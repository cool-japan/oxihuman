#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct RgbNode { name: String }

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct RgbEdge { from: usize, to: usize }

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderGraphBuilder {
    nodes: Vec<RgbNode>,
    edges: Vec<RgbEdge>,
}

#[allow(dead_code)]
pub fn new_render_graph_builder() -> RenderGraphBuilder {
    RenderGraphBuilder { nodes: Vec::new(), edges: Vec::new() }
}

#[allow(dead_code)]
pub fn add_node_rgb(b: &mut RenderGraphBuilder, name: &str) -> usize {
    let idx = b.nodes.len();
    b.nodes.push(RgbNode { name: name.to_string() });
    idx
}

#[allow(dead_code)]
pub fn add_edge_rgb(b: &mut RenderGraphBuilder, from: usize, to: usize) {
    if from < b.nodes.len() && to < b.nodes.len() {
        b.edges.push(RgbEdge { from, to });
    }
}

#[allow(dead_code)]
pub fn build_render_graph(b: &RenderGraphBuilder) -> Vec<usize> {
    // topological-ish order: just return 0..n for now
    (0..b.nodes.len()).collect()
}

#[allow(dead_code)]
pub fn node_count_rgb(b: &RenderGraphBuilder) -> usize { b.nodes.len() }

#[allow(dead_code)]
pub fn edge_count_rgb(b: &RenderGraphBuilder) -> usize { b.edges.len() }

#[allow(dead_code)]
pub fn builder_to_json(b: &RenderGraphBuilder) -> String {
    format!("{{\"nodes\":{},\"edges\":{}}}", b.nodes.len(), b.edges.len())
}

#[allow(dead_code)]
pub fn builder_clear(b: &mut RenderGraphBuilder) { b.nodes.clear(); b.edges.clear(); }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let b = new_render_graph_builder(); assert_eq!(node_count_rgb(&b), 0); }
    #[test] fn test_add_node() { let mut b = new_render_graph_builder(); add_node_rgb(&mut b, "pass0"); assert_eq!(node_count_rgb(&b), 1); }
    #[test] fn test_add_edge() { let mut b = new_render_graph_builder(); let a = add_node_rgb(&mut b, "a"); let c = add_node_rgb(&mut b, "b"); add_edge_rgb(&mut b, a, c); assert_eq!(edge_count_rgb(&b), 1); }
    #[test] fn test_build() { let mut b = new_render_graph_builder(); add_node_rgb(&mut b, "x"); let order = build_render_graph(&b); assert_eq!(order, vec![0]); }
    #[test] fn test_json() { let b = new_render_graph_builder(); assert!(builder_to_json(&b).contains("nodes")); }
    #[test] fn test_clear() { let mut b = new_render_graph_builder(); add_node_rgb(&mut b, "x"); builder_clear(&mut b); assert_eq!(node_count_rgb(&b), 0); }
    #[test] fn test_edge_oob() { let mut b = new_render_graph_builder(); add_edge_rgb(&mut b, 0, 1); assert_eq!(edge_count_rgb(&b), 0); }
    #[test] fn test_edge_count() { let b = new_render_graph_builder(); assert_eq!(edge_count_rgb(&b), 0); }
    #[test] fn test_multiple_nodes() { let mut b = new_render_graph_builder(); for i in 0..5 { add_node_rgb(&mut b, &format!("n{}", i)); } assert_eq!(node_count_rgb(&b), 5); }
    #[test] fn test_build_order() { let mut b = new_render_graph_builder(); add_node_rgb(&mut b, "a"); add_node_rgb(&mut b, "b"); let o = build_render_graph(&b); assert_eq!(o.len(), 2); }
}
