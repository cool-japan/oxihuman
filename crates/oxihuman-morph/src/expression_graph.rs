#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub name: String,
    pub value: f32,
    pub inputs: Vec<usize>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionGraph {
    nodes: Vec<GraphNode>,
    output: Vec<f32>,
}

#[allow(dead_code)]
pub fn new_expression_graph() -> ExpressionGraph {
    ExpressionGraph { nodes: Vec::new(), output: Vec::new() }
}

#[allow(dead_code)]
pub fn add_graph_node_eg(g: &mut ExpressionGraph, name: &str, value: f32) -> usize {
    let idx = g.nodes.len();
    g.nodes.push(GraphNode { name: name.to_string(), value, inputs: Vec::new() });
    idx
}

#[allow(dead_code)]
pub fn evaluate_graph(g: &mut ExpressionGraph) -> Vec<f32> {
    let n = g.nodes.len();
    let mut vals = vec![0.0_f32; n];
    for i in 0..n {
        let mut sum = g.nodes[i].value;
        for &inp in &g.nodes[i].inputs {
            if inp < i { sum += vals[inp] * 0.5; }
        }
        vals[i] = sum.clamp(-1.0, 1.0);
    }
    g.output = vals.clone();
    vals
}

#[allow(dead_code)]
pub fn graph_node_count_eg(g: &ExpressionGraph) -> usize { g.nodes.len() }

#[allow(dead_code)]
pub fn graph_output(g: &ExpressionGraph) -> &[f32] { &g.output }

#[allow(dead_code)]
pub fn graph_to_json_eg(g: &ExpressionGraph) -> String {
    format!("{{\"nodes\":{}}}", g.nodes.len())
}

#[allow(dead_code)]
pub fn graph_clear_eg(g: &mut ExpressionGraph) { g.nodes.clear(); g.output.clear(); }

#[allow(dead_code)]
pub fn graph_is_valid(g: &ExpressionGraph) -> bool {
    for node in &g.nodes {
        for &inp in &node.inputs {
            if inp >= g.nodes.len() { return false; }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let g = new_expression_graph(); assert_eq!(graph_node_count_eg(&g), 0); }
    #[test] fn test_add() { let mut g = new_expression_graph(); add_graph_node_eg(&mut g, "a", 0.5); assert_eq!(graph_node_count_eg(&g), 1); }
    #[test] fn test_eval() { let mut g = new_expression_graph(); add_graph_node_eg(&mut g, "a", 0.3); let v = evaluate_graph(&mut g); assert!((v[0] - 0.3).abs() < 1e-6); }
    #[test] fn test_output() { let g = new_expression_graph(); assert!(graph_output(&g).is_empty()); }
    #[test] fn test_json() { let g = new_expression_graph(); assert!(graph_to_json_eg(&g).contains("nodes")); }
    #[test] fn test_clear() { let mut g = new_expression_graph(); add_graph_node_eg(&mut g, "a", 0.5); graph_clear_eg(&mut g); assert_eq!(graph_node_count_eg(&g), 0); }
    #[test] fn test_valid_empty() { let g = new_expression_graph(); assert!(graph_is_valid(&g)); }
    #[test] fn test_valid_with_node() { let mut g = new_expression_graph(); add_graph_node_eg(&mut g, "a", 0.1); assert!(graph_is_valid(&g)); }
    #[test] fn test_eval_clamp() { let mut g = new_expression_graph(); add_graph_node_eg(&mut g, "a", 2.0); let v = evaluate_graph(&mut g); assert!((v[0] - 1.0).abs() < 1e-6); }
    #[test] fn test_multiple() { let mut g = new_expression_graph(); for i in 0..4 { add_graph_node_eg(&mut g, &format!("n{}", i), 0.1); } assert_eq!(graph_node_count_eg(&g), 4); }
}
