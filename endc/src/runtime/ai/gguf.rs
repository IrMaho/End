use super::error::AiError;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::Path;

pub const GGUF_MAGIC: u32 = 0x46554747; // "GGUF" in ASCII LE

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GgufTensorMeta {
    pub name: String,
    pub dimensions: Vec<usize>,
    pub tensor_type: String,
    pub offset: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GgufMetadata {
    pub version: u32,
    pub tensor_count: u64,
    pub kv_count: u64,
    pub architecture: String,
    pub model_name: Option<String>,
    pub context_length: Option<u64>,
    pub embedding_length: Option<u64>,
    pub block_count: Option<u64>,
    pub feed_forward_length: Option<u64>,
    pub head_count: Option<u64>,
    pub head_count_kv: Option<u64>,
    pub rope_dimension_count: Option<u64>,
    pub rope_freq_base: Option<f32>,
    pub expert_count: Option<u64>,
    pub expert_used_count: Option<u64>,
    pub tensor_info: Vec<GgufTensorMeta>,
    pub raw_metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GgufMetadataValueType {
    Uint8 = 0,
    Int8 = 1,
    Uint16 = 2,
    Int16 = 3,
    Uint32 = 4,
    Int32 = 5,
    Float32 = 6,
    Bool = 7,
    String = 8,
    Array = 9,
    Uint64 = 10,
    Int64 = 11,
    Float64 = 12,
}

impl GgufMetadataValueType {
    pub fn from_u32(val: u32) -> Result<Self, AiError> {
        match val {
            0 => Ok(Self::Uint8),
            1 => Ok(Self::Int8),
            2 => Ok(Self::Uint16),
            3 => Ok(Self::Int16),
            4 => Ok(Self::Uint32),
            5 => Ok(Self::Int32),
            6 => Ok(Self::Float32),
            7 => Ok(Self::Bool),
            8 => Ok(Self::String),
            9 => Ok(Self::Array),
            10 => Ok(Self::Uint64),
            11 => Ok(Self::Int64),
            12 => Ok(Self::Float64),
            _ => Err(AiError::InvalidGguf(format!(
                "Unknown GGUF metadata value type: {}",
                val
            ))),
        }
    }
}

pub fn parse_gguf_metadata<R: Read + Seek>(reader: &mut R) -> Result<GgufMetadata, AiError> {
    let magic = reader
        .read_u32::<LittleEndian>()
        .map_err(|e| AiError::InvalidGguf(format!("Failed to read magic bytes: {}", e)))?;

    if magic != GGUF_MAGIC {
        return Err(AiError::InvalidGguf(format!(
            "Invalid GGUF magic header 0x{:08X} (expected 0x{:08X} 'GGUF')",
            magic, GGUF_MAGIC
        )));
    }

    let version = reader
        .read_u32::<LittleEndian>()
        .map_err(|e| AiError::InvalidGguf(format!("Failed to read version: {}", e)))?;

    if version != 1 && version != 2 && version != 3 {
        return Err(AiError::InvalidGguf(format!(
            "Unsupported GGUF version {} (supported: 1, 2, 3)",
            version
        )));
    }

    let tensor_count = if version == 1 {
        reader
            .read_u32::<LittleEndian>()
            .map_err(|e| AiError::InvalidGguf(format!("Failed to read tensor count: {}", e)))?
            as u64
    } else {
        reader
            .read_u64::<LittleEndian>()
            .map_err(|e| AiError::InvalidGguf(format!("Failed to read tensor count: {}", e)))?
    };

    let kv_count = if version == 1 {
        reader
            .read_u32::<LittleEndian>()
            .map_err(|e| AiError::InvalidGguf(format!("Failed to read metadata KV count: {}", e)))?
            as u64
    } else {
        reader
            .read_u64::<LittleEndian>()
            .map_err(|e| AiError::InvalidGguf(format!("Failed to read metadata KV count: {}", e)))?
    };

    let mut raw_metadata: HashMap<String, String> = HashMap::new();

    for _ in 0..kv_count {
        let key = read_gguf_string(reader, version)?;
        let val_type_u32 = reader
            .read_u32::<LittleEndian>()
            .map_err(|e| AiError::InvalidGguf(format!("Failed to read value type for key '{}': {}", key, e)))?;
        let val_type = GgufMetadataValueType::from_u32(val_type_u32)?;
        let val_str = read_gguf_value(reader, val_type, version)?;
        raw_metadata.insert(key, val_str);
    }

    // Identify architecture
    let arch = raw_metadata
        .get("general.architecture")
        .cloned()
        .unwrap_or_else(|| "unknown".to_string());

    let model_name = raw_metadata.get("general.name").cloned();

    let context_length = raw_metadata
        .get(&format!("{}.context_length", arch))
        .and_then(|s| s.parse::<u64>().ok());

    let embedding_length = raw_metadata
        .get(&format!("{}.embedding_length", arch))
        .and_then(|s| s.parse::<u64>().ok());

    let block_count = raw_metadata
        .get(&format!("{}.block_count", arch))
        .and_then(|s| s.parse::<u64>().ok());

    let feed_forward_length = raw_metadata
        .get(&format!("{}.feed_forward_length", arch))
        .and_then(|s| s.parse::<u64>().ok());

    let head_count = raw_metadata
        .get(&format!("{}.attention.head_count", arch))
        .and_then(|s| s.parse::<u64>().ok());

    let head_count_kv = raw_metadata
        .get(&format!("{}.attention.head_count_kv", arch))
        .and_then(|s| s.parse::<u64>().ok());

    let rope_dimension_count = raw_metadata
        .get(&format!("{}.rope.dimension_count", arch))
        .and_then(|s| s.parse::<u64>().ok());

    let rope_freq_base = raw_metadata
        .get(&format!("{}.rope.freq_base", arch))
        .and_then(|s| s.parse::<f32>().ok());

    let expert_count = raw_metadata
        .get(&format!("{}.expert_count", arch))
        .and_then(|s| s.parse::<u64>().ok());

    let expert_used_count = raw_metadata
        .get(&format!("{}.expert_used_count", arch))
        .and_then(|s| s.parse::<u64>().ok());

    // Read Tensor Info Table
    let mut tensor_info = Vec::with_capacity(tensor_count as usize);
    for _ in 0..tensor_count {
        let name = read_gguf_string(reader, version)?;
        let n_dims = reader
            .read_u32::<LittleEndian>()
            .map_err(|e| AiError::InvalidGguf(format!("Failed to read n_dims for tensor '{}': {}", name, e)))?;

        let mut dimensions = Vec::with_capacity(n_dims as usize);
        for _ in 0..n_dims {
            let dim = if version == 1 {
                reader
                    .read_u32::<LittleEndian>()
                    .map_err(|e| AiError::InvalidGguf(format!("Failed to read dim: {}", e)))? as usize
            } else {
                reader
                    .read_u64::<LittleEndian>()
                    .map_err(|e| AiError::InvalidGguf(format!("Failed to read dim: {}", e)))? as usize
            };
            dimensions.push(dim);
        }

        let type_u32 = reader
            .read_u32::<LittleEndian>()
            .map_err(|e| AiError::InvalidGguf(format!("Failed to read type for tensor '{}': {}", name, e)))?;

        let offset = reader
            .read_u64::<LittleEndian>()
            .map_err(|e| AiError::InvalidGguf(format!("Failed to read offset for tensor '{}': {}", name, e)))?;

        let type_name = match type_u32 {
            0 => "F32",
            1 => "F16",
            2 => "Q4_0",
            3 => "Q4_1",
            6 => "Q5_0",
            7 => "Q5_1",
            8 => "Q8_0",
            9 => "Q8_1",
            10 => "Q2_K",
            11 => "Q3_K",
            12 => "Q4_K",
            13 => "Q5_K",
            14 => "Q6_K",
            15 => "Q8_K",
            16 => "IQ2_XXS",
            17 => "IQ2_XS",
            18 => "IQ3_XXS",
            19 => "IQ1_S",
            20 => "IQ4_NL",
            21 => "IQ3_S",
            22 => "IQ2_S",
            23 => "IQ4_XS",
            24 => "I8",
            25 => "I16",
            26 => "I32",
            27 => "I64",
            28 => "F64",
            29 => "IQ1_M",
            30 => "BF16",
            _ => "UNKNOWN",
        }
        .to_string();

        tensor_info.push(GgufTensorMeta {
            name,
            dimensions,
            tensor_type: type_name,
            offset,
        });
    }

    Ok(GgufMetadata {
        version,
        tensor_count,
        kv_count,
        architecture: arch,
        model_name,
        context_length,
        embedding_length,
        block_count,
        feed_forward_length,
        head_count,
        head_count_kv,
        rope_dimension_count,
        rope_freq_base,
        expert_count,
        expert_used_count,
        tensor_info,
        raw_metadata,
    })
}

pub fn validate_gguf_file(path: &Path) -> Result<GgufMetadata, AiError> {
    if !path.exists() {
        return Err(AiError::ModelNotFound(path.display().to_string()));
    }
    let mut file = File::open(path)
        .map_err(|e| AiError::IoError(format!("Failed to open '{}': {}", path.display(), e)))?;
    parse_gguf_metadata(&mut file)
}

fn read_gguf_string<R: Read>(reader: &mut R, version: u32) -> Result<String, AiError> {
    let len = if version == 1 {
        reader
            .read_u32::<LittleEndian>()
            .map_err(|e| AiError::InvalidGguf(format!("Failed to read string length: {}", e)))? as usize
    } else {
        reader
            .read_u64::<LittleEndian>()
            .map_err(|e| AiError::InvalidGguf(format!("Failed to read string length: {}", e)))? as usize
    };

    if len > 100 * 1024 * 1024 {
        return Err(AiError::InvalidGguf(format!(
            "Unreasonable string length {} in GGUF metadata",
            len
        )));
    }

    let mut buf = vec![0u8; len];
    reader
        .read_exact(&mut buf)
        .map_err(|e| AiError::InvalidGguf(format!("Truncated string bytes: {}", e)))?;

    String::from_utf8(buf).map_err(|e| AiError::InvalidGguf(format!("Invalid UTF-8 in GGUF: {}", e)))
}

fn read_gguf_value<R: Read + Seek>(
    reader: &mut R,
    val_type: GgufMetadataValueType,
    version: u32,
) -> Result<String, AiError> {
    match val_type {
        GgufMetadataValueType::Uint8 => {
            let v = reader
                .read_u8()
                .map_err(|e| AiError::InvalidGguf(format!("Truncated uint8: {}", e)))?;
            Ok(v.to_string())
        }
        GgufMetadataValueType::Int8 => {
            let v = reader
                .read_i8()
                .map_err(|e| AiError::InvalidGguf(format!("Truncated int8: {}", e)))?;
            Ok(v.to_string())
        }
        GgufMetadataValueType::Uint16 => {
            let v = reader
                .read_u16::<LittleEndian>()
                .map_err(|e| AiError::InvalidGguf(format!("Truncated uint16: {}", e)))?;
            Ok(v.to_string())
        }
        GgufMetadataValueType::Int16 => {
            let v = reader
                .read_i16::<LittleEndian>()
                .map_err(|e| AiError::InvalidGguf(format!("Truncated int16: {}", e)))?;
            Ok(v.to_string())
        }
        GgufMetadataValueType::Uint32 => {
            let v = reader
                .read_u32::<LittleEndian>()
                .map_err(|e| AiError::InvalidGguf(format!("Truncated uint32: {}", e)))?;
            Ok(v.to_string())
        }
        GgufMetadataValueType::Int32 => {
            let v = reader
                .read_i32::<LittleEndian>()
                .map_err(|e| AiError::InvalidGguf(format!("Truncated int32: {}", e)))?;
            Ok(v.to_string())
        }
        GgufMetadataValueType::Float32 => {
            let v = reader
                .read_f32::<LittleEndian>()
                .map_err(|e| AiError::InvalidGguf(format!("Truncated float32: {}", e)))?;
            Ok(v.to_string())
        }
        GgufMetadataValueType::Bool => {
            let v = reader
                .read_u8()
                .map_err(|e| AiError::InvalidGguf(format!("Truncated bool: {}", e)))?;
            Ok((v != 0).to_string())
        }
        GgufMetadataValueType::String => read_gguf_string(reader, version),
        GgufMetadataValueType::Uint64 => {
            let v = reader
                .read_u64::<LittleEndian>()
                .map_err(|e| AiError::InvalidGguf(format!("Truncated uint64: {}", e)))?;
            Ok(v.to_string())
        }
        GgufMetadataValueType::Int64 => {
            let v = reader
                .read_i64::<LittleEndian>()
                .map_err(|e| AiError::InvalidGguf(format!("Truncated int64: {}", e)))?;
            Ok(v.to_string())
        }
        GgufMetadataValueType::Float64 => {
            let v = reader
                .read_f64::<LittleEndian>()
                .map_err(|e| AiError::InvalidGguf(format!("Truncated float64: {}", e)))?;
            Ok(v.to_string())
        }
        GgufMetadataValueType::Array => {
            let item_type_u32 = reader
                .read_u32::<LittleEndian>()
                .map_err(|e| AiError::InvalidGguf(format!("Truncated array item type: {}", e)))?;
            let item_type = GgufMetadataValueType::from_u32(item_type_u32)?;

            let arr_len = if version == 1 {
                reader
                    .read_u32::<LittleEndian>()
                    .map_err(|e| AiError::InvalidGguf(format!("Truncated array length: {}", e)))? as usize
            } else {
                reader
                    .read_u64::<LittleEndian>()
                    .map_err(|e| AiError::InvalidGguf(format!("Truncated array length: {}", e)))? as usize
            };

            // For very large arrays (e.g. tokenizer token list), record summary or parse items
            if arr_len > 100_000 {
                // Skip payload if needed or read elements
                for _ in 0..arr_len {
                    let _ = read_gguf_value(reader, item_type, version)?;
                }
                Ok(format!("[Array of {} elements]", arr_len))
            } else {
                let mut elements = Vec::with_capacity(arr_len.min(64));
                for i in 0..arr_len {
                    let val = read_gguf_value(reader, item_type, version)?;
                    if i < 64 {
                        elements.push(val);
                    }
                }
                if arr_len > 64 {
                    Ok(format!("[{}, ... (total {})]", elements.join(", "), arr_len))
                } else {
                    Ok(format!("[{}]", elements.join(", ")))
                }
            }
        }
    }
}
