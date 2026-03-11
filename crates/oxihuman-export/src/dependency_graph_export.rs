// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export an asset dependency graph as DOT (Graphviz) or JSON format.

#![allow(dead_code)]

/// Configuration for dependency-graph export.
#[derive(Debug, Clone)]
pub struct DepGraphExportConfig {
    /// Name of the graph in DOT output.
    pub graph_name: String,
    /// Whether to use directed edges (`digraph`) in DOT.
    pub directed: bool,
    /// Pretty-print JSON output.
    pub pretty: bool,
}

/// A node in the dependency graph.
#[derive(Debug, Clone)]
pub struct DepNode {
    /// Unique node identifier.
    pub id: String,
    /// Human-readable label.
    pub label: String,
    /// Optional type tag (e.g. "texture", "mesh", "shader").
    pub node_type: String,
}

/// A directed or undirected edge between two nodes.
#[derive(Debug, Clone)]
pub struct DepEdge {
    /// Source node id.
    pub from: String,
    /// Target node id.
    pub to: String,
    /// Optional edge label.
    pub label: String,
}

/// The dependency graph export container.
#[derive(Debug, Clone)]
pub struct DepGraphExport {
    /// All nodes.
    pub nodes: Vec<DepNode>,
    /// All edges.
    pub edges: Vec<DepEdge>,
    /// Total byte size of the last serialised output.
    pub total_bytes: usize,
}

/// Returns the default [`DepGraphExportConfig`].
pub fn default_dep_graph_export_config() -> DepGraphExportConfig {
    DepGraphExportConfig {
        graph_name: "asset_deps".to_string(),
        directed: true,
        pretty: true,
    }
}

/// Creates a new, empty [`DepGraphExport`].
pub fn new_dep_graph_export() -> DepGraphExport {
    DepGraphExport {
        nodes: Vec::new(),
        edges: Vec::new(),
        total_bytes: 0,
    }
}

/// Adds a node to the graph.
pub fn dep_graph_add_node(graph: &mut DepGraphExport, node: DepNode) {
    graph.nodes.push(node);
}

/// Adds an edge to the graph.
pub fn dep_graph_add_edge(graph: &mut DepGraphExport, edge: DepEdge) {
    graph.edges.push(edge);
}

/// Serialises the graph as a DOT string.
pub fn dep_graph_to_dot(graph: &mut DepGraphExport, cfg: &DepGraphExportConfig) -> String {
    let kw = if cfg.directed { "digraph" } else { "graph" };
    let arrow = if cfg.directed { "->" } else { "--" };
    let mut out = format!("{} {} {{\n", kw, cfg.graph_name);
    for n in &graph.nodes {
        out.push_str(&format!(
            "  \"{}\" [label=\"{}\" type=\"{}\"];\n",
            n.id, n.label, n.node_type
        ));
    }
    for e in &graph.edges {
        if e.label.is_empty() {
            out.push_str(&format!("  \"{}\" {} \"{}\";\n", e.from, arrow, e.to));
        } else {
            out.push_str(&format!(
                "  \"{}\" {} \"{}\" [label=\"{}\"];\n",
                e.from, arrow, e.to, e.label
            ));
        }
    }
    out.push('}');
    graph.total_bytes = out.len();
    out
}

/// Serialises the graph as a JSON string.
pub fn dep_graph_to_json(graph: &mut DepGraphExport, cfg: &DepGraphExportConfig) -> String {
    let indent = if cfg.pretty { "  " } else { "" };
    let nl = if cfg.pretty { "\n" } else { "" };
    let mut out = format!("{{{nl}");

    // nodes array
    out.push_str(&format!("{indent}\"nodes\":[{nl}"));
    for (i, n) in graph.nodes.iter().enumerate() {
        let comma = if i + 1 < graph.nodes.len() { "," } else { "" };
        out.push_str(&format!(
            "{indent}{indent}{{\"id\":\"{}\",\"label\":\"{}\",\"type\":\"{}\"}}{}{nl}",
            n.id, n.label, n.node_type, comma
        ));
    }
    out.push_str(&format!("{indent}],{nl}"));

    // edges array
    out.push_str(&format!("{indent}\"edges\":[{nl}"));
    for (i, e) in graph.edges.iter().enumerate() {
        let comma = if i + 1 < graph.edges.len() { "," } else { "" };
        out.push_str(&format!(
            "{indent}{indent}{{\"from\":\"{}\",\"to\":\"{}\",\"label\":\"{}\"}}{}{nl}",
            e.from, e.to, e.label, comma
        ));
    }
    out.push_str(&format!("{indent}]{nl}}}"));

    graph.total_bytes = out.len();
    out
}

/// Returns the number of nodes in the graph.
pub fn dep_graph_node_count(graph: &DepGraphExport) -> usize {
    graph.nodes.len()
}

/// Returns the number of edges in the graph.
pub fn dep_graph_edge_count(graph: &DepGraphExport) -> usize {
    graph.edges.len()
}

/// Writes DOT output to a file path (stub — returns byte count).
pub fn dep_graph_write_to_file(
    graph: &mut DepGraphExport,
    cfg: &DepGraphExportConfig,
    _path: &str,
) -> usize {
    let dot = dep_graph_to_dot(graph, cfg);
    graph.total_bytes = dot.len();
    graph.total_bytes
}

/// Clears all nodes and edges, resets state.
pub fn dep_graph_clear(graph: &mut DepGraphExport) {
    graph.nodes.clear();
    graph.edges.clear();
    graph.total_bytes = 0;
}

// ── internal helpers ───────────────────────────────────────────────────────────

fn make_node(id: &str, label: &str, node_type: &str) -> DepNode {
    DepNode {
        id: id.to_string(),
        label: label.to_string(),
        node_type: node_type.to_string(),
    }
}

fn make_edge(from: &str, to: &str, label: &str) -> DepEdge {
    DepEdge {
        from: from.to_string(),
        to: to.to_string(),
        label: label.to_string(),
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_dep_graph_export_config();
        assert!(cfg.directed);
        assert!(cfg.pretty);
        assert_eq!(cfg.graph_name, "asset_deps");
    }

    #[test]
    fn new_graph_is_empty() {
        let g = new_dep_graph_export();
        assert_eq!(dep_graph_node_count(&g), 0);
        assert_eq!(dep_graph_edge_count(&g), 0);
    }

    #[test]
    fn add_node_increments_count() {
        let mut g = new_dep_graph_export();
        dep_graph_add_node(&mut g, make_node("n1", "Body Mesh", "mesh"));
        assert_eq!(dep_graph_node_count(&g), 1);
    }

    #[test]
    fn add_edge_increments_count() {
        let mut g = new_dep_graph_export();
        dep_graph_add_node(&mut g, make_node("n1", "Mesh", "mesh"));
        dep_graph_add_node(&mut g, make_node("n2", "Texture", "texture"));
        dep_graph_add_edge(&mut g, make_edge("n1", "n2", "uses"));
        assert_eq!(dep_graph_edge_count(&g), 1);
    }

    #[test]
    fn dot_output_contains_digraph() {
        let mut g = new_dep_graph_export();
        dep_graph_add_node(&mut g, make_node("n1", "Mesh", "mesh"));
        let cfg = default_dep_graph_export_config();
        let dot = dep_graph_to_dot(&mut g, &cfg);
        assert!(dot.contains("digraph"));
        assert!(dot.contains("n1"));
    }

    #[test]
    fn dot_undirected_uses_graph_keyword() {
        let mut g = new_dep_graph_export();
        dep_graph_add_node(&mut g, make_node("n1", "Mesh", "mesh"));
        let cfg = DepGraphExportConfig {
            graph_name: "deps".to_string(),
            directed: false,
            pretty: true,
        };
        let dot = dep_graph_to_dot(&mut g, &cfg);
        assert!(dot.contains("graph deps"));
        assert!(!dot.contains("digraph"));
    }

    #[test]
    fn json_output_contains_nodes_and_edges() {
        let mut g = new_dep_graph_export();
        dep_graph_add_node(&mut g, make_node("n1", "Mesh", "mesh"));
        dep_graph_add_edge(&mut g, make_edge("n1", "n2", ""));
        let cfg = default_dep_graph_export_config();
        let json = dep_graph_to_json(&mut g, &cfg);
        assert!(json.contains("\"nodes\""));
        assert!(json.contains("\"edges\""));
    }

    #[test]
    fn write_to_file_sets_total_bytes() {
        let mut g = new_dep_graph_export();
        dep_graph_add_node(&mut g, make_node("n1", "Mesh", "mesh"));
        let cfg = default_dep_graph_export_config();
        let n = dep_graph_write_to_file(&mut g, &cfg, "/tmp/deps.dot");
        assert!(n > 0);
        assert_eq!(g.total_bytes, n);
    }

    #[test]
    fn clear_resets_state() {
        let mut g = new_dep_graph_export();
        dep_graph_add_node(&mut g, make_node("n1", "Mesh", "mesh"));
        dep_graph_add_edge(&mut g, make_edge("n1", "n2", ""));
        let cfg = default_dep_graph_export_config();
        dep_graph_write_to_file(&mut g, &cfg, "/tmp/deps.dot");
        dep_graph_clear(&mut g);
        assert_eq!(dep_graph_node_count(&g), 0);
        assert_eq!(dep_graph_edge_count(&g), 0);
        assert_eq!(g.total_bytes, 0);
    }
}
