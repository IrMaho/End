use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    Void,
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Str,
    Custom(String),
    Pointer(Box<Type>),
    Slice(Box<Type>),
    Array(Box<Type>, usize),
    Simd(Box<Type>, usize), // e.g. f32x4, i32x8
    Tuple(Vec<Type>),
    Generic(String, Vec<Type>),
    Result(Box<Type>, Option<Box<Type>>), // Result<T, E> or !T
    Region(String),                       // Region reference
    Box(Box<Type>),                       // Heap Box<T> (Tier 2)
    Rc(Box<Type>),                        // Reference Counted Rc<T> (Tier 3)
    Arc(Box<Type>),                       // Atomic Ref Counted Arc<T> (Tier 3)
    Channel(Box<Type>),                   // MPSC Channel<T>
    Allocator,
    Operation(Option<Box<Type>>, Option<Box<Type>>), // Operation<TIn, TOut>
    Event(String),                                   // Event type
    OperationResult,                                 // Rich OperationResult
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Void => write!(f, "void"),
            Type::Bool => write!(f, "bool"),
            Type::I8 => write!(f, "i8"),
            Type::I16 => write!(f, "i16"),
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            Type::U8 => write!(f, "u8"),
            Type::U16 => write!(f, "u16"),
            Type::U32 => write!(f, "u32"),
            Type::U64 => write!(f, "u64"),
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),
            Type::Str => write!(f, "str"),
            Type::Custom(name) => write!(f, "{}", name),
            Type::Pointer(inner) => write!(f, "*{}", inner),
            Type::Slice(inner) => write!(f, "[]{}", inner),
            Type::Array(inner, size) => write!(f, "[{}]{}", size, inner),
            Type::Simd(inner, lanes) => write!(f, "{}x{}", inner, lanes),
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            Type::Generic(name, params) => {
                write!(f, "{}<", name)?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ">")
            }
            Type::Result(inner, None) => write!(f, "!{}", inner),
            Type::Result(inner, Some(err)) => write!(f, "Result<{}, {}>", inner, err),
            Type::Region(name) => write!(f, "region<{}>", name),
            Type::Box(inner) => write!(f, "Box<{}>", inner),
            Type::Rc(inner) => write!(f, "Rc<{}>", inner),
            Type::Arc(inner) => write!(f, "Arc<{}>", inner),
            Type::Channel(inner) => write!(f, "Channel<{}>", inner),
            Type::Allocator => write!(f, "Allocator"),
            Type::Operation(tin, tout) => {
                write!(f, "Operation")?;
                if tin.is_some() || tout.is_some() {
                    write!(f, "<{}, {}>", tin.as_ref().map(|t| t.to_string()).unwrap_or_else(|| "void".to_string()),
                                          tout.as_ref().map(|t| t.to_string()).unwrap_or_else(|| "void".to_string()))?;
                }
                Ok(())
            }
            Type::Event(name) => write!(f, "Event<{}>", name),
            Type::OperationResult => write!(f, "OperationResult"),
        }
    }
}

