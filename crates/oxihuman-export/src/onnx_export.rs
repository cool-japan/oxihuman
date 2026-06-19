// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! ONNX model graph export and real protobuf binary serialisation.
//!
//! `to_onnx_bytes` produces a valid ONNX ModelProto binary following the
//! onnx.proto3 schema, using the crate-local `ProtoEncoder`.  Nested
//! messages are encoded bottom-up (inner → outer) as required by the
//! length-delimited wire format.

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

// ─── ONNX elem_type constants ────────────────────────────────────────────────
/// ONNX DataType enum values as defined in onnx.proto3.
const ONNX_ELEM_FLOAT32: u64 = 1;
const ONNX_ELEM_UINT8: u64 = 2;
const ONNX_ELEM_INT32: u64 = 6;
const ONNX_ELEM_INT64: u64 = 7;
const ONNX_ELEM_BOOL: u64 = 9;
const ONNX_ELEM_FLOAT16: u64 = 10;

/// Map `OnnxTensorType` to its ONNX protobuf elem_type integer.
fn tensor_type_to_elem_type(dtype: &OnnxTensorType) -> u64 {
    match dtype {
        OnnxTensorType::Float32 => ONNX_ELEM_FLOAT32,
        OnnxTensorType::Float16 => ONNX_ELEM_FLOAT16,
        OnnxTensorType::Int64 => ONNX_ELEM_INT64,
        OnnxTensorType::Int32 => ONNX_ELEM_INT32,
        OnnxTensorType::Uint8 => ONNX_ELEM_UINT8,
        OnnxTensorType::Bool => ONNX_ELEM_BOOL,
    }
}

// ─── Nested proto encoders (bottom-up) ───────────────────────────────────────

/// Encode a single `Dimension` message.
///
/// ```protobuf
/// message Dimension { int64 dim_value = 1; }
/// ```
///
/// Dynamic (unknown) dimensions are encoded as `dim_param` (field 2, string "?")
/// which lets ONNX runtime handle variable batch sizes gracefully.
fn encode_dimension(dim: Option<i64>) -> Vec<u8> {
    use crate::protobuf_export::ProtoEncoder;
    let mut enc = ProtoEncoder::new();
    match dim {
        Some(v) => {
            // field 1 = dim_value (int64 as varint, re-interpret as u64)
            enc.write_varint_field(1, v as u64);
        }
        None => {
            // field 2 = dim_param (string "?") — marks a dynamic dimension
            enc.write_string_field(2, "?");
        }
    }
    enc.buf
}

/// Encode a `TensorShapeProto` message.
///
/// ```protobuf
/// message TensorShapeProto { repeated Dimension dim = 1; }
/// ```
fn encode_tensor_shape(shape: &[Option<i64>]) -> Vec<u8> {
    let mut enc = crate::protobuf_export::ProtoEncoder::new();
    for dim in shape {
        let dim_bytes = encode_dimension(*dim);
        enc.write_bytes_field(1, &dim_bytes);
    }
    enc.buf
}

/// Encode a `TypeProto.Tensor` sub-message.
///
/// ```protobuf
/// message Tensor { int32 elem_type = 1; TensorShapeProto shape = 2; }
/// ```
fn encode_type_proto_tensor(dtype: &OnnxTensorType, shape: &[Option<i64>]) -> Vec<u8> {
    let mut enc = crate::protobuf_export::ProtoEncoder::new();
    enc.write_varint_field(1, tensor_type_to_elem_type(dtype));
    let shape_bytes = encode_tensor_shape(shape);
    enc.write_bytes_field(2, &shape_bytes);
    enc.buf
}

/// Encode a `TypeProto` message (wraps the Tensor sub-message at field 1).
///
/// ```protobuf
/// message TypeProto { oneof value { Tensor tensor_type = 1; } }
/// ```
fn encode_type_proto(dtype: &OnnxTensorType, shape: &[Option<i64>]) -> Vec<u8> {
    let mut enc = crate::protobuf_export::ProtoEncoder::new();
    let tensor_bytes = encode_type_proto_tensor(dtype, shape);
    enc.write_bytes_field(1, &tensor_bytes);
    enc.buf
}

/// Encode a `ValueInfoProto` message (model input or output).
///
/// ```protobuf
/// message ValueInfoProto { string name = 1; TypeProto type = 2; }
/// ```
fn encode_value_info_proto(desc: &OnnxTensorDesc) -> Vec<u8> {
    let mut enc = crate::protobuf_export::ProtoEncoder::new();
    enc.write_string_field(1, &desc.name);
    let type_bytes = encode_type_proto(&desc.dtype, &desc.shape);
    enc.write_bytes_field(2, &type_bytes);
    enc.buf
}

/// Encode a `NodeProto` message.
///
/// ```protobuf
/// message NodeProto {
///   repeated string input  = 1;
///   repeated string output = 2;
///   string          name   = 3;
///   string          op_type = 4;
/// }
/// ```
fn encode_node_proto(node: &OnnxNode) -> Vec<u8> {
    let mut enc = crate::protobuf_export::ProtoEncoder::new();
    for inp in &node.inputs {
        enc.write_string_field(1, inp);
    }
    for out in &node.outputs {
        enc.write_string_field(2, out);
    }
    if !node.name.is_empty() {
        enc.write_string_field(3, &node.name);
    }
    enc.write_string_field(4, &node.op_type);
    enc.buf
}

/// Encode an `OperatorSetIdProto` message.
///
/// ```protobuf
/// message OperatorSetIdProto { string domain = 1; int64 version = 2; }
/// ```
///
/// The empty `domain` string selects the default ONNX operator set.
fn encode_opset_import(domain: &str, version: u64) -> Vec<u8> {
    let mut enc = crate::protobuf_export::ProtoEncoder::new();
    // Always write the domain even if empty — ONNX runtime expects it.
    enc.write_string_field(1, domain);
    enc.write_varint_field(2, version);
    enc.buf
}

/// Encode a `GraphProto` message.
///
/// Field layout (onnx.proto3):
/// - 1: node  NodeProto (repeated, length-delimited)
/// - 2: name  string
/// - 11: input  ValueInfoProto (repeated)
/// - 12: output ValueInfoProto (repeated)
fn encode_graph_proto(export: &OnnxExport) -> Vec<u8> {
    let mut enc = crate::protobuf_export::ProtoEncoder::new();
    // nodes
    for node in &export.nodes {
        let node_bytes = encode_node_proto(node);
        enc.write_bytes_field(1, &node_bytes);
    }
    // graph name
    enc.write_string_field(2, "oxihuman_graph");
    // inputs (field 11)
    for input_desc in &export.inputs {
        let vi_bytes = encode_value_info_proto(input_desc);
        enc.write_bytes_field(11, &vi_bytes);
    }
    // outputs (field 12)
    for output_desc in &export.outputs {
        let vi_bytes = encode_value_info_proto(output_desc);
        enc.write_bytes_field(12, &vi_bytes);
    }
    enc.buf
}

// ─── Public API ──────────────────────────────────────────────────────────────

/// Serialise an `OnnxExport` to a valid ONNX `ModelProto` binary (protobuf3).
///
/// The encoding follows onnx.proto3 field numbering:
/// - ModelProto field 1: ir_version int64
/// - ModelProto field 5: model_version int64
/// - ModelProto field 7: graph GraphProto (length-delimited)
/// - ModelProto field 8: opset_import OperatorSetIdProto (repeated)
///
/// # Example
/// ```rust,ignore
/// let mut export = new_onnx_export(17);
/// add_onnx_input(&mut export, OnnxTensorDesc { name: "x".into(), dtype: OnnxTensorType::Float32, shape: vec![Some(1), Some(3)] });
/// add_onnx_output(&mut export, OnnxTensorDesc { name: "y".into(), dtype: OnnxTensorType::Float32, shape: vec![Some(1), Some(3)] });
/// let bytes = to_onnx_bytes(&export);
/// assert!(!bytes.is_empty());
/// ```
pub fn to_onnx_bytes(export: &OnnxExport) -> Vec<u8> {
    let mut enc = crate::protobuf_export::ProtoEncoder::new();

    // field 1: ir_version (int64 / varint)
    enc.write_varint_field(1, export.ir_version as u64);

    // field 5: model_version (int64 / varint) — always 1 for OxiHuman exports
    enc.write_varint_field(5, 1_u64);

    // field 7: graph (GraphProto, length-delimited)
    let graph_bytes = encode_graph_proto(export);
    enc.write_bytes_field(7, &graph_bytes);

    // field 8: opset_import (OperatorSetIdProto, repeated, length-delimited)
    // Emit the standard ONNX opset (empty domain = "ai.onnx").
    let opset_bytes = encode_opset_import("", export.opset_version as u64);
    enc.write_bytes_field(8, &opset_bytes);

    enc.buf
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

    // ─── to_onnx_bytes tests ────────────────────────────────────────────────

    /// An export with no nodes still encodes a non-empty ModelProto because
    /// ir_version, model_version, an empty graph, and the opset import are
    /// always written.
    #[test]
    fn to_onnx_bytes_empty_graph_produces_non_empty_bytes() {
        let export = new_onnx_export(11);
        let bytes = to_onnx_bytes(&export);
        assert!(
            !bytes.is_empty(),
            "to_onnx_bytes must produce non-empty output even for an empty graph"
        );
    }

    /// The very first byte(s) of the ModelProto binary must encode the varint
    /// for ir_version (field 1, wire type 0).  The protobuf tag for field 1 /
    /// varint is 0x08, followed immediately by the ir_version value.  For
    /// ir_version = 7 that second byte is 0x07, so the stream starts with
    /// [0x08, 0x07, ...].
    #[test]
    fn to_onnx_bytes_starts_with_ir_version_varint() {
        let export = new_onnx_export(11); // ir_version defaults to 7
        let bytes = to_onnx_bytes(&export);
        // tag = (field=1 << 3) | wire_type=0 = 0x08
        assert_eq!(
            bytes[0], 0x08,
            "first byte must be protobuf tag for field 1 / varint (0x08)"
        );
        // ir_version = 7  →  varint = 0x07
        assert_eq!(bytes[1], 0x07, "second byte must be varint-encoded ir_version=7");
    }

    /// A ModelProto containing a Gemm node with two inputs ("A", "B"), one
    /// output ("C"), plus one graph input and one graph output must produce
    /// substantially more than 10 bytes.
    #[test]
    fn to_onnx_bytes_gemm_node_length_gt_10() {
        let mut export = new_onnx_export(17);
        add_onnx_node(
            &mut export,
            OnnxNode {
                name: "gemm0".into(),
                op_type: "Gemm".into(),
                inputs: vec!["A".into(), "B".into()],
                outputs: vec!["C".into()],
            },
        );
        add_onnx_input(
            &mut export,
            OnnxTensorDesc {
                name: "A".into(),
                dtype: OnnxTensorType::Float32,
                shape: vec![Some(4), Some(8)],
            },
        );
        add_onnx_input(
            &mut export,
            OnnxTensorDesc {
                name: "B".into(),
                dtype: OnnxTensorType::Float32,
                shape: vec![Some(8), Some(4)],
            },
        );
        add_onnx_output(
            &mut export,
            OnnxTensorDesc {
                name: "C".into(),
                dtype: OnnxTensorType::Float32,
                shape: vec![Some(4), Some(4)],
            },
        );
        let bytes = to_onnx_bytes(&export);
        assert!(
            bytes.len() > 10,
            "Gemm ModelProto must be longer than 10 bytes, got {}",
            bytes.len()
        );
    }

    /// Calling `to_onnx_bytes` twice on the same export must return byte
    /// slices of identical length (deterministic, pure function).
    #[test]
    fn to_onnx_bytes_deterministic_length() {
        let mut export = new_onnx_export(13);
        add_onnx_node(
            &mut export,
            OnnxNode {
                name: "relu0".into(),
                op_type: "Relu".into(),
                inputs: vec!["x".into()],
                outputs: vec!["y".into()],
            },
        );
        add_onnx_input(
            &mut export,
            OnnxTensorDesc {
                name: "x".into(),
                dtype: OnnxTensorType::Float32,
                shape: vec![None, Some(256)],
            },
        );
        add_onnx_output(
            &mut export,
            OnnxTensorDesc {
                name: "y".into(),
                dtype: OnnxTensorType::Float32,
                shape: vec![None, Some(256)],
            },
        );
        let bytes_a = to_onnx_bytes(&export);
        let bytes_b = to_onnx_bytes(&export);
        assert_eq!(
            bytes_a.len(),
            bytes_b.len(),
            "to_onnx_bytes must be deterministic: both calls must produce identical length"
        );
        assert_eq!(
            bytes_a, bytes_b,
            "to_onnx_bytes must be deterministic: byte content must be identical"
        );
    }

    /// Verify that all ONNX tensor types are mapped to distinct non-zero
    /// elem_type values and that the expected standard values match.
    #[test]
    fn tensor_type_elem_type_mapping() {
        assert_eq!(tensor_type_to_elem_type(&OnnxTensorType::Float32), 1);
        assert_eq!(tensor_type_to_elem_type(&OnnxTensorType::Float16), 10);
        assert_eq!(tensor_type_to_elem_type(&OnnxTensorType::Int64), 7);
        assert_eq!(tensor_type_to_elem_type(&OnnxTensorType::Int32), 6);
        assert_eq!(tensor_type_to_elem_type(&OnnxTensorType::Uint8), 2);
        assert_eq!(tensor_type_to_elem_type(&OnnxTensorType::Bool), 9);
    }

    /// Verify that a dynamic dimension (`None`) is encoded differently from a
    /// static dimension (`Some(1)`) — dynamic uses `dim_param` (field 2) while
    /// static uses `dim_value` (field 1).
    #[test]
    fn encode_dimension_static_vs_dynamic() {
        let static_bytes = encode_dimension(Some(4));
        let dynamic_bytes = encode_dimension(None);
        // Static: tag 0x08 (field 1 / varint), value 4 → [0x08, 0x04]
        assert_eq!(static_bytes, vec![0x08, 0x04]);
        // Dynamic: tag 0x12 (field 2 / len-delimited), length 1, byte '?'
        assert_eq!(dynamic_bytes[0], 0x12, "dynamic dimension must use field 2");
        // Must contain the ASCII '?' byte somewhere
        assert!(
            dynamic_bytes.contains(&b'?'),
            "dynamic dimension must encode '?'"
        );
    }

    /// Encode a full Gemm graph and verify that the graph bytes contain the
    /// UTF-8 bytes for "Gemm" (op_type) and "oxihuman_graph" (graph name).
    #[test]
    fn to_onnx_bytes_gemm_contains_op_type_and_graph_name() {
        let mut export = new_onnx_export(17);
        add_onnx_node(
            &mut export,
            OnnxNode {
                name: "g0".into(),
                op_type: "Gemm".into(),
                inputs: vec!["X".into()],
                outputs: vec!["Y".into()],
            },
        );
        add_onnx_input(
            &mut export,
            OnnxTensorDesc {
                name: "X".into(),
                dtype: OnnxTensorType::Float32,
                shape: vec![Some(1), Some(4)],
            },
        );
        add_onnx_output(
            &mut export,
            OnnxTensorDesc {
                name: "Y".into(),
                dtype: OnnxTensorType::Float32,
                shape: vec![Some(1), Some(4)],
            },
        );
        let bytes = to_onnx_bytes(&export);
        let gemm_pattern = b"Gemm";
        let graph_name = b"oxihuman_graph";
        assert!(
            bytes.windows(gemm_pattern.len()).any(|w| w == gemm_pattern),
            "serialised bytes must contain the op_type string 'Gemm'"
        );
        assert!(
            bytes.windows(graph_name.len()).any(|w| w == graph_name),
            "serialised bytes must contain the graph name 'oxihuman_graph'"
        );
    }
}
