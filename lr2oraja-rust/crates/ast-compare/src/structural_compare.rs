use crate::ir::*;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct StructuralReport {
    pub comparisons: Vec<MethodComparison>,
    pub summary: StructuralSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct MethodComparison {
    pub java_file: String,
    pub rust_file: String,
    pub type_name: String,
    pub method_name: String,
    pub similarity: f64,
    pub differences: Vec<String>,
    pub java_cf_summary: CfSummary,
    pub rust_cf_summary: CfSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct CfSummary {
    pub if_count: usize,
    pub switch_count: usize,
    pub for_count: usize,
    pub while_count: usize,
    pub try_catch_count: usize,
    pub return_count: usize,
    pub max_depth: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct StructuralSummary {
    pub total_compared: usize,
    pub high_similarity: usize,
    pub medium_similarity: usize,
    pub low_similarity: usize,
    pub avg_similarity: f64,
}

/// Compare the control flow structure of two method bodies.
pub fn compare_methods(
    java_body: &MethodBody,
    rust_body: &MethodBody,
    java_file: &str,
    rust_file: &str,
    type_name: &str,
    method_name: &str,
) -> MethodComparison {
    let java_summary = summarize_control_flow(&java_body.control_flow, 0);
    let rust_summary = summarize_control_flow(&rust_body.control_flow, 0);

    let mut differences = Vec::new();

    // Compare control flow counts
    compare_count(
        "if",
        java_summary.if_count,
        rust_summary.if_count,
        &mut differences,
    );
    compare_count(
        "switch/match",
        java_summary.switch_count,
        rust_summary.switch_count,
        &mut differences,
    );
    compare_count(
        "for",
        java_summary.for_count,
        rust_summary.for_count,
        &mut differences,
    );
    compare_count(
        "while/loop",
        java_summary.while_count,
        rust_summary.while_count,
        &mut differences,
    );
    compare_count(
        "try-catch",
        java_summary.try_catch_count,
        rust_summary.try_catch_count,
        &mut differences,
    );
    compare_count(
        "return",
        java_summary.return_count,
        rust_summary.return_count,
        &mut differences,
    );

    if java_summary.max_depth != rust_summary.max_depth {
        differences.push(format!(
            "max nesting depth: Java={} Rust={}",
            java_summary.max_depth, rust_summary.max_depth
        ));
    }

    // Structural tree comparison
    let tree_diffs = compare_cf_trees(&java_body.control_flow, &rust_body.control_flow, 0);
    differences.extend(tree_diffs);

    let similarity = compute_similarity(&java_summary, &rust_summary);

    MethodComparison {
        java_file: java_file.to_string(),
        rust_file: rust_file.to_string(),
        type_name: type_name.to_string(),
        method_name: method_name.to_string(),
        similarity,
        differences,
        java_cf_summary: java_summary,
        rust_cf_summary: rust_summary,
    }
}

fn compare_count(name: &str, java: usize, rust: usize, diffs: &mut Vec<String>) {
    if java != rust {
        diffs.push(format!("{name} count: Java={java} Rust={rust}"));
    }
}

fn summarize_control_flow(nodes: &[ControlFlowNode], depth: usize) -> CfSummary {
    let mut summary = CfSummary {
        if_count: 0,
        switch_count: 0,
        for_count: 0,
        while_count: 0,
        try_catch_count: 0,
        return_count: 0,
        max_depth: depth,
    };

    for node in nodes {
        match node {
            ControlFlowNode::If { children, .. } => {
                summary.if_count += 1;
                merge_summary(&mut summary, &summarize_control_flow(children, depth + 1));
            }
            ControlFlowNode::Switch { children, .. } => {
                summary.switch_count += 1;
                merge_summary(&mut summary, &summarize_control_flow(children, depth + 1));
            }
            ControlFlowNode::ForLoop { children } => {
                summary.for_count += 1;
                merge_summary(&mut summary, &summarize_control_flow(children, depth + 1));
            }
            ControlFlowNode::WhileLoop { children } => {
                summary.while_count += 1;
                merge_summary(&mut summary, &summarize_control_flow(children, depth + 1));
            }
            ControlFlowNode::TryCatch { children, .. } => {
                summary.try_catch_count += 1;
                merge_summary(&mut summary, &summarize_control_flow(children, depth + 1));
            }
            ControlFlowNode::Return => summary.return_count += 1,
            ControlFlowNode::Break => {}
            ControlFlowNode::Continue => {}
        }
    }
    summary
}

fn merge_summary(target: &mut CfSummary, other: &CfSummary) {
    target.if_count += other.if_count;
    target.switch_count += other.switch_count;
    target.for_count += other.for_count;
    target.while_count += other.while_count;
    target.try_catch_count += other.try_catch_count;
    target.return_count += other.return_count;
    target.max_depth = target.max_depth.max(other.max_depth);
}

fn compute_similarity(java: &CfSummary, rust: &CfSummary) -> f64 {
    let counts_java = [
        java.if_count,
        java.switch_count,
        java.for_count,
        java.while_count,
        java.try_catch_count,
        java.return_count,
    ];
    let counts_rust = [
        rust.if_count,
        rust.switch_count,
        rust.for_count,
        rust.while_count,
        rust.try_catch_count,
        rust.return_count,
    ];

    let total_java: usize = counts_java.iter().sum();
    let total_rust: usize = counts_rust.iter().sum();

    if total_java == 0 && total_rust == 0 {
        return 1.0;
    }

    let max_total = total_java.max(total_rust) as f64;
    let matching: usize = counts_java
        .iter()
        .zip(counts_rust.iter())
        .map(|(j, r)| (*j).min(*r))
        .sum();

    matching as f64 / max_total
}

/// Compare two control flow trees at the structural level.
fn compare_cf_trees(
    java_nodes: &[ControlFlowNode],
    rust_nodes: &[ControlFlowNode],
    depth: usize,
) -> Vec<String> {
    let mut diffs = Vec::new();
    let indent = "  ".repeat(depth);
    let max_len = java_nodes.len().max(rust_nodes.len());

    for i in 0..max_len {
        match (java_nodes.get(i), rust_nodes.get(i)) {
            (Some(jn), Some(rn)) => {
                if jn.kind_name() != rn.kind_name() {
                    diffs.push(format!(
                        "{indent}[{i}] node type mismatch: Java={} Rust={}",
                        jn.kind_name(),
                        rn.kind_name()
                    ));
                } else {
                    // Compare node-specific properties
                    match (jn, rn) {
                        (
                            ControlFlowNode::If {
                                branches: jb,
                                has_else: je,
                                children: jc,
                            },
                            ControlFlowNode::If {
                                branches: rb,
                                has_else: re,
                                children: rc,
                            },
                        ) => {
                            if jb != rb {
                                diffs.push(format!(
                                    "{indent}[{i}] if branches: Java={jb} Rust={rb}"
                                ));
                            }
                            if je != re {
                                diffs.push(format!(
                                    "{indent}[{i}] if has_else: Java={je} Rust={re}"
                                ));
                            }
                            diffs.extend(compare_cf_trees(jc, rc, depth + 1));
                        }
                        (
                            ControlFlowNode::Switch {
                                arm_count: ja,
                                children: jc,
                            },
                            ControlFlowNode::Switch {
                                arm_count: ra,
                                children: rc,
                            },
                        ) => {
                            if ja != ra {
                                diffs.push(format!(
                                    "{indent}[{i}] switch/match arms: Java={ja} Rust={ra}"
                                ));
                            }
                            diffs.extend(compare_cf_trees(jc, rc, depth + 1));
                        }
                        (
                            ControlFlowNode::ForLoop { children: jc },
                            ControlFlowNode::ForLoop { children: rc },
                        )
                        | (
                            ControlFlowNode::WhileLoop { children: jc },
                            ControlFlowNode::WhileLoop { children: rc },
                        ) => {
                            diffs.extend(compare_cf_trees(jc, rc, depth + 1));
                        }
                        (
                            ControlFlowNode::TryCatch {
                                catch_count: jcc,
                                has_finally: jf,
                                children: jc,
                            },
                            ControlFlowNode::TryCatch {
                                catch_count: rcc,
                                has_finally: rf,
                                children: rc,
                            },
                        ) => {
                            if jcc != rcc {
                                diffs.push(format!(
                                    "{indent}[{i}] catch count: Java={jcc} Rust={rcc}"
                                ));
                            }
                            if jf != rf {
                                diffs.push(format!(
                                    "{indent}[{i}] has_finally: Java={jf} Rust={rf}"
                                ));
                            }
                            diffs.extend(compare_cf_trees(jc, rc, depth + 1));
                        }
                        _ => {}
                    }
                }
            }
            (Some(jn), None) => {
                diffs.push(format!("{indent}[{i}] extra in Java: {}", jn.kind_name()));
            }
            (None, Some(rn)) => {
                diffs.push(format!("{indent}[{i}] extra in Rust: {}", rn.kind_name()));
            }
            (None, None) => unreachable!(),
        }
    }

    diffs
}

/// Build a full structural comparison report for matched method pairs.
pub fn build_structural_report(
    comparisons: Vec<MethodComparison>,
    threshold: f64,
) -> StructuralReport {
    let filtered: Vec<MethodComparison> = comparisons
        .into_iter()
        .filter(|c| c.similarity < threshold)
        .collect();

    let total = filtered.len();
    let high = filtered.iter().filter(|c| c.similarity >= 0.8).count();
    let medium = filtered
        .iter()
        .filter(|c| c.similarity >= 0.5 && c.similarity < 0.8)
        .count();
    let low = filtered.iter().filter(|c| c.similarity < 0.5).count();
    let avg = if total > 0 {
        filtered.iter().map(|c| c.similarity).sum::<f64>() / total as f64
    } else {
        1.0
    };

    StructuralReport {
        comparisons: filtered,
        summary: StructuralSummary {
            total_compared: total,
            high_similarity: high,
            medium_similarity: medium,
            low_similarity: low,
            avg_similarity: avg,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_control_flow() {
        let body = MethodBody {
            control_flow: vec![
                ControlFlowNode::If {
                    branches: 2,
                    has_else: true,
                    children: vec![ControlFlowNode::ForLoop {
                        children: vec![ControlFlowNode::Continue],
                    }],
                },
                ControlFlowNode::Return,
            ],
            literals: Vec::new(),
            is_stub: false,
        };

        let comparison = compare_methods(&body, &body, "Foo.java", "foo.rs", "Foo", "bar");
        assert_eq!(comparison.similarity, 1.0);
        assert!(comparison.differences.is_empty());
    }

    #[test]
    fn test_different_control_flow() {
        let java_body = MethodBody {
            control_flow: vec![ControlFlowNode::If {
                branches: 2,
                has_else: true,
                children: vec![],
            }],
            literals: Vec::new(),
            is_stub: false,
        };
        let rust_body = MethodBody {
            control_flow: vec![ControlFlowNode::Switch {
                arm_count: 3,
                children: vec![],
            }],
            literals: Vec::new(),
            is_stub: false,
        };

        let comparison =
            compare_methods(&java_body, &rust_body, "Foo.java", "foo.rs", "Foo", "bar");
        assert!(comparison.similarity < 1.0);
        assert!(!comparison.differences.is_empty());
    }

    #[test]
    fn test_empty_bodies() {
        let body = MethodBody {
            control_flow: vec![],
            literals: Vec::new(),
            is_stub: false,
        };
        let comparison = compare_methods(&body, &body, "Foo.java", "foo.rs", "Foo", "bar");
        assert_eq!(comparison.similarity, 1.0);
    }
}
