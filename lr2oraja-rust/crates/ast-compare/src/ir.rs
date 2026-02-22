use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFile {
    pub path: PathBuf,
    pub language: Language,
    pub types: Vec<TypeDecl>,
    /// Top-level functions not inside any type (Rust free functions)
    pub free_functions: Vec<MethodDecl>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Java,
    Rust,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDecl {
    pub name: String,
    pub kind: TypeKind,
    pub visibility: Visibility,
    pub fields: Vec<FieldDecl>,
    pub methods: Vec<MethodDecl>,
    pub inner_types: Vec<TypeDecl>,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeKind {
    Class,
    AbstractClass,
    Interface,
    Enum,
    Struct,
    Trait,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Protected,
    PackagePrivate,
    Private,
    PubCrate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDecl {
    pub name: String,
    pub type_name: String,
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_final: bool,
    pub line: usize,
    /// Value from `#[serde(rename = "...")]` attribute (Rust only)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serde_rename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodDecl {
    pub name: String,
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_abstract: bool,
    pub params: Vec<ParamDecl>,
    pub return_type: Option<String>,
    pub body: Option<MethodBody>,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDecl {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodBody {
    pub control_flow: Vec<ControlFlowNode>,
    pub literals: Vec<Literal>,
    /// Whether the body is a stub (contains only todo!/unimplemented!/log::warn!)
    #[serde(default)]
    pub is_stub: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlFlowNode {
    If {
        branches: usize,
        has_else: bool,
        children: Vec<ControlFlowNode>,
    },
    Switch {
        arm_count: usize,
        children: Vec<ControlFlowNode>,
    },
    ForLoop {
        children: Vec<ControlFlowNode>,
    },
    WhileLoop {
        children: Vec<ControlFlowNode>,
    },
    TryCatch {
        catch_count: usize,
        has_finally: bool,
        children: Vec<ControlFlowNode>,
    },
    Return,
    Break,
    Continue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Char(char),
    Hex(i64),
    Long(i64),
}

impl Literal {
    /// Numeric value for comparison (ignoring literal form)
    pub fn numeric_value(&self) -> Option<f64> {
        match self {
            Literal::Integer(v) | Literal::Hex(v) | Literal::Long(v) => Some(*v as f64),
            Literal::Float(v) => Some(*v),
            _ => None,
        }
    }

    pub fn is_trivial(&self) -> bool {
        match self {
            Literal::Integer(v) => *v >= -1 && *v <= 1,
            Literal::Float(v) => *v == 0.0 || *v == 1.0 || *v == -1.0,
            Literal::Boolean(_) => true,
            _ => false,
        }
    }
}

impl ControlFlowNode {
    pub fn children(&self) -> &[ControlFlowNode] {
        match self {
            ControlFlowNode::If { children, .. }
            | ControlFlowNode::Switch { children, .. }
            | ControlFlowNode::ForLoop { children, .. }
            | ControlFlowNode::WhileLoop { children, .. }
            | ControlFlowNode::TryCatch { children, .. } => children,
            ControlFlowNode::Return | ControlFlowNode::Break | ControlFlowNode::Continue => &[],
        }
    }

    pub fn kind_name(&self) -> &'static str {
        match self {
            ControlFlowNode::If { .. } => "if",
            ControlFlowNode::Switch { .. } => "switch/match",
            ControlFlowNode::ForLoop { .. } => "for",
            ControlFlowNode::WhileLoop { .. } => "while/loop",
            ControlFlowNode::TryCatch { .. } => "try-catch",
            ControlFlowNode::Return => "return",
            ControlFlowNode::Break => "break",
            ControlFlowNode::Continue => "continue",
        }
    }
}
