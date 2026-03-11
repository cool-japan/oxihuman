// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! SafeTensors format stub export.

/// SafeTensors element dtype tag.
#[derive(Debug, Clone, PartialEq)]
pub enum StDtype {
    F32,
    F16,
    Bf16,
    F64,
    I64,
    I32,
    I16,
    I8,
    Bool,
}

impl StDtype {
    /// Returns the byte width of the dtype element.
    pub fn byte_width(&self) -> usize {
        match self {
            StDtype::F64 | StDtype::I64 => 8,
            StDtype::F32 | StDtype::I32 => 4,
            StDtype::F16 | StDtype::Bf16 | StDtype::I16 => 2,
            StDtype::I8 | StDtype::Bool => 1,
        }
    }
}

/// A single tensor entry in the SafeTensors file.
#[derive(Debug, Clone)]
pub struct StTensorEntry {
    pub name: String,
    pub dtype: StDtype,
    pub shape: Vec<u64>,
    pub data_offsets: (u64, u64), /* (begin, end) */
}

impl StTensorEntry {
    /// Returns the element count for this tensor.
    pub fn element_count(&self) -> u64 {
        self.shape.iter().product::<u64>().max(1)
    }
}

/// SafeTensors file stub.
#[derive(Debug, Clone, Default)]
pub struct SafeTensorsExport {
    pub tensors: Vec<StTensorEntry>,
    pub metadata: Vec<(String, String)>,
}

/// Creates a new SafeTensors export stub.
pub fn new_safetensors_export() -> SafeTensorsExport {
    SafeTensorsExport::default()
}

/// Adds a tensor entry.
pub fn add_st_tensor(export: &mut SafeTensorsExport, entry: StTensorEntry) {
    export.tensors.push(entry);
}

/// Adds a metadata key-value pair.
pub fn add_st_metadata(export: &mut SafeTensorsExport, key: &str, value: &str) {
    export.metadata.push((key.to_string(), value.to_string()));
}

/// Finds a tensor entry by name.
pub fn find_st_tensor<'a>(export: &'a SafeTensorsExport, name: &str) -> Option<&'a StTensorEntry> {
    export.tensors.iter().find(|t| t.name == name)
}

/// Returns total tensor count.
pub fn st_tensor_count(export: &SafeTensorsExport) -> usize {
    export.tensors.len()
}

/// Validates the export (at least one tensor).
pub fn validate_safetensors(export: &SafeTensorsExport) -> bool {
    !export.tensors.is_empty()
}

/// Estimates the total serialised data size in bytes.
pub fn st_data_size_estimate(export: &SafeTensorsExport) -> usize {
    let data_bytes: usize = export
        .tensors
        .iter()
        .map(|t| t.element_count() as usize * t.dtype.byte_width())
        .sum();
    let header_bytes: usize = export
        .tensors
        .iter()
        .map(|t| t.name.len() + 64)
        .sum::<usize>()
        + export
            .metadata
            .iter()
            .map(|(k, v)| k.len() + v.len() + 4)
            .sum::<usize>()
        + 8; /* header length prefix */
    data_bytes + header_bytes
}

/// Returns a JSON-like header summary.
pub fn st_header_json(export: &SafeTensorsExport) -> String {
    format!(
        "{{\"tensors\":{},\"metadata_keys\":{}}}",
        export.tensors.len(),
        export.metadata.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_export() -> SafeTensorsExport {
        let mut e = new_safetensors_export();
        add_st_tensor(
            &mut e,
            StTensorEntry {
                name: "model.embed.weight".into(),
                dtype: StDtype::F16,
                shape: vec![32000, 4096],
                data_offsets: (0, 32000 * 4096 * 2),
            },
        );
        add_st_metadata(&mut e, "format", "pt");
        e
    }

    #[test]
    fn tensor_count() {
        let e = sample_export();
        assert_eq!(st_tensor_count(&e), 1);
    }

    #[test]
    fn validate_complete() {
        let e = sample_export();
        assert!(validate_safetensors(&e));
    }

    #[test]
    fn validate_empty_false() {
        let e = new_safetensors_export();
        assert!(!validate_safetensors(&e));
    }

    #[test]
    fn find_tensor_found() {
        let e = sample_export();
        assert!(find_st_tensor(&e, "model.embed.weight").is_some());
    }

    #[test]
    fn find_tensor_missing() {
        let e = sample_export();
        assert!(find_st_tensor(&e, "nonexistent").is_none());
    }

    #[test]
    fn dtype_byte_width_f32() {
        assert_eq!(StDtype::F32.byte_width(), 4);
    }

    #[test]
    fn dtype_byte_width_f16() {
        assert_eq!(StDtype::F16.byte_width(), 2);
    }

    #[test]
    fn size_estimate_positive() {
        let e = sample_export();
        assert!(st_data_size_estimate(&e) > 0);
    }

    #[test]
    fn header_json_has_tensors() {
        let e = sample_export();
        let json = st_header_json(&e);
        assert!(json.contains("tensors"));
    }
}
