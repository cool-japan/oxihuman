// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Houdini HIP scene stub export.

/// HIP format type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HipFormat {
    Hip,
    Hiplc,
    Hipnc,
}

impl HipFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            HipFormat::Hip => "hip",
            HipFormat::Hiplc => "hiplc",
            HipFormat::Hipnc => "hipnc",
        }
    }
}

/// A HIP node stub.
#[derive(Debug, Clone)]
pub struct HipNode {
    pub name: String,
    pub node_type: String,
    pub parms: Vec<(String, String)>,
}

/// A HIP scene export stub.
#[derive(Debug, Clone)]
pub struct HipExport {
    pub format: HipFormat,
    pub hip_version: String,
    pub nodes: Vec<HipNode>,
}

/// Create a new HIP export.
pub fn new_hip_export(format: HipFormat) -> HipExport {
    HipExport {
        format,
        hip_version: "20.0.506".to_string(),
        nodes: Vec::new(),
    }
}

/// Add a node to the HIP scene.
pub fn hip_add_node(export: &mut HipExport, name: &str, node_type: &str) {
    export.nodes.push(HipNode {
        name: name.to_string(),
        node_type: node_type.to_string(),
        parms: Vec::new(),
    });
}

/// Set a parameter on the last added node.
pub fn hip_set_parm(export: &mut HipExport, key: &str, value: &str) {
    if let Some(node) = export.nodes.last_mut() {
        node.parms.push((key.to_string(), value.to_string()));
    }
}

/// Return the node count.
pub fn hip_node_count(export: &HipExport) -> usize {
    export.nodes.len()
}

/// Validate the HIP export.
pub fn validate_hip(export: &HipExport) -> bool {
    !export.hip_version.is_empty()
}

/// Generate a stub HIP script string.
pub fn hip_to_string(export: &HipExport) -> String {
    let mut out = format!("# Houdini {} Scene\n", export.hip_version);
    for node in &export.nodes {
        out.push_str(&format!("opadd -e {} {}\n", node.node_type, node.name));
        for (k, v) in &node.parms {
            out.push_str(&format!("opparm {} {} ({})\n", node.name, k, v));
        }
    }
    out
}

/// Estimate the HIP file size in bytes.
pub fn hip_size_estimate(export: &HipExport) -> usize {
    hip_to_string(export).len()
}

/// Find a node by name.
pub fn hip_find_node<'a>(export: &'a HipExport, name: &str) -> Option<&'a HipNode> {
    export.nodes.iter().find(|n| n.name == name)
}

/// Get the file extension for this format.
pub fn hip_extension(export: &HipExport) -> &'static str {
    export.format.extension()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_hip_export() {
        let exp = new_hip_export(HipFormat::Hip);
        assert_eq!(exp.format, HipFormat::Hip);
        assert_eq!(hip_node_count(&exp), 0);
    }

    #[test]
    fn test_add_node() {
        let mut exp = new_hip_export(HipFormat::Hip);
        hip_add_node(&mut exp, "geo1", "geo");
        assert_eq!(hip_node_count(&exp), 1);
    }

    #[test]
    fn test_set_parm() {
        let mut exp = new_hip_export(HipFormat::Hip);
        hip_add_node(&mut exp, "geo1", "geo");
        hip_set_parm(&mut exp, "tx", "1.0");
        assert_eq!(exp.nodes[0].parms.len(), 1);
    }

    #[test]
    fn test_validate() {
        let exp = new_hip_export(HipFormat::Hiplc);
        assert!(validate_hip(&exp));
    }

    #[test]
    fn test_to_string() {
        let mut exp = new_hip_export(HipFormat::Hip);
        hip_add_node(&mut exp, "geo1", "geo");
        let s = hip_to_string(&exp);
        assert!(s.contains("geo1"));
    }

    #[test]
    fn test_size_estimate() {
        let exp = new_hip_export(HipFormat::Hip);
        assert!(hip_size_estimate(&exp) > 0);
    }

    #[test]
    fn test_find_node() {
        let mut exp = new_hip_export(HipFormat::Hip);
        hip_add_node(&mut exp, "mynode", "null");
        let found = hip_find_node(&exp, "mynode");
        assert!(found.is_some());
    }

    #[test]
    fn test_extension() {
        let exp = new_hip_export(HipFormat::Hipnc);
        assert_eq!(hip_extension(&exp), "hipnc");
    }

    #[test]
    fn test_format_extensions() {
        assert_eq!(HipFormat::Hip.extension(), "hip");
        assert_eq!(HipFormat::Hiplc.extension(), "hiplc");
        assert_eq!(HipFormat::Hipnc.extension(), "hipnc");
    }
}
