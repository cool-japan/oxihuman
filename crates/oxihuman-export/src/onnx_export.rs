// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! ONNX model graph export stub.
//! Provides a lightweight representation of an ONNX graph for export.

/// An ONNX tensor type tag.
#[derive(Debug, Clone, PartialEq)]
pub enum OnnxTensorType {
    Float32,
    Float16,
    Int64,
    Int32,
    Uint8,
    Bool,
}

/// A node in the ONNX graph.
#[derive(Debug, Clone)]
pub struct OnnxNode {
    pub name: String,
    pub op_type: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

/// An input/output tensor descriptor.
#[derive(Debug, Clone)]
pub struct OnnxTensorDesc {
    pub name: String,
    pub dtype: OnnxTensorType,
    pub shape: Vec<Option<i64>>,
}

/// Stub representation of an ONNX model graph.
#[derive(Debug, Clone, Default)]
pub struct OnnxExport {
    pub nodes: Vec<OnnxNode>,
    pub inputs: Vec<OnnxTensorDesc>,
    pub outputs: Vec<OnnxTensorDesc>,
    pub ir_version: u32,
    pub opset_version: u32,
}

/// Creates a new ONNX export stub with given opset.
pub fn new_onnx_export(opset: u32) -> OnnxExport {
    OnnxExport {
        ir_version: 7,
        opset_version: opset,
        ..Default::default()
    }
}

/// Adds a node to the ONNX graph.
pub fn add_onnx_node(export: &mut OnnxExport, node: OnnxNode) {
    export.nodes.push(node);
}

/// Adds an input tensor descriptor.
pub fn add_onnx_input(export: &mut OnnxExport, desc: OnnxTensorDesc) {
    export.inputs.push(desc);
}

/// Adds an output tensor descriptor.
pub fn add_onnx_output(export: &mut OnnxExport, desc: OnnxTensorDesc) {
    export.outputs.push(desc);
}

/// Returns the total node count.
pub fn onnx_node_count(export: &OnnxExport) -> usize {
    export.nodes.len()
}

/// Returns estimated byte size of the stub (not a real ONNX serialisation).
pub fn onnx_size_estimate(export: &OnnxExport) -> usize {
    let node_bytes: usize = export
        .nodes
        .iter()
        .map(|n| n.name.len() + n.op_type.len() + 64)
        .sum();
    let tensor_bytes: usize = export.inputs.len() * 64 + export.outputs.len() * 64;
    node_bytes + tensor_bytes + 128
}

/// Validates that inputs and outputs are non-empty.
pub fn validate_onnx(export: &OnnxExport) -> bool {
    !export.inputs.is_empty() && !export.outputs.is_empty()
}

/// Serialises a minimal JSON-like header for the ONNX stub.
pub fn onnx_header_json(export: &OnnxExport) -> String {
    format!(
        "{{\"ir_version\":{},\"opset\":{},\"nodes\":{},\"inputs\":{},\"outputs\":{}}}",
        export.ir_version,
        export.opset_version,
        export.nodes.len(),
        export.inputs.len(),
        export.outputs.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_export() -> OnnxExport {
        let mut e = new_onnx_export(17);
        add_onnx_input(
            &mut e,
            OnnxTensorDesc {
                name: "input".into(),
                dtype: OnnxTensorType::Float32,
                shape: vec![None, Some(3), Some(224), Some(224)],
            },
        );
        add_onnx_output(
            &mut e,
            OnnxTensorDesc {
                name: "output".into(),
                dtype: OnnxTensorType::Float32,
                shape: vec![None, Some(1000)],
            },
        );
        e
    }

    #[test]
    fn new_export_opset() {
        let e = new_onnx_export(17);
        assert_eq!(e.opset_version, 17);
    }

    #[test]
    fn add_node_increments_count() {
        let mut e = new_onnx_export(17);
        add_onnx_node(
            &mut e,
            OnnxNode {
                name: "relu0".into(),
                op_type: "Relu".into(),
                inputs: vec!["x".into()],
                outputs: vec!["y".into()],
            },
        );
        assert_eq!(onnx_node_count(&e), 1);
    }

    #[test]
    fn validate_with_io() {
        let e = sample_export();
        assert!(validate_onnx(&e));
    }

    #[test]
    fn validate_empty_false() {
        let e = new_onnx_export(17);
        assert!(!validate_onnx(&e));
    }

    #[test]
    fn size_estimate_positive() {
        let e = sample_export();
        assert!(onnx_size_estimate(&e) > 0);
    }

    #[test]
    fn header_json_contains_opset() {
        let e = sample_export();
        let json = onnx_header_json(&e);
        assert!(json.contains("17"));
    }

    #[test]
    fn ir_version_default() {
        let e = new_onnx_export(11);
        assert_eq!(e.ir_version, 7);
    }

    #[test]
    fn input_output_counts() {
        let e = sample_export();
        assert_eq!(e.inputs.len(), 1);
        assert_eq!(e.outputs.len(), 1);
    }

    #[test]
    fn tensor_type_eq() {
        assert_eq!(OnnxTensorType::Float32, OnnxTensorType::Float32);
        assert_ne!(OnnxTensorType::Float32, OnnxTensorType::Int64);
    }
}
