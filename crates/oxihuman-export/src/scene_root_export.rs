#![allow(dead_code)]
//! Scene root export.

/// Scene root export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SceneRootExport {
    pub name: String,
    pub node_indices: Vec<u32>,
    pub default_scene: bool,
}

/// Export a scene root.
#[allow(dead_code)]
pub fn export_scene_root(name: &str, node_indices: Vec<u32>, default_scene: bool) -> SceneRootExport {
    SceneRootExport {
        name: name.to_string(),
        node_indices,
        default_scene,
    }
}

/// Get root node count.
#[allow(dead_code)]
pub fn root_node_count(e: &SceneRootExport) -> usize {
    e.node_indices.len()
}

/// Get root name.
#[allow(dead_code)]
pub fn root_name(e: &SceneRootExport) -> &str {
    &e.name
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn root_to_json(e: &SceneRootExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"nodes\":{},\"default\":{}}}",
        e.name,
        e.node_indices.len(),
        e.default_scene
    )
}

/// Get child node indices.
#[allow(dead_code)]
pub fn root_child_indices(e: &SceneRootExport) -> &[u32] {
    &e.node_indices
}

/// Check if default scene.
#[allow(dead_code)]
pub fn root_default_scene(e: &SceneRootExport) -> bool {
    e.default_scene
}

/// Get export size estimate.
#[allow(dead_code)]
pub fn root_export_size(e: &SceneRootExport) -> usize {
    e.name.len() + e.node_indices.len() * 4
}

/// Validate scene root.
#[allow(dead_code)]
pub fn validate_scene_root(e: &SceneRootExport) -> bool {
    !e.name.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_scene_root() {
        let e = export_scene_root("Scene", vec![0, 1, 2], true);
        assert_eq!(e.name, "Scene");
    }

    #[test]
    fn test_root_node_count() {
        let e = export_scene_root("s", vec![0, 1], false);
        assert_eq!(root_node_count(&e), 2);
    }

    #[test]
    fn test_root_name() {
        let e = export_scene_root("MyScene", vec![], true);
        assert_eq!(root_name(&e), "MyScene");
    }

    #[test]
    fn test_root_to_json() {
        let e = export_scene_root("s", vec![0], true);
        let j = root_to_json(&e);
        assert!(j.contains("name"));
    }

    #[test]
    fn test_root_child_indices() {
        let e = export_scene_root("s", vec![3, 5], false);
        assert_eq!(root_child_indices(&e), &[3, 5]);
    }

    #[test]
    fn test_root_default_scene() {
        let e = export_scene_root("s", vec![], true);
        assert!(root_default_scene(&e));
    }

    #[test]
    fn test_root_not_default() {
        let e = export_scene_root("s", vec![], false);
        assert!(!root_default_scene(&e));
    }

    #[test]
    fn test_root_export_size() {
        let e = export_scene_root("ab", vec![0], false);
        assert_eq!(root_export_size(&e), 6);
    }

    #[test]
    fn test_validate_ok() {
        let e = export_scene_root("Scene", vec![], true);
        assert!(validate_scene_root(&e));
    }

    #[test]
    fn test_validate_empty_name() {
        let e = export_scene_root("", vec![], true);
        assert!(!validate_scene_root(&e));
    }
}
