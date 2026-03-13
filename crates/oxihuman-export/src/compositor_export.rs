// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Compositor node tree export settings.

/* ── legacy API (kept) ── */

pub struct CompositorExport {
    pub use_compositor: bool,
    pub use_sequencer: bool,
    pub use_nodes: bool,
    pub node_count: u32,
}

pub fn default_compositor_export() -> CompositorExport {
    CompositorExport {
        use_compositor: true,
        use_sequencer: false,
        use_nodes: true,
        node_count: 0,
    }
}

pub fn export_compositor_to_json(c: &CompositorExport) -> String {
    format!(
        r#"{{"use_compositor":{},"use_sequencer":{},"use_nodes":{},"node_count":{}}}"#,
        c.use_compositor, c.use_sequencer, c.use_nodes, c.node_count
    )
}

/* ── spec functions (wave 150B) ── */

/// Spec-style compositor node.
#[derive(Debug, Clone)]
pub struct CompositorNode {
    pub name: String,
    pub node_type: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub is_output: bool,
}

/// Create a new `CompositorNode`.
pub fn new_compositor_node(name: &str, node_type: &str) -> CompositorNode {
    CompositorNode {
        name: name.to_string(),
        node_type: node_type.to_string(),
        inputs: Vec::new(),
        outputs: Vec::new(),
        is_output: false,
    }
}

/// Add an input socket name.
pub fn comp_push_input(node: &mut CompositorNode, input: &str) {
    node.inputs.push(input.to_string());
}

/// Add an output socket name.
pub fn comp_push_output(node: &mut CompositorNode, output: &str) {
    node.outputs.push(output.to_string());
}

/// Serialize a single node to JSON.
pub fn comp_node_to_json(n: &CompositorNode) -> String {
    format!(
        "{{\"name\":\"{}\",\"type\":\"{}\",\"inputs\":{},\"outputs\":{},\"is_output\":{}}}",
        n.name,
        n.node_type,
        n.inputs.len(),
        n.outputs.len(),
        n.is_output
    )
}

/// Serialize multiple nodes to a JSON array.
pub fn comp_nodes_to_json(nodes: &[CompositorNode]) -> String {
    let inner: Vec<String> = nodes.iter().map(comp_node_to_json).collect();
    format!("[{}]", inner.join(","))
}

/// Returns true if the node is an output node.
pub fn comp_node_is_output(n: &CompositorNode) -> bool {
    n.is_output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_compositor_node() {
        let n = new_compositor_node("output", "CompositorNodeComposite");
        assert_eq!(n.name, "output");
    }

    #[test]
    fn test_comp_push_input() {
        let mut n = new_compositor_node("n", "T");
        comp_push_input(&mut n, "Image");
        assert_eq!(n.inputs.len(), 1);
    }

    #[test]
    fn test_comp_push_output() {
        let mut n = new_compositor_node("n", "T");
        comp_push_output(&mut n, "Value");
        assert_eq!(n.outputs.len(), 1);
    }

    #[test]
    fn test_comp_node_to_json() {
        let n = new_compositor_node("blur", "Blur");
        let j = comp_node_to_json(&n);
        assert!(j.contains("blur"));
    }

    #[test]
    fn test_comp_node_is_output() {
        let mut n = new_compositor_node("out", "Composite");
        n.is_output = true;
        assert!(comp_node_is_output(&n));
    }
}
