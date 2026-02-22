use crate::ir::*;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ConstantsReport {
    pub comparisons: Vec<MethodConstantComparison>,
    pub summary: ConstantsSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct MethodConstantComparison {
    pub java_file: String,
    pub rust_file: String,
    pub type_name: String,
    pub method_name: String,
    pub java_literals: Vec<LiteralInfo>,
    pub rust_literals: Vec<LiteralInfo>,
    pub missing_in_rust: Vec<LiteralInfo>,
    pub extra_in_rust: Vec<LiteralInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LiteralInfo {
    pub value: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConstantsSummary {
    pub total_compared: usize,
    pub methods_with_diffs: usize,
    pub total_missing: usize,
    pub total_extra: usize,
}

impl From<&Literal> for LiteralInfo {
    fn from(lit: &Literal) -> Self {
        let (value, kind) = match lit {
            Literal::Integer(v) => (v.to_string(), "int"),
            Literal::Float(v) => (format!("{v}"), "float"),
            Literal::String(s) => (format!("\"{s}\""), "string"),
            Literal::Boolean(b) => (b.to_string(), "bool"),
            Literal::Char(c) => (format!("'{c}'"), "char"),
            Literal::Hex(v) => (format!("0x{v:X}"), "hex"),
            Literal::Long(v) => (format!("{v}L"), "long"),
        };
        LiteralInfo {
            value,
            kind: kind.to_string(),
        }
    }
}

/// Compare constants between a Java and Rust method body.
pub fn compare_constants(
    java_body: &MethodBody,
    rust_body: &MethodBody,
    java_file: &str,
    rust_file: &str,
    type_name: &str,
    method_name: &str,
    exclude_trivial: bool,
) -> MethodConstantComparison {
    let java_lits: Vec<&Literal> = java_body
        .literals
        .iter()
        .filter(|l| !exclude_trivial || !l.is_trivial())
        .collect();

    let rust_lits: Vec<&Literal> = rust_body
        .literals
        .iter()
        .filter(|l| !exclude_trivial || !l.is_trivial())
        .collect();

    let mut missing_in_rust = Vec::new();
    let mut matched_rust_indices = Vec::new();

    // Find literals in Java that don't have a counterpart in Rust
    for jl in &java_lits {
        let found = rust_lits
            .iter()
            .enumerate()
            .find(|(i, rl)| !matched_rust_indices.contains(i) && literals_match(jl, rl));

        if let Some((idx, _)) = found {
            matched_rust_indices.push(idx);
        } else {
            missing_in_rust.push(LiteralInfo::from(*jl));
        }
    }

    // Unmatched Rust literals are "extra"
    let extra_in_rust: Vec<LiteralInfo> = rust_lits
        .iter()
        .enumerate()
        .filter(|(i, _)| !matched_rust_indices.contains(i))
        .map(|(_, rl)| LiteralInfo::from(*rl))
        .collect();

    MethodConstantComparison {
        java_file: java_file.to_string(),
        rust_file: rust_file.to_string(),
        type_name: type_name.to_string(),
        method_name: method_name.to_string(),
        java_literals: java_lits.iter().map(|l| LiteralInfo::from(*l)).collect(),
        rust_literals: rust_lits.iter().map(|l| LiteralInfo::from(*l)).collect(),
        missing_in_rust,
        extra_in_rust,
    }
}

fn literals_match(java: &Literal, rust: &Literal) -> bool {
    match (java, rust) {
        // Numeric comparison (ignoring literal form: hex vs decimal, int vs long)
        (a, b) if a.numeric_value().is_some() && b.numeric_value().is_some() => {
            let ja = a.numeric_value().unwrap();
            let ra = b.numeric_value().unwrap();
            (ja - ra).abs() < 0.001
        }
        (Literal::String(a), Literal::String(b)) => a == b,
        (Literal::Boolean(a), Literal::Boolean(b)) => a == b,
        (Literal::Char(a), Literal::Char(b)) => a == b,
        _ => false,
    }
}

/// Build a full constants comparison report.
pub fn build_constants_report(comparisons: Vec<MethodConstantComparison>) -> ConstantsReport {
    let total = comparisons.len();
    let with_diffs = comparisons
        .iter()
        .filter(|c| !c.missing_in_rust.is_empty() || !c.extra_in_rust.is_empty())
        .count();
    let total_missing: usize = comparisons.iter().map(|c| c.missing_in_rust.len()).sum();
    let total_extra: usize = comparisons.iter().map(|c| c.extra_in_rust.len()).sum();

    ConstantsReport {
        comparisons: comparisons
            .into_iter()
            .filter(|c| !c.missing_in_rust.is_empty() || !c.extra_in_rust.is_empty())
            .collect(),
        summary: ConstantsSummary {
            total_compared: total,
            methods_with_diffs: with_diffs,
            total_missing,
            total_extra,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matching_literals() {
        let java_body = MethodBody {
            control_flow: vec![],
            literals: vec![
                Literal::Integer(42),
                Literal::Hex(0xFF),
                Literal::String("hello".to_string()),
            ],
        };
        let rust_body = MethodBody {
            control_flow: vec![],
            literals: vec![
                Literal::Integer(42),
                Literal::Integer(255), // same value as 0xFF
                Literal::String("hello".to_string()),
            ],
        };

        let comparison = compare_constants(
            &java_body, &rust_body, "Foo.java", "foo.rs", "Foo", "bar", false,
        );
        assert!(comparison.missing_in_rust.is_empty());
        assert!(comparison.extra_in_rust.is_empty());
    }

    #[test]
    fn test_missing_literal() {
        let java_body = MethodBody {
            control_flow: vec![],
            literals: vec![Literal::Integer(42), Literal::Integer(99)],
        };
        let rust_body = MethodBody {
            control_flow: vec![],
            literals: vec![Literal::Integer(42)],
        };

        let comparison = compare_constants(
            &java_body, &rust_body, "Foo.java", "foo.rs", "Foo", "bar", false,
        );
        assert_eq!(comparison.missing_in_rust.len(), 1);
        assert_eq!(comparison.missing_in_rust[0].value, "99");
    }

    #[test]
    fn test_exclude_trivial() {
        let java_body = MethodBody {
            control_flow: vec![],
            literals: vec![
                Literal::Integer(0),
                Literal::Integer(1),
                Literal::Integer(42),
                Literal::Boolean(true),
            ],
        };
        let rust_body = MethodBody {
            control_flow: vec![],
            literals: vec![Literal::Integer(42)],
        };

        let comparison = compare_constants(
            &java_body, &rust_body, "Foo.java", "foo.rs", "Foo", "bar", true,
        );
        // 0, 1, true are trivial → excluded → only 42 remains, which matches
        assert!(comparison.missing_in_rust.is_empty());
    }
}
