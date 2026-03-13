// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! FBX export v2 (ASCII format stub).

#[allow(dead_code)]
pub struct FbxNodeV2 {
    pub name: String,
    pub properties: Vec<(String, String)>,
}

#[allow(dead_code)]
pub struct FbxExportV2 {
    pub nodes: Vec<FbxNodeV2>,
    pub version: u32,
}

#[allow(dead_code)]
pub fn new_fbx_export_v2() -> FbxExportV2 {
    FbxExportV2 { nodes: Vec::new(), version: 7400 }
}

#[allow(dead_code)]
pub fn fbx2_add_node(e: &mut FbxExportV2, name: &str) -> usize {
    let idx = e.nodes.len();
    e.nodes.push(FbxNodeV2 { name: name.to_string(), properties: Vec::new() });
    idx
}

#[allow(dead_code)]
pub fn fbx2_set_property(e: &mut FbxExportV2, node_idx: usize, key: &str, value: &str) {
    if node_idx < e.nodes.len() {
        e.nodes[node_idx].properties.push((key.to_string(), value.to_string()));
    }
}

#[allow(dead_code)]
pub fn fbx2_node_count(e: &FbxExportV2) -> usize {
    e.nodes.len()
}

#[allow(dead_code)]
pub fn fbx2_to_ascii(e: &FbxExportV2) -> String {
    let mut out = format!("; FBX {}.0.0 project file\n", e.version);
    out.push_str("; Created by oxihuman\n\n");
    out.push_str("FBXHeaderExtension:  {\n");
    out.push_str(&format!("    FBXHeaderVersion: {}\n", e.version));
    out.push_str("}\n\n");
    for node in &e.nodes {
        out.push_str(&format!("{}:  {{\n", node.name));
        for (k, v) in &node.properties {
            out.push_str(&format!("    P: \"{}\", \"KString\", \"\", \"{}\"\n", k, v));
        }
        out.push_str("}\n\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let e = new_fbx_export_v2();
        assert_eq!(fbx2_node_count(&e), 0);
    }

    #[test]
    fn test_add_node() {
        let mut e = new_fbx_export_v2();
        let idx = fbx2_add_node(&mut e, "Objects");
        assert_eq!(idx, 0);
        assert_eq!(fbx2_node_count(&e), 1);
    }

    #[test]
    fn test_set_property() {
        let mut e = new_fbx_export_v2();
        fbx2_add_node(&mut e, "Objects");
        fbx2_set_property(&mut e, 0, "Scale", "1.0");
        assert_eq!(e.nodes[0].properties.len(), 1);
    }

    #[test]
    fn test_node_count() {
        let mut e = new_fbx_export_v2();
        for i in 0..3 {
            fbx2_add_node(&mut e, &format!("Node{}", i));
        }
        assert_eq!(fbx2_node_count(&e), 3);
    }

    #[test]
    fn test_to_ascii_contains_fbx() {
        let e = new_fbx_export_v2();
        let s = fbx2_to_ascii(&e);
        assert!(s.contains("FBX"));
    }

    #[test]
    fn test_to_ascii_contains_node_name() {
        let mut e = new_fbx_export_v2();
        fbx2_add_node(&mut e, "Geometry");
        let s = fbx2_to_ascii(&e);
        assert!(s.contains("Geometry"));
    }

    #[test]
    fn test_version_default() {
        let e = new_fbx_export_v2();
        assert_eq!(e.version, 7400);
    }

    #[test]
    fn test_property_in_ascii() {
        let mut e = new_fbx_export_v2();
        fbx2_add_node(&mut e, "Node");
        fbx2_set_property(&mut e, 0, "Color", "red");
        let s = fbx2_to_ascii(&e);
        assert!(s.contains("Color"));
    }
}
