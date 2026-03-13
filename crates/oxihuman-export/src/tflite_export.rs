// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! TensorFlow Lite flatbuffer stub export.

/// TFLite tensor quantization parameters.
#[derive(Debug, Clone, Default)]
pub struct TfliteQuantParams {
    pub scale: Vec<f32>,
    pub zero_point: Vec<i32>,
    pub quantized_dimension: i32,
}

/// A TFLite tensor descriptor.
#[derive(Debug, Clone)]
pub struct TfliteTensor {
    pub name: String,
    pub shape: Vec<i32>,
    pub dtype: u8, /* 0=float32, 1=int8, 2=uint8, 3=int32 */
    pub quantization: TfliteQuantParams,
}

/// A TFLite operator stub.
#[derive(Debug, Clone)]
pub struct TfliteOperator {
    pub opcode_index: u32,
    pub inputs: Vec<i32>,
    pub outputs: Vec<i32>,
}

/// TFLite subgraph stub.
#[derive(Debug, Clone, Default)]
pub struct TfliteSubgraph {
    pub tensors: Vec<TfliteTensor>,
    pub operators: Vec<TfliteOperator>,
    pub inputs: Vec<i32>,
    pub outputs: Vec<i32>,
    pub name: String,
}

/// TFLite model stub.
#[derive(Debug, Clone, Default)]
pub struct TfliteExport {
    pub version: u32,
    pub subgraphs: Vec<TfliteSubgraph>,
    pub description: String,
}

/// Creates a new TFLite export stub.
pub fn new_tflite_export(description: &str) -> TfliteExport {
    TfliteExport {
        version: 3,
        description: description.to_string(),
        ..Default::default()
    }
}

/// Adds a subgraph to the model.
pub fn add_tflite_subgraph(export: &mut TfliteExport, subgraph: TfliteSubgraph) {
    export.subgraphs.push(subgraph);
}

/// Returns the total tensor count across all subgraphs.
pub fn tflite_tensor_count(export: &TfliteExport) -> usize {
    export.subgraphs.iter().map(|sg| sg.tensors.len()).sum()
}

/// Returns the total operator count across all subgraphs.
pub fn tflite_operator_count(export: &TfliteExport) -> usize {
    export.subgraphs.iter().map(|sg| sg.operators.len()).sum()
}

/// Estimates the byte size of the flatbuffer stub.
pub fn tflite_size_estimate(export: &TfliteExport) -> usize {
    let tensor_bytes: usize = export
        .subgraphs
        .iter()
        .flat_map(|sg| &sg.tensors)
        .map(|t| t.name.len() + t.shape.len() * 4 + 32)
        .sum();
    tensor_bytes + export.subgraphs.len() * 64 + 256
}

/// Returns `true` if the model has at least one subgraph with tensors.
pub fn validate_tflite(export: &TfliteExport) -> bool {
    export.subgraphs.iter().any(|sg| !sg.tensors.is_empty())
}

/// Returns a JSON-like header string for the TFLite stub.
pub fn tflite_header_json(export: &TfliteExport) -> String {
    format!(
        "{{\"version\":{},\"subgraphs\":{},\"tensors\":{},\"ops\":{}}}",
        export.version,
        export.subgraphs.len(),
        tflite_tensor_count(export),
        tflite_operator_count(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_subgraph() -> TfliteSubgraph {
        TfliteSubgraph {
            name: "main".into(),
            tensors: vec![TfliteTensor {
                name: "input".into(),
                shape: vec![1, 224, 224, 3],
                dtype: 0,
                quantization: TfliteQuantParams::default(),
            }],
            operators: vec![],
            inputs: vec![0],
            outputs: vec![0],
        }
    }

    #[test]
    fn new_export_version() {
        let e = new_tflite_export("test");
        assert_eq!(e.version, 3);
    }

    #[test]
    fn add_subgraph_increments() {
        let mut e = new_tflite_export("test");
        add_tflite_subgraph(&mut e, sample_subgraph());
        assert_eq!(e.subgraphs.len(), 1);
    }

    #[test]
    fn tensor_count_correct() {
        let mut e = new_tflite_export("test");
        add_tflite_subgraph(&mut e, sample_subgraph());
        assert_eq!(tflite_tensor_count(&e), 1);
    }

    #[test]
    fn operator_count_zero() {
        let mut e = new_tflite_export("test");
        add_tflite_subgraph(&mut e, sample_subgraph());
        assert_eq!(tflite_operator_count(&e), 0);
    }

    #[test]
    fn validate_with_tensors() {
        let mut e = new_tflite_export("test");
        add_tflite_subgraph(&mut e, sample_subgraph());
        assert!(validate_tflite(&e));
    }

    #[test]
    fn validate_empty_false() {
        let e = new_tflite_export("test");
        assert!(!validate_tflite(&e));
    }

    #[test]
    fn size_estimate_positive() {
        let mut e = new_tflite_export("test");
        add_tflite_subgraph(&mut e, sample_subgraph());
        assert!(tflite_size_estimate(&e) > 0);
    }

    #[test]
    fn header_json_contains_version() {
        let mut e = new_tflite_export("test");
        add_tflite_subgraph(&mut e, sample_subgraph());
        let json = tflite_header_json(&e);
        assert!(json.contains("\"version\":3"));
    }

    #[test]
    fn description_stored() {
        let e = new_tflite_export("my model");
        assert_eq!(e.description, "my model");
    }
}
