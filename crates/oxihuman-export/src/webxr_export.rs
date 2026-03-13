#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export WebXR scene manifest.

#[allow(dead_code)]
pub struct WebXrNode {
    pub name: String,
    pub mesh_ref: Option<String>,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

#[allow(dead_code)]
pub struct WebXrScene {
    pub nodes: Vec<WebXrNode>,
}

#[allow(dead_code)]
pub fn new_webxr_scene() -> WebXrScene {
    WebXrScene { nodes: Vec::new() }
}

#[allow(dead_code)]
pub fn add_node(scene: &mut WebXrScene, name: &str, pos: [f32; 3]) -> usize {
    let idx = scene.nodes.len();
    scene.nodes.push(WebXrNode {
        name: name.to_string(),
        mesh_ref: None,
        position: pos,
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0, 1.0, 1.0],
    });
    idx
}

#[allow(dead_code)]
pub fn node_count(scene: &WebXrScene) -> usize {
    scene.nodes.len()
}

#[allow(dead_code)]
pub fn export_webxr_to_json(scene: &WebXrScene) -> String {
    let mut s = "{\"nodes\":[".to_string();
    for (i, n) in scene.nodes.iter().enumerate() {
        if i > 0 { s.push(','); }
        let mesh = n.mesh_ref.as_deref().unwrap_or("null");
        s.push_str(&format!(
            "{{\"name\":\"{}\",\"mesh\":\"{}\",\"position\":[{},{},{}]}}",
            n.name, mesh, n.position[0], n.position[1], n.position[2]
        ));
    }
    s.push_str("]}");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_scene_empty() {
        let s = new_webxr_scene();
        assert!(s.nodes.is_empty());
    }

    #[test]
    fn add_node_returns_index() {
        let mut s = new_webxr_scene();
        let idx = add_node(&mut s, "root", [0.0, 0.0, 0.0]);
        assert_eq!(idx, 0);
    }

    #[test]
    fn add_second_node_returns_one() {
        let mut s = new_webxr_scene();
        add_node(&mut s, "a", [0.0, 0.0, 0.0]);
        let idx = add_node(&mut s, "b", [1.0, 0.0, 0.0]);
        assert_eq!(idx, 1);
    }

    #[test]
    fn node_count_correct() {
        let mut s = new_webxr_scene();
        add_node(&mut s, "a", [0.0, 0.0, 0.0]);
        add_node(&mut s, "b", [1.0, 0.0, 0.0]);
        assert_eq!(node_count(&s), 2);
    }

    #[test]
    fn node_name_stored() {
        let mut s = new_webxr_scene();
        add_node(&mut s, "myNode", [0.0, 0.0, 0.0]);
        assert_eq!(s.nodes[0].name, "myNode");
    }

    #[test]
    fn node_position_stored() {
        let mut s = new_webxr_scene();
        add_node(&mut s, "n", [1.0, 2.0, 3.0]);
        assert!((s.nodes[0].position[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn node_default_rotation_identity() {
        let mut s = new_webxr_scene();
        add_node(&mut s, "n", [0.0, 0.0, 0.0]);
        assert!((s.nodes[0].rotation[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn export_json_contains_name() {
        let mut s = new_webxr_scene();
        add_node(&mut s, "scene_root", [0.0, 0.0, 0.0]);
        let j = export_webxr_to_json(&s);
        assert!(j.contains("scene_root"));
    }

    #[test]
    fn export_json_empty_scene() {
        let s = new_webxr_scene();
        let j = export_webxr_to_json(&s);
        assert!(j.contains("nodes"));
    }
}
