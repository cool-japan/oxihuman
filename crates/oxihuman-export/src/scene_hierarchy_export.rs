#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// A node in the scene hierarchy.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HierarchyNode {
    pub name: String,
    pub children: Vec<HierarchyNode>,
}

/// Scene hierarchy export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SceneHierarchyExport {
    pub roots: Vec<HierarchyNode>,
}

/// Build a scene hierarchy from root nodes.
#[allow(dead_code)]
pub fn export_scene_hierarchy(roots: Vec<HierarchyNode>) -> SceneHierarchyExport {
    SceneHierarchyExport { roots }
}

/// Count total nodes recursively.
#[allow(dead_code)]
pub fn hierarchy_node_count(exp: &SceneHierarchyExport) -> usize {
    fn count(node: &HierarchyNode) -> usize {
        1 + node.children.iter().map(count).sum::<usize>()
    }
    exp.roots.iter().map(count).sum()
}

/// Return number of root nodes.
#[allow(dead_code)]
pub fn hierarchy_root_count(exp: &SceneHierarchyExport) -> usize {
    exp.roots.len()
}

/// Return maximum depth of the hierarchy.
#[allow(dead_code)]
pub fn hierarchy_depth(exp: &SceneHierarchyExport) -> usize {
    fn depth(node: &HierarchyNode) -> usize {
        if node.children.is_empty() {
            1
        } else {
            1 + node.children.iter().map(depth).max().unwrap_or(0)
        }
    }
    exp.roots.iter().map(depth).max().unwrap_or(0)
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn hierarchy_to_json(exp: &SceneHierarchyExport) -> String {
    fn node_json(n: &HierarchyNode) -> String {
        let children: Vec<String> = n.children.iter().map(node_json).collect();
        format!("{{\"name\":\"{}\",\"children\":[{}]}}", n.name, children.join(","))
    }
    let roots: Vec<String> = exp.roots.iter().map(node_json).collect();
    format!("{{\"roots\":[{}]}}", roots.join(","))
}

/// Return number of children of a root node at index.
#[allow(dead_code)]
pub fn node_children_count(exp: &SceneHierarchyExport, root_index: usize) -> usize {
    exp.roots.get(root_index).map_or(0, |n| n.children.len())
}

/// Return name of a root node.
#[allow(dead_code)]
pub fn node_name(exp: &SceneHierarchyExport, root_index: usize) -> Option<&str> {
    exp.roots.get(root_index).map(|n| n.name.as_str())
}

/// Validate: all nodes have non-empty names.
#[allow(dead_code)]
pub fn validate_hierarchy_export(exp: &SceneHierarchyExport) -> bool {
    fn valid(n: &HierarchyNode) -> bool {
        !n.name.is_empty() && n.children.iter().all(valid)
    }
    exp.roots.iter().all(valid)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> SceneHierarchyExport {
        export_scene_hierarchy(vec![
            HierarchyNode {
                name: "root".to_string(),
                children: vec![
                    HierarchyNode { name: "child_a".to_string(), children: vec![] },
                    HierarchyNode {
                        name: "child_b".to_string(),
                        children: vec![
                            HierarchyNode { name: "grandchild".to_string(), children: vec![] },
                        ],
                    },
                ],
            },
        ])
    }

    #[test]
    fn test_node_count() {
        assert_eq!(hierarchy_node_count(&sample()), 4);
    }

    #[test]
    fn test_root_count() {
        assert_eq!(hierarchy_root_count(&sample()), 1);
    }

    #[test]
    fn test_depth() {
        assert_eq!(hierarchy_depth(&sample()), 3);
    }

    #[test]
    fn test_to_json() {
        let j = hierarchy_to_json(&sample());
        assert!(j.contains("\"root\""));
        assert!(j.contains("\"grandchild\""));
    }

    #[test]
    fn test_children_count() {
        assert_eq!(node_children_count(&sample(), 0), 2);
        assert_eq!(node_children_count(&sample(), 5), 0);
    }

    #[test]
    fn test_node_name() {
        assert_eq!(node_name(&sample(), 0), Some("root"));
        assert_eq!(node_name(&sample(), 10), None);
    }

    #[test]
    fn test_validate_ok() {
        assert!(validate_hierarchy_export(&sample()));
    }

    #[test]
    fn test_validate_empty_name() {
        let e = export_scene_hierarchy(vec![HierarchyNode { name: String::new(), children: vec![] }]);
        assert!(!validate_hierarchy_export(&e));
    }

    #[test]
    fn test_empty_hierarchy() {
        let e = export_scene_hierarchy(vec![]);
        assert_eq!(hierarchy_node_count(&e), 0);
        assert_eq!(hierarchy_depth(&e), 0);
    }

    #[test]
    fn test_flat_hierarchy() {
        let e = export_scene_hierarchy(vec![
            HierarchyNode { name: "a".to_string(), children: vec![] },
            HierarchyNode { name: "b".to_string(), children: vec![] },
        ]);
        assert_eq!(hierarchy_depth(&e), 1);
        assert_eq!(hierarchy_root_count(&e), 2);
    }
}
