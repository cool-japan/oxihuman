#![allow(dead_code)]

/// Node in a morph network graph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NetworkNode {
    pub name: String,
    pub weight: f32,
    pub connections: Vec<usize>,
}

/// Directed graph of morph operations.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphNetwork {
    nodes: Vec<NetworkNode>,
    output_cache: Vec<f32>,
}

#[allow(dead_code)]
pub fn new_morph_network() -> MorphNetwork {
    MorphNetwork { nodes: Vec::new(), output_cache: Vec::new() }
}

#[allow(dead_code)]
pub fn add_network_node(net: &mut MorphNetwork, name: &str, weight: f32) -> usize {
    let idx = net.nodes.len();
    net.nodes.push(NetworkNode { name: name.to_string(), weight, connections: Vec::new() });
    idx
}

#[allow(dead_code)]
pub fn connect_nodes(net: &mut MorphNetwork, from: usize, to: usize) {
    if from < net.nodes.len() && to < net.nodes.len() {
        net.nodes[from].connections.push(to);
    }
}

#[allow(dead_code)]
pub fn evaluate_network(net: &mut MorphNetwork) -> Vec<f32> {
    let n = net.nodes.len();
    let mut values: Vec<f32> = net.nodes.iter().map(|node| node.weight).collect();
    for i in 0..n {
        let conns: Vec<usize> = net.nodes[i].connections.clone();
        for &c in &conns {
            if c < n {
                values[c] += values[i] * 0.5;
            }
        }
    }
    net.output_cache = values.clone();
    values
}

#[allow(dead_code)]
pub fn network_node_count(net: &MorphNetwork) -> usize { net.nodes.len() }

#[allow(dead_code)]
pub fn network_to_json(net: &MorphNetwork) -> String {
    format!("{{\"node_count\":{}}}", net.nodes.len())
}

#[allow(dead_code)]
pub fn network_clear(net: &mut MorphNetwork) {
    net.nodes.clear();
    net.output_cache.clear();
}

#[allow(dead_code)]
pub fn network_output(net: &MorphNetwork) -> &[f32] { &net.output_cache }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let n = new_morph_network(); assert_eq!(network_node_count(&n), 0); }
    #[test] fn test_add_node() {
        let mut n = new_morph_network();
        let idx = add_network_node(&mut n, "a", 1.0);
        assert_eq!(idx, 0);
        assert_eq!(network_node_count(&n), 1);
    }
    #[test] fn test_connect() {
        let mut n = new_morph_network();
        add_network_node(&mut n, "a", 1.0);
        add_network_node(&mut n, "b", 0.5);
        connect_nodes(&mut n, 0, 1);
        assert_eq!(n.nodes[0].connections.len(), 1);
    }
    #[test] fn test_evaluate() {
        let mut n = new_morph_network();
        add_network_node(&mut n, "a", 1.0);
        add_network_node(&mut n, "b", 0.0);
        connect_nodes(&mut n, 0, 1);
        let vals = evaluate_network(&mut n);
        assert!(vals[1] > 0.0);
    }
    #[test] fn test_json() {
        let n = new_morph_network();
        assert!(network_to_json(&n).contains("node_count"));
    }
    #[test] fn test_clear() {
        let mut n = new_morph_network();
        add_network_node(&mut n, "a", 1.0);
        network_clear(&mut n);
        assert_eq!(network_node_count(&n), 0);
    }
    #[test] fn test_output_empty() {
        let n = new_morph_network();
        assert!(network_output(&n).is_empty());
    }
    #[test] fn test_output_after_eval() {
        let mut n = new_morph_network();
        add_network_node(&mut n, "x", 2.0);
        evaluate_network(&mut n);
        assert!(!network_output(&n).is_empty());
    }
    #[test] fn test_connect_oob() {
        let mut n = new_morph_network();
        connect_nodes(&mut n, 0, 1);
        assert_eq!(network_node_count(&n), 0);
    }
    #[test] fn test_multiple_nodes() {
        let mut n = new_morph_network();
        for i in 0..5 { add_network_node(&mut n, &format!("n{}", i), i as f32); }
        assert_eq!(network_node_count(&n), 5);
    }
}
