//! DAG-based blend shape evaluation graph.
//! Supports Add, Multiply, Override, and Screen blend operations.

#[allow(dead_code)]
#[derive(Clone)]
pub enum BlendOp {
    Add,
    Multiply,
    Override,
    Screen,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct BlendNode {
    pub id: u32,
    pub name: String,
    pub weight: f32,
    pub op: BlendOp,
    pub inputs: Vec<u32>,
    pub target_name: Option<String>,
}

#[allow(dead_code)]
pub struct BlendGraph {
    pub nodes: Vec<BlendNode>,
    pub roots: Vec<u32>,
    pub next_id: u32,
}

#[allow(dead_code)]
pub struct EvalResult {
    pub target_weights: Vec<(String, f32)>,
}

#[allow(dead_code)]
pub fn new_blend_graph() -> BlendGraph {
    BlendGraph {
        nodes: Vec::new(),
        roots: Vec::new(),
        next_id: 0,
    }
}

#[allow(dead_code)]
pub fn add_blend_node(graph: &mut BlendGraph, name: &str, op: BlendOp, weight: f32) -> u32 {
    let id = graph.next_id;
    graph.next_id += 1;
    graph.nodes.push(BlendNode {
        id,
        name: name.to_string(),
        weight,
        op,
        inputs: Vec::new(),
        target_name: None,
    });
    id
}

#[allow(dead_code)]
pub fn add_leaf_node(graph: &mut BlendGraph, target: &str, weight: f32) -> u32 {
    let id = graph.next_id;
    graph.next_id += 1;
    graph.nodes.push(BlendNode {
        id,
        name: target.to_string(),
        weight,
        op: BlendOp::Add,
        inputs: Vec::new(),
        target_name: Some(target.to_string()),
    });
    id
}

#[allow(dead_code)]
pub fn connect_nodes(graph: &mut BlendGraph, parent: u32, child: u32) {
    if let Some(node) = graph.nodes.iter_mut().find(|n| n.id == parent) {
        if !node.inputs.contains(&child) {
            node.inputs.push(child);
        }
    }
}

#[allow(dead_code)]
pub fn add_root(graph: &mut BlendGraph, node_id: u32) {
    if !graph.roots.contains(&node_id) {
        graph.roots.push(node_id);
    }
}

/// Recursively evaluate a node and accumulate target weights.
fn eval_node(
    graph: &BlendGraph,
    node_id: u32,
    parent_weight: f32,
    results: &mut Vec<(String, f32)>,
    visited: &mut Vec<u32>,
) {
    if visited.contains(&node_id) {
        return;
    }
    visited.push(node_id);

    let node = match graph.nodes.iter().find(|n| n.id == node_id) {
        Some(n) => n.clone(),
        None => return,
    };

    let effective_weight = match node.op {
        BlendOp::Add => parent_weight + node.weight,
        BlendOp::Multiply => parent_weight * node.weight,
        BlendOp::Override => node.weight,
        BlendOp::Screen => 1.0 - (1.0 - parent_weight) * (1.0 - node.weight),
    };

    if let Some(ref target) = node.target_name {
        // leaf node
        if let Some(entry) = results.iter_mut().find(|(name, _)| name == target) {
            entry.1 += effective_weight;
        } else {
            results.push((target.clone(), effective_weight));
        }
    }

    for child_id in &node.inputs {
        eval_node(graph, *child_id, effective_weight, results, visited);
    }
}

/// DFS from all roots, accumulate weights per target.
#[allow(dead_code)]
pub fn evaluate_graph(graph: &BlendGraph) -> EvalResult {
    let mut results: Vec<(String, f32)> = Vec::new();
    let mut visited = Vec::new();

    for &root_id in &graph.roots {
        eval_node(graph, root_id, 1.0, &mut results, &mut visited);
    }

    EvalResult {
        target_weights: results,
    }
}

#[allow(dead_code)]
pub fn set_node_weight(graph: &mut BlendGraph, id: u32, weight: f32) {
    if let Some(node) = graph.nodes.iter_mut().find(|n| n.id == id) {
        node.weight = weight;
    }
}

#[allow(dead_code)]
pub fn get_node(graph: &BlendGraph, id: u32) -> Option<&BlendNode> {
    graph.nodes.iter().find(|n| n.id == id)
}

#[allow(dead_code)]
pub fn node_count(graph: &BlendGraph) -> usize {
    graph.nodes.len()
}

#[allow(dead_code)]
pub fn leaf_nodes(graph: &BlendGraph) -> Vec<&BlendNode> {
    graph
        .nodes
        .iter()
        .filter(|n| n.target_name.is_some())
        .collect()
}

/// Topological sort using DFS post-order (Kahn's-style via recursion).
fn topo_visit(graph: &BlendGraph, node_id: u32, visited: &mut Vec<u32>, order: &mut Vec<u32>) {
    if visited.contains(&node_id) {
        return;
    }
    visited.push(node_id);
    if let Some(node) = graph.nodes.iter().find(|n| n.id == node_id) {
        for &child in &node.inputs {
            topo_visit(graph, child, visited, order);
        }
    }
    order.push(node_id);
}

/// Returns node IDs in evaluation order (roots first, leaves last via reverse post-order).
#[allow(dead_code)]
pub fn topological_sort_graph(graph: &BlendGraph) -> Vec<u32> {
    let mut visited = Vec::new();
    let mut order = Vec::new();

    for &root in &graph.roots {
        topo_visit(graph, root, &mut visited, &mut order);
    }
    // also handle disconnected nodes
    for node in &graph.nodes {
        if !visited.contains(&node.id) {
            topo_visit(graph, node.id, &mut visited, &mut order);
        }
    }

    order.reverse();
    order
}

/// Remove leaf nodes (no children) with weight approximately zero.
#[allow(dead_code)]
pub fn prune_zero_weight(graph: &mut BlendGraph) {
    // Keep a node if it has children OR its weight is non-zero.
    graph
        .nodes
        .retain(|n| !n.inputs.is_empty() || n.weight.abs() >= 1e-6);
}

#[allow(dead_code)]
pub fn blend_graph_to_json(graph: &BlendGraph) -> String {
    let mut parts = Vec::new();
    for node in &graph.nodes {
        let op_str = match node.op {
            BlendOp::Add => "Add",
            BlendOp::Multiply => "Multiply",
            BlendOp::Override => "Override",
            BlendOp::Screen => "Screen",
        };
        let target_str = node
            .target_name
            .as_deref()
            .map(|s| format!("\"{}\"", s))
            .unwrap_or_else(|| "null".to_string());
        let inputs_str = node
            .inputs
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        parts.push(format!(
            "{{\"id\":{},\"name\":\"{}\",\"weight\":{},\"op\":\"{}\",\"target\":{},\"inputs\":[{}]}}",
            node.id, node.name, node.weight, op_str, target_str, inputs_str
        ));
    }
    let roots_str = graph
        .roots
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"nodes\":[{}],\"roots\":[{}]}}",
        parts.join(","),
        roots_str
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_blend_graph() {
        let g = new_blend_graph();
        assert!(g.nodes.is_empty());
        assert!(g.roots.is_empty());
        assert_eq!(g.next_id, 0);
    }

    #[test]
    fn test_add_blend_node() {
        let mut g = new_blend_graph();
        let id = add_blend_node(&mut g, "parent", BlendOp::Add, 0.8);
        assert_eq!(id, 0);
        assert_eq!(g.nodes.len(), 1);
        assert_eq!(g.nodes[0].name, "parent");
    }

    #[test]
    fn test_add_leaf_node() {
        let mut g = new_blend_graph();
        let id = add_leaf_node(&mut g, "smile", 0.5);
        assert_eq!(id, 0);
        assert!(g.nodes[0].target_name.is_some());
        assert_eq!(g.nodes[0].target_name.as_deref(), Some("smile"));
    }

    #[test]
    fn test_connect_nodes() {
        let mut g = new_blend_graph();
        let p = add_blend_node(&mut g, "parent", BlendOp::Add, 1.0);
        let c = add_leaf_node(&mut g, "child_target", 0.5);
        connect_nodes(&mut g, p, c);
        assert!(g.nodes[0].inputs.contains(&c));
    }

    #[test]
    fn test_add_root() {
        let mut g = new_blend_graph();
        let id = add_blend_node(&mut g, "root", BlendOp::Add, 1.0);
        add_root(&mut g, id);
        assert!(g.roots.contains(&id));
        // duplicate should not be added
        add_root(&mut g, id);
        assert_eq!(g.roots.len(), 1);
    }

    #[test]
    fn test_evaluate_single_leaf() {
        let mut g = new_blend_graph();
        let leaf = add_leaf_node(&mut g, "brow_raise", 0.7);
        add_root(&mut g, leaf);
        let result = evaluate_graph(&g);
        assert_eq!(result.target_weights.len(), 1);
        let (name, w) = &result.target_weights[0];
        assert_eq!(name, "brow_raise");
        // Override op is Add here, so parent_weight(1.0) + node.weight(0.7) for Add
        assert!(*w > 0.0);
    }

    #[test]
    fn test_evaluate_tree_add() {
        let mut g = new_blend_graph();
        let parent = add_blend_node(&mut g, "group", BlendOp::Multiply, 0.5);
        let leaf = add_leaf_node(&mut g, "jaw_open", 1.0);
        connect_nodes(&mut g, parent, leaf);
        add_root(&mut g, parent);
        let result = evaluate_graph(&g);
        assert!(!result.target_weights.is_empty());
    }

    #[test]
    fn test_set_node_weight() {
        let mut g = new_blend_graph();
        let id = add_blend_node(&mut g, "n", BlendOp::Add, 0.0);
        set_node_weight(&mut g, id, 0.9);
        assert!((get_node(&g, id).unwrap().weight - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_get_node() {
        let mut g = new_blend_graph();
        let id = add_leaf_node(&mut g, "target", 0.3);
        let node = get_node(&g, id);
        assert!(node.is_some());
        assert_eq!(node.unwrap().id, id);
    }

    #[test]
    fn test_node_count() {
        let mut g = new_blend_graph();
        assert_eq!(node_count(&g), 0);
        add_blend_node(&mut g, "a", BlendOp::Add, 1.0);
        add_leaf_node(&mut g, "b", 0.5);
        assert_eq!(node_count(&g), 2);
    }

    #[test]
    fn test_leaf_nodes() {
        let mut g = new_blend_graph();
        add_blend_node(&mut g, "group", BlendOp::Add, 1.0);
        add_leaf_node(&mut g, "target_a", 0.5);
        add_leaf_node(&mut g, "target_b", 0.3);
        let leaves = leaf_nodes(&g);
        assert_eq!(leaves.len(), 2);
    }

    #[test]
    fn test_topological_sort() {
        let mut g = new_blend_graph();
        let p = add_blend_node(&mut g, "root", BlendOp::Add, 1.0);
        let c1 = add_leaf_node(&mut g, "t1", 0.5);
        let c2 = add_leaf_node(&mut g, "t2", 0.5);
        connect_nodes(&mut g, p, c1);
        connect_nodes(&mut g, p, c2);
        add_root(&mut g, p);
        let order = topological_sort_graph(&g);
        assert_eq!(order.len(), 3);
        // root should appear before children
        let root_pos = order.iter().position(|&id| id == p).unwrap();
        let c1_pos = order.iter().position(|&id| id == c1).unwrap();
        assert!(root_pos < c1_pos);
    }

    #[test]
    fn test_prune_zero_weight() {
        let mut g = new_blend_graph();
        add_leaf_node(&mut g, "zero_target", 0.0);
        let _keep = add_leaf_node(&mut g, "keep_target", 0.5);
        prune_zero_weight(&mut g);
        // zero-weight leaf with no children should be removed
        assert_eq!(g.nodes.len(), 1);
        assert_eq!(g.nodes[0].name, "keep_target");
    }

    #[test]
    fn test_blend_graph_to_json() {
        let mut g = new_blend_graph();
        let leaf = add_leaf_node(&mut g, "smile", 0.8);
        add_root(&mut g, leaf);
        let json = blend_graph_to_json(&g);
        assert!(json.contains("smile"));
        assert!(json.contains("nodes"));
        assert!(json.contains("roots"));
    }
}
