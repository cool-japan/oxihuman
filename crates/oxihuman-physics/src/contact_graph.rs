//! Graph of active contact pairs used to order constraint solving.
//!
//! A contact graph records which pairs of rigid bodies are currently
//! in contact. It supports connected-component queries so that the
//! constraint solver can process independent islands in parallel.

#![allow(dead_code)]

/// Configuration for the contact graph.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ContactGraphConfig {
    /// Initial capacity for the node and edge lists.
    pub initial_capacity: usize,
}

/// A single node in the contact graph, representing one body.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ContactNode {
    /// Body identifier.
    pub body_id: u32,
    /// Connected-component label (set by `contact_graph_connected_components`).
    pub component: u32,
}

/// An undirected edge connecting two bodies that are in contact.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ContactEdge {
    /// Index into the node list for the first body.
    pub node_a: usize,
    /// Index into the node list for the second body.
    pub node_b: usize,
}

/// The full contact graph.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ContactGraph {
    /// All registered body nodes.
    pub nodes: Vec<ContactNode>,
    /// All active contact edges.
    pub edges: Vec<ContactEdge>,
}

/// Return sensible defaults for [`ContactGraphConfig`].
#[allow(dead_code)]
pub fn default_contact_graph_config() -> ContactGraphConfig {
    ContactGraphConfig { initial_capacity: 64 }
}

/// Create an empty contact graph.
#[allow(dead_code)]
pub fn new_contact_graph(config: &ContactGraphConfig) -> ContactGraph {
    ContactGraph {
        nodes: Vec::with_capacity(config.initial_capacity),
        edges: Vec::with_capacity(config.initial_capacity * 2),
    }
}

/// Add a contact pair (body_id_a, body_id_b) to the graph.
///
/// Body nodes are created on demand; edges are only added if the pair
/// is not already present.
#[allow(dead_code)]
pub fn contact_graph_add_pair(graph: &mut ContactGraph, body_id_a: u32, body_id_b: u32) {
    let idx_a = get_or_insert_node(graph, body_id_a);
    let idx_b = get_or_insert_node(graph, body_id_b);
    // Avoid duplicate edges
    let already = graph
        .edges
        .iter()
        .any(|e| (e.node_a == idx_a && e.node_b == idx_b) || (e.node_a == idx_b && e.node_b == idx_a));
    if !already {
        graph.edges.push(ContactEdge { node_a: idx_a, node_b: idx_b });
    }
}

fn get_or_insert_node(graph: &mut ContactGraph, body_id: u32) -> usize {
    if let Some(pos) = graph.nodes.iter().position(|n| n.body_id == body_id) {
        return pos;
    }
    graph.nodes.push(ContactNode { body_id, component: 0 });
    graph.nodes.len() - 1
}

/// Remove all contact edges involving the body pair.
#[allow(dead_code)]
pub fn contact_graph_remove_pair(graph: &mut ContactGraph, body_id_a: u32, body_id_b: u32) {
    let idx_a = graph.nodes.iter().position(|n| n.body_id == body_id_a);
    let idx_b = graph.nodes.iter().position(|n| n.body_id == body_id_b);
    if let (Some(a), Some(b)) = (idx_a, idx_b) {
        graph.edges.retain(|e| {
            !((e.node_a == a && e.node_b == b) || (e.node_a == b && e.node_b == a))
        });
    }
}

/// Return the node indices of all bodies that share a contact edge with `body_id`.
#[allow(dead_code)]
pub fn contact_graph_neighbors(graph: &ContactGraph, body_id: u32) -> Vec<u32> {
    let idx = match graph.nodes.iter().position(|n| n.body_id == body_id) {
        Some(i) => i,
        None => return vec![],
    };
    let mut result = Vec::new();
    for e in &graph.edges {
        if e.node_a == idx {
            result.push(graph.nodes[e.node_b].body_id);
        } else if e.node_b == idx {
            result.push(graph.nodes[e.node_a].body_id);
        }
    }
    result
}

/// Return the number of active contact pairs.
#[allow(dead_code)]
pub fn contact_graph_pair_count(graph: &ContactGraph) -> usize {
    graph.edges.len()
}

/// Label every node with its connected-component index.
///
/// Returns the total number of components found.
#[allow(dead_code)]
pub fn contact_graph_connected_components(graph: &mut ContactGraph) -> u32 {
    let n = graph.nodes.len();
    // Union-Find
    let mut parent: Vec<usize> = (0..n).collect();

    fn find(parent: &mut Vec<usize>, x: usize) -> usize {
        if parent[x] != x {
            parent[x] = find(parent, parent[x]);
        }
        parent[x]
    }

    for e in &graph.edges {
        let ra = find(&mut parent, e.node_a);
        let rb = find(&mut parent, e.node_b);
        if ra != rb {
            parent[ra] = rb;
        }
    }

    // Assign component labels
    let mut label_map = std::collections::HashMap::new();
    let mut next_label = 0_u32;
    for i in 0..n {
        let root = find(&mut parent, i);
        let label = *label_map.entry(root).or_insert_with(|| {
            let l = next_label;
            next_label += 1;
            l
        });
        graph.nodes[i].component = label;
    }
    next_label
}

/// Serialise the graph to compact JSON.
#[allow(dead_code)]
pub fn contact_graph_to_json(graph: &ContactGraph) -> String {
    let nodes: Vec<String> = graph
        .nodes
        .iter()
        .map(|n| format!(r#"{{"id":{},"comp":{}}}"#, n.body_id, n.component))
        .collect();
    let edges: Vec<String> = graph
        .edges
        .iter()
        .map(|e| format!(r#"{{"a":{},"b":{}}}"#, e.node_a, e.node_b))
        .collect();
    format!(
        r#"{{"nodes":[{}],"edges":[{}]}}"#,
        nodes.join(","),
        edges.join(",")
    )
}

/// Clear all nodes and edges.
#[allow(dead_code)]
pub fn contact_graph_clear(graph: &mut ContactGraph) {
    graph.nodes.clear();
    graph.edges.clear();
}

/// Return `true` if the entire graph is a single connected component.
#[allow(dead_code)]
pub fn contact_graph_is_connected(graph: &ContactGraph) -> bool {
    if graph.nodes.is_empty() {
        return true;
    }
    // BFS from node 0
    let n = graph.nodes.len();
    let mut visited = vec![false; n];
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(0_usize);
    visited[0] = true;
    let mut count = 1_usize;
    while let Some(cur) = queue.pop_front() {
        for e in &graph.edges {
            let other = if e.node_a == cur {
                Some(e.node_b)
            } else if e.node_b == cur {
                Some(e.node_a)
            } else {
                None
            };
            if let Some(o) = other {
                if !visited[o] {
                    visited[o] = true;
                    count += 1;
                    queue.push_back(o);
                }
            }
        }
    }
    count == n
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_contact_graph_config();
        assert_eq!(cfg.initial_capacity, 64);
    }

    #[test]
    fn test_add_pair_creates_nodes() {
        let cfg = default_contact_graph_config();
        let mut g = new_contact_graph(&cfg);
        contact_graph_add_pair(&mut g, 1, 2);
        assert_eq!(g.nodes.len(), 2);
    }

    #[test]
    fn test_pair_count() {
        let cfg = default_contact_graph_config();
        let mut g = new_contact_graph(&cfg);
        contact_graph_add_pair(&mut g, 1, 2);
        contact_graph_add_pair(&mut g, 2, 3);
        assert_eq!(contact_graph_pair_count(&g), 2);
    }

    #[test]
    fn test_no_duplicate_edges() {
        let cfg = default_contact_graph_config();
        let mut g = new_contact_graph(&cfg);
        contact_graph_add_pair(&mut g, 1, 2);
        contact_graph_add_pair(&mut g, 1, 2);
        contact_graph_add_pair(&mut g, 2, 1);
        assert_eq!(contact_graph_pair_count(&g), 1);
    }

    #[test]
    fn test_remove_pair() {
        let cfg = default_contact_graph_config();
        let mut g = new_contact_graph(&cfg);
        contact_graph_add_pair(&mut g, 1, 2);
        contact_graph_add_pair(&mut g, 2, 3);
        contact_graph_remove_pair(&mut g, 1, 2);
        assert_eq!(contact_graph_pair_count(&g), 1);
    }

    #[test]
    fn test_neighbors() {
        let cfg = default_contact_graph_config();
        let mut g = new_contact_graph(&cfg);
        contact_graph_add_pair(&mut g, 1, 2);
        contact_graph_add_pair(&mut g, 1, 3);
        let mut nb = contact_graph_neighbors(&g, 1);
        nb.sort();
        assert_eq!(nb, vec![2, 3]);
    }

    #[test]
    fn test_connected_components_two_islands() {
        let cfg = default_contact_graph_config();
        let mut g = new_contact_graph(&cfg);
        contact_graph_add_pair(&mut g, 1, 2);
        contact_graph_add_pair(&mut g, 3, 4);
        let count = contact_graph_connected_components(&mut g);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_is_connected_true() {
        let cfg = default_contact_graph_config();
        let mut g = new_contact_graph(&cfg);
        contact_graph_add_pair(&mut g, 1, 2);
        contact_graph_add_pair(&mut g, 2, 3);
        assert!(contact_graph_is_connected(&g));
    }

    #[test]
    fn test_is_connected_false() {
        let cfg = default_contact_graph_config();
        let mut g = new_contact_graph(&cfg);
        contact_graph_add_pair(&mut g, 1, 2);
        contact_graph_add_pair(&mut g, 3, 4);
        assert!(!contact_graph_is_connected(&g));
    }

    #[test]
    fn test_to_json_contains_nodes() {
        let cfg = default_contact_graph_config();
        let mut g = new_contact_graph(&cfg);
        contact_graph_add_pair(&mut g, 5, 6);
        let json = contact_graph_to_json(&g);
        assert!(json.contains("nodes"));
        assert!(json.contains("edges"));
    }

    #[test]
    fn test_clear_empties_graph() {
        let cfg = default_contact_graph_config();
        let mut g = new_contact_graph(&cfg);
        contact_graph_add_pair(&mut g, 1, 2);
        contact_graph_clear(&mut g);
        assert_eq!(contact_graph_pair_count(&g), 0);
        assert_eq!(g.nodes.len(), 0);
    }
}
