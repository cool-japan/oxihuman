// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Topological dependency resolver using Kahn's algorithm.

use std::collections::{HashMap, HashSet, VecDeque};

/// A dependency graph for topological resolution.
#[derive(Debug, Default, Clone)]
pub struct DependencyResolverV2 {
    nodes: HashSet<String>,
    deps: HashMap<String, Vec<String>>, /* node -> list of dependencies */
}

/// Result of a resolution attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum ResolveResult {
    /// Successful topological order.
    Order(Vec<String>),
    /// A cycle was detected involving these nodes.
    Cycle(Vec<String>),
}

impl DependencyResolverV2 {
    /// Create a new empty resolver.
    pub fn new() -> Self {
        DependencyResolverV2 { nodes: HashSet::new(), deps: HashMap::new() }
    }

    /// Register a node with no dependencies.
    pub fn add_node(&mut self, node: impl Into<String>) {
        let n = node.into();
        self.nodes.insert(n.clone());
        self.deps.entry(n).or_default();
    }

    /// Declare that `node` depends on `dep`.
    pub fn add_dep(&mut self, node: impl Into<String>, dep: impl Into<String>) {
        let n = node.into();
        let d = dep.into();
        self.nodes.insert(n.clone());
        self.nodes.insert(d.clone());
        self.deps.entry(d.clone()).or_default();
        self.deps.entry(n).or_default().push(d);
    }

    /// Resolve the dependency order (Kahn's algorithm).
    pub fn resolve(&self) -> ResolveResult {
        /* build in-degree map */
        let mut in_degree: HashMap<&str, usize> = self.nodes.iter().map(|n| (n.as_str(), 0)).collect();
        /* adjacency: dep -> nodes that depend on dep */
        let mut adj: HashMap<&str, Vec<&str>> = self.nodes.iter().map(|n| (n.as_str(), vec![])).collect();
        for (node, deps) in &self.deps {
            for dep in deps {
                *in_degree.entry(node.as_str()).or_insert(0) += 1;
                adj.entry(dep.as_str()).or_default().push(node.as_str());
            }
        }
        let mut queue: VecDeque<&str> =
            in_degree.iter().filter(|(_, &d)| d == 0).map(|(&n, _)| n).collect();
        let mut order = Vec::new();
        while let Some(n) = queue.pop_front() {
            order.push(n.to_owned());
            if let Some(dependents) = adj.get(n) {
                for &dep in dependents {
                    let Some(cnt) = in_degree.get_mut(dep) else { continue };
                    *cnt = cnt.saturating_sub(1);
                    if *cnt == 0 {
                        queue.push_back(dep);
                    }
                }
            }
        }
        if order.len() == self.nodes.len() {
            ResolveResult::Order(order)
        } else {
            let in_cycle: Vec<String> = self
                .nodes
                .iter()
                .filter(|n| !order.contains(&n.to_string()))
                .cloned()
                .collect();
            ResolveResult::Cycle(in_cycle)
        }
    }

    /// Number of registered nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Number of dependency edges.
    pub fn edge_count(&self) -> usize {
        self.deps.values().map(|v| v.len()).sum()
    }
}

/// Create a new dependency resolver.
pub fn new_dependency_resolver_v2() -> DependencyResolverV2 {
    DependencyResolverV2::new()
}

/// Add a node.
pub fn dr_add_node(r: &mut DependencyResolverV2, node: &str) {
    r.add_node(node);
}

/// Add a dependency edge.
pub fn dr_add_dep(r: &mut DependencyResolverV2, node: &str, dep: &str) {
    r.add_dep(node, dep);
}

/// Resolve order.
pub fn dr_resolve(r: &DependencyResolverV2) -> ResolveResult {
    r.resolve()
}

/// Node count.
pub fn dr_node_count(r: &DependencyResolverV2) -> usize {
    r.node_count()
}

/// Edge count.
pub fn dr_edge_count(r: &DependencyResolverV2) -> usize {
    r.edge_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_node() {
        let mut r = new_dependency_resolver_v2();
        dr_add_node(&mut r, "A");
        match dr_resolve(&r) {
            ResolveResult::Order(o) => assert_eq!(o, vec!["A"] /* single node */),
            _ => panic!("expected order"),
        }
    }

    #[test]
    fn test_chain() {
        let mut r = new_dependency_resolver_v2();
        dr_add_dep(&mut r, "B", "A"); /* B depends on A */
        dr_add_dep(&mut r, "C", "B"); /* C depends on B */
        match dr_resolve(&r) {
            ResolveResult::Order(o) => {
                assert!(o.iter().position(|x| x == "A") < o.iter().position(|x| x == "B") /* A before B */);
                assert!(o.iter().position(|x| x == "B") < o.iter().position(|x| x == "C"));
            }
            _ => panic!("expected order"),
        }
    }

    #[test]
    fn test_cycle_detection() {
        let mut r = new_dependency_resolver_v2();
        dr_add_dep(&mut r, "A", "B");
        dr_add_dep(&mut r, "B", "A");
        match dr_resolve(&r) {
            ResolveResult::Cycle(_) => { /* cycle detected */ }
            _ => panic!("expected cycle"),
        }
    }

    #[test]
    fn test_node_count() {
        let mut r = new_dependency_resolver_v2();
        dr_add_node(&mut r, "X");
        dr_add_node(&mut r, "Y");
        assert_eq!(dr_node_count(&r), 2 /* two nodes */);
    }

    #[test]
    fn test_edge_count() {
        let mut r = new_dependency_resolver_v2();
        dr_add_dep(&mut r, "B", "A");
        assert_eq!(dr_edge_count(&r), 1 /* one edge */);
    }

    #[test]
    fn test_diamond() {
        let mut r = new_dependency_resolver_v2();
        dr_add_dep(&mut r, "B", "A");
        dr_add_dep(&mut r, "C", "A");
        dr_add_dep(&mut r, "D", "B");
        dr_add_dep(&mut r, "D", "C");
        match dr_resolve(&r) {
            ResolveResult::Order(o) => {
                let ai = o.iter().position(|x| x == "A").expect("should succeed");
                let di = o.iter().position(|x| x == "D").expect("should succeed");
                assert!(ai < di /* A before D */);
            }
            _ => panic!("expected order"),
        }
    }

    #[test]
    fn test_empty_resolver() {
        let r = new_dependency_resolver_v2();
        match dr_resolve(&r) {
            ResolveResult::Order(o) => assert!(o.is_empty() /* empty */),
            _ => panic!("expected empty order"),
        }
    }

    #[test]
    fn test_no_deps() {
        let mut r = new_dependency_resolver_v2();
        dr_add_node(&mut r, "P");
        dr_add_node(&mut r, "Q");
        match dr_resolve(&r) {
            ResolveResult::Order(o) => assert_eq!(o.len(), 2 /* two independent nodes */),
            _ => panic!("expected order"),
        }
    }

    #[test]
    fn test_auto_register_deps() {
        let mut r = new_dependency_resolver_v2();
        dr_add_dep(&mut r, "B", "A");
        assert_eq!(dr_node_count(&r), 2 /* both auto-registered */);
    }
}
