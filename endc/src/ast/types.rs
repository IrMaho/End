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
    Unknown,                                         // Transient inference state (cannot reach codegen)
}

impl Type {
    pub fn is_unknown(&self) -> bool {
        matches!(self, Type::Unknown)
    }

    pub fn is_numeric(&self) -> bool {
        self.is_integer() || self.is_float()
    }

    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
        )
    }

    pub fn is_signed_integer(&self) -> bool {
        matches!(self, Type::I8 | Type::I16 | Type::I32 | Type::I64)
    }

    pub fn is_unsigned_integer(&self) -> bool {
        matches!(self, Type::U8 | Type::U16 | Type::U32 | Type::U64)
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Type::F32 | Type::F64)
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Type::Bool)
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Type::Str)
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self, Type::Pointer(_))
    }

    /// Check whether `self` is type-compatible with `expected`.
    pub fn is_compatible_with(&self, expected: &Type) -> bool {
        if self == expected {
            return true;
        }
        if self.is_unknown() || expected.is_unknown() {
            return true; // Unknown represents in-progress inference
        }

        // Implicit numeric conversions for integers
        if self.is_integer() && expected.is_integer() {
            return true;
        }

        // Float conversions
        if self.is_float() && expected.is_float() {
            return true;
        }

        // Generic and Custom alias compatibility
        match (self, expected) {
            (Type::Pointer(a), Type::Pointer(b)) => {
                if **a == Type::Void || **b == Type::Void {
                    true
                } else {
                    a.is_compatible_with(b)
                }
            }
            (Type::Slice(a), Type::Slice(b)) => a.is_compatible_with(b),
            (Type::Slice(a), Type::Array(b, _)) | (Type::Array(a, _), Type::Slice(b)) => {
                a.is_compatible_with(b)
            }
            (Type::Array(a, s1), Type::Array(b, s2)) => s1 == s2 && a.is_compatible_with(b),
            (Type::Custom(a), Type::Custom(b)) => a == b,
            (Type::Tuple(a), Type::Tuple(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.is_compatible_with(y))
            }
            (Type::Custom(s), Type::Tuple(tys)) | (Type::Tuple(tys), Type::Custom(s))
                if s == &format!("tuple_{}", tys.len()) =>
            {
                true
            }
            (val, Type::Generic(name, params)) if !params.is_empty() => {
                if name == "secret"
                    || name == "tainted"
                    || name == "verified"
                    || name == "authority"
                    || name == "Box"
                    || name == "Rc"
                    || name == "Arc"
                {
                    val.is_compatible_with(&params[0])
                } else {
                    false
                }
            }
            (Type::Generic(name, params), expected) if !params.is_empty() => {
                if name == "secret"
                    || name == "tainted"
                    || name == "verified"
                    || name == "authority"
                    || name == "Box"
                    || name == "Rc"
                    || name == "Arc"
                {
                    params[0].is_compatible_with(expected)
                } else {
                    false
                }
            }
            (val, Type::Result(inner, _)) => val.is_compatible_with(inner),
            (Type::Result(inner, _), expected) => inner.is_compatible_with(expected),
            _ => false,
        }
    }
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
            Type::Unknown => write!(f, "unknown"),
        }
    }
}

