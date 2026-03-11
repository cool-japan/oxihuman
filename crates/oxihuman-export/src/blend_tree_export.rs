// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]

/// Blend tree export for animation state machines.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendTreeExport {
    pub nodes: Vec<BlendNode>,
}

/// A node in the blend tree.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendNode {
    pub name: String,
    pub node_type: BlendNodeType,
    pub children: Vec<usize>,
    pub weight: f32,
}

/// Type of blend node.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum BlendNodeType {
    Clip,
    Blend1D,
    Blend2D,
    Additive,
}

#[allow(dead_code)]
impl BlendTreeExport {
    pub fn new() -> Self { Self { nodes: Vec::new() } }

    pub fn add_clip(&mut self, name: &str) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(BlendNode { name: name.to_string(), node_type: BlendNodeType::Clip, children: vec![], weight: 1.0 });
        idx
    }

    pub fn add_blend(&mut self, name: &str, children: Vec<usize>, weight: f32) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(BlendNode { name: name.to_string(), node_type: BlendNodeType::Blend1D, children, weight });
        idx
    }

    pub fn node_count(&self) -> usize { self.nodes.len() }

    pub fn leaf_count(&self) -> usize {
        self.nodes.iter().filter(|n| n.children.is_empty()).count()
    }

    pub fn find(&self, name: &str) -> Option<usize> {
        self.nodes.iter().position(|n| n.name == name)
    }

    pub fn to_json(&self) -> String {
        let mut s = String::from("[");
        for (i, n) in self.nodes.iter().enumerate() {
            if i > 0 { s.push(','); }
            s.push_str(&format!("{{\"name\":\"{}\",\"children\":{}}}", n.name, n.children.len()));
        }
        s.push(']');
        s
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self.nodes.len() as u32).to_le_bytes());
        for n in &self.nodes {
            bytes.extend_from_slice(&n.weight.to_le_bytes());
            bytes.extend_from_slice(&(n.children.len() as u32).to_le_bytes());
            for &c in &n.children { bytes.extend_from_slice(&(c as u32).to_le_bytes()); }
        }
        bytes
    }
}

impl Default for BlendTreeExport {
    fn default() -> Self { Self::new() }
}

/// Validate blend tree.
#[allow(dead_code)]
pub fn validate_blend_tree(bt: &BlendTreeExport) -> bool {
    bt.nodes.iter().all(|n| n.children.iter().all(|&c| c < bt.nodes.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> BlendTreeExport {
        let mut bt = BlendTreeExport::new();
        let c0 = bt.add_clip("idle");
        let c1 = bt.add_clip("walk");
        bt.add_blend("locomotion", vec![c0, c1], 0.5);
        bt
    }

    #[test]
    fn test_node_count() { assert_eq!(sample().node_count(), 3); }

    #[test]
    fn test_leaf_count() { assert_eq!(sample().leaf_count(), 2); }

    #[test]
    fn test_find() { assert_eq!(sample().find("idle"), Some(0)); }

    #[test]
    fn test_find_missing() { assert!(sample().find("run").is_none()); }

    #[test]
    fn test_validate() { assert!(validate_blend_tree(&sample())); }

    #[test]
    fn test_to_json() { assert!(sample().to_json().contains("idle")); }

    #[test]
    fn test_to_bytes() { assert!(!sample().to_bytes().is_empty()); }

    #[test]
    fn test_empty() { assert_eq!(BlendTreeExport::new().node_count(), 0); }

    #[test]
    fn test_default() { assert_eq!(BlendTreeExport::default().node_count(), 0); }

    #[test]
    fn test_invalid_child() {
        let mut bt = BlendTreeExport::new();
        bt.nodes.push(BlendNode { name: "bad".to_string(), node_type: BlendNodeType::Blend1D, children: vec![99], weight: 1.0 });
        assert!(!validate_blend_tree(&bt));
    }
}
