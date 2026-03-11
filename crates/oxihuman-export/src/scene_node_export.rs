#![allow(dead_code)]
//! Export scene node data.

/// Scene node export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SceneNodeExport {
    pub name: String,
    pub transform: [f32; 16],
    pub mesh_index: Option<usize>,
    pub children: Vec<SceneNodeExport>,
}

/// Export a scene node.
#[allow(dead_code)]
pub fn export_scene_node(name: &str, transform: [f32; 16], mesh_index: Option<usize>) -> SceneNodeExport {
    SceneNodeExport {
        name: name.to_string(),
        transform,
        mesh_index,
        children: Vec::new(),
    }
}

/// Get the node name.
#[allow(dead_code)]
pub fn node_name_export(export: &SceneNodeExport) -> &str {
    &export.name
}

/// Get the node transform.
#[allow(dead_code)]
pub fn node_transform(export: &SceneNodeExport) -> [f32; 16] {
    export.transform
}

/// Get the children of the node.
#[allow(dead_code)]
pub fn node_children(export: &SceneNodeExport) -> &[SceneNodeExport] {
    &export.children
}

/// Get the mesh index.
#[allow(dead_code)]
pub fn node_mesh_index(export: &SceneNodeExport) -> Option<usize> {
    export.mesh_index
}

/// Convert node to JSON.
#[allow(dead_code)]
pub fn node_to_json(export: &SceneNodeExport) -> String {
    let mesh_str = match export.mesh_index {
        Some(i) => format!("{}", i),
        None => "null".to_string(),
    };
    format!(
        "{{\"name\":\"{}\",\"mesh_index\":{},\"children\":{}}}",
        export.name,
        mesh_str,
        export.children.len()
    )
}

/// Get the depth of a node (0 = root).
#[allow(dead_code)]
pub fn node_depth(export: &SceneNodeExport) -> usize {
    fn max_child_depth(node: &SceneNodeExport) -> usize {
        if node.children.is_empty() {
            0
        } else {
            node.children.iter().map(|c| 1 + max_child_depth(c)).max().unwrap_or(0)
        }
    }
    max_child_depth(export)
}

/// Validate a scene node (name must be non-empty, transform must be finite).
#[allow(dead_code)]
pub fn validate_scene_node(export: &SceneNodeExport) -> bool {
    if export.name.is_empty() {
        return false;
    }
    if !export.transform.iter().all(|v| v.is_finite()) {
        return false;
    }
    export.children.iter().all(validate_scene_node)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity_mat() -> [f32; 16] {
        [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ]
    }

    #[test]
    fn test_export_scene_node() {
        let n = export_scene_node("root", identity_mat(), None);
        assert_eq!(node_name_export(&n), "root");
    }

    #[test]
    fn test_node_transform() {
        let n = export_scene_node("root", identity_mat(), None);
        let t = node_transform(&n);
        assert!((t[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_node_children_empty() {
        let n = export_scene_node("root", identity_mat(), None);
        assert!(node_children(&n).is_empty());
    }

    #[test]
    fn test_node_mesh_index() {
        let n = export_scene_node("mesh0", identity_mat(), Some(0));
        assert_eq!(node_mesh_index(&n), Some(0));
    }

    #[test]
    fn test_node_to_json() {
        let n = export_scene_node("root", identity_mat(), None);
        let j = node_to_json(&n);
        assert!(j.contains("root"));
    }

    #[test]
    fn test_node_depth_no_children() {
        let n = export_scene_node("root", identity_mat(), None);
        assert_eq!(node_depth(&n), 0);
    }

    #[test]
    fn test_node_depth_with_children() {
        let mut root = export_scene_node("root", identity_mat(), None);
        let child = export_scene_node("child", identity_mat(), Some(0));
        root.children.push(child);
        assert_eq!(node_depth(&root), 1);
    }

    #[test]
    fn test_validate_scene_node() {
        let n = export_scene_node("root", identity_mat(), None);
        assert!(validate_scene_node(&n));
    }

    #[test]
    fn test_validate_scene_node_empty_name() {
        let n = export_scene_node("", identity_mat(), None);
        assert!(!validate_scene_node(&n));
    }

    #[test]
    fn test_validate_scene_node_nan_transform() {
        let mut mat = identity_mat();
        mat[0] = f32::NAN;
        let n = export_scene_node("root", mat, None);
        assert!(!validate_scene_node(&n));
    }
}
