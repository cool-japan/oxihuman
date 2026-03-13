// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! FrameGraph — render frame graph for pass scheduling.

#![allow(dead_code)]

/// A single render pass in the frame graph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderPass {
    pub name: String,
    pub dependencies: Vec<String>,
    pub executed: bool,
}

/// A frame graph holding ordered render passes.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct FrameGraph {
    pub passes: Vec<RenderPass>,
}

/// Create an empty frame graph.
#[allow(dead_code)]
pub fn new_frame_graph() -> FrameGraph {
    FrameGraph::default()
}

/// Add a render pass with the given name and dependencies.
#[allow(dead_code)]
pub fn add_render_pass(graph: &mut FrameGraph, name: &str, dependencies: &[&str]) {
    graph.passes.push(RenderPass {
        name: name.to_owned(),
        dependencies: dependencies.iter().map(|s| (*s).to_owned()).collect(),
        executed: false,
    });
}

/// Number of passes in the graph.
#[allow(dead_code)]
pub fn pass_count(graph: &FrameGraph) -> usize {
    graph.passes.len()
}

/// Execute all passes (stub — marks them as executed).
#[allow(dead_code)]
pub fn execute_frame_graph(graph: &mut FrameGraph) {
    for pass in &mut graph.passes {
        pass.executed = true;
    }
}

/// Return the name of a pass by index.
#[allow(dead_code)]
pub fn pass_name(graph: &FrameGraph, index: usize) -> Option<&str> {
    graph.passes.get(index).map(|p| p.name.as_str())
}

/// Return the dependencies of a pass by index.
#[allow(dead_code)]
pub fn pass_dependencies(graph: &FrameGraph, index: usize) -> Option<&[String]> {
    graph.passes.get(index).map(|p| p.dependencies.as_slice())
}

/// Export the frame graph as a DOT-format string for visualization.
#[allow(dead_code)]
pub fn frame_graph_to_dot(graph: &FrameGraph) -> String {
    let mut dot = String::from("digraph FrameGraph {\n");
    for pass in &graph.passes {
        for dep in &pass.dependencies {
            dot.push_str(&format!("  \"{}\" -> \"{}\";\n", dep, pass.name));
        }
    }
    dot.push('}');
    dot
}

/// Validate the frame graph: all dependency names must exist as pass names.
#[allow(dead_code)]
pub fn validate_frame_graph(graph: &FrameGraph) -> bool {
    let names: Vec<&str> = graph.passes.iter().map(|p| p.name.as_str()).collect();
    for pass in &graph.passes {
        for dep in &pass.dependencies {
            if !names.contains(&dep.as_str()) {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_frame_graph() {
        let g = new_frame_graph();
        assert_eq!(pass_count(&g), 0);
    }

    #[test]
    fn test_add_render_pass() {
        let mut g = new_frame_graph();
        add_render_pass(&mut g, "depth", &[]);
        assert_eq!(pass_count(&g), 1);
    }

    #[test]
    fn test_pass_count() {
        let mut g = new_frame_graph();
        add_render_pass(&mut g, "depth", &[]);
        add_render_pass(&mut g, "color", &["depth"]);
        assert_eq!(pass_count(&g), 2);
    }

    #[test]
    fn test_execute_frame_graph() {
        let mut g = new_frame_graph();
        add_render_pass(&mut g, "depth", &[]);
        execute_frame_graph(&mut g);
        assert!(g.passes[0].executed);
    }

    #[test]
    fn test_pass_name() {
        let mut g = new_frame_graph();
        add_render_pass(&mut g, "shadow", &[]);
        assert_eq!(pass_name(&g, 0), Some("shadow"));
        assert_eq!(pass_name(&g, 1), None);
    }

    #[test]
    fn test_pass_dependencies() {
        let mut g = new_frame_graph();
        add_render_pass(&mut g, "depth", &[]);
        add_render_pass(&mut g, "color", &["depth"]);
        let deps = pass_dependencies(&g, 1).expect("should succeed");
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], "depth");
    }

    #[test]
    fn test_frame_graph_to_dot() {
        let mut g = new_frame_graph();
        add_render_pass(&mut g, "depth", &[]);
        add_render_pass(&mut g, "color", &["depth"]);
        let dot = frame_graph_to_dot(&g);
        assert!(dot.contains("digraph"));
        assert!(dot.contains("depth"));
    }

    #[test]
    fn test_validate_frame_graph_valid() {
        let mut g = new_frame_graph();
        add_render_pass(&mut g, "depth", &[]);
        add_render_pass(&mut g, "color", &["depth"]);
        assert!(validate_frame_graph(&g));
    }

    #[test]
    fn test_validate_frame_graph_invalid() {
        let mut g = new_frame_graph();
        add_render_pass(&mut g, "color", &["missing"]);
        assert!(!validate_frame_graph(&g));
    }

    #[test]
    fn test_pass_dependencies_empty() {
        let mut g = new_frame_graph();
        add_render_pass(&mut g, "depth", &[]);
        let deps = pass_dependencies(&g, 0).expect("should succeed");
        assert!(deps.is_empty());
    }
}
