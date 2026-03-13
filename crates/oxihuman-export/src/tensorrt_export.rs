// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! TensorRT engine stub export.

/// TensorRT precision mode.
#[derive(Debug, Clone, PartialEq)]
pub enum TrtPrecision {
    Fp32,
    Fp16,
    Int8,
    BFloat16,
}

/// TensorRT binding (input or output tensor).
#[derive(Debug, Clone)]
pub struct TrtBinding {
    pub name: String,
    pub is_input: bool,
    pub shape: Vec<i64>,
    pub precision: TrtPrecision,
}

/// TensorRT engine stub.
#[derive(Debug, Clone, Default)]
pub struct TensorRtExport {
    pub engine_name: String,
    pub trt_version: String,
    pub bindings: Vec<TrtBinding>,
    pub workspace_mb: u32,
    pub max_batch_size: u32,
}

/// Creates a new TensorRT export stub.
pub fn new_tensorrt_export(name: &str, version: &str) -> TensorRtExport {
    TensorRtExport {
        engine_name: name.to_string(),
        trt_version: version.to_string(),
        workspace_mb: 1024,
        max_batch_size: 1,
        ..Default::default()
    }
}

/// Adds a binding (input or output tensor).
pub fn add_trt_binding(export: &mut TensorRtExport, binding: TrtBinding) {
    export.bindings.push(binding);
}

/// Returns the number of input bindings.
pub fn trt_input_count(export: &TensorRtExport) -> usize {
    export.bindings.iter().filter(|b| b.is_input).count()
}

/// Returns the number of output bindings.
pub fn trt_output_count(export: &TensorRtExport) -> usize {
    export.bindings.iter().filter(|b| !b.is_input).count()
}

/// Finds a binding by name.
pub fn find_trt_binding<'a>(export: &'a TensorRtExport, name: &str) -> Option<&'a TrtBinding> {
    export.bindings.iter().find(|b| b.name == name)
}

/// Validates the engine stub.
pub fn validate_trt_export(export: &TensorRtExport) -> bool {
    trt_input_count(export) > 0 && trt_output_count(export) > 0
}

/// Estimates the serialised engine size in bytes.
pub fn trt_size_estimate(export: &TensorRtExport) -> usize {
    let binding_bytes: usize = export
        .bindings
        .iter()
        .map(|b| b.name.len() + b.shape.len() * 8 + 64)
        .sum();
    binding_bytes + (export.workspace_mb as usize) * 1024 * 1024 / 1000 + 512
}

/// Returns a JSON-like summary.
pub fn trt_summary_json(export: &TensorRtExport) -> String {
    format!(
        "{{\"engine\":\"{}\",\"trt_version\":\"{}\",\"inputs\":{},\"outputs\":{},\"workspace_mb\":{}}}",
        export.engine_name,
        export.trt_version,
        trt_input_count(export),
        trt_output_count(export),
        export.workspace_mb
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_export() -> TensorRtExport {
        let mut e = new_tensorrt_export("my_engine", "8.6.1");
        add_trt_binding(
            &mut e,
            TrtBinding {
                name: "input".into(),
                is_input: true,
                shape: vec![1, 3, 224, 224],
                precision: TrtPrecision::Fp16,
            },
        );
        add_trt_binding(
            &mut e,
            TrtBinding {
                name: "output".into(),
                is_input: false,
                shape: vec![1, 1000],
                precision: TrtPrecision::Fp32,
            },
        );
        e
    }

    #[test]
    fn engine_name() {
        let e = new_tensorrt_export("net", "8.0");
        assert_eq!(e.engine_name, "net");
    }

    #[test]
    fn input_output_counts() {
        let e = sample_export();
        assert_eq!(trt_input_count(&e), 1);
        assert_eq!(trt_output_count(&e), 1);
    }

    #[test]
    fn validate_complete() {
        let e = sample_export();
        assert!(validate_trt_export(&e));
    }

    #[test]
    fn validate_no_output_false() {
        let mut e = new_tensorrt_export("net", "8.0");
        add_trt_binding(
            &mut e,
            TrtBinding {
                name: "input".into(),
                is_input: true,
                shape: vec![1, 3],
                precision: TrtPrecision::Fp32,
            },
        );
        assert!(!validate_trt_export(&e));
    }

    #[test]
    fn find_binding_found() {
        let e = sample_export();
        assert!(find_trt_binding(&e, "input").is_some());
    }

    #[test]
    fn find_binding_not_found() {
        let e = sample_export();
        assert!(find_trt_binding(&e, "missing").is_none());
    }

    #[test]
    fn size_estimate_positive() {
        let e = sample_export();
        assert!(trt_size_estimate(&e) > 0);
    }

    #[test]
    fn summary_json_contains_version() {
        let e = sample_export();
        let json = trt_summary_json(&e);
        assert!(json.contains("8.6.1"));
    }

    #[test]
    fn precision_eq() {
        assert_eq!(TrtPrecision::Fp16, TrtPrecision::Fp16);
    }
}
