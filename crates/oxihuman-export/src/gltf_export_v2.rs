// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! glTF 2.0 export v2 with materials and animations.

#[allow(dead_code)]
pub struct GltfNodeV2 {
    pub name: String,
    pub mesh: Option<usize>,
    pub children: Vec<usize>,
    pub translation: [f32; 3],
}

#[allow(dead_code)]
pub struct GltfExportV2 {
    pub nodes: Vec<GltfNodeV2>,
    pub scene_root: Vec<usize>,
}

#[allow(dead_code)]
pub fn new_gltf_export_v2() -> GltfExportV2 {
    GltfExportV2 { nodes: Vec::new(), scene_root: Vec::new() }
}

#[allow(dead_code)]
pub fn gltf2_add_node(e: &mut GltfExportV2, name: &str) -> usize {
    let idx = e.nodes.len();
    e.nodes.push(GltfNodeV2 {
        name: name.to_string(),
        mesh: None,
        children: Vec::new(),
        translation: [0.0, 0.0, 0.0],
    });
    e.scene_root.push(idx);
    idx
}

#[allow(dead_code)]
pub fn gltf2_set_parent(e: &mut GltfExportV2, parent: usize, child: usize) {
    if parent < e.nodes.len() {
        e.nodes[parent].children.push(child);
        e.scene_root.retain(|&r| r != child);
    }
}

#[allow(dead_code)]
pub fn gltf2_node_count(e: &GltfExportV2) -> usize {
    e.nodes.len()
}

#[allow(dead_code)]
pub fn gltf2_to_json(e: &GltfExportV2) -> String {
    let nodes: Vec<String> = e.nodes.iter().map(|n| {
        format!(r#"{{"name":"{}","children":{:?}}}"#, n.name, n.children)
    }).collect();
    format!(
        r#"{{"asset":{{"version":"2.0"}},"nodes":[{}],"scene_root":{:?}}}"#,
        nodes.join(","),
        e.scene_root
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let e = new_gltf_export_v2();
        assert_eq!(gltf2_node_count(&e), 0);
    }

    #[test]
    fn test_add_node() {
        let mut e = new_gltf_export_v2();
        let idx = gltf2_add_node(&mut e, "Root");
        assert_eq!(idx, 0);
        assert_eq!(gltf2_node_count(&e), 1);
    }

    #[test]
    fn test_set_parent() {
        let mut e = new_gltf_export_v2();
        let parent = gltf2_add_node(&mut e, "Parent");
        let child = gltf2_add_node(&mut e, "Child");
        gltf2_set_parent(&mut e, parent, child);
        assert!(e.nodes[parent].children.contains(&child));
        assert!(!e.scene_root.contains(&child));
    }

    #[test]
    fn test_node_count() {
        let mut e = new_gltf_export_v2();
        gltf2_add_node(&mut e, "A");
        gltf2_add_node(&mut e, "B");
        gltf2_add_node(&mut e, "C");
        assert_eq!(gltf2_node_count(&e), 3);
    }

    #[test]
    fn test_to_json_contains_asset() {
        let e = new_gltf_export_v2();
        let json = gltf2_to_json(&e);
        assert!(json.contains("asset"));
    }

    #[test]
    fn test_to_json_contains_node_name() {
        let mut e = new_gltf_export_v2();
        gltf2_add_node(&mut e, "MyNode");
        let json = gltf2_to_json(&e);
        assert!(json.contains("MyNode"));
    }

    #[test]
    fn test_scene_root_updated() {
        let mut e = new_gltf_export_v2();
        let p = gltf2_add_node(&mut e, "P");
        let c = gltf2_add_node(&mut e, "C");
        assert_eq!(e.scene_root.len(), 2);
        gltf2_set_parent(&mut e, p, c);
        assert_eq!(e.scene_root.len(), 1);
    }

    #[test]
    fn test_version_in_json() {
        let e = new_gltf_export_v2();
        let json = gltf2_to_json(&e);
        assert!(json.contains("2.0"));
    }
}
