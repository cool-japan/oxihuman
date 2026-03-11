// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! GGUF (llama.cpp) model file stub export.

/// GGUF tensor quantisation type.
#[derive(Debug, Clone, PartialEq)]
pub enum GgufQuantType {
    F32,
    F16,
    Q4_0,
    Q4_1,
    Q5_0,
    Q5_1,
    Q8_0,
}

/// GGUF tensor stub.
#[derive(Debug, Clone)]
pub struct GgufTensor {
    pub name: String,
    pub dims: Vec<u64>,
    pub quant: GgufQuantType,
    pub offset: u64,
}

/// GGUF model metadata key-value pair.
#[derive(Debug, Clone)]
pub struct GgufMetadata {
    pub key: String,
    pub value: String,
}

/// GGUF model file stub.
#[derive(Debug, Clone, Default)]
pub struct GgufExport {
    pub magic: u32,
    pub version: u32,
    pub tensors: Vec<GgufTensor>,
    pub metadata: Vec<GgufMetadata>,
}

/// Creates a new GGUF export stub (magic = 0x46554747 = "GGUF").
pub fn new_gguf_export() -> GgufExport {
    GgufExport {
        magic: 0x4647_4755, /* "GGUF" little-endian */
        version: 3,
        ..Default::default()
    }
}

/// Adds a tensor to the GGUF model.
pub fn add_gguf_tensor(export: &mut GgufExport, tensor: GgufTensor) {
    export.tensors.push(tensor);
}

/// Adds a metadata key-value pair.
pub fn add_gguf_metadata(export: &mut GgufExport, key: &str, value: &str) {
    export.metadata.push(GgufMetadata {
        key: key.to_string(),
        value: value.to_string(),
    });
}

/// Finds metadata by key.
pub fn find_gguf_metadata<'a>(export: &'a GgufExport, key: &str) -> Option<&'a GgufMetadata> {
    export.metadata.iter().find(|m| m.key == key)
}

/// Returns total tensor count.
pub fn gguf_tensor_count(export: &GgufExport) -> usize {
    export.tensors.len()
}

/// Validates the GGUF stub (has tensors and magic is correct).
pub fn validate_gguf(export: &GgufExport) -> bool {
    export.magic == 0x4647_4755 && !export.tensors.is_empty()
}

/// Estimates the file data size in bytes.
pub fn gguf_size_estimate(export: &GgufExport) -> usize {
    let tensor_bytes: usize = export
        .tensors
        .iter()
        .map(|t| t.name.len() + t.dims.len() * 8 + 64)
        .sum();
    let meta_bytes: usize = export
        .metadata
        .iter()
        .map(|m| m.key.len() + m.value.len() + 16)
        .sum();
    tensor_bytes + meta_bytes + 64 /* header */
}

/// Returns a JSON-like header summary.
pub fn gguf_header_json(export: &GgufExport) -> String {
    format!(
        "{{\"magic\":\"0x{:08X}\",\"version\":{},\"tensors\":{},\"metadata\":{}}}",
        export.magic,
        export.version,
        export.tensors.len(),
        export.metadata.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_export() -> GgufExport {
        let mut e = new_gguf_export();
        add_gguf_tensor(
            &mut e,
            GgufTensor {
                name: "token_embd.weight".into(),
                dims: vec![32000, 4096],
                quant: GgufQuantType::Q4_0,
                offset: 0,
            },
        );
        add_gguf_metadata(&mut e, "general.name", "Llama-2-7B");
        add_gguf_metadata(&mut e, "llama.context_length", "4096");
        e
    }

    #[test]
    fn magic_value_correct() {
        let e = new_gguf_export();
        assert_eq!(e.magic, 0x4647_4755);
    }

    #[test]
    fn version_is_three() {
        let e = new_gguf_export();
        assert_eq!(e.version, 3);
    }

    #[test]
    fn tensor_count() {
        let e = sample_export();
        assert_eq!(gguf_tensor_count(&e), 1);
    }

    #[test]
    fn validate_complete() {
        let e = sample_export();
        assert!(validate_gguf(&e));
    }

    #[test]
    fn validate_empty_false() {
        let e = new_gguf_export();
        assert!(!validate_gguf(&e));
    }

    #[test]
    fn find_metadata_found() {
        let e = sample_export();
        assert!(find_gguf_metadata(&e, "general.name").is_some());
    }

    #[test]
    fn find_metadata_missing() {
        let e = sample_export();
        assert!(find_gguf_metadata(&e, "nonexistent").is_none());
    }

    #[test]
    fn size_estimate_positive() {
        let e = sample_export();
        assert!(gguf_size_estimate(&e) > 0);
    }

    #[test]
    fn header_json_has_magic() {
        let e = sample_export();
        let json = gguf_header_json(&e);
        assert!(json.contains("magic"));
    }
}
