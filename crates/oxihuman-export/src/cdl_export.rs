// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! ASC CDL (Color Decision List) export.

/// A single CDL correction node (SOP + SAT).
#[derive(Debug, Clone)]
pub struct CdlNode {
    pub id: String,
    pub slope: [f32; 3],
    pub offset: [f32; 3],
    pub power: [f32; 3],
    pub saturation: f32,
}

impl Default for CdlNode {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            slope: [1.0, 1.0, 1.0],
            offset: [0.0, 0.0, 0.0],
            power: [1.0, 1.0, 1.0],
            saturation: 1.0,
        }
    }
}

/// A CDL export containing one or more nodes.
#[derive(Debug, Clone)]
pub struct CdlExport {
    pub version: String,
    pub nodes: Vec<CdlNode>,
}

/// Create a new CDL export.
pub fn new_cdl_export() -> CdlExport {
    CdlExport {
        version: "1.01".to_string(),
        nodes: Vec::new(),
    }
}

/// Add a CDL node.
pub fn cdl_add_node(export: &mut CdlExport, node: CdlNode) {
    export.nodes.push(node);
}

/// Add a default identity CDL node.
pub fn cdl_add_identity(export: &mut CdlExport, id: &str) {
    let node = CdlNode {
        id: id.to_string(),
        ..CdlNode::default()
    };
    export.nodes.push(node);
}

/// Return the node count.
pub fn cdl_node_count(export: &CdlExport) -> usize {
    export.nodes.len()
}

/// Find a node by ID.
pub fn cdl_find_node<'a>(export: &'a CdlExport, id: &str) -> Option<&'a CdlNode> {
    export.nodes.iter().find(|n| n.id == id)
}

/// Validate: saturation >= 0, slope components > 0.
pub fn validate_cdl(export: &CdlExport) -> bool {
    export
        .nodes
        .iter()
        .all(|n| n.saturation >= 0.0 && n.slope[0] >= 0.0 && n.slope[1] >= 0.0 && n.slope[2] >= 0.0)
}

/// Serialize to CDL XML.
pub fn cdl_to_xml(export: &CdlExport) -> String {
    let mut out = format!(
        "<?xml version=\"1.0\"?>\n<ColorDecisionList xmlns=\"urn:ASC:CDL:v{}\">\n",
        export.version
    );
    for node in &export.nodes {
        out.push_str(&format!(
            "  <ColorDecision>\n    <ColorCorrection id=\"{}\">\n      <SOPNode>\n        <Slope>{:.6} {:.6} {:.6}</Slope>\n        <Offset>{:.6} {:.6} {:.6}</Offset>\n        <Power>{:.6} {:.6} {:.6}</Power>\n      </SOPNode>\n      <SatNode><Saturation>{:.6}</Saturation></SatNode>\n    </ColorCorrection>\n  </ColorDecision>\n",
            node.id,
            node.slope[0], node.slope[1], node.slope[2],
            node.offset[0], node.offset[1], node.offset[2],
            node.power[0], node.power[1], node.power[2],
            node.saturation
        ));
    }
    out.push_str("</ColorDecisionList>\n");
    out
}

/// Estimate the CDL file size.
pub fn cdl_size_bytes(export: &CdlExport) -> usize {
    cdl_to_xml(export).len()
}

/// Apply CDL to a single RGB value (float, linear).
pub fn apply_cdl(node: &CdlNode, rgb: [f32; 3]) -> [f32; 3] {
    let sop: [f32; 3] = [
        (rgb[0] * node.slope[0] + node.offset[0])
            .max(0.0)
            .powf(node.power[0]),
        (rgb[1] * node.slope[1] + node.offset[1])
            .max(0.0)
            .powf(node.power[1]),
        (rgb[2] * node.slope[2] + node.offset[2])
            .max(0.0)
            .powf(node.power[2]),
    ];
    let lum = 0.2126 * sop[0] + 0.7152 * sop[1] + 0.0722 * sop[2];
    [
        lum + node.saturation * (sop[0] - lum),
        lum + node.saturation * (sop[1] - lum),
        lum + node.saturation * (sop[2] - lum),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> CdlExport {
        let mut exp = new_cdl_export();
        cdl_add_identity(&mut exp, "shot_010");
        let node = CdlNode {
            id: "shot_020".to_string(),
            slope: [1.1, 1.0, 0.9],
            saturation: 1.2,
            ..CdlNode::default()
        };
        cdl_add_node(&mut exp, node);
        exp
    }

    #[test]
    fn test_node_count() {
        assert_eq!(cdl_node_count(&sample()), 2);
    }

    #[test]
    fn test_find_node() {
        let exp = sample();
        assert!(cdl_find_node(&exp, "shot_010").is_some());
        assert!(cdl_find_node(&exp, "none").is_none());
    }

    #[test]
    fn test_validate_valid() {
        assert!(validate_cdl(&sample()));
    }

    #[test]
    fn test_validate_bad_slope() {
        let mut exp = new_cdl_export();
        let mut node = CdlNode::default();
        node.slope[0] = -1.0;
        cdl_add_node(&mut exp, node);
        assert!(!validate_cdl(&exp));
    }

    #[test]
    fn test_to_xml() {
        let s = cdl_to_xml(&sample());
        assert!(s.contains("shot_010"));
    }

    #[test]
    fn test_size_positive() {
        assert!(cdl_size_bytes(&sample()) > 0);
    }

    #[test]
    fn test_apply_cdl_identity() {
        let node = CdlNode::default();
        let rgb = [0.5f32, 0.5, 0.5];
        let out = apply_cdl(&node, rgb);
        assert!((out[0] - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_apply_cdl_slope() {
        let node = CdlNode {
            slope: [2.0, 2.0, 2.0],
            ..CdlNode::default()
        };
        let out = apply_cdl(&node, [0.5, 0.5, 0.5]);
        assert!(out[0] > 0.5);
    }

    #[test]
    fn test_default_node_identity() {
        let node = CdlNode::default();
        assert!((node.saturation - 1.0).abs() < 1e-6);
    }
}
