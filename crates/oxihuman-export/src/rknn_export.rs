// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! RKNPU RKNN model stub export.

/// RKNN tensor data type.
#[derive(Debug, Clone, PartialEq)]
pub enum RknnDtype {
    Float32,
    Float16,
    Int8,
    Uint8,
    Int16,
}

/// RKNN tensor format.
#[derive(Debug, Clone, PartialEq)]
pub enum RknnFormat {
    Nchw,
    Nhwc,
    Undefined,
}

/// RKNN tensor descriptor.
#[derive(Debug, Clone)]
pub struct RknnTensor {
    pub name: String,
    pub dims: Vec<u32>,
    pub dtype: RknnDtype,
    pub fmt: RknnFormat,
}

/// RKNN model stub.
#[derive(Debug, Clone, Default)]
pub struct RknnExport {
    pub model_version: String,
    pub sdk_version: String,
    pub inputs: Vec<RknnTensor>,
    pub outputs: Vec<RknnTensor>,
    pub mean_values: Vec<f32>,
    pub std_values: Vec<f32>,
}

/// Creates a new RKNN export stub.
pub fn new_rknn_export(model_version: &str, sdk_version: &str) -> RknnExport {
    RknnExport {
        model_version: model_version.to_string(),
        sdk_version: sdk_version.to_string(),
        ..Default::default()
    }
}

/// Adds an input tensor descriptor.
pub fn add_rknn_input(export: &mut RknnExport, tensor: RknnTensor) {
    export.inputs.push(tensor);
}

/// Adds an output tensor descriptor.
pub fn add_rknn_output(export: &mut RknnExport, tensor: RknnTensor) {
    export.outputs.push(tensor);
}

/// Sets normalisation parameters (mean and std per channel).
pub fn set_rknn_normalisation(export: &mut RknnExport, mean: Vec<f32>, std: Vec<f32>) {
    export.mean_values = mean;
    export.std_values = std;
}

/// Returns the total number of input + output tensors.
pub fn rknn_tensor_count(export: &RknnExport) -> usize {
    export.inputs.len() + export.outputs.len()
}

/// Validates the model stub (at least one input and one output).
pub fn validate_rknn(export: &RknnExport) -> bool {
    !export.inputs.is_empty() && !export.outputs.is_empty()
}

/// Estimates the serialised size in bytes.
pub fn rknn_size_estimate(export: &RknnExport) -> usize {
    let tensor_bytes: usize = export
        .inputs
        .iter()
        .chain(export.outputs.iter())
        .map(|t| t.name.len() + t.dims.len() * 4 + 64)
        .sum();
    tensor_bytes + (export.mean_values.len() + export.std_values.len()) * 4 + 512
}

/// Returns a JSON-like summary string.
pub fn rknn_summary_json(export: &RknnExport) -> String {
    format!(
        "{{\"model_version\":\"{}\",\"sdk_version\":\"{}\",\"inputs\":{},\"outputs\":{}}}",
        export.model_version,
        export.sdk_version,
        export.inputs.len(),
        export.outputs.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_export() -> RknnExport {
        let mut e = new_rknn_export("1.0", "1.6.0");
        add_rknn_input(
            &mut e,
            RknnTensor {
                name: "input".into(),
                dims: vec![1, 3, 640, 640],
                dtype: RknnDtype::Uint8,
                fmt: RknnFormat::Nhwc,
            },
        );
        add_rknn_output(
            &mut e,
            RknnTensor {
                name: "output".into(),
                dims: vec![1, 85, 80, 80],
                dtype: RknnDtype::Float32,
                fmt: RknnFormat::Nchw,
            },
        );
        e
    }

    #[test]
    fn new_export_stores_versions() {
        let e = new_rknn_export("2.0", "1.7.0");
        assert_eq!(e.model_version, "2.0");
        assert_eq!(e.sdk_version, "1.7.0");
    }

    #[test]
    fn add_tensors_count() {
        let e = sample_export();
        assert_eq!(rknn_tensor_count(&e), 2);
    }

    #[test]
    fn validate_complete() {
        let e = sample_export();
        assert!(validate_rknn(&e));
    }

    #[test]
    fn validate_empty_false() {
        let e = new_rknn_export("1.0", "1.6.0");
        assert!(!validate_rknn(&e));
    }

    #[test]
    fn size_estimate_positive() {
        let e = sample_export();
        assert!(rknn_size_estimate(&e) > 0);
    }

    #[test]
    fn summary_json_has_version() {
        let e = sample_export();
        let json = rknn_summary_json(&e);
        assert!(json.contains("1.6.0"));
    }

    #[test]
    fn normalisation_stored() {
        let mut e = sample_export();
        set_rknn_normalisation(&mut e, vec![127.5, 127.5, 127.5], vec![128.0, 128.0, 128.0]);
        assert_eq!(e.mean_values.len(), 3);
        assert_eq!(e.std_values.len(), 3);
    }

    #[test]
    fn dtype_eq() {
        assert_eq!(RknnDtype::Float16, RknnDtype::Float16);
    }

    #[test]
    fn format_eq() {
        assert_eq!(RknnFormat::Nhwc, RknnFormat::Nhwc);
    }
}
