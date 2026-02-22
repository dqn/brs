use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};

use ast_compare::constants::{build_constants_report, compare_constants};
use ast_compare::file_mapping::{
    MappingConfidence, build_file_mappings, collect_java_files, collect_rust_files,
};
use ast_compare::ir::SourceFile;
use ast_compare::java_parser::parse_java_file;
use ast_compare::naming;
use ast_compare::report::{
    format_constants_report, format_signature_report, format_structural_report,
};
use ast_compare::rust_parser::parse_rust_file;
use ast_compare::signature_map::{
    MapConfig, VisibilityFilter, build_signature_map, load_ignore_patterns,
};
use ast_compare::structural_compare::{build_structural_report, compare_methods};

#[derive(Parser)]
#[command(name = "ast-compare")]
#[command(about = "Java-Rust AST structural comparison tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Java source root directories (searched recursively for .java files)
    #[arg(long, num_args = 1.., default_values_t = default_java_roots_str())]
    java_root: Vec<String>,

    /// Rust crates root directory
    #[arg(long, default_value = default_rust_root())]
    rust_root: PathBuf,

    /// Output format
    #[arg(long, default_value = "text")]
    format: OutputFormat,

    /// Output file (stdout if not specified)
    #[arg(long)]
    output: Option<PathBuf>,
}

fn default_java_roots_str() -> Vec<String> {
    vec![
        "../../lr2oraja-java/core/src".to_string(),
        "../../lr2oraja-java/core/dependencies/jbms-parser/src".to_string(),
        "../../lr2oraja-java/core/dependencies/jbmstable-parser/src".to_string(),
    ]
}

fn default_rust_root() -> &'static str {
    "../crates"
}

#[derive(Subcommand)]
enum Commands {
    /// Generate Java-Rust file and signature mapping
    Map {
        /// Filter to specific Java package
        #[arg(long)]
        package: Option<String>,

        /// Show only unmapped items
        #[arg(long)]
        unmapped_only: bool,

        /// Visibility filter: all, public, public-protected
        #[arg(long, default_value = "all")]
        visibility: String,

        /// Include stub methods as matched (default: report separately)
        #[arg(long)]
        include_stubs: bool,

        /// Path to ignore patterns file (one suffix pattern per line, # comments)
        #[arg(long)]
        ignore_file: Option<PathBuf>,
    },

    /// Compare control flow structure of matched methods
    Compare {
        /// Filter to specific file (Java class name)
        #[arg(long)]
        file: Option<String>,

        /// Minimum similarity threshold to suppress output (0.0-1.0)
        #[arg(long, default_value = "0.8")]
        threshold: f64,
    },

    /// Extract and compare constants/magic numbers
    Constants {
        /// Filter to specific file (Java class name)
        #[arg(long)]
        file: Option<String>,

        /// Exclude common trivial constants (0, 1, -1, true, false)
        #[arg(long)]
        exclude_trivial: bool,
    },

    /// Full report (all three features combined)
    Full {
        /// Minimum similarity threshold
        #[arg(long, default_value = "0.8")]
        threshold: f64,
    },
}

#[derive(Clone, Debug, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::Map {
            package,
            unmapped_only,
            visibility,
            include_stubs,
            ignore_file,
        } => run_map(
            &cli,
            package.as_deref(),
            *unmapped_only,
            visibility,
            *include_stubs,
            ignore_file.as_deref(),
        ),
        Commands::Compare { file, threshold } => run_compare(&cli, file.as_deref(), *threshold),
        Commands::Constants {
            file,
            exclude_trivial,
        } => run_constants(&cli, file.as_deref(), *exclude_trivial),
        Commands::Full { threshold } => run_full(&cli, *threshold),
    }
}

fn run_map(
    cli: &Cli,
    package: Option<&str>,
    unmapped_only: bool,
    visibility: &str,
    include_stubs: bool,
    ignore_file: Option<&std::path::Path>,
) -> Result<()> {
    let (file_mappings, java_sources, rust_sources) = load_all(cli)?;

    let mut filtered_mappings = file_mappings;
    if let Some(pkg) = package {
        filtered_mappings.retain(|fm| fm.java_package.starts_with(pkg));
    }
    if unmapped_only {
        filtered_mappings.retain(|fm| fm.confidence == MappingConfidence::NotFound);
    }

    let visibility_filter = match visibility {
        "public" => VisibilityFilter::Public,
        "public-protected" => VisibilityFilter::PublicProtected,
        _ => VisibilityFilter::All,
    };

    let ignore_patterns = if let Some(path) = ignore_file {
        load_ignore_patterns(path)
    } else {
        // Try default .ast-compare-ignore
        let default_path = std::path::Path::new(".ast-compare-ignore");
        if default_path.exists() {
            load_ignore_patterns(default_path)
        } else {
            Vec::new()
        }
    };

    let config = MapConfig {
        visibility_filter,
        include_stubs,
        ignore_patterns,
    };

    let report = build_signature_map(&filtered_mappings, &java_sources, &rust_sources, &config);

    let text = format_signature_report(&report);
    output_result(cli, &text, &report)
}

fn run_compare(cli: &Cli, file_filter: Option<&str>, threshold: f64) -> Result<()> {
    let (file_mappings, java_sources, rust_sources) = load_all(cli)?;

    let mut comparisons = Vec::new();

    for fm in &file_mappings {
        if let Some(filter) = file_filter
            && !fm.java_class.contains(filter)
        {
            continue;
        }

        let java_source = java_sources.iter().find(|f| f.path == fm.java_path);
        let rust_source = fm
            .rust_path
            .as_ref()
            .and_then(|rp| rust_sources.iter().find(|f| f.path == *rp));

        if let (Some(java), Some(rust)) = (java_source, rust_source) {
            for jt in &java.types {
                let rust_type = rust.types.iter().find(|rt| rt.name == jt.name);
                if let Some(rt) = rust_type {
                    for jm in &jt.methods {
                        let snake_name = naming::method_to_snake(&jm.name);
                        let rm = rt.methods.iter().find(|m| m.name == snake_name);
                        if let (Some(jbody), Some(rm)) = (&jm.body, rm)
                            && let Some(rbody) = &rm.body
                        {
                            comparisons.push(compare_methods(
                                jbody,
                                rbody,
                                &fm.java_path.display().to_string(),
                                &fm.rust_path.as_ref().unwrap().display().to_string(),
                                &jt.name,
                                &jm.name,
                            ));
                        }
                    }
                }
            }
        }
    }

    let report = build_structural_report(comparisons, threshold);
    let text = format_structural_report(&report);
    output_result(cli, &text, &report)
}

fn run_constants(cli: &Cli, file_filter: Option<&str>, exclude_trivial: bool) -> Result<()> {
    let (file_mappings, java_sources, rust_sources) = load_all(cli)?;

    let mut comparisons = Vec::new();

    for fm in &file_mappings {
        if let Some(filter) = file_filter
            && !fm.java_class.contains(filter)
        {
            continue;
        }

        let java_source = java_sources.iter().find(|f| f.path == fm.java_path);
        let rust_source = fm
            .rust_path
            .as_ref()
            .and_then(|rp| rust_sources.iter().find(|f| f.path == *rp));

        if let (Some(java), Some(rust)) = (java_source, rust_source) {
            for jt in &java.types {
                let rust_type = rust.types.iter().find(|rt| rt.name == jt.name);
                if let Some(rt) = rust_type {
                    for jm in &jt.methods {
                        let snake_name = naming::method_to_snake(&jm.name);
                        let rm = rt.methods.iter().find(|m| m.name == snake_name);
                        if let (Some(jbody), Some(rm)) = (&jm.body, rm)
                            && let Some(rbody) = &rm.body
                        {
                            comparisons.push(compare_constants(
                                jbody,
                                rbody,
                                &fm.java_path.display().to_string(),
                                &fm.rust_path.as_ref().unwrap().display().to_string(),
                                &jt.name,
                                &jm.name,
                                exclude_trivial,
                            ));
                        }
                    }
                }
            }
        }
    }

    let report = build_constants_report(comparisons);
    let text = format_constants_report(&report);
    output_result(cli, &text, &report)
}

fn run_full(cli: &Cli, threshold: f64) -> Result<()> {
    eprintln!("Running full analysis...");
    eprintln!();

    eprintln!("=== Phase 1: Signature Mapping ===");
    run_map(cli, None, false, "all", false, None)?;

    eprintln!();
    eprintln!("=== Phase 2: Structural Comparison ===");
    run_compare(cli, None, threshold)?;

    eprintln!();
    eprintln!("=== Phase 3: Constants Comparison ===");
    run_constants(cli, None, true)?;

    Ok(())
}

fn load_all(
    cli: &Cli,
) -> Result<(
    Vec<ast_compare::file_mapping::FileMapping>,
    Vec<SourceFile>,
    Vec<SourceFile>,
)> {
    // Canonicalize Rust root first so all paths are consistent
    let rust_root = std::fs::canonicalize(&cli.rust_root)
        .with_context(|| format!("canonicalizing Rust root: {}", cli.rust_root.display()))?;

    // Build file mappings from all Java roots
    let mut all_file_mappings = Vec::new();
    let mut all_java_sources = Vec::new();

    for java_root_str in &cli.java_root {
        let java_root = std::fs::canonicalize(java_root_str)
            .with_context(|| format!("canonicalizing Java root: {java_root_str}"))?;

        let file_mappings =
            build_file_mappings(&java_root, &rust_root).context("building file mappings")?;

        // Parse Java files
        let java_files = collect_java_files(&java_root)?;
        for path in &java_files {
            match parse_java_file(path) {
                Ok(source) => all_java_sources.push(source),
                Err(e) => log::warn!("Failed to parse Java file {}: {e}", path.display()),
            }
        }

        all_file_mappings.extend(file_mappings);
    }
    let rust_files = collect_rust_files(&rust_root)?;
    let mut all_rust_sources = Vec::new();
    for path in &rust_files {
        match parse_rust_file(path) {
            Ok(source) => all_rust_sources.push(source),
            Err(e) => log::warn!("Failed to parse Rust file {}: {e}", path.display()),
        }
    }

    eprintln!(
        "Loaded {} Java files, {} Rust files, {} mappings",
        all_java_sources.len(),
        all_rust_sources.len(),
        all_file_mappings.len()
    );

    Ok((all_file_mappings, all_java_sources, all_rust_sources))
}

fn output_result<T: serde::Serialize>(cli: &Cli, text: &str, report: &T) -> Result<()> {
    let content = match cli.format {
        OutputFormat::Text => text.to_string(),
        OutputFormat::Json => serde_json::to_string_pretty(report)?,
    };

    if let Some(output_path) = &cli.output {
        std::fs::write(output_path, &content)?;
        eprintln!("Report written to {}", output_path.display());
    } else {
        print!("{content}");
    }

    Ok(())
}
