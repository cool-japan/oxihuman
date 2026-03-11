// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! NumPy NPZ archive stub export.

/// NumPy dtype string tag.
#[derive(Debug, Clone, PartialEq)]
pub enum NpyDtype {
    Float32,
    Float64,
    Int32,
    Int64,
    Uint8,
    Bool,
}

impl NpyDtype {
    /// Returns the NumPy dtype descriptor string.
    pub fn descriptor(&self) -> &'static str {
        match self {
            NpyDtype::Float32 => "<f4",
            NpyDtype::Float64 => "<f8",
            NpyDtype::Int32 => "<i4",
            NpyDtype::Int64 => "<i8",
            NpyDtype::Uint8 => "|u1",
            NpyDtype::Bool => "|b1",
        }
    }

    /// Byte size per element.
    pub fn itemsize(&self) -> usize {
        match self {
            NpyDtype::Float32 | NpyDtype::Int32 => 4,
            NpyDtype::Float64 | NpyDtype::Int64 => 8,
            NpyDtype::Uint8 | NpyDtype::Bool => 1,
        }
    }
}

/// A single array entry in the NPZ archive.
#[derive(Debug, Clone)]
pub struct NpzArray {
    pub key: String,
    pub dtype: NpyDtype,
    pub shape: Vec<usize>,
    pub fortran_order: bool,
}

impl NpzArray {
    /// Returns total element count.
    pub fn numel(&self) -> usize {
        self.shape.iter().product::<usize>().max(1)
    }

    /// Returns byte size of this array.
    pub fn byte_size(&self) -> usize {
        self.numel() * self.dtype.itemsize()
    }
}

/// NPZ archive stub.
#[derive(Debug, Clone, Default)]
pub struct NpzExport {
    pub arrays: Vec<NpzArray>,
    pub compressed: bool,
}

/// Creates a new NPZ export stub.
pub fn new_npz_export(compressed: bool) -> NpzExport {
    NpzExport {
        compressed,
        ..Default::default()
    }
}

/// Adds an array to the archive.
pub fn add_npz_array(export: &mut NpzExport, array: NpzArray) {
    export.arrays.push(array);
}

/// Finds an array by key.
pub fn find_npz_array<'a>(export: &'a NpzExport, key: &str) -> Option<&'a NpzArray> {
    export.arrays.iter().find(|a| a.key == key)
}

/// Returns total array count.
pub fn npz_array_count(export: &NpzExport) -> usize {
    export.arrays.len()
}

/// Returns total estimated uncompressed data size in bytes.
pub fn npz_data_size(export: &NpzExport) -> usize {
    export
        .arrays
        .iter()
        .map(|a| a.byte_size() + 128 /* .npy header */)
        .sum()
}

/// Validates the export (at least one array).
pub fn validate_npz(export: &NpzExport) -> bool {
    !export.arrays.is_empty()
}

/// Returns a JSON-like summary.
pub fn npz_summary_json(export: &NpzExport) -> String {
    format!(
        "{{\"arrays\":{},\"compressed\":{},\"total_bytes\":{}}}",
        export.arrays.len(),
        export.compressed,
        npz_data_size(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_export() -> NpzExport {
        let mut e = new_npz_export(false);
        add_npz_array(
            &mut e,
            NpzArray {
                key: "weights".into(),
                dtype: NpyDtype::Float32,
                shape: vec![512, 256],
                fortran_order: false,
            },
        );
        add_npz_array(
            &mut e,
            NpzArray {
                key: "bias".into(),
                dtype: NpyDtype::Float32,
                shape: vec![256],
                fortran_order: false,
            },
        );
        e
    }

    #[test]
    fn array_count() {
        let e = sample_export();
        assert_eq!(npz_array_count(&e), 2);
    }

    #[test]
    fn validate_complete() {
        let e = sample_export();
        assert!(validate_npz(&e));
    }

    #[test]
    fn validate_empty_false() {
        let e = new_npz_export(true);
        assert!(!validate_npz(&e));
    }

    #[test]
    fn find_array_found() {
        let e = sample_export();
        assert!(find_npz_array(&e, "weights").is_some());
    }

    #[test]
    fn find_array_missing() {
        let e = sample_export();
        assert!(find_npz_array(&e, "nonexistent").is_none());
    }

    #[test]
    fn byte_size_weights() {
        let arr = NpzArray {
            key: "w".into(),
            dtype: NpyDtype::Float32,
            shape: vec![4, 4],
            fortran_order: false,
        };
        assert_eq!(arr.byte_size(), 64);
    }

    #[test]
    fn dtype_descriptor_f32() {
        assert_eq!(NpyDtype::Float32.descriptor(), "<f4");
    }

    #[test]
    fn data_size_positive() {
        let e = sample_export();
        assert!(npz_data_size(&e) > 0);
    }

    #[test]
    fn summary_json_has_arrays() {
        let e = sample_export();
        let json = npz_summary_json(&e);
        assert!(json.contains("arrays"));
    }
}
