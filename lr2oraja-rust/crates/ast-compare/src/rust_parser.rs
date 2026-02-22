use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use tree_sitter::{Node, Parser};

use crate::ir::*;

/// Parse a Rust source file into the common IR.
pub fn parse_rust_file(path: &Path) -> Result<SourceFile> {
    let source =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;

    let mut parser = Parser::new();
    let language = tree_sitter_rust::LANGUAGE;
    parser
        .set_language(&language.into())
        .context("setting Rust language")?;

    let tree = parser.parse(&source, None).context("parsing Rust source")?;

    let root = tree.root_node();
    let (mut types, free_functions, impl_blocks) = extract_top_level(root, source.as_bytes());

    // Merge impl blocks into their corresponding type declarations
    merge_impl_blocks(&mut types, &impl_blocks);

    Ok(SourceFile {
        path: path.to_path_buf(),
        language: Language::Rust,
        types,
        free_functions,
    })
}

struct ImplBlock {
    type_name: String,
    #[allow(dead_code)]
    trait_name: Option<String>,
    methods: Vec<MethodDecl>,
}

fn extract_top_level(
    node: Node,
    source: &[u8],
) -> (Vec<TypeDecl>, Vec<MethodDecl>, Vec<ImplBlock>) {
    let mut types = Vec::new();
    let mut free_functions = Vec::new();
    let mut impl_blocks = Vec::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind() {
            "struct_item" => {
                if let Some(decl) = extract_struct_decl(child, source) {
                    types.push(decl);
                }
            }
            "enum_item" => {
                if let Some(decl) = extract_enum_decl(child, source) {
                    types.push(decl);
                }
            }
            "trait_item" => {
                if let Some(decl) = extract_trait_decl(child, source) {
                    types.push(decl);
                }
            }
            "impl_item" => {
                if let Some(block) = extract_impl_block(child, source) {
                    impl_blocks.push(block);
                }
            }
            "function_item" => {
                if let Some(func) = extract_function_decl(child, source) {
                    free_functions.push(func);
                }
            }
            _ => {}
        }
    }

    (types, free_functions, impl_blocks)
}

fn extract_struct_decl(node: Node, source: &[u8]) -> Option<TypeDecl> {
    let name = node
        .child_by_field_name("name")
        .and_then(|n| n.utf8_text(source).ok())
        .unwrap_or("")
        .to_string();

    let visibility = extract_rust_visibility(node, source);
    let line = node.start_position().row + 1;

    let fields = extract_struct_fields(node, source);

    Some(TypeDecl {
        name,
        kind: TypeKind::Struct,
        visibility,
        fields,
        methods: Vec::new(),
        inner_types: Vec::new(),
        line,
    })
}

fn extract_enum_decl(node: Node, source: &[u8]) -> Option<TypeDecl> {
    let name = node
        .child_by_field_name("name")
        .and_then(|n| n.utf8_text(source).ok())
        .unwrap_or("")
        .to_string();

    let visibility = extract_rust_visibility(node, source);
    let line = node.start_position().row + 1;

    let fields = extract_enum_variants(node, source);

    Some(TypeDecl {
        name,
        kind: TypeKind::Enum,
        visibility,
        fields,
        methods: Vec::new(),
        inner_types: Vec::new(),
        line,
    })
}

fn extract_trait_decl(node: Node, source: &[u8]) -> Option<TypeDecl> {
    let name = node
        .child_by_field_name("name")
        .and_then(|n| n.utf8_text(source).ok())
        .unwrap_or("")
        .to_string();

    let visibility = extract_rust_visibility(node, source);
    let line = node.start_position().row + 1;

    let mut methods = Vec::new();
    if let Some(body) = node.child_by_field_name("body") {
        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            if (child.kind() == "function_signature_item" || child.kind() == "function_item")
                && let Some(func) = extract_function_decl(child, source)
            {
                methods.push(func);
            }
        }
    }

    Some(TypeDecl {
        name,
        kind: TypeKind::Trait,
        visibility,
        fields: Vec::new(),
        methods,
        inner_types: Vec::new(),
        line,
    })
}

fn extract_impl_block(node: Node, source: &[u8]) -> Option<ImplBlock> {
    // Determine the type name and optional trait name
    let (type_name, trait_name) = extract_impl_names(node, source)?;

    let mut methods = Vec::new();
    if let Some(body) = node.child_by_field_name("body") {
        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            if child.kind() == "function_item"
                && let Some(func) = extract_function_decl(child, source)
            {
                methods.push(func);
            }
        }
    }

    Some(ImplBlock {
        type_name,
        trait_name,
        methods,
    })
}

fn extract_impl_names(node: Node, source: &[u8]) -> Option<(String, Option<String>)> {
    // impl [Trait for] Type { ... }
    // tree-sitter-rust: `type` field is the implementing type,
    // `trait` field is the trait being implemented (if any)
    let type_node = node.child_by_field_name("type")?;
    let type_name = type_node.utf8_text(source).ok()?.to_string();

    let trait_name = node
        .child_by_field_name("trait")
        .and_then(|n| n.utf8_text(source).ok())
        .map(|s| s.to_string());

    Some((type_name, trait_name))
}

fn extract_function_decl(node: Node, source: &[u8]) -> Option<MethodDecl> {
    let name = node
        .child_by_field_name("name")
        .and_then(|n| n.utf8_text(source).ok())
        .unwrap_or("")
        .to_string();

    let visibility = extract_rust_visibility(node, source);
    let is_abstract = node.kind() == "function_signature_item";

    let params = extract_rust_params(node, source);
    let is_static = !params
        .iter()
        .any(|p| p.name == "self" || p.name == "&self" || p.name == "&mut self");

    // Filter out self params from the reported params
    let params: Vec<ParamDecl> = params
        .into_iter()
        .filter(|p| p.name != "self" && p.name != "&self" && p.name != "&mut self")
        .collect();

    let return_type = extract_rust_return_type(node, source);

    let body_node = node.child_by_field_name("body");
    let body = body_node.map(|b| extract_rust_method_body(b, source));

    Some(MethodDecl {
        name,
        visibility,
        is_static,
        is_abstract,
        params,
        return_type,
        body,
        line: node.start_position().row + 1,
    })
}

fn extract_struct_fields(node: Node, source: &[u8]) -> Vec<FieldDecl> {
    let mut fields = Vec::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if child.kind() == "field_declaration_list" {
            let mut field_cursor = child.walk();
            for field_child in child.children(&mut field_cursor) {
                if field_child.kind() == "field_declaration" {
                    let name = field_child
                        .child_by_field_name("name")
                        .and_then(|n| n.utf8_text(source).ok())
                        .unwrap_or("")
                        .to_string();
                    let type_name = field_child
                        .child_by_field_name("type")
                        .and_then(|n| n.utf8_text(source).ok())
                        .unwrap_or("")
                        .to_string();
                    let visibility = extract_rust_visibility(field_child, source);
                    let serde_rename = extract_serde_rename(field_child, source);

                    fields.push(FieldDecl {
                        name,
                        type_name,
                        visibility,
                        is_static: false,
                        is_final: false,
                        line: field_child.start_position().row + 1,
                        serde_rename,
                    });
                }
            }
        }
    }
    fields
}

fn extract_enum_variants(node: Node, source: &[u8]) -> Vec<FieldDecl> {
    let mut fields = Vec::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if child.kind() == "enum_variant_list" {
            let mut variant_cursor = child.walk();
            for variant in child.children(&mut variant_cursor) {
                if variant.kind() == "enum_variant" {
                    let name = variant
                        .child_by_field_name("name")
                        .and_then(|n| n.utf8_text(source).ok())
                        .unwrap_or("")
                        .to_string();
                    fields.push(FieldDecl {
                        name,
                        type_name: "enum_variant".to_string(),
                        visibility: Visibility::Public,
                        is_static: true,
                        is_final: true,
                        line: variant.start_position().row + 1,
                        serde_rename: None,
                    });
                }
            }
        }
    }
    fields
}

fn extract_rust_params(node: Node, source: &[u8]) -> Vec<ParamDecl> {
    let mut params = Vec::new();
    let params_node = node.child_by_field_name("parameters");
    if let Some(pnode) = params_node {
        let mut cursor = pnode.walk();
        for child in pnode.children(&mut cursor) {
            match child.kind() {
                "self_parameter" => {
                    let text = child.utf8_text(source).unwrap_or("self").to_string();
                    params.push(ParamDecl {
                        name: text,
                        type_name: "Self".to_string(),
                    });
                }
                "parameter" => {
                    let name = child
                        .child_by_field_name("pattern")
                        .and_then(|n| n.utf8_text(source).ok())
                        .unwrap_or("")
                        .to_string();
                    let type_name = child
                        .child_by_field_name("type")
                        .and_then(|n| n.utf8_text(source).ok())
                        .unwrap_or("")
                        .to_string();
                    params.push(ParamDecl { name, type_name });
                }
                _ => {}
            }
        }
    }
    params
}

fn extract_rust_return_type(node: Node, source: &[u8]) -> Option<String> {
    node.child_by_field_name("return_type")
        .and_then(|n| {
            // The return type node wraps the actual type
            let mut cursor = n.walk();
            n.children(&mut cursor)
                .find(|c| c.kind() != "->")
                .and_then(|c| c.utf8_text(source).ok())
                .map(|s| s.to_string())
        })
        .filter(|s| s != "()")
}

fn extract_rust_method_body(node: Node, source: &[u8]) -> MethodBody {
    let control_flow = extract_rust_control_flow(node, source);
    let literals = extract_rust_literals(node, source);
    MethodBody {
        control_flow,
        literals,
    }
}

fn extract_rust_control_flow(node: Node, source: &[u8]) -> Vec<ControlFlowNode> {
    let mut nodes = Vec::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind() {
            "if_expression" => {
                let (branches, has_else, children) = extract_rust_if_chain(child, source);
                nodes.push(ControlFlowNode::If {
                    branches,
                    has_else,
                    children,
                });
            }
            "match_expression" => {
                let arm_count = count_match_arms(child);
                let children = extract_rust_control_flow(child, source);
                nodes.push(ControlFlowNode::Switch {
                    arm_count,
                    children,
                });
            }
            "for_expression" => {
                let children = extract_rust_control_flow_from_body(child, source);
                nodes.push(ControlFlowNode::ForLoop { children });
            }
            "while_expression" => {
                let children = extract_rust_control_flow_from_body(child, source);
                nodes.push(ControlFlowNode::WhileLoop { children });
            }
            "loop_expression" => {
                let children = extract_rust_control_flow_from_body(child, source);
                nodes.push(ControlFlowNode::WhileLoop { children });
            }
            "return_expression" => {
                nodes.push(ControlFlowNode::Return);
            }
            "break_expression" => {
                nodes.push(ControlFlowNode::Break);
            }
            "continue_expression" => {
                nodes.push(ControlFlowNode::Continue);
            }
            _ => {
                nodes.extend(extract_rust_control_flow(child, source));
            }
        }
    }
    nodes
}

fn extract_rust_control_flow_from_body(node: Node, source: &[u8]) -> Vec<ControlFlowNode> {
    if let Some(body) = node.child_by_field_name("body") {
        return extract_rust_control_flow(body, source);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "block" {
            return extract_rust_control_flow(child, source);
        }
    }
    extract_rust_control_flow(node, source)
}

fn extract_rust_if_chain(node: Node, source: &[u8]) -> (usize, bool, Vec<ControlFlowNode>) {
    let mut branches = 1;
    let mut has_else = false;
    let mut children = Vec::new();

    if let Some(consequence) = node.child_by_field_name("consequence") {
        children.extend(extract_rust_control_flow(consequence, source));
    }

    if let Some(alternative) = node.child_by_field_name("alternative") {
        // In tree-sitter-rust, `else_clause` wraps the else body.
        // For `else if`, the else_clause contains an if_expression.
        // For `else`, the else_clause contains a block.
        let inner = find_if_in_else_clause(alternative);
        if let Some(inner_if) = inner {
            let (sub_branches, sub_has_else, sub_children) =
                extract_rust_if_chain(inner_if, source);
            branches += sub_branches;
            has_else = sub_has_else;
            children.extend(sub_children);
        } else {
            has_else = true;
            children.extend(extract_rust_control_flow(alternative, source));
        }
    }

    (branches, has_else, children)
}

/// Find an if_expression inside an else_clause node.
fn find_if_in_else_clause(node: Node) -> Option<Node> {
    // The else_clause might directly contain an if_expression,
    // or the alternative field of the if might be an if_expression itself.
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "if_expression" {
            return Some(child);
        }
        // Also check inside else_clause
        if child.kind() == "else_clause" {
            return find_if_in_else_clause(child);
        }
    }
    None
}

fn count_match_arms(node: Node) -> usize {
    let mut count = 0;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "match_block" {
            let mut arm_cursor = child.walk();
            for arm in child.children(&mut arm_cursor) {
                if arm.kind() == "match_arm" {
                    count += 1;
                }
            }
        }
    }
    count
}

fn extract_rust_literals(node: Node, source: &[u8]) -> Vec<Literal> {
    let mut literals = Vec::new();
    collect_rust_literals_recursive(node, source, &mut literals);
    literals
}

fn collect_rust_literals_recursive(node: Node, source: &[u8], literals: &mut Vec<Literal>) {
    match node.kind() {
        "integer_literal" => {
            if let Ok(text) = node.utf8_text(source) {
                let text = text.replace('_', "");
                if text.starts_with("0x") || text.starts_with("0X") {
                    let hex = text.trim_start_matches("0x").trim_start_matches("0X");
                    let hex = hex.trim_end_matches(['i', 'u', '8', '6', '4', '2', '1', '3']);
                    if let Ok(v) = i64::from_str_radix(hex, 16) {
                        literals.push(Literal::Hex(v));
                    }
                } else {
                    // Strip type suffix (i32, u64, etc.)
                    let cleaned: String = text
                        .trim_end_matches(|c: char| c.is_ascii_alphabetic())
                        .to_string();
                    if let Ok(v) = cleaned.parse::<i64>() {
                        literals.push(Literal::Integer(v));
                    }
                }
            }
        }
        "float_literal" => {
            if let Ok(text) = node.utf8_text(source) {
                let text = text
                    .replace('_', "")
                    .trim_end_matches("f32")
                    .trim_end_matches("f64")
                    .to_string();
                if let Ok(v) = text.parse::<f64>() {
                    literals.push(Literal::Float(v));
                }
            }
        }
        "string_literal" => {
            if let Ok(text) = node.utf8_text(source) {
                let inner = text
                    .strip_prefix('"')
                    .and_then(|s| s.strip_suffix('"'))
                    .unwrap_or(text);
                literals.push(Literal::String(inner.to_string()));
            }
        }
        "char_literal" => {
            if let Ok(text) = node.utf8_text(source) {
                let inner = text
                    .strip_prefix('\'')
                    .and_then(|s| s.strip_suffix('\''))
                    .unwrap_or(text);
                if let Some(ch) = inner.chars().next() {
                    literals.push(Literal::Char(ch));
                }
            }
        }
        "boolean_literal" => {
            if let Ok(text) = node.utf8_text(source) {
                match text {
                    "true" => literals.push(Literal::Boolean(true)),
                    "false" => literals.push(Literal::Boolean(false)),
                    _ => {}
                }
            }
        }
        _ => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                collect_rust_literals_recursive(child, source, literals);
            }
        }
    }
}

/// Extract `#[serde(rename = "...")]` value from a field's attributes.
fn extract_serde_rename(node: Node, source: &[u8]) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "attribute_item" || child.kind() == "attribute" {
            let text = child.utf8_text(source).unwrap_or("");
            // Match patterns like #[serde(rename = "camelCase")]
            if let Some(start) = text.find("rename") {
                let rest = &text[start..];
                if let Some(q1) = rest.find('"') {
                    let after_q1 = &rest[q1 + 1..];
                    if let Some(q2) = after_q1.find('"') {
                        return Some(after_q1[..q2].to_string());
                    }
                }
            }
        }
    }
    None
}

fn extract_rust_visibility(node: Node, source: &[u8]) -> Visibility {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "visibility_modifier" {
            let text = child.utf8_text(source).unwrap_or("");
            if text.contains("pub(crate)") {
                return Visibility::PubCrate;
            }
            if text.contains("pub") {
                return Visibility::Public;
            }
        }
    }
    Visibility::Private
}

/// Merge impl blocks into matching type declarations.
fn merge_impl_blocks(types: &mut [TypeDecl], impl_blocks: &[ImplBlock]) {
    // Build index first, then mutate separately to avoid borrow conflict
    let type_index: HashMap<String, usize> = types
        .iter()
        .enumerate()
        .map(|(i, t)| (t.name.clone(), i))
        .collect();

    for block in impl_blocks {
        // Strip generic parameters: "BmsTable<T>" → "BmsTable"
        let base_name = block
            .type_name
            .split('<')
            .next()
            .unwrap_or(&block.type_name);
        if let Some(&idx) = type_index.get(base_name) {
            types[idx].methods.extend(block.methods.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_rust(source: &str) -> SourceFile {
        let mut parser = Parser::new();
        let language = tree_sitter_rust::LANGUAGE;
        parser.set_language(&language.into()).unwrap();
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        let (mut types, free_functions, impl_blocks) = extract_top_level(root, source.as_bytes());
        merge_impl_blocks(&mut types, &impl_blocks);
        SourceFile {
            path: "test.rs".into(),
            language: Language::Rust,
            types,
            free_functions,
        }
    }

    #[test]
    fn test_simple_struct() {
        let source = r#"
pub struct Config {
    pub name: String,
    score: i32,
}

impl Config {
    pub fn new() -> Self { Config { name: String::new(), score: 0 } }
    pub fn get_name(&self) -> &str { &self.name }
    pub fn set_name(&mut self, name: String) { self.name = name; }
    pub fn load(path: &str) -> Config { todo!() }
}
"#;
        let file = parse_rust(source);
        assert_eq!(file.types.len(), 1);
        let s = &file.types[0];
        assert_eq!(s.name, "Config");
        assert_eq!(s.kind, TypeKind::Struct);
        assert_eq!(s.fields.len(), 2);
        assert_eq!(s.methods.len(), 4);

        // `new` and `load` should be static (no self param)
        assert!(
            s.methods
                .iter()
                .find(|m| m.name == "new")
                .unwrap()
                .is_static
        );
        assert!(
            s.methods
                .iter()
                .find(|m| m.name == "load")
                .unwrap()
                .is_static
        );
        // `get_name` and `set_name` should not be static
        assert!(
            !s.methods
                .iter()
                .find(|m| m.name == "get_name")
                .unwrap()
                .is_static
        );
        assert!(
            !s.methods
                .iter()
                .find(|m| m.name == "set_name")
                .unwrap()
                .is_static
        );
    }

    #[test]
    fn test_enum_with_impl() {
        let source = r#"
pub enum ClearType {
    NoPlay,
    Failed,
    AssistEasy,
}

impl ClearType {
    pub fn from_int(i: i32) -> Self { todo!() }
}
"#;
        let file = parse_rust(source);
        assert_eq!(file.types[0].kind, TypeKind::Enum);
        assert_eq!(file.types[0].fields.len(), 3);
        assert_eq!(file.types[0].methods.len(), 1);
        assert_eq!(file.types[0].methods[0].name, "from_int");
    }

    #[test]
    fn test_trait() {
        let source = r#"
pub trait Validatable {
    fn validate(&self) -> bool;
}
"#;
        let file = parse_rust(source);
        assert_eq!(file.types[0].kind, TypeKind::Trait);
        assert_eq!(file.types[0].methods.len(), 1);
        assert_eq!(file.types[0].methods[0].name, "validate");
    }

    #[test]
    fn test_control_flow() {
        let source = r#"
pub fn bar(x: i32) {
    if x > 0 {
        for i in 0..10 {
            if i % 2 == 0 {
                continue;
            }
        }
    } else if x < 0 {
        while true {
            break;
        }
    } else {
        return;
    }
}
"#;
        let file = parse_rust(source);
        let body = file.free_functions[0].body.as_ref().unwrap();
        assert_eq!(body.control_flow.len(), 1);
        if let ControlFlowNode::If {
            branches, has_else, ..
        } = &body.control_flow[0]
        {
            assert_eq!(*branches, 2);
            assert!(has_else);
        } else {
            panic!("expected If node");
        }
    }

    #[test]
    fn test_literals() {
        let source = r#"
pub fn bar() {
    let x = 42;
    let y = 0x5DEECE66Di64;
    let f = 3.14_f32;
    let s = "hello";
    let b = true;
}
"#;
        let file = parse_rust(source);
        let body = file.free_functions[0].body.as_ref().unwrap();
        assert!(body.literals.len() >= 4);
        assert!(
            body.literals
                .iter()
                .any(|l| matches!(l, Literal::Integer(42)))
        );
        assert!(
            body.literals
                .iter()
                .any(|l| matches!(l, Literal::Hex(0x5DEECE66D)))
        );
        assert!(
            body.literals
                .iter()
                .any(|l| matches!(l, Literal::String(s) if s == "hello"))
        );
        assert!(
            body.literals
                .iter()
                .any(|l| matches!(l, Literal::Boolean(true)))
        );
    }

    #[test]
    fn test_free_functions() {
        let source = r#"
pub fn helper() -> i32 { 42 }
fn internal() {}
"#;
        let file = parse_rust(source);
        assert_eq!(file.free_functions.len(), 2);
        assert_eq!(file.free_functions[0].name, "helper");
        assert_eq!(file.free_functions[0].visibility, Visibility::Public);
        assert_eq!(file.free_functions[1].name, "internal");
        assert_eq!(file.free_functions[1].visibility, Visibility::Private);
    }

    #[test]
    fn test_match_expression() {
        let source = r#"
pub fn foo(x: i32) -> &str {
    match x {
        0 => "zero",
        1 => "one",
        _ => "other",
    }
}
"#;
        let file = parse_rust(source);
        let body = file.free_functions[0].body.as_ref().unwrap();
        assert_eq!(body.control_flow.len(), 1);
        if let ControlFlowNode::Switch { arm_count, .. } = &body.control_flow[0] {
            assert_eq!(*arm_count, 3);
        } else {
            panic!("expected Switch node");
        }
    }
}
