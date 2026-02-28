use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use walkdir::WalkDir;

use crate::naming::{self, class_to_module};

#[derive(Debug, Clone)]
pub struct FileMapping {
    pub java_path: PathBuf,
    pub rust_path: Option<PathBuf>,
    pub java_class: String,
    pub java_package: String,
    pub rust_crate: Option<String>,
    pub confidence: MappingConfidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingConfidence {
    Exact,
    NameMismatch,
    NotFound,
}

/// Build the Java package → Rust crate mapping table.
fn package_to_crate_table() -> HashMap<&'static str, (&'static str, &'static str)> {
    // Maps Java package → (Rust crate directory name, subdirectory within src/)
    [
        ("bms.model", ("bms-model", "")),
        ("bms.model.bmson", ("bms-model", "bmson")),
        ("bms.model.osu", ("bms-model", "osu")),
        ("bms.table", ("bms-table", "")),
        ("bms.player.beatoraja", ("beatoraja-core", "")),
        ("bms.player.beatoraja.audio", ("beatoraja-audio", "")),
        (
            "bms.player.beatoraja.config",
            ("beatoraja-core", "config_pkg"),
        ),
        (
            "bms.player.beatoraja.controller",
            ("beatoraja-controller", ""),
        ),
        ("bms.player.beatoraja.decide", ("beatoraja-decide", "")),
        ("bms.player.beatoraja.external", ("beatoraja-external", "")),
        (
            "bms.player.beatoraja.external.DiscordRPC",
            ("discord-rpc", ""),
        ),
        ("bms.player.beatoraja.input", ("beatoraja-input", "")),
        ("bms.player.beatoraja.ir", ("beatoraja-ir", "")),
        ("bms.player.beatoraja.launcher", ("beatoraja-launcher", "")),
        ("bms.player.beatoraja.modmenu", ("beatoraja-modmenu", "")),
        ("bms.player.beatoraja.obs", ("beatoraja-obs", "")),
        ("bms.player.beatoraja.pattern", ("beatoraja-pattern", "")),
        ("bms.player.beatoraja.play", ("beatoraja-play", "")),
        ("bms.player.beatoraja.play.bga", ("beatoraja-play", "bga")),
        ("bms.player.beatoraja.result", ("beatoraja-result", "")),
        ("bms.player.beatoraja.select", ("beatoraja-select", "")),
        (
            "bms.player.beatoraja.select.bar",
            ("beatoraja-select", "bar"),
        ),
        ("bms.player.beatoraja.skin", ("beatoraja-skin", "")),
        ("bms.player.beatoraja.skin.json", ("beatoraja-skin", "json")),
        ("bms.player.beatoraja.skin.lr2", ("beatoraja-skin", "lr2")),
        ("bms.player.beatoraja.skin.lua", ("beatoraja-skin", "lua")),
        (
            "bms.player.beatoraja.skin.property",
            ("beatoraja-skin", "property"),
        ),
        ("bms.player.beatoraja.song", ("beatoraja-song", "")),
        ("bms.player.beatoraja.stream", ("beatoraja-stream", "")),
        (
            "bms.player.beatoraja.stream.command",
            ("beatoraja-stream", "command"),
        ),
        ("bms.player.beatoraja.system", ("beatoraja-system", "")),
        ("bms.tool.mdprocessor", ("md-processor", "")),
        ("bms.tool.util", ("md-processor", "")),
    ]
    .into_iter()
    .collect()
}

/// Collect all .java files under the given root directory.
pub fn collect_java_files(java_root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(java_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "java"))
    {
        files.push(entry.into_path());
    }
    files.sort();
    Ok(files)
}

/// Collect all .rs files under the given root directory, excluding test files and stubs.
pub fn collect_rust_files(rust_root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(rust_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let p = e.path();
            p.extension().is_some_and(|ext| ext == "rs")
                && !p.to_string_lossy().contains("/tests/")
                && !p.to_string_lossy().contains("/target/")
        })
    {
        files.push(entry.into_path());
    }
    files.sort();
    Ok(files)
}

/// Extract the Java package from a file path.
///
/// Given a path like `.../core/src/bms/player/beatoraja/audio/PCM.java`,
/// extracts `bms.player.beatoraja.audio`.
pub fn extract_java_package(java_path: &Path, java_root: &Path) -> Option<String> {
    let relative = java_path.strip_prefix(java_root).ok()?;
    let parent = relative.parent()?;
    let components: Vec<&str> = parent
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    // Find the index of "bms" to start the package path
    let bms_idx = components.iter().position(|c| *c == "bms")?;
    Some(components[bms_idx..].join("."))
}

/// Extract the Java class name from a file path.
pub fn extract_java_class(java_path: &Path) -> Option<String> {
    java_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

/// Build file mappings between Java and Rust files.
pub fn build_file_mappings(java_root: &Path, rust_crates_root: &Path) -> Result<Vec<FileMapping>> {
    let java_files = collect_java_files(java_root).context("collecting Java files")?;
    let rust_files = collect_rust_files(rust_crates_root).context("collecting Rust files")?;

    // Index Rust files by (crate_name, relative_path) for fast lookup
    let rust_index = build_rust_index(&rust_files, rust_crates_root);
    let pkg_table = package_to_crate_table();

    let mut mappings = Vec::new();

    for java_path in &java_files {
        let java_class = match extract_java_class(java_path) {
            Some(c) => c,
            None => continue,
        };
        let java_package = extract_java_package(java_path, java_root).unwrap_or_default();

        let (rust_path, rust_crate, confidence) =
            find_rust_counterpart(&java_class, &java_package, &pkg_table, &rust_index);

        mappings.push(FileMapping {
            java_path: java_path.clone(),
            rust_path,
            java_class,
            java_package,
            rust_crate,
            confidence,
        });
    }

    Ok(mappings)
}

/// Build an index of Rust files: maps (crate_dir_name, sub_path_stem) → full path.
fn build_rust_index(
    rust_files: &[PathBuf],
    rust_crates_root: &Path,
) -> HashMap<(String, String), PathBuf> {
    let mut index = HashMap::new();
    for path in rust_files {
        if let Ok(relative) = path.strip_prefix(rust_crates_root) {
            let components: Vec<&str> = relative
                .components()
                .filter_map(|c| c.as_os_str().to_str())
                .collect();
            // Expected: crate_dir/src/[subdir/]file.rs
            if components.len() >= 2 {
                let crate_dir = components[0].to_string();
                // Get path after "src/"
                let src_idx = components.iter().position(|c| *c == "src");
                if let Some(idx) = src_idx {
                    let after_src: Vec<&str> = components[idx + 1..].to_vec();
                    if let Some(file_name) = after_src.last()
                        && let Some(stem) = file_name.strip_suffix(".rs")
                    {
                        // sub_path = subdir components + stem
                        let sub_dir = if after_src.len() > 1 {
                            after_src[..after_src.len() - 1].join("/")
                        } else {
                            String::new()
                        };
                        let key = if sub_dir.is_empty() {
                            stem.to_string()
                        } else {
                            format!("{sub_dir}/{stem}")
                        };
                        index.insert((crate_dir, key), path.clone());
                    }
                }
            }
        }
    }
    index
}

/// Find the Rust counterpart for a Java class.
fn find_rust_counterpart(
    java_class: &str,
    java_package: &str,
    pkg_table: &HashMap<&str, (&str, &str)>,
    rust_index: &HashMap<(String, String), PathBuf>,
) -> (Option<PathBuf>, Option<String>, MappingConfidence) {
    let rust_module_name = class_to_module(java_class);

    // Look up the crate for this package
    if let Some(&(crate_dir, sub_dir)) = pkg_table.get(java_package) {
        let key = if sub_dir.is_empty() {
            rust_module_name.clone()
        } else {
            format!("{sub_dir}/{rust_module_name}")
        };

        if let Some(path) = rust_index.get(&(crate_dir.to_string(), key)) {
            return (
                Some(path.clone()),
                Some(crate_dir.to_string()),
                MappingConfidence::Exact,
            );
        }

        // Also check beatoraja-types for re-exported types
        let types_key = rust_module_name.clone();
        if let Some(path) = rust_index.get(&("beatoraja-types".to_string(), types_key)) {
            return (
                Some(path.clone()),
                Some("beatoraja-types".to_string()),
                MappingConfidence::Exact,
            );
        }

        // Fuzzy file name matching within the target crate
        for ((cd, file_key), path) in rust_index {
            if cd != crate_dir {
                continue;
            }
            let stem = file_key.rsplit('/').next().unwrap_or(file_key);
            if naming::edit_distance_within(stem, &rust_module_name, 2) {
                return (
                    Some(path.clone()),
                    Some(crate_dir.to_string()),
                    MappingConfidence::NameMismatch,
                );
            }
        }

        return (
            None,
            Some(crate_dir.to_string()),
            MappingConfidence::NotFound,
        );
    }

    // No package mapping found — try brute-force search across all crates
    for ((crate_dir, file_key), path) in rust_index {
        let stem = file_key.rsplit('/').next().unwrap_or(file_key);
        if stem == rust_module_name {
            return (
                Some(path.clone()),
                Some(crate_dir.clone()),
                MappingConfidence::NameMismatch,
            );
        }
    }

    // Fuzzy brute-force search across all crates
    for ((crate_dir, file_key), path) in rust_index {
        let stem = file_key.rsplit('/').next().unwrap_or(file_key);
        if naming::edit_distance_within(stem, &rust_module_name, 2) {
            return (
                Some(path.clone()),
                Some(crate_dir.clone()),
                MappingConfidence::NameMismatch,
            );
        }
    }

    (None, None, MappingConfidence::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_java_package() {
        let root = Path::new("/project/lr2oraja-java/core/src");
        let path = Path::new("/project/lr2oraja-java/core/src/bms/player/beatoraja/audio/PCM.java");
        assert_eq!(
            extract_java_package(path, root),
            Some("bms.player.beatoraja.audio".to_string())
        );
    }

    #[test]
    fn test_extract_java_package_model() {
        let root = Path::new("/project/lr2oraja-java/core/dependencies/jbms-parser/src");
        let path = Path::new(
            "/project/lr2oraja-java/core/dependencies/jbms-parser/src/bms/model/BMSDecoder.java",
        );
        assert_eq!(
            extract_java_package(path, root),
            Some("bms.model".to_string())
        );
    }

    #[test]
    fn test_extract_java_class() {
        let path = Path::new("/foo/bar/BMSDecoder.java");
        assert_eq!(extract_java_class(path), Some("BMSDecoder".to_string()));
    }

    #[test]
    fn test_package_table_completeness() {
        let table = package_to_crate_table();
        assert!(table.contains_key("bms.model"));
        assert!(table.contains_key("bms.player.beatoraja"));
        assert!(table.contains_key("bms.player.beatoraja.audio"));
        assert!(table.contains_key("bms.player.beatoraja.play"));
        assert!(table.contains_key("bms.player.beatoraja.skin.json"));
    }
}
