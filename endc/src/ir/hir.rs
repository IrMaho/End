// ?? End High-Level Intermediate Representation (HIR)
// Typed AST with fully resolved types, scope bindings, and capability invariants

use crate::ast::Type;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HirType {
    Void,
    Bool,
    I8, I16, I32, I64,
    U8, U16, U32, U64,
    F32, F64,
    Str,
    Pointer(Box<HirType>),
    RegionPointer(Box<HirType>, String), // Pointer tied to a specific region lifetime
    Array(Box<HirType>, usize),
    Struct(String, Vec<(String, HirType)>),
    Enum(String, Vec<String>),
    Generic(String, Vec<HirType>),
    Custom(String),
}

impl std::fmt::Display for HirType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HirType::Void => write!(f, "void"),
            HirType::Bool => write!(f, "bool"),
            HirType::I32 => write!(f, "i32"),
            HirType::I64 => write!(f, "i64"),
            HirType::U64 => write!(f, "u64"),
            HirType::F32 => write!(f, "f32"),
            HirType::F64 => write!(f, "f64"),
            HirType::Str => write!(f, "str"),
            HirType::Pointer(t) => write!(f, "*{}", t),
            HirType::RegionPointer(t, r) => write!(f, "*{} @'{}", t, r),
            HirType::Array(t, sz) => write!(f, "[{}]{}", sz, t),
            HirType::Struct(name, _) => write!(f, "struct {}", name),
            HirType::Enum(name, _) => write!(f, "enum {}", name),
            HirType::Generic(name, args) => {
                let args_str = args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ");
                write!(f, "{}<{}>", name, args_str)
            }
            HirType::Custom(name) => write!(f, "{}", name),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HirModule {
    pub name: String,
    pub structs: Vec<HirStruct>,
    pub functions: Vec<HirFunction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HirStruct {
    pub name: String,
    pub fields: Vec<(String, HirType)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HirFunction {
    pub name: String,
    pub params: Vec<(String, HirType, bool)>, // (name, type, is_mut)
    pub return_type: HirType,
    pub body: Vec<HirStatement>,
    pub is_pure: bool,
    pub is_async: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HirStatement {
    VarDecl {
        name: String,
        ty: HirType,
        is_mut: bool,
        init: Option<HirExpression>,
        line: usize,
    },
    Assign {
        target: String,
        value: HirExpression,
        line: usize,
    },
    Return {
        val: Option<HirExpression>,
        line: usize,
    },
    Expression(HirExpression),
    If {
        cond: HirExpression,
        then_branch: Vec<HirStatement>,
        else_branch: Option<Vec<HirStatement>>,
        line: usize,
    },
    While {
        cond: HirExpression,
        body: Vec<HirStatement>,
        line: usize,
    },
    RegionEnter {
        name: String,
        line: usize,
    },
    RegionExit {
        name: String,
        line: usize,
    },
    Drop {
        var_name: String,
        line: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HirExpression {
    LitInt(i64, HirType),
    LitFloat(f64, HirType),
    LitStr(String),
    LitBool(bool),
    Var(String, HirType),
    Binary {
        op: String,
        left: Box<HirExpression>,
        right: Box<HirExpression>,
        result_type: HirType,
    },
    Unary {
        op: String,
        expr: Box<HirExpression>,
        result_type: HirType,
    },
    Call {
        callee: String,
        args: Vec<HirExpression>,
        result_type: HirType,
    },
    FieldAccess {
        object: Box<HirExpression>,
        field: String,
        result_type: HirType,
    },
    Alloc {
        element_type: HirType,
        count: Box<HirExpression>,
        region_name: Option<String>,
        result_type: HirType,
    },
}

impl HirExpression {
    pub fn get_type(&self) -> HirType {
        match self {
            HirExpression::LitInt(_, ty) => ty.clone(),
            HirExpression::LitFloat(_, ty) => ty.clone(),
            HirExpression::LitStr(_) => HirType::Str,
            HirExpression::LitBool(_) => HirType::Bool,
            HirExpression::Var(_, ty) => ty.clone(),
            HirExpression::Binary { result_type, .. } => result_type.clone(),
            HirExpression::Unary { result_type, .. } => result_type.clone(),
            HirExpression::Call { result_type, .. } => result_type.clone(),
            HirExpression::FieldAccess { result_type, .. } => result_type.clone(),
            HirExpression::Alloc { result_type, .. } => result_type.clone(),
        }
    }
}
