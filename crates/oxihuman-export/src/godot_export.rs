// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Godot scene (.tscn stub) export.

/// A Godot node descriptor.
#[derive(Debug, Clone)]
pub struct GodotNode {
    pub name: String,
    pub type_: String,
    pub parent: Option<String>,
    pub position: [f32; 3],
}

impl GodotNode {
    pub fn new(name: &str, type_: &str) -> Self {
        Self {
            name: name.to_string(),
            type_: type_.to_string(),
            parent: None,
            position: [0.0; 3],
        }
    }

    pub fn with_parent(mut self, parent: &str) -> Self {
        self.parent = Some(parent.to_string());
        self
    }

    pub fn at(mut self, pos: [f32; 3]) -> Self {
        self.position = pos;
        self
    }
}

/// A Godot resource reference.
#[derive(Debug, Clone)]
pub struct GodotResource {
    pub path: String,
    pub resource_type: String,
}

impl GodotResource {
    pub fn new(path: &str, resource_type: &str) -> Self {
        Self { path: path.to_string(), resource_type: resource_type.to_string() }
    }
}

/// Godot scene export.
#[derive(Debug, Clone, Default)]
pub struct GodotExport {
    pub load_steps: u32,
    pub format: u32,
    pub nodes: Vec<GodotNode>,
    pub resources: Vec<GodotResource>,
}

impl GodotExport {
    pub fn new() -> Self {
        Self { load_steps: 1, format: 2, ..Default::default() }
    }

    pub fn add_node(&mut self, node: GodotNode) {
        self.nodes.push(node);
    }

    pub fn add_resource(&mut self, res: GodotResource) {
        self.resources.push(res);
        self.load_steps += 1;
    }
}

/// Serialize to Godot .tscn format.
pub fn to_godot_tscn(d: &GodotExport) -> String {
    let mut out = format!(
        "[gd_scene load_steps={} format={}]\n\n",
        d.load_steps, d.format
    );
    for r in &d.resources {
        out.push_str(&format!(
            "[ext_resource type=\"{}\" path=\"{}\"] \n",
            r.resource_type, r.path
        ));
    }
    for n in &d.nodes {
        let parent_str = if let Some(p) = &n.parent {
            format!(" parent=\"{}\"", p)
        } else {
            String::new()
        };
        out.push_str(&format!(
            "[node name=\"{}\" type=\"{}\"{}]\n",
            n.name, n.type_, parent_str
        ));
        out.push_str(&format!(
            "transform/pos = Vector3({}, {}, {})\n\n",
            n.position[0], n.position[1], n.position[2]
        ));
    }
    out
}

/// Count nodes.
pub fn godot_node_count(d: &GodotExport) -> usize {
    d.nodes.len()
}

/// Count resources.
pub fn godot_resource_count(d: &GodotExport) -> usize {
    d.resources.len()
}

/// Create a new Godot export.
pub fn new_godot_export() -> GodotExport {
    GodotExport::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_godot_export() {
        let d = new_godot_export();
        assert_eq!(d.format, 2);
    }

    #[test]
    fn test_add_node() {
        let mut d = GodotExport::new();
        d.add_node(GodotNode::new("Player", "KinematicBody"));
        assert_eq!(godot_node_count(&d), 1);
    }

    #[test]
    fn test_add_resource() {
        let mut d = GodotExport::new();
        d.add_resource(GodotResource::new("res://mesh.glb", "PackedScene"));
        assert_eq!(godot_resource_count(&d), 1);
    }

    #[test]
    fn test_to_tscn_header() {
        let d = GodotExport::new();
        let s = to_godot_tscn(&d);
        assert!(s.contains("gd_scene"));
    }

    #[test]
    fn test_to_tscn_has_node() {
        let mut d = GodotExport::new();
        d.add_node(GodotNode::new("Root", "Spatial"));
        let s = to_godot_tscn(&d);
        assert!(s.contains("Root"));
        assert!(s.contains("Spatial"));
    }

    #[test]
    fn test_to_tscn_has_resource() {
        let mut d = GodotExport::new();
        d.add_resource(GodotResource::new("res://mat.tres", "Material"));
        let s = to_godot_tscn(&d);
        assert!(s.contains("Material"));
    }

    #[test]
    fn test_node_with_parent() {
        let n = GodotNode::new("Child", "Spatial").with_parent("Root");
        assert_eq!(n.parent.as_deref(), Some("Root"));
    }

    #[test]
    fn test_node_at_position() {
        let n = GodotNode::new("N", "Spatial").at([1.0, 2.0, 3.0]);
        assert!((n.position[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_load_steps_increments_with_resource() {
        let mut d = GodotExport::new();
        let initial = d.load_steps;
        d.add_resource(GodotResource::new("res://a.glb", "PackedScene"));
        assert_eq!(d.load_steps, initial + 1);
    }

    #[test]
    fn test_to_tscn_node_with_parent_in_output() {
        let mut d = GodotExport::new();
        d.add_node(GodotNode::new("Child", "Node").with_parent("."));
        let s = to_godot_tscn(&d);
        assert!(s.contains("parent="));
    }
}
