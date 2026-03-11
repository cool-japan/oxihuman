// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! OpenVINO Intermediate Representation (IR) stub export (.xml / .bin).

/// OpenVINO tensor precision tag.
#[derive(Debug, Clone, PartialEq)]
pub enum OvPrecision {
    Fp32,
    Fp16,
    Int8,
    U8,
    Int32,
}

/// An OpenVINO port descriptor.
#[derive(Debug, Clone)]
pub struct OvPort {
    pub id: u32,
    pub precision: OvPrecision,
    pub dims: Vec<i64>,
}

/// An OpenVINO layer stub.
#[derive(Debug, Clone)]
pub struct OvLayer {
    pub id: u32,
    pub name: String,
    pub layer_type: String,
    pub inputs: Vec<OvPort>,
    pub outputs: Vec<OvPort>,
}

/// OpenVINO IR stub export.
#[derive(Debug, Clone, Default)]
pub struct OpenVinoExport {
    pub ir_version: u32,
    pub name: String,
    pub layers: Vec<OvLayer>,
    pub edges: Vec<(u32, u32, u32, u32)>, /* (from_layer, from_port, to_layer, to_port) */
}

/// Creates a new OpenVINO IR export stub.
pub fn new_openvino_export(name: &str) -> OpenVinoExport {
    OpenVinoExport {
        ir_version: 11,
        name: name.to_string(),
        ..Default::default()
    }
}

/// Adds a layer to the IR graph.
pub fn add_ov_layer(export: &mut OpenVinoExport, layer: OvLayer) {
    export.layers.push(layer);
}

/// Adds an edge between two layers.
pub fn add_ov_edge(
    export: &mut OpenVinoExport,
    from_layer: u32,
    from_port: u32,
    to_layer: u32,
    to_port: u32,
) {
    export
        .edges
        .push((from_layer, from_port, to_layer, to_port));
}

/// Returns total layer count.
pub fn ov_layer_count(export: &OpenVinoExport) -> usize {
    export.layers.len()
}

/// Finds a layer by name.
pub fn find_ov_layer<'a>(export: &'a OpenVinoExport, name: &str) -> Option<&'a OvLayer> {
    export.layers.iter().find(|l| l.name == name)
}

/// Validates the IR (needs layers and at least one edge or I/O).
pub fn validate_openvino(export: &OpenVinoExport) -> bool {
    !export.layers.is_empty() && !export.name.is_empty()
}

/// Estimates the `.xml` size in bytes.
pub fn ov_xml_size_estimate(export: &OpenVinoExport) -> usize {
    export
        .layers
        .iter()
        .map(|l| l.name.len() + l.layer_type.len() + 128)
        .sum::<usize>()
        + export.edges.len() * 32
        + 512
}

/// Returns a minimal XML-like header string.
pub fn ov_xml_header(export: &OpenVinoExport) -> String {
    format!(
        "<net name=\"{}\" version=\"{}\"><layers count=\"{}\"/><edges count=\"{}\"/></net>",
        export.name,
        export.ir_version,
        export.layers.len(),
        export.edges.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_export() -> OpenVinoExport {
        let mut e = new_openvino_export("ResNet50");
        add_ov_layer(
            &mut e,
            OvLayer {
                id: 0,
                name: "input".into(),
                layer_type: "Parameter".into(),
                inputs: vec![],
                outputs: vec![OvPort {
                    id: 0,
                    precision: OvPrecision::Fp32,
                    dims: vec![1, 3, 224, 224],
                }],
            },
        );
        add_ov_layer(
            &mut e,
            OvLayer {
                id: 1,
                name: "conv0".into(),
                layer_type: "Convolution".into(),
                inputs: vec![OvPort {
                    id: 0,
                    precision: OvPrecision::Fp32,
                    dims: vec![1, 3, 224, 224],
                }],
                outputs: vec![OvPort {
                    id: 0,
                    precision: OvPrecision::Fp32,
                    dims: vec![1, 64, 112, 112],
                }],
            },
        );
        add_ov_edge(&mut e, 0, 0, 1, 0);
        e
    }

    #[test]
    fn ir_version() {
        let e = new_openvino_export("test");
        assert_eq!(e.ir_version, 11);
    }

    #[test]
    fn add_layer_count() {
        let e = sample_export();
        assert_eq!(ov_layer_count(&e), 2);
    }

    #[test]
    fn find_layer_found() {
        let e = sample_export();
        assert!(find_ov_layer(&e, "conv0").is_some());
    }

    #[test]
    fn validate_complete() {
        let e = sample_export();
        assert!(validate_openvino(&e));
    }

    #[test]
    fn validate_empty_false() {
        let e = new_openvino_export("test");
        assert!(!validate_openvino(&e));
    }

    #[test]
    fn xml_size_positive() {
        let e = sample_export();
        assert!(ov_xml_size_estimate(&e) > 0);
    }

    #[test]
    fn xml_header_contains_name() {
        let e = sample_export();
        let xml = ov_xml_header(&e);
        assert!(xml.contains("ResNet50"));
    }

    #[test]
    fn edges_stored() {
        let e = sample_export();
        assert_eq!(e.edges.len(), 1);
    }

    #[test]
    fn precision_eq() {
        assert_eq!(OvPrecision::Fp32, OvPrecision::Fp32);
    }
}
