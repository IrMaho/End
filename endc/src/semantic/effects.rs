use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Effect {
    Alloc(Option<String>), // Allocation with optional allocator name (e.g. Arena, Stack, Heap)
    IO,
    Network,
    Database,
    Filesystem,
    Panic,
    ForeignCall(String),   // FFI Call
}

impl std::fmt::Display for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Effect::Alloc(None) => write!(f, "alloc"),
            Effect::Alloc(Some(a)) => write!(f, "alloc({})", a),
            Effect::IO => write!(f, "io"),
            Effect::Network => write!(f, "network"),
            Effect::Database => write!(f, "database"),
            Effect::Filesystem => write!(f, "filesystem"),
            Effect::Panic => write!(f, "panic"),
            Effect::ForeignCall(lang) => write!(f, "ffi({})", lang),
        }
    }
}
