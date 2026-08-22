use crate::ast::Type;

pub trait TypeMapper {
    fn map_void(&self) -> String;
    fn map_bool(&self) -> String;
    fn map_int(&self, signed: bool, bits: u8) -> String;
    fn map_float(&self, bits: u8) -> String;
    fn map_str(&self) -> String;
    fn map_pointer(&self, inner: &Type) -> String;
    fn map_array(&self, inner: &Type, size: usize) -> String;
    fn map_slice(&self, inner: &Type) -> String;
    fn map_custom(&self, name: &str) -> String;
    fn map_simd(&self, inner: &Type, lanes: usize) -> String;
    fn map_tuple(&self, types: &[Type]) -> String;
    fn map_channel(&self, inner: &Type) -> String;
    fn map_region(&self, name: &str) -> String;
    fn map_operation(&self) -> String;
    fn map_type(&self, ty: &Type) -> String;
}

pub struct CTypeMapper;

impl TypeMapper for CTypeMapper {
    fn map_void(&self) -> String { "void".to_string() }
    fn map_bool(&self) -> String { "bool".to_string() }
    fn map_int(&self, signed: bool, bits: u8) -> String {
        if signed {
            format!("int{}_t", bits)
        } else {
            format!("uint{}_t", bits)
        }
    }
    fn map_float(&self, bits: u8) -> String {
        if bits <= 32 { "float".to_string() } else { "double".to_string() }
    }
    fn map_str(&self) -> String { "const char*".to_string() }
    fn map_pointer(&self, inner: &Type) -> String {
        format!("{}*", self.map_type(inner))
    }
    fn map_array(&self, inner: &Type, size: usize) -> String {
        format!("{}[{}]", self.map_type(inner), size)
    }
    fn map_slice(&self, inner: &Type) -> String {
        format!("EndSlice_{}", self.map_type(inner).replace("*", "_ptr").replace(" ", "_"))
    }
    fn map_custom(&self, name: &str) -> String {
        format!("struct {}", name)
    }
    fn map_simd(&self, inner: &Type, lanes: usize) -> String {
        format!("EndSimd_{}_{}", self.map_type(inner).replace(" ", "_"), lanes)
    }
    fn map_tuple(&self, types: &[Type]) -> String {
        let mapped: Vec<String> = types.iter().map(|t| self.map_type(t)).collect();
        format!("EndTuple_{}", mapped.join("_"))
    }
    fn map_channel(&self, inner: &Type) -> String {
        format!("EndChannel_{}*", self.map_type(inner).replace(" ", "_"))
    }
    fn map_region(&self, _name: &str) -> String { "EndArena*".to_string() }
    fn map_operation(&self) -> String { "EndOperation*".to_string() }
    fn map_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => self.map_void(),
            Type::Bool => self.map_bool(),
            Type::I8 => self.map_int(true, 8),
            Type::I16 => self.map_int(true, 16),
            Type::I32 => self.map_int(true, 32),
            Type::I64 => self.map_int(true, 64),
            Type::U8 => self.map_int(false, 8),
            Type::U16 => self.map_int(false, 16),
            Type::U32 => self.map_int(false, 32),
            Type::U64 => self.map_int(false, 64),
            Type::F32 => self.map_float(32),
            Type::F64 => self.map_float(64),
            Type::Str => self.map_str(),
            Type::Pointer(inner) => self.map_pointer(inner),
            Type::Array(inner, sz) => self.map_array(inner, *sz),
            Type::Slice(inner) => self.map_slice(inner),
            Type::Custom(name) => self.map_custom(name),
            Type::Simd(inner, lanes) => self.map_simd(inner, *lanes),
            Type::Tuple(types) => self.map_tuple(types),
            Type::Channel(inner) => self.map_channel(inner),
            Type::Region(name) => self.map_region(name),
            Type::Operation(_, _) => self.map_operation(),
            _ => "void*".to_string(),
        }
    }
}

pub struct LlvmTypeMapper;

impl TypeMapper for LlvmTypeMapper {
    fn map_void(&self) -> String { "void".to_string() }
    fn map_bool(&self) -> String { "i1".to_string() }
    fn map_int(&self, _signed: bool, bits: u8) -> String {
        format!("i{}", bits)
    }
    fn map_float(&self, bits: u8) -> String {
        if bits <= 32 { "float".to_string() } else { "double".to_string() }
    }
    fn map_str(&self) -> String { "i8*".to_string() }
    fn map_pointer(&self, inner: &Type) -> String {
        format!("{}*", self.map_type(inner))
    }
    fn map_array(&self, inner: &Type, size: usize) -> String {
        format!("[{} x {}]", size, self.map_type(inner))
    }
    fn map_slice(&self, inner: &Type) -> String {
        format!("{{ {}*, i64 }}", self.map_type(inner))
    }
    fn map_custom(&self, name: &str) -> String {
        format!("%struct.{}*", name)
    }
    fn map_simd(&self, inner: &Type, lanes: usize) -> String {
        format!("<{} x {}>", lanes, self.map_type(inner))
    }
    fn map_tuple(&self, types: &[Type]) -> String {
        let mapped: Vec<String> = types.iter().map(|t| self.map_type(t)).collect();
        format!("{{ {} }}", mapped.join(", "))
    }
    fn map_channel(&self, _inner: &Type) -> String {
        "%struct.EndChannel*".to_string()
    }
    fn map_region(&self, _name: &str) -> String { "i8*".to_string() }
    fn map_operation(&self) -> String { "i8*".to_string() }
    fn map_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => self.map_void(),
            Type::Bool => self.map_bool(),
            Type::I8 | Type::U8 => self.map_int(true, 8),
            Type::I16 | Type::U16 => self.map_int(true, 16),
            Type::I32 | Type::U32 => self.map_int(true, 32),
            Type::I64 | Type::U64 => self.map_int(true, 64),
            Type::F32 => self.map_float(32),
            Type::F64 => self.map_float(64),
            Type::Str => self.map_str(),
            Type::Pointer(inner) => self.map_pointer(inner),
            Type::Array(inner, sz) => self.map_array(inner, *sz),
            Type::Slice(inner) => self.map_slice(inner),
            Type::Custom(name) => self.map_custom(name),
            Type::Simd(inner, lanes) => self.map_simd(inner, *lanes),
            Type::Tuple(types) => self.map_tuple(types),
            Type::Channel(inner) => self.map_channel(inner),
            Type::Region(name) => self.map_region(name),
            Type::Operation(_, _) => self.map_operation(),
            _ => "i8*".to_string(),
        }
    }
}

pub struct CraneliftTypeMapper;

impl TypeMapper for CraneliftTypeMapper {
    fn map_void(&self) -> String { "types::INVALID".to_string() }
    fn map_bool(&self) -> String { "types::I8".to_string() }
    fn map_int(&self, _signed: bool, bits: u8) -> String {
        match bits {
            8 => "types::I8".to_string(),
            16 => "types::I16".to_string(),
            32 => "types::I32".to_string(),
            _ => "types::I64".to_string(),
        }
    }
    fn map_float(&self, bits: u8) -> String {
        if bits <= 32 { "types::F32".to_string() } else { "types::F64".to_string() }
    }
    fn map_str(&self) -> String { "types::I64".to_string() }
    fn map_pointer(&self, _inner: &Type) -> String { "types::I64".to_string() }
    fn map_array(&self, _inner: &Type, _size: usize) -> String { "types::I64".to_string() }
    fn map_slice(&self, _inner: &Type) -> String { "types::I64".to_string() }
    fn map_custom(&self, _name: &str) -> String { "types::I64".to_string() }
    fn map_simd(&self, inner: &Type, lanes: usize) -> String {
        match (inner, lanes) {
            (Type::F32, 4) => "types::F32X4".to_string(),
            (Type::F32, 8) => "types::F32X8".to_string(),
            (Type::I32, 4) => "types::I32X4".to_string(),
            (Type::I32, 8) => "types::I32X8".to_string(),
            _ => "types::I64X2".to_string(),
        }
    }
    fn map_tuple(&self, _types: &[Type]) -> String { "types::I64".to_string() }
    fn map_channel(&self, _inner: &Type) -> String { "types::I64".to_string() }
    fn map_region(&self, _name: &str) -> String { "types::I64".to_string() }
    fn map_operation(&self) -> String { "types::I64".to_string() }
    fn map_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => self.map_void(),
            Type::Bool => self.map_bool(),
            Type::I8 | Type::U8 => self.map_int(true, 8),
            Type::I16 | Type::U16 => self.map_int(true, 16),
            Type::I32 | Type::U32 => self.map_int(true, 32),
            Type::I64 | Type::U64 => self.map_int(true, 64),
            Type::F32 => self.map_float(32),
            Type::F64 => self.map_float(64),
            Type::Str => self.map_str(),
            Type::Pointer(_) => "types::I64".to_string(),
            Type::Simd(inner, lanes) => self.map_simd(inner, *lanes),
            _ => "types::I64".to_string(),
        }
    }
}
