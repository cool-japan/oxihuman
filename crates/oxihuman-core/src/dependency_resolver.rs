//! Generic dependency resolver using topological sort (Kahn's algorithm).

#[allow(dead_code)]
#[derive(Clone)]
pub struct Dependency {
    pub name: String,
    pub required: bool,
    pub version_req: Option<String>,
}

#[allow(dead_code)]
pub struct DependencyNode {
    pub id: String,
    pub version: String,
    pub deps: Vec<Dependency>,
}

#[allow(dead_code)]
pub struct DependencyGraph {
    pub nodes: Vec<DependencyNode>,
}

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum ResolveError {
    CircularDependency(Vec<String>),
    MissingDependency(String),
    VersionConflict(String),
}

#[allow(dead_code)]
pub struct ResolveResult {
    /// Resolved load order.
    pub order: Vec<String>,
    pub warnings: Vec<String>,
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn new_dependency_graph() -> DependencyGraph {
    DependencyGraph { nodes: Vec::new() }
}

#[allow(dead_code)]
pub fn add_dep_node(graph: &mut DependencyGraph, node: DependencyNode) {
    graph.nodes.push(node);
}

// ---------------------------------------------------------------------------
// Query helpers
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn dep_node_count(graph: &DependencyGraph) -> usize {
    graph.nodes.len()
}

#[allow(dead_code)]
pub fn get_dep_node<'a>(graph: &'a DependencyGraph, id: &str) -> Option<&'a DependencyNode> {
    graph.nodes.iter().find(|n| n.id == id)
}

#[allow(dead_code)]
pub fn remove_dep_node(graph: &mut DependencyGraph, id: &str) -> bool {
    let before = graph.nodes.len();
    graph.nodes.retain(|n| n.id != id);
    graph.nodes.len() < before
}

#[allow(dead_code)]
pub fn missing_dependencies(graph: &DependencyGraph) -> Vec<String> {
    let ids: std::collections::HashSet<&str> = graph.nodes.iter().map(|n| n.id.as_str()).collect();
    let mut missing = Vec::new();
    for node in &graph.nodes {
        for dep in &node.deps {
            if dep.required && !ids.contains(dep.name.as_str()) && !missing.contains(&dep.name) {
                missing.push(dep.name.clone());
            }
        }
    }
    missing
}

#[allow(dead_code)]
pub fn optional_dep_count(graph: &DependencyGraph) -> usize {
    graph
        .nodes
        .iter()
        .flat_map(|n| n.deps.iter())
        .filter(|d| !d.required)
        .count()
}

#[allow(dead_code)]
pub fn direct_dependents<'a>(graph: &'a DependencyGraph, id: &str) -> Vec<&'a str> {
    graph
        .nodes
        .iter()
        .filter(|n| n.deps.iter().any(|d| d.name == id))
        .map(|n| n.id.as_str())
        .collect()
}

#[allow(dead_code)]
pub fn all_dependents_transitive(graph: &DependencyGraph, id: &str) -> Vec<String> {
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(id.to_string());
    while let Some(current) = queue.pop_front() {
        let dependents = direct_dependents(graph, &current);
        for dep in dependents {
            if !visited.contains(dep) {
                visited.insert(dep.to_string());
                queue.push_back(dep.to_string());
            }
        }
    }
    visited.into_iter().collect()
}

// ---------------------------------------------------------------------------
// Topological sort (Kahn's algorithm)
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn resolve_dependencies(graph: &DependencyGraph) -> Result<ResolveResult, ResolveError> {
    use std::collections::HashMap;
    use std::collections::VecDeque;

    // Check for missing required deps first.
    let missing = missing_dependencies(graph);
    if !missing.is_empty() {
        return Err(ResolveError::MissingDependency(missing[0].clone()));
    }

    // Build adjacency: dep_name → dependents (nodes that depend on it).
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    for node in &graph.nodes {
        in_degree.entry(node.id.as_str()).or_insert(0);
        for dep in &node.deps {
            in_degree.entry(dep.name.as_str()).or_insert(0);
        }
    }
    // In-degree for Kahn's: count how many required deps each node has.
    let mut in_deg: HashMap<&str, usize> = HashMap::new();
    for node in &graph.nodes {
        let required_count = node.deps.iter().filter(|d| d.required).count();
        in_deg.insert(node.id.as_str(), required_count);
    }

    // Build reverse mapping: dep → nodes that need it.
    let mut dependents_map: HashMap<&str, Vec<&str>> = HashMap::new();
    for node in &graph.nodes {
        for dep in &node.deps {
            if dep.required {
                dependents_map
                    .entry(dep.name.as_str())
                    .or_default()
                    .push(node.id.as_str());
            }
        }
    }

    let mut queue: VecDeque<&str> = VecDeque::new();
    for node in &graph.nodes {
        if *in_deg.get(node.id.as_str()).unwrap_or(&0) == 0 {
            queue.push_back(node.id.as_str());
        }
    }

    let mut order: Vec<String> = Vec::new();
    while let Some(current) = queue.pop_front() {
        order.push(current.to_string());
        if let Some(deps_of_current) = dependents_map.get(current) {
            for &dependent in deps_of_current {
                let entry = in_deg.entry(dependent).or_insert(0);
                *entry = entry.saturating_sub(1);
                if *entry == 0 {
                    queue.push_back(dependent);
                }
            }
        }
    }

    if order.len() < graph.nodes.len() {
        // Cycle detected — collect nodes not in order.
        let cycle_nodes: Vec<String> = graph
            .nodes
            .iter()
            .filter(|n| !order.contains(&n.id))
            .map(|n| n.id.clone())
            .collect();
        return Err(ResolveError::CircularDependency(cycle_nodes));
    }

    Ok(ResolveResult {
        order,
        warnings: Vec::new(),
    })
}

#[allow(dead_code)]
pub fn has_circular_dependency(graph: &DependencyGraph) -> bool {
    matches!(
        resolve_dependencies(graph),
        Err(ResolveError::CircularDependency(_))
    )
}

// ---------------------------------------------------------------------------
// Serialization
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn dep_graph_to_json(graph: &DependencyGraph) -> String {
    let mut s = String::from("{\"nodes\":[");
    for (i, node) in graph.nodes.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"id\":\"{}\",\"version\":\"{}\",\"deps\":[",
            node.id, node.version
        ));
        for (j, dep) in node.deps.iter().enumerate() {
            if j > 0 {
                s.push(',');
            }
            let req_str = if dep.required { "true" } else { "false" };
            let ver = dep.version_req.as_deref().unwrap_or("");
            s.push_str(&format!(
                "{{\"name\":\"{}\",\"required\":{},\"version_req\":\"{}\"}}",
                dep.name, req_str, ver
            ));
        }
        s.push_str("]}");
    }
    s.push_str("]}");
    s
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(id: &str, deps: &[(&str, bool)]) -> DependencyNode {
        DependencyNode {
            id: id.to_string(),
            version: "1.0.0".to_string(),
            deps: deps
                .iter()
                .map(|(name, required)| Dependency {
                    name: name.to_string(),
                    required: *required,
                    version_req: None,
                })
                .collect(),
        }
    }

    #[test]
    fn test_new_graph() {
        let g = new_dependency_graph();
        assert_eq!(dep_node_count(&g), 0);
    }

    #[test]
    fn test_add_node() {
        let mut g = new_dependency_graph();
        add_dep_node(&mut g, make_node("A", &[]));
        assert_eq!(dep_node_count(&g), 1);
    }

    #[test]
    fn test_resolve_simple_chain() {
        let mut g = new_dependency_graph();
        add_dep_node(&mut g, make_node("A", &[]));
        add_dep_node(&mut g, make_node("B", &[("A", true)]));
        add_dep_node(&mut g, make_node("C", &[("B", true)]));
        let result = resolve_dependencies(&g).unwrap();
        let order = &result.order;
        let a_pos = order.iter().position(|x| x == "A").unwrap();
        let b_pos = order.iter().position(|x| x == "B").unwrap();
        let c_pos = order.iter().position(|x| x == "C").unwrap();
        assert!(a_pos < b_pos);
        assert!(b_pos < c_pos);
    }

    #[test]
    fn test_circular_detection() {
        let mut g = new_dependency_graph();
        add_dep_node(&mut g, make_node("A", &[("B", true)]));
        add_dep_node(&mut g, make_node("B", &[("A", true)]));
        assert!(has_circular_dependency(&g));
    }

    #[test]
    fn test_no_circular_simple() {
        let mut g = new_dependency_graph();
        add_dep_node(&mut g, make_node("A", &[]));
        add_dep_node(&mut g, make_node("B", &[("A", true)]));
        assert!(!has_circular_dependency(&g));
    }

    #[test]
    fn test_missing_dependencies() {
        let mut g = new_dependency_graph();
        add_dep_node(&mut g, make_node("A", &[("Z", true)]));
        let missing = missing_dependencies(&g);
        assert!(missing.contains(&"Z".to_string()));
    }

    #[test]
    fn test_missing_deps_returns_error() {
        let mut g = new_dependency_graph();
        add_dep_node(&mut g, make_node("A", &[("Z", true)]));
        assert!(matches!(
            resolve_dependencies(&g),
            Err(ResolveError::MissingDependency(_))
        ));
    }

    #[test]
    fn test_dep_node_count() {
        let mut g = new_dependency_graph();
        assert_eq!(dep_node_count(&g), 0);
        add_dep_node(&mut g, make_node("A", &[]));
        add_dep_node(&mut g, make_node("B", &[]));
        assert_eq!(dep_node_count(&g), 2);
    }

    #[test]
    fn test_get_dep_node() {
        let mut g = new_dependency_graph();
        add_dep_node(&mut g, make_node("A", &[]));
        assert!(get_dep_node(&g, "A").is_some());
        assert!(get_dep_node(&g, "X").is_none());
    }

    #[test]
    fn test_direct_dependents() {
        let mut g = new_dependency_graph();
        add_dep_node(&mut g, make_node("A", &[]));
        add_dep_node(&mut g, make_node("B", &[("A", true)]));
        add_dep_node(&mut g, make_node("C", &[("A", false)]));
        let deps = direct_dependents(&g, "A");
        assert!(deps.contains(&"B"));
        assert!(deps.contains(&"C"));
    }

    #[test]
    fn test_all_dependents_transitive() {
        let mut g = new_dependency_graph();
        add_dep_node(&mut g, make_node("A", &[]));
        add_dep_node(&mut g, make_node("B", &[("A", true)]));
        add_dep_node(&mut g, make_node("C", &[("B", true)]));
        let trans = all_dependents_transitive(&g, "A");
        assert!(trans.contains(&"B".to_string()));
        assert!(trans.contains(&"C".to_string()));
    }

    #[test]
    fn test_remove_dep_node() {
        let mut g = new_dependency_graph();
        add_dep_node(&mut g, make_node("A", &[]));
        add_dep_node(&mut g, make_node("B", &[]));
        let removed = remove_dep_node(&mut g, "A");
        assert!(removed);
        assert_eq!(dep_node_count(&g), 1);
        let not_removed = remove_dep_node(&mut g, "X");
        assert!(!not_removed);
    }

    #[test]
    fn test_optional_dep_count() {
        let mut g = new_dependency_graph();
        add_dep_node(&mut g, make_node("A", &[("B", false), ("C", true)]));
        add_dep_node(&mut g, make_node("B", &[]));
        add_dep_node(&mut g, make_node("C", &[]));
        assert_eq!(optional_dep_count(&g), 1);
    }

    #[test]
    fn test_dep_graph_to_json() {
        let mut g = new_dependency_graph();
        add_dep_node(&mut g, make_node("A", &[]));
        let json = dep_graph_to_json(&g);
        assert!(json.contains("\"id\":\"A\""));
    }

    #[test]
    fn test_resolve_no_deps() {
        let mut g = new_dependency_graph();
        add_dep_node(&mut g, make_node("X", &[]));
        add_dep_node(&mut g, make_node("Y", &[]));
        let result = resolve_dependencies(&g).unwrap();
        assert_eq!(result.order.len(), 2);
    }
}
