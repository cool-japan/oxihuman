// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Qualcomm SNPE (Snapdragon Neural Processing Engine) DLC stub export.

/// SNPE runtime target.
#[derive(Debug, Clone, PartialEq)]
pub enum SnpeRuntime {
    Cpu,
    Gpu,
    Dsp,
    AipDsp,
}

/// SNPE quantisation encoding.
#[derive(Debug, Clone, PartialEq)]
pub enum SnpeEncoding {
    Float,
    Tf8,
    Int4,
}

/// SNPE tensor stub.
#[derive(Debug, Clone)]
pub struct SnpeTensor {
    pub name: String,
    pub dims: Vec<u32>,
    pub encoding: SnpeEncoding,
}

/// SNPE DLC model stub.
#[derive(Debug, Clone, Default)]
pub struct SnpeExport {
    pub dlc_version: String,
    pub model_name: String,
    pub target_runtime: Option<SnpeRuntime>,
    pub inputs: Vec<SnpeTensor>,
    pub outputs: Vec<SnpeTensor>,
    pub layer_count: usize,
}

/// Creates a new SNPE export stub.
pub fn new_snpe_export(model_name: &str, dlc_version: &str) -> SnpeExport {
    SnpeExport {
        model_name: model_name.to_string(),
        dlc_version: dlc_version.to_string(),
        ..Default::default()
    }
}

/// Sets the preferred runtime target.
pub fn set_snpe_runtime(export: &mut SnpeExport, runtime: SnpeRuntime) {
    export.target_runtime = Some(runtime);
}

/// Adds an input tensor.
pub fn add_snpe_input(export: &mut SnpeExport, tensor: SnpeTensor) {
    export.inputs.push(tensor);
}

/// Adds an output tensor.
pub fn add_snpe_output(export: &mut SnpeExport, tensor: SnpeTensor) {
    export.outputs.push(tensor);
}

/// Sets the layer count in the DLC graph.
pub fn set_snpe_layer_count(export: &mut SnpeExport, count: usize) {
    export.layer_count = count;
}

/// Validates the export stub (model name, at least one input and output).
pub fn validate_snpe(export: &SnpeExport) -> bool {
    !export.model_name.is_empty() && !export.inputs.is_empty() && !export.outputs.is_empty()
}

/// Estimates the DLC file size in bytes.
pub fn snpe_size_estimate(export: &SnpeExport) -> usize {
    let tensor_bytes: usize = export
        .inputs
        .iter()
        .chain(export.outputs.iter())
        .map(|t| t.name.len() + t.dims.len() * 4 + 64)
        .sum();
    tensor_bytes + export.layer_count * 256 + 512
}

/// Returns a JSON-like summary.
pub fn snpe_summary_json(export: &SnpeExport) -> String {
    format!(
        "{{\"model\":\"{}\",\"dlc_version\":\"{}\",\"inputs\":{},\"outputs\":{},\"layers\":{}}}",
        export.model_name,
        export.dlc_version,
        export.inputs.len(),
        export.outputs.len(),
        export.layer_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_export() -> SnpeExport {
        let mut e = new_snpe_export("MobileNet", "1.68.0");
        add_snpe_input(
            &mut e,
            SnpeTensor {
                name: "input_1".into(),
                dims: vec![1, 224, 224, 3],
                encoding: SnpeEncoding::Tf8,
            },
        );
        add_snpe_output(
            &mut e,
            SnpeTensor {
                name: "Softmax".into(),
                dims: vec![1, 1000],
                encoding: SnpeEncoding::Float,
            },
        );
        set_snpe_layer_count(&mut e, 52);
        e
    }

    #[test]
    fn new_export_stores_name() {
        let e = new_snpe_export("ResNet", "1.70.0");
        assert_eq!(e.model_name, "ResNet");
    }

    #[test]
    fn validate_complete() {
        let e = sample_export();
        assert!(validate_snpe(&e));
    }

    #[test]
    fn validate_no_outputs_false() {
        let mut e = new_snpe_export("Net", "1.68.0");
        add_snpe_input(
            &mut e,
            SnpeTensor {
                name: "in".into(),
                dims: vec![1, 3, 224, 224],
                encoding: SnpeEncoding::Float,
            },
        );
        assert!(!validate_snpe(&e));
    }

    #[test]
    fn runtime_set() {
        let mut e = sample_export();
        set_snpe_runtime(&mut e, SnpeRuntime::Dsp);
        assert_eq!(e.target_runtime, Some(SnpeRuntime::Dsp));
    }

    #[test]
    fn layer_count_stored() {
        let e = sample_export();
        assert_eq!(e.layer_count, 52);
    }

    #[test]
    fn size_estimate_positive() {
        let e = sample_export();
        assert!(snpe_size_estimate(&e) > 0);
    }

    #[test]
    fn summary_json_has_model() {
        let e = sample_export();
        let json = snpe_summary_json(&e);
        assert!(json.contains("MobileNet"));
    }

    #[test]
    fn encoding_eq() {
        assert_eq!(SnpeEncoding::Tf8, SnpeEncoding::Tf8);
    }

    #[test]
    fn runtime_eq() {
        assert_eq!(SnpeRuntime::Gpu, SnpeRuntime::Gpu);
    }
}
