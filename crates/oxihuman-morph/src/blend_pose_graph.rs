//! Directed graph of pose states with weighted edge transitions.
//!
//! Models a simplified state machine for pose blending. Nodes represent pose
//! states; directed edges carry a blend weight that is accumulated when the
//! graph is evaluated. The active node is advanced along the edge with the
//! highest outgoing weight that exceeds a configurable threshold.

/// Configuration for the blend pose graph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendPoseGraphConfig {
    /// Maximum number of nodes.
    pub max_nodes: usize,
    /// Maximum number of edges.
    pub max_edges: usize,
    /// Minimum edge weight required to trigger a transition.
    pub transition_threshold: f32,
}

#[allow(dead_code)]
impl BlendPoseGraphConfig {
    fn new() -> Self {
        Self {
            max_nodes: 32,
            max_edges: 128,
            transition_threshold: 0.5,
        }
    }
}

/// Returns the default blend pose graph configuration.
#[allow(dead_code)]
pub fn default_blend_pose_graph_config() -> BlendPoseGraphConfig {
    BlendPoseGraphConfig::new()
}

/// A single node (pose state) in the graph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendPoseNode {
    /// Unique node id.
    pub id: u32,
    /// Human-readable label (e.g. "idle", "walk", "run").
    pub label: String,
    /// Accumulated blend weight from all incoming edges.
    pub blend_weight: f32,
}

/// A directed weighted edge between two nodes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendPoseEdge {
    /// Source node id.
    pub from: u32,
    /// Destination node id.
    pub to: u32,
    /// Edge weight [0.0, 1.0].
    pub weight: f32,
}

/// Directed blend pose graph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendPoseGraph {
    config: BlendPoseGraphConfig,
    nodes: Vec<BlendPoseNode>,
    edges: Vec<BlendPoseEdge>,
    current_node: Option<u32>,
    next_id: u32,
}

/// Creates a new `BlendPoseGraph` with the given configuration.
#[allow(dead_code)]
pub fn new_blend_pose_graph(config: BlendPoseGraphConfig) -> BlendPoseGraph {
    BlendPoseGraph {
        config,
        nodes: Vec::new(),
        edges: Vec::new(),
        current_node: None,
        next_id: 0,
    }
}

/// Adds a node with a label. Returns the new node's id, or `None` if full.
#[allow(dead_code)]
pub fn bpg_add_node(graph: &mut BlendPoseGraph, label: &str) -> Option<u32> {
    if graph.nodes.len() >= graph.config.max_nodes {
        return None;
    }
    let id = graph.next_id;
    graph.next_id += 1;
    if graph.current_node.is_none() {
        graph.current_node = Some(id);
    }
    graph.nodes.push(BlendPoseNode {
        id,
        label: label.to_string(),
        blend_weight: 0.0,
    });
    Some(id)
}

/// Adds a directed edge between two nodes. Returns `false` if the edge limit is reached
/// or if either node does not exist.
#[allow(dead_code)]
pub fn bpg_add_edge(graph: &mut BlendPoseGraph, from: u32, to: u32, weight: f32) -> bool {
    if graph.edges.len() >= graph.config.max_edges {
        return false;
    }
    let from_ok = graph.nodes.iter().any(|n| n.id == from);
    let to_ok = graph.nodes.iter().any(|n| n.id == to);
    if !from_ok || !to_ok {
        return false;
    }
    graph.edges.push(BlendPoseEdge {
        from,
        to,
        weight: weight.clamp(0.0, 1.0),
    });
    true
}

/// Evaluates the graph.
///
/// Steps performed:
///   1. Accumulates incoming edge weights onto each node's `blend_weight`.
///   2. From the current node, follows the edge with the highest weight if it
///      exceeds `transition_threshold`.
///
/// Returns the id of the (possibly updated) current node.
#[allow(dead_code)]
pub fn bpg_evaluate(graph: &mut BlendPoseGraph) -> Option<u32> {
    // Reset blend weights.
    for n in &mut graph.nodes {
        n.blend_weight = 0.0;
    }
    // Accumulate incoming weights.
    for e in &graph.edges {
        if let Some(n) = graph.nodes.iter_mut().find(|n| n.id == e.to) {
            n.blend_weight += e.weight;
        }
    }
    // Clamp blend weights to [0, 1].
    for n in &mut graph.nodes {
        n.blend_weight = n.blend_weight.clamp(0.0, 1.0);
    }
    // Transition from current node along the best outgoing edge.
    if let Some(cur) = graph.current_node {
        let best = graph
            .edges
            .iter()
            .filter(|e| e.from == cur && e.weight >= graph.config.transition_threshold)
            .max_by(|a, b| a.weight.partial_cmp(&b.weight).unwrap_or(std::cmp::Ordering::Equal));
        if let Some(edge) = best {
            graph.current_node = Some(edge.to);
        }
    }
    graph.current_node
}

/// Returns the number of nodes in the graph.
#[allow(dead_code)]
pub fn bpg_node_count(graph: &BlendPoseGraph) -> usize {
    graph.nodes.len()
}

/// Returns the number of edges in the graph.
#[allow(dead_code)]
pub fn bpg_edge_count(graph: &BlendPoseGraph) -> usize {
    graph.edges.len()
}

/// Returns the id of the current node, if any.
#[allow(dead_code)]
pub fn bpg_current_node(graph: &BlendPoseGraph) -> Option<u32> {
    graph.current_node
}

/// Serialises the graph to a simple JSON string.
#[allow(dead_code)]
pub fn bpg_to_json(graph: &BlendPoseGraph) -> String {
    let nodes: Vec<String> = graph
        .nodes
        .iter()
        .map(|n| format!("{{\"id\":{},\"label\":\"{}\",\"blend_weight\":{:.4}}}", n.id, n.label, n.blend_weight))
        .collect();
    let edges: Vec<String> = graph
        .edges
        .iter()
        .map(|e| format!("{{\"from\":{},\"to\":{},\"weight\":{:.4}}}", e.from, e.to, e.weight))
        .collect();
    format!(
        "{{\"current_node\":{},\"nodes\":[{}],\"edges\":[{}]}}",
        graph.current_node.map(|id| id.to_string()).unwrap_or_else(|| "null".to_string()),
        nodes.join(","),
        edges.join(",")
    )
}

/// Clears all nodes and edges, resetting the graph.
#[allow(dead_code)]
pub fn bpg_clear(graph: &mut BlendPoseGraph) {
    graph.nodes.clear();
    graph.edges.clear();
    graph.current_node = None;
    graph.next_id = 0;
}

// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make_graph() -> (BlendPoseGraph, u32, u32, u32) {
        let cfg = default_blend_pose_graph_config();
        let mut g = new_blend_pose_graph(cfg);
        let idle = bpg_add_node(&mut g, "idle").unwrap();
        let walk = bpg_add_node(&mut g, "walk").unwrap();
        let run = bpg_add_node(&mut g, "run").unwrap();
        bpg_add_edge(&mut g, idle, walk, 0.8);
        bpg_add_edge(&mut g, walk, run, 0.9);
        (g, idle, walk, run)
    }

    #[test]
    fn test_node_count() {
        let (g, _, _, _) = make_graph();
        assert_eq!(bpg_node_count(&g), 3);
    }

    #[test]
    fn test_edge_count() {
        let (g, _, _, _) = make_graph();
        assert_eq!(bpg_edge_count(&g), 2);
    }

    #[test]
    fn test_initial_current_node() {
        let (g, idle, _, _) = make_graph();
        assert_eq!(bpg_current_node(&g), Some(idle));
    }

    #[test]
    fn test_evaluate_transitions() {
        let (mut g, _idle, walk, _run) = make_graph();
        // idle → walk (weight 0.8 > threshold 0.5).
        let cur = bpg_evaluate(&mut g);
        assert_eq!(cur, Some(walk));
    }

    #[test]
    fn test_evaluate_chain() {
        let (mut g, _idle, _walk, run) = make_graph();
        bpg_evaluate(&mut g); // idle → walk
        let cur = bpg_evaluate(&mut g); // walk → run
        assert_eq!(cur, Some(run));
    }

    #[test]
    fn test_low_weight_no_transition() {
        let cfg = BlendPoseGraphConfig {
            max_nodes: 8,
            max_edges: 16,
            transition_threshold: 0.95,
        };
        let mut g = new_blend_pose_graph(cfg);
        let a = bpg_add_node(&mut g, "a").unwrap();
        let b = bpg_add_node(&mut g, "b").unwrap();
        bpg_add_edge(&mut g, a, b, 0.5);
        let cur = bpg_evaluate(&mut g);
        // Weight 0.5 < threshold 0.95, no transition.
        assert_eq!(cur, Some(a));
    }

    #[test]
    fn test_clear_resets() {
        let (mut g, _, _, _) = make_graph();
        bpg_clear(&mut g);
        assert_eq!(bpg_node_count(&g), 0);
        assert_eq!(bpg_edge_count(&g), 0);
        assert_eq!(bpg_current_node(&g), None);
    }

    #[test]
    fn test_to_json_contains_nodes() {
        let (g, _, _, _) = make_graph();
        let json = bpg_to_json(&g);
        assert!(json.contains("idle"));
        assert!(json.contains("walk"));
        assert!(json.contains("edges"));
    }

    #[test]
    fn test_add_edge_invalid_node_rejected() {
        let (mut g, _, _, _) = make_graph();
        assert!(!bpg_add_edge(&mut g, 99, 0, 1.0));
    }

    #[test]
    fn test_max_nodes_limit() {
        let cfg = BlendPoseGraphConfig {
            max_nodes: 2,
            max_edges: 8,
            transition_threshold: 0.5,
        };
        let mut g = new_blend_pose_graph(cfg);
        assert!(bpg_add_node(&mut g, "a").is_some());
        assert!(bpg_add_node(&mut g, "b").is_some());
        assert!(bpg_add_node(&mut g, "c").is_none());
    }
}
