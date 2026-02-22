use std::path::Path;

use anyhow::{Context, Result};
use tree_sitter::{Node, Parser};

use crate::ir::*;

/// Parse a Java source file into the common IR.
pub fn parse_java_file(path: &Path) -> Result<SourceFile> {
    let source =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;

    let mut parser = Parser::new();
    let language = tree_sitter_java::LANGUAGE;
    parser
        .set_language(&language.into())
        .context("setting Java language")?;

    let tree = parser.parse(&source, None).context("parsing Java source")?;

    let root = tree.root_node();
    let types = extract_types(root, source.as_bytes());

    Ok(SourceFile {
        path: path.to_path_buf(),
        language: Language::Java,
        types,
        free_functions: Vec::new(),
    })
}

fn extract_types(node: Node, source: &[u8]) -> Vec<TypeDecl> {
    let mut types = Vec::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind() {
            "class_declaration" | "interface_declaration" | "enum_declaration" => {
                if let Some(decl) = extract_type_decl(child, source) {
                    types.push(decl);
                }
            }
            "program" => {
                types.extend(extract_types(child, source));
            }
            _ => {}
        }
    }
    types
}

fn extract_type_decl(node: Node, source: &[u8]) -> Option<TypeDecl> {
    let kind = match node.kind() {
        "class_declaration" => {
            if has_modifier(node, source, "abstract") {
                TypeKind::AbstractClass
            } else {
                TypeKind::Class
            }
        }
        "interface_declaration" => TypeKind::Interface,
        "enum_declaration" => TypeKind::Enum,
        _ => return None,
    };

    let name = node
        .child_by_field_name("name")
        .and_then(|n| n.utf8_text(source).ok())
        .unwrap_or("")
        .to_string();

    let visibility = extract_visibility(node, source);
    let line = node.start_position().row + 1;

    let body = node.child_by_field_name("body");
    let mut fields = Vec::new();
    let mut methods = Vec::new();
    let mut inner_types = Vec::new();

    if let Some(body_node) = body {
        let mut cursor = body_node.walk();
        for child in body_node.children(&mut cursor) {
            match child.kind() {
                "field_declaration" => {
                    fields.extend(extract_field_decl(child, source));
                }
                "method_declaration" | "constructor_declaration" => {
                    if let Some(m) = extract_method_decl(child, source) {
                        methods.push(m);
                    }
                }
                "class_declaration" | "interface_declaration" | "enum_declaration" => {
                    if let Some(inner) = extract_type_decl(child, source) {
                        inner_types.push(inner);
                    }
                }
                "enum_body_declarations" => {
                    let mut inner_cursor = child.walk();
                    for inner_child in child.children(&mut inner_cursor) {
                        match inner_child.kind() {
                            "method_declaration" | "constructor_declaration" => {
                                if let Some(m) = extract_method_decl(inner_child, source) {
                                    methods.push(m);
                                }
                            }
                            "field_declaration" => {
                                fields.extend(extract_field_decl(inner_child, source));
                            }
                            "class_declaration" | "interface_declaration" | "enum_declaration" => {
                                if let Some(inner) = extract_type_decl(inner_child, source) {
                                    inner_types.push(inner);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                // Enum constants — count them as fields
                "enum_constant" => {
                    if let Some(const_name) = child
                        .child_by_field_name("name")
                        .and_then(|n| n.utf8_text(source).ok())
                    {
                        fields.push(FieldDecl {
                            name: const_name.to_string(),
                            type_name: "enum_constant".to_string(),
                            visibility: Visibility::Public,
                            is_static: true,
                            is_final: true,
                            line: child.start_position().row + 1,
                            serde_rename: None,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    Some(TypeDecl {
        name,
        kind,
        visibility,
        fields,
        methods,
        inner_types,
        line,
    })
}

fn extract_field_decl(node: Node, source: &[u8]) -> Vec<FieldDecl> {
    let visibility = extract_visibility(node, source);
    let is_static = has_modifier(node, source, "static");
    let is_final = has_modifier(node, source, "final");

    let type_name = node
        .child_by_field_name("type")
        .and_then(|n| n.utf8_text(source).ok())
        .unwrap_or("")
        .to_string();

    let mut fields = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "variable_declarator"
            && let Some(name_node) = child.child_by_field_name("name")
        {
            let name = name_node.utf8_text(source).unwrap_or("").to_string();
            fields.push(FieldDecl {
                name,
                type_name: type_name.clone(),
                visibility,
                is_static,
                is_final,
                line: child.start_position().row + 1,
                serde_rename: None,
            });
        }
    }
    fields
}

fn extract_method_decl(node: Node, source: &[u8]) -> Option<MethodDecl> {
    let is_constructor = node.kind() == "constructor_declaration";

    let name = if is_constructor {
        node.child_by_field_name("name")
            .and_then(|n| n.utf8_text(source).ok())
            .unwrap_or("<init>")
            .to_string()
    } else {
        node.child_by_field_name("name")
            .and_then(|n| n.utf8_text(source).ok())
            .unwrap_or("")
            .to_string()
    };

    let visibility = extract_visibility(node, source);
    let is_static = has_modifier(node, source, "static");
    let is_abstract = has_modifier(node, source, "abstract");

    let return_type = if is_constructor {
        None
    } else {
        node.child_by_field_name("type")
            .and_then(|n| n.utf8_text(source).ok())
            .map(|s| s.to_string())
            .filter(|s| s != "void")
    };

    let params = extract_params(node, source);

    let body_node = node.child_by_field_name("body");
    let body = body_node.map(|b| extract_method_body(b, source));

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

fn extract_params(node: Node, source: &[u8]) -> Vec<ParamDecl> {
    let mut params = Vec::new();
    let params_node = node.child_by_field_name("parameters");
    if let Some(pnode) = params_node {
        let mut cursor = pnode.walk();
        for child in pnode.children(&mut cursor) {
            if child.kind() == "formal_parameter" || child.kind() == "spread_parameter" {
                let type_name = child
                    .child_by_field_name("type")
                    .and_then(|n| n.utf8_text(source).ok())
                    .unwrap_or("")
                    .to_string();
                let name = child
                    .child_by_field_name("name")
                    .and_then(|n| n.utf8_text(source).ok())
                    .unwrap_or("")
                    .to_string();
                params.push(ParamDecl { name, type_name });
            }
        }
    }
    params
}

fn extract_method_body(node: Node, source: &[u8]) -> MethodBody {
    let control_flow = extract_control_flow(node, source);
    let literals = extract_literals(node, source);
    MethodBody {
        control_flow,
        literals,
    }
}

fn extract_control_flow(node: Node, source: &[u8]) -> Vec<ControlFlowNode> {
    let mut nodes = Vec::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind() {
            "if_statement" => {
                let (branches, has_else, children) = extract_if_chain(child, source);
                nodes.push(ControlFlowNode::If {
                    branches,
                    has_else,
                    children,
                });
            }
            "switch_expression" | "switch_block_statement_group" => {
                let arm_count = count_switch_arms(child);
                let children = extract_control_flow(child, source);
                nodes.push(ControlFlowNode::Switch {
                    arm_count,
                    children,
                });
            }
            "for_statement" | "enhanced_for_statement" => {
                let children = extract_control_flow_from_body(child, source);
                nodes.push(ControlFlowNode::ForLoop { children });
            }
            "while_statement" | "do_statement" => {
                let children = extract_control_flow_from_body(child, source);
                nodes.push(ControlFlowNode::WhileLoop { children });
            }
            "try_statement" | "try_with_resources_statement" => {
                let (catch_count, has_finally) = count_catches(child);
                let children = extract_control_flow(child, source);
                nodes.push(ControlFlowNode::TryCatch {
                    catch_count,
                    has_finally,
                    children,
                });
            }
            "return_statement" => {
                nodes.push(ControlFlowNode::Return);
            }
            "break_statement" => {
                nodes.push(ControlFlowNode::Break);
            }
            "continue_statement" => {
                nodes.push(ControlFlowNode::Continue);
            }
            _ => {
                // Recurse into other statement types (blocks, etc.)
                nodes.extend(extract_control_flow(child, source));
            }
        }
    }
    nodes
}

fn extract_control_flow_from_body(node: Node, source: &[u8]) -> Vec<ControlFlowNode> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "block" || child.kind() == "statement" {
            return extract_control_flow(child, source);
        }
    }
    // If no explicit block, try the node's body field
    if let Some(body) = node.child_by_field_name("body") {
        return extract_control_flow(body, source);
    }
    extract_control_flow(node, source)
}

fn extract_if_chain(node: Node, source: &[u8]) -> (usize, bool, Vec<ControlFlowNode>) {
    let mut branches = 1;
    let mut has_else = false;
    let mut children = Vec::new();

    // Extract control flow from the "consequence" (then-branch)
    if let Some(consequence) = node.child_by_field_name("consequence") {
        children.extend(extract_control_flow(consequence, source));
    }

    // Check for else/else-if chain
    if let Some(alternative) = node.child_by_field_name("alternative") {
        if alternative.kind() == "if_statement" {
            let (sub_branches, sub_has_else, sub_children) = extract_if_chain(alternative, source);
            branches += sub_branches;
            has_else = sub_has_else;
            children.extend(sub_children);
        } else {
            has_else = true;
            children.extend(extract_control_flow(alternative, source));
        }
    }

    (branches, has_else, children)
}

fn count_switch_arms(node: Node) -> usize {
    let mut count = 0;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "switch_block_statement_group" || child.kind() == "switch_rule" {
            count += 1;
        }
        // Also recurse into switch_block
        if child.kind() == "switch_block" {
            count += count_switch_arms(child);
        }
    }
    count
}

fn count_catches(node: Node) -> (usize, bool) {
    let mut catch_count = 0;
    let mut has_finally = false;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "catch_clause" {
            catch_count += 1;
        }
        if child.kind() == "finally_clause" {
            has_finally = true;
        }
    }
    (catch_count, has_finally)
}

fn extract_literals(node: Node, source: &[u8]) -> Vec<Literal> {
    let mut literals = Vec::new();
    collect_literals_recursive(node, source, &mut literals);
    literals
}

fn collect_literals_recursive(node: Node, source: &[u8], literals: &mut Vec<Literal>) {
    match node.kind() {
        "decimal_integer_literal" => {
            if let Ok(text) = node.utf8_text(source) {
                let text = text.trim_end_matches(['l', 'L']);
                if let Ok(v) = text.replace('_', "").parse::<i64>() {
                    literals.push(Literal::Integer(v));
                }
            }
        }
        "hex_integer_literal" => {
            if let Ok(text) = node.utf8_text(source) {
                let text = text.trim_end_matches(['l', 'L']);
                let text = text.trim_start_matches("0x").trim_start_matches("0X");
                if let Ok(v) = i64::from_str_radix(&text.replace('_', ""), 16) {
                    literals.push(Literal::Hex(v));
                }
            }
        }
        "decimal_floating_point_literal" => {
            if let Ok(text) = node.utf8_text(source) {
                let text = text.trim_end_matches(['f', 'F', 'd', 'D']);
                if let Ok(v) = text.replace('_', "").parse::<f64>() {
                    literals.push(Literal::Float(v));
                }
            }
        }
        "string_literal" => {
            if let Ok(text) = node.utf8_text(source) {
                // Remove surrounding quotes
                let inner = text
                    .strip_prefix('"')
                    .and_then(|s| s.strip_suffix('"'))
                    .unwrap_or(text);
                literals.push(Literal::String(inner.to_string()));
            }
        }
        "character_literal" => {
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
        "true" => literals.push(Literal::Boolean(true)),
        "false" => literals.push(Literal::Boolean(false)),
        _ => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                collect_literals_recursive(child, source, literals);
            }
        }
    }
}

fn extract_visibility(node: Node, source: &[u8]) -> Visibility {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "modifiers" || child.kind() == "modifier" {
            let mut mod_cursor = child.walk();
            for modifier in child.children(&mut mod_cursor) {
                if let Ok(text) = modifier.utf8_text(source) {
                    match text {
                        "public" => return Visibility::Public,
                        "protected" => return Visibility::Protected,
                        "private" => return Visibility::Private,
                        _ => {}
                    }
                }
            }
            // Check the modifier node itself
            if let Ok(text) = child.utf8_text(source) {
                if text.contains("public") {
                    return Visibility::Public;
                }
                if text.contains("protected") {
                    return Visibility::Protected;
                }
                if text.contains("private") {
                    return Visibility::Private;
                }
            }
        }
    }
    Visibility::PackagePrivate
}

fn has_modifier(node: Node, source: &[u8], modifier_name: &str) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "modifiers" {
            let text = child.utf8_text(source).unwrap_or("");
            if text.contains(modifier_name) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_java(source: &str) -> SourceFile {
        let mut parser = Parser::new();
        let language = tree_sitter_java::LANGUAGE;
        parser.set_language(&language.into()).unwrap();
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        let types = extract_types(root, source.as_bytes());
        SourceFile {
            path: "test.java".into(),
            language: Language::Java,
            types,
            free_functions: Vec::new(),
        }
    }

    #[test]
    fn test_simple_class() {
        let source = r#"
public class Config {
    private String name;
    public int score;

    public String getName() { return name; }
    public void setName(String name) { this.name = name; }
    public static Config load(String path) { return null; }
}
"#;
        let file = parse_java(source);
        assert_eq!(file.types.len(), 1);
        let cls = &file.types[0];
        assert_eq!(cls.name, "Config");
        assert_eq!(cls.kind, TypeKind::Class);
        assert_eq!(cls.fields.len(), 2);
        assert_eq!(cls.methods.len(), 3);

        assert_eq!(cls.methods[0].name, "getName");
        assert!(!cls.methods[0].is_static);
        assert_eq!(cls.methods[2].name, "load");
        assert!(cls.methods[2].is_static);
    }

    #[test]
    fn test_abstract_class() {
        let source = r#"
public abstract class ChartDecoder {
    public abstract BMSModel decode(Path f);
    public void setLntype(int lntype) {}
}
"#;
        let file = parse_java(source);
        assert_eq!(file.types[0].kind, TypeKind::AbstractClass);
        assert!(file.types[0].methods[0].is_abstract);
        assert!(!file.types[0].methods[1].is_abstract);
    }

    #[test]
    fn test_enum_with_methods() {
        let source = r#"
public enum ClearType {
    NoPlay, Failed, AssistEasy;

    public static ClearType fromInt(int i) { return values()[i]; }
}
"#;
        let file = parse_java(source);
        assert_eq!(file.types[0].kind, TypeKind::Enum);
        assert_eq!(file.types[0].fields.len(), 3); // enum constants
        assert_eq!(file.types[0].methods.len(), 1);
        assert_eq!(file.types[0].methods[0].name, "fromInt");
    }

    #[test]
    fn test_interface() {
        let source = r#"
public interface Validatable {
    boolean validate();
}
"#;
        let file = parse_java(source);
        assert_eq!(file.types[0].kind, TypeKind::Interface);
        assert_eq!(file.types[0].methods.len(), 1);
    }

    #[test]
    fn test_control_flow() {
        let source = r#"
public class Foo {
    public void bar() {
        if (x > 0) {
            for (int i = 0; i < 10; i++) {
                if (i % 2 == 0) {
                    continue;
                }
            }
        } else if (x < 0) {
            while (true) {
                break;
            }
        } else {
            return;
        }
    }
}
"#;
        let file = parse_java(source);
        let body = file.types[0].methods[0].body.as_ref().unwrap();
        assert_eq!(body.control_flow.len(), 1); // one top-level if
        if let ControlFlowNode::If {
            branches, has_else, ..
        } = &body.control_flow[0]
        {
            assert_eq!(*branches, 2); // if + else if
            assert!(has_else);
        } else {
            panic!("expected If node");
        }
    }

    #[test]
    fn test_literals() {
        let source = r#"
public class Foo {
    public void bar() {
        int x = 42;
        long y = 0x5DEECE66DL;
        float f = 3.14f;
        String s = "hello";
        boolean b = true;
    }
}
"#;
        let file = parse_java(source);
        let body = file.types[0].methods[0].body.as_ref().unwrap();
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
    fn test_inner_class() {
        let source = r#"
public class Outer {
    public static class Inner {
        public void doStuff() {}
    }
    public void outerMethod() {}
}
"#;
        let file = parse_java(source);
        assert_eq!(file.types[0].methods.len(), 1);
        assert_eq!(file.types[0].inner_types.len(), 1);
        assert_eq!(file.types[0].inner_types[0].name, "Inner");
        assert_eq!(file.types[0].inner_types[0].methods.len(), 1);
    }
}
