#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! DAG-based task scheduler with topological sort.

use std::collections::{HashMap, VecDeque};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DagNode {
    pub id: usize,
    pub name: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DagScheduler {
    nodes: HashMap<usize, DagNode>,
    edges: Vec<(usize, usize)>,
    next_id: usize,
}

#[allow(dead_code)]
pub fn new_dag_scheduler() -> DagScheduler {
    DagScheduler {
        nodes: HashMap::new(),
        edges: Vec::new(),
        next_id: 0,
    }
}

#[allow(dead_code)]
pub fn add_dag_node(dag: &mut DagScheduler, name: &str) -> usize {
    let id = dag.next_id;
    dag.next_id += 1;
    dag.nodes.insert(id, DagNode { id, name: name.to_string() });
    id
}

#[allow(dead_code)]
pub fn add_dag_edge(dag: &mut DagScheduler, from: usize, to: usize) -> bool {
    if dag.nodes.contains_key(&from) && dag.nodes.contains_key(&to) {
        dag.edges.push((from, to));
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn topological_sort(dag: &DagScheduler) -> Option<Vec<usize>> {
    let mut in_degree: HashMap<usize, usize> = HashMap::new();
    for &id in dag.nodes.keys() {
        in_degree.insert(id, 0);
    }
    for &(_, to) in &dag.edges {
        *in_degree.entry(to).or_insert(0) += 1;
    }
    let mut queue: VecDeque<usize> = in_degree
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(&id, _)| id)
        .collect();
    let mut result = Vec::new();
    while let Some(id) = queue.pop_front() {
        result.push(id);
        for &(from, to) in &dag.edges {
            if from == id {
                if let Some(d) = in_degree.get_mut(&to) {
                    *d -= 1;
                    if *d == 0 {
                        queue.push_back(to);
                    }
                }
            }
        }
    }
    if result.len() == dag.nodes.len() {
        Some(result)
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn dag_node_count(dag: &DagScheduler) -> usize {
    dag.nodes.len()
}

#[allow(dead_code)]
pub fn dag_edge_count(dag: &DagScheduler) -> usize {
    dag.edges.len()
}

#[allow(dead_code)]
pub fn dag_is_acyclic(dag: &DagScheduler) -> bool {
    topological_sort(dag).is_some()
}

#[allow(dead_code)]
pub fn dag_to_json(dag: &DagScheduler) -> String {
    format!(
        r#"{{"nodes":{},"edges":{}}}"#,
        dag.nodes.len(),
        dag.edges.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dag() {
        let dag = new_dag_scheduler();
        assert_eq!(dag_node_count(&dag), 0);
    }

    #[test]
    fn test_add_node() {
        let mut dag = new_dag_scheduler();
        let id = add_dag_node(&mut dag, "a");
        assert_eq!(id, 0);
        assert_eq!(dag_node_count(&dag), 1);
    }

    #[test]
    fn test_add_edge() {
        let mut dag = new_dag_scheduler();
        let a = add_dag_node(&mut dag, "a");
        let b = add_dag_node(&mut dag, "b");
        assert!(add_dag_edge(&mut dag, a, b));
        assert_eq!(dag_edge_count(&dag), 1);
    }

    #[test]
    fn test_edge_invalid_node() {
        let mut dag = new_dag_scheduler();
        assert!(!add_dag_edge(&mut dag, 0, 1));
    }

    #[test]
    fn test_topological_sort_simple() {
        let mut dag = new_dag_scheduler();
        let a = add_dag_node(&mut dag, "a");
        let b = add_dag_node(&mut dag, "b");
        add_dag_edge(&mut dag, a, b);
        let sorted = topological_sort(&dag).expect("should succeed");
        assert_eq!(sorted.len(), 2);
        let pos_a = sorted.iter().position(|&x| x == a).expect("should succeed");
        let pos_b = sorted.iter().position(|&x| x == b).expect("should succeed");
        assert!(pos_a < pos_b);
    }

    #[test]
    fn test_topological_sort_cycle() {
        let mut dag = new_dag_scheduler();
        let a = add_dag_node(&mut dag, "a");
        let b = add_dag_node(&mut dag, "b");
        add_dag_edge(&mut dag, a, b);
        add_dag_edge(&mut dag, b, a);
        assert!(topological_sort(&dag).is_none());
    }

    #[test]
    fn test_dag_is_acyclic() {
        let mut dag = new_dag_scheduler();
        let a = add_dag_node(&mut dag, "a");
        let b = add_dag_node(&mut dag, "b");
        add_dag_edge(&mut dag, a, b);
        assert!(dag_is_acyclic(&dag));
    }

    #[test]
    fn test_dag_to_json() {
        let dag = new_dag_scheduler();
        let json = dag_to_json(&dag);
        assert!(json.contains("\"nodes\":0"));
    }

    #[test]
    fn test_empty_topo_sort() {
        let dag = new_dag_scheduler();
        let sorted = topological_sort(&dag).expect("should succeed");
        assert!(sorted.is_empty());
    }

    #[test]
    fn test_three_node_chain() {
        let mut dag = new_dag_scheduler();
        let a = add_dag_node(&mut dag, "a");
        let b = add_dag_node(&mut dag, "b");
        let c = add_dag_node(&mut dag, "c");
        add_dag_edge(&mut dag, a, b);
        add_dag_edge(&mut dag, b, c);
        let sorted = topological_sort(&dag).expect("should succeed");
        assert_eq!(sorted.len(), 3);
    }
}
