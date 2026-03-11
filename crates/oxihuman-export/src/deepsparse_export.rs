// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Neural Magic DeepSparse deployment stub export.

/// DeepSparse sparsity mode.
#[derive(Debug, Clone, PartialEq)]
pub enum SparsityMode {
    Dense,
    Unstructured,
    Structured,
    BlockSparse,
}

/// A sparse layer descriptor.
#[derive(Debug, Clone)]
pub struct SparseLayer {
    pub name: String,
    pub sparsity_ratio: f32,
    pub mode: SparsityMode,
}

/// DeepSparse deployment model stub.
#[derive(Debug, Clone, Default)]
pub struct DeepSparseExport {
    pub model_path: String,
    pub engine_version: String,
    pub num_cores: u32,
    pub batch_size: u32,
    pub layers: Vec<SparseLayer>,
}

/// Creates a new DeepSparse export stub.
pub fn new_deepsparse_export(model_path: &str, engine_version: &str) -> DeepSparseExport {
    DeepSparseExport {
        model_path: model_path.to_string(),
        engine_version: engine_version.to_string(),
        num_cores: 1,
        batch_size: 1,
        ..Default::default()
    }
}

/// Sets the number of CPU cores to use.
pub fn set_deepsparse_cores(export: &mut DeepSparseExport, cores: u32) {
    export.num_cores = cores;
}

/// Adds a sparse layer descriptor.
pub fn add_sparse_layer(export: &mut DeepSparseExport, layer: SparseLayer) {
    export.layers.push(layer);
}

/// Returns average sparsity across all layers (0.0 if no layers).
pub fn average_sparsity(export: &DeepSparseExport) -> f32 {
    if export.layers.is_empty() {
        return 0.0;
    }
    let total: f32 = export.layers.iter().map(|l| l.sparsity_ratio).sum();
    total / export.layers.len() as f32
}

/// Returns the count of sparse layers above a given ratio.
pub fn sparse_layer_count_above(export: &DeepSparseExport, threshold: f32) -> usize {
    export
        .layers
        .iter()
        .filter(|l| l.sparsity_ratio > threshold)
        .count()
}

/// Validates the export (non-empty model path).
pub fn validate_deepsparse(export: &DeepSparseExport) -> bool {
    !export.model_path.is_empty() && export.num_cores > 0
}

/// Estimates the serialised size in bytes.
pub fn deepsparse_size_estimate(export: &DeepSparseExport) -> usize {
    export
        .layers
        .iter()
        .map(|l| l.name.len() + 64)
        .sum::<usize>()
        + 256
}

/// Returns a JSON-like summary.
pub fn deepsparse_summary_json(export: &DeepSparseExport) -> String {
    format!(
        "{{\"model\":\"{}\",\"engine\":\"{}\",\"cores\":{},\"batch\":{},\"layers\":{}}}",
        export.model_path,
        export.engine_version,
        export.num_cores,
        export.batch_size,
        export.layers.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_export() -> DeepSparseExport {
        let mut e = new_deepsparse_export("model.onnx", "1.7.0");
        add_sparse_layer(
            &mut e,
            SparseLayer {
                name: "fc1".into(),
                sparsity_ratio: 0.8,
                mode: SparsityMode::Unstructured,
            },
        );
        add_sparse_layer(
            &mut e,
            SparseLayer {
                name: "fc2".into(),
                sparsity_ratio: 0.5,
                mode: SparsityMode::Dense,
            },
        );
        set_deepsparse_cores(&mut e, 4);
        e
    }

    #[test]
    fn new_export_stores_path() {
        let e = new_deepsparse_export("net.onnx", "1.6.0");
        assert_eq!(e.model_path, "net.onnx");
    }

    #[test]
    fn validate_complete() {
        let e = sample_export();
        assert!(validate_deepsparse(&e));
    }

    #[test]
    fn validate_empty_path_false() {
        let e = new_deepsparse_export("", "1.7.0");
        assert!(!validate_deepsparse(&e));
    }

    #[test]
    fn average_sparsity_correct() {
        let e = sample_export();
        let avg = average_sparsity(&e);
        assert!((avg - 0.65).abs() < 1e-5);
    }

    #[test]
    fn average_sparsity_no_layers() {
        let e = new_deepsparse_export("model.onnx", "1.7.0");
        assert_eq!(average_sparsity(&e), 0.0);
    }

    #[test]
    fn sparse_count_above_threshold() {
        let e = sample_export();
        assert_eq!(sparse_layer_count_above(&e, 0.6), 1);
    }

    #[test]
    fn size_estimate_positive() {
        let e = sample_export();
        assert!(deepsparse_size_estimate(&e) > 0);
    }

    #[test]
    fn summary_json_has_engine() {
        let e = sample_export();
        let json = deepsparse_summary_json(&e);
        assert!(json.contains("1.7.0"));
    }

    #[test]
    fn sparsity_mode_eq() {
        assert_eq!(SparsityMode::Structured, SparsityMode::Structured);
    }
}
