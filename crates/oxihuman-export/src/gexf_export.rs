//! GEXF (Graph Exchange XML Format) export for skeleton/joint graph data.

use std::fs;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct GexfExportConfig {
    pub version: String,
    pub default_edge_type: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct GexfNode {
    pub id: u32,
    pub label: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct GexfEdge {
    pub source: u32,
    pub target: u32,
    pub weight: f32,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct GexfGraph {
    pub config: GexfExportConfig,
    pub nodes: Vec<GexfNode>,
    pub edges: Vec<GexfEdge>,
}

#[allow(dead_code)]
pub fn default_gexf_config() -> GexfExportConfig {
    GexfExportConfig {
        version: "1.3".to_string(),
        default_edge_type: "directed".to_string(),
    }
}

#[allow(dead_code)]
pub fn new_gexf_graph(cfg: &GexfExportConfig) -> GexfGraph {
    GexfGraph {
        config: cfg.clone(),
        nodes: Vec::new(),
        edges: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn gexf_add_node(graph: &mut GexfGraph, id: u32, label: &str) {
    graph.nodes.push(GexfNode {
        id,
        label: label.to_string(),
    });
}

#[allow(dead_code)]
pub fn gexf_add_edge(graph: &mut GexfGraph, source: u32, target: u32, weight: f32) {
    graph.edges.push(GexfEdge { source, target, weight });
}

#[allow(dead_code)]
pub fn gexf_to_xml_string(graph: &GexfGraph) -> String {
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str(&format!(
        "<gexf xmlns=\"http://gexf.net/{}\" version=\"{}\">\n",
        graph.config.version, graph.config.version
    ));
    xml.push_str(&format!(
        "  <graph defaultedgetype=\"{}\">\n",
        graph.config.default_edge_type
    ));
    xml.push_str("    <nodes>\n");
    for node in &graph.nodes {
        xml.push_str(&format!(
            "      <node id=\"{}\" label=\"{}\"/>\n",
            node.id, node.label
        ));
    }
    xml.push_str("    </nodes>\n");
    xml.push_str("    <edges>\n");
    for (idx, edge) in graph.edges.iter().enumerate() {
        xml.push_str(&format!(
            "      <edge id=\"{}\" source=\"{}\" target=\"{}\" weight=\"{}\"/>\n",
            idx, edge.source, edge.target, edge.weight
        ));
    }
    xml.push_str("    </edges>\n");
    xml.push_str("  </graph>\n");
    xml.push_str("</gexf>\n");
    xml
}

#[allow(dead_code)]
pub fn gexf_write_to_file(graph: &GexfGraph, path: &str) -> Result<(), String> {
    let xml = gexf_to_xml_string(graph);
    fs::write(path, xml).map_err(|e| e.to_string())
}

#[allow(dead_code)]
pub fn gexf_node_count(graph: &GexfGraph) -> usize {
    graph.nodes.len()
}

#[allow(dead_code)]
pub fn gexf_edge_count(graph: &GexfGraph) -> usize {
    graph.edges.len()
}

#[allow(dead_code)]
pub fn gexf_graph_clear(graph: &mut GexfGraph) {
    graph.nodes.clear();
    graph.edges.clear();
}

#[allow(dead_code)]
pub fn gexf_from_skeleton(joint_names: &[&str], parents: &[i32]) -> GexfGraph {
    let cfg = default_gexf_config();
    let mut graph = new_gexf_graph(&cfg);
    for (i, name) in joint_names.iter().enumerate() {
        gexf_add_node(&mut graph, i as u32, name);
    }
    for (i, &parent) in parents.iter().enumerate() {
        if parent >= 0 {
            gexf_add_edge(&mut graph, parent as u32, i as u32, 1.0);
        }
    }
    graph
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_gexf_config();
        assert_eq!(cfg.version, "1.3");
        assert_eq!(cfg.default_edge_type, "directed");
    }

    #[test]
    fn test_new_graph_empty() {
        let cfg = default_gexf_config();
        let g = new_gexf_graph(&cfg);
        assert_eq!(gexf_node_count(&g), 0);
        assert_eq!(gexf_edge_count(&g), 0);
    }

    #[test]
    fn test_add_node() {
        let cfg = default_gexf_config();
        let mut g = new_gexf_graph(&cfg);
        gexf_add_node(&mut g, 0, "Root");
        assert_eq!(gexf_node_count(&g), 1);
        assert_eq!(g.nodes[0].label, "Root");
    }

    #[test]
    fn test_add_edge() {
        let cfg = default_gexf_config();
        let mut g = new_gexf_graph(&cfg);
        gexf_add_node(&mut g, 0, "A");
        gexf_add_node(&mut g, 1, "B");
        gexf_add_edge(&mut g, 0, 1, 1.5);
        assert_eq!(gexf_edge_count(&g), 1);
        assert!((g.edges[0].weight - 1.5).abs() < 1e-5);
    }

    #[test]
    fn test_gexf_to_xml_contains_node() {
        let cfg = default_gexf_config();
        let mut g = new_gexf_graph(&cfg);
        gexf_add_node(&mut g, 42, "Spine");
        let xml = gexf_to_xml_string(&g);
        assert!(xml.contains("Spine"));
        assert!(xml.contains("id=\"42\""));
    }

    #[test]
    fn test_gexf_to_xml_contains_edge() {
        let cfg = default_gexf_config();
        let mut g = new_gexf_graph(&cfg);
        gexf_add_node(&mut g, 0, "Root");
        gexf_add_node(&mut g, 1, "Child");
        gexf_add_edge(&mut g, 0, 1, 1.0);
        let xml = gexf_to_xml_string(&g);
        assert!(xml.contains("source=\"0\""));
        assert!(xml.contains("target=\"1\""));
    }

    #[test]
    fn test_gexf_graph_clear() {
        let cfg = default_gexf_config();
        let mut g = new_gexf_graph(&cfg);
        gexf_add_node(&mut g, 0, "X");
        gexf_add_edge(&mut g, 0, 0, 1.0);
        gexf_graph_clear(&mut g);
        assert_eq!(gexf_node_count(&g), 0);
        assert_eq!(gexf_edge_count(&g), 0);
    }

    #[test]
    fn test_gexf_from_skeleton() {
        let names = ["Hips", "Spine", "Chest"];
        let parents: [i32; 3] = [-1, 0, 1];
        let g = gexf_from_skeleton(&names, &parents);
        assert_eq!(gexf_node_count(&g), 3);
        assert_eq!(gexf_edge_count(&g), 2);
    }

    #[test]
    fn test_gexf_from_skeleton_no_root_edge() {
        let names = ["Root"];
        let parents: [i32; 1] = [-1];
        let g = gexf_from_skeleton(&names, &parents);
        assert_eq!(gexf_edge_count(&g), 0);
    }

    #[test]
    fn test_gexf_xml_header() {
        let cfg = default_gexf_config();
        let g = new_gexf_graph(&cfg);
        let xml = gexf_to_xml_string(&g);
        assert!(xml.starts_with("<?xml"));
        assert!(xml.contains("<gexf"));
    }
}
