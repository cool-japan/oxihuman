#![allow(dead_code)]
//! Vertex skin export.

/// Vertex skin export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct VertexSkinExport {
    pub joint_indices: Vec<[u16; 4]>,
    pub joint_weights: Vec<[f32; 4]>,
    pub max_joints: u32,
}

/// Export vertex skin data.
#[allow(dead_code)]
pub fn export_vertex_skin(
    joint_indices: Vec<[u16; 4]>,
    joint_weights: Vec<[f32; 4]>,
    max_joints: u32,
) -> VertexSkinExport {
    VertexSkinExport { joint_indices, joint_weights, max_joints }
}

/// Get joint indices for a vertex.
#[allow(dead_code)]
pub fn skin_joint_indices(e: &VertexSkinExport, index: usize) -> [u16; 4] {
    if index < e.joint_indices.len() {
        e.joint_indices[index]
    } else {
        [0; 4]
    }
}

/// Get joint weights for a vertex.
#[allow(dead_code)]
pub fn skin_joint_weights(e: &VertexSkinExport, index: usize) -> [f32; 4] {
    if index < e.joint_weights.len() {
        e.joint_weights[index]
    } else {
        [0.0; 4]
    }
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn skin_to_json(e: &VertexSkinExport) -> String {
    format!(
        "{{\"vertex_count\":{},\"max_joints\":{}}}",
        e.joint_indices.len(),
        e.max_joints
    )
}

/// Get max joints per vertex.
#[allow(dead_code)]
pub fn skin_max_joints(e: &VertexSkinExport) -> u32 {
    e.max_joints
}

/// Get vertex count.
#[allow(dead_code)]
pub fn skin_vertex_count_vse(e: &VertexSkinExport) -> usize {
    e.joint_indices.len()
}

/// Get export size estimate.
#[allow(dead_code)]
pub fn skin_export_size(e: &VertexSkinExport) -> usize {
    e.joint_indices.len() * (8 + 16) // 4 u16 + 4 f32
}

/// Validate skin data.
#[allow(dead_code)]
pub fn validate_vertex_skin(e: &VertexSkinExport) -> bool {
    if e.joint_indices.len() != e.joint_weights.len() {
        return false;
    }
    for w in &e.joint_weights {
        let sum: f32 = w.iter().sum();
        if (sum - 1.0).abs() > 0.01 && sum > 0.01 {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_vertex_skin() {
        let e = export_vertex_skin(
            vec![[0, 1, 0, 0]],
            vec![[0.5, 0.5, 0.0, 0.0]],
            4,
        );
        assert_eq!(e.joint_indices.len(), 1);
    }

    #[test]
    fn test_skin_joint_indices() {
        let e = export_vertex_skin(vec![[1, 2, 3, 4]], vec![[0.25; 4]], 4);
        assert_eq!(skin_joint_indices(&e, 0), [1, 2, 3, 4]);
        assert_eq!(skin_joint_indices(&e, 5), [0; 4]);
    }

    #[test]
    fn test_skin_joint_weights() {
        let e = export_vertex_skin(vec![[0; 4]], vec![[1.0, 0.0, 0.0, 0.0]], 4);
        assert!((skin_joint_weights(&e, 0)[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_skin_to_json() {
        let e = export_vertex_skin(vec![], vec![], 4);
        let j = skin_to_json(&e);
        assert!(j.contains("max_joints"));
    }

    #[test]
    fn test_skin_max_joints() {
        let e = export_vertex_skin(vec![], vec![], 8);
        assert_eq!(skin_max_joints(&e), 8);
    }

    #[test]
    fn test_skin_vertex_count() {
        let e = export_vertex_skin(vec![[0; 4]; 5], vec![[0.25; 4]; 5], 4);
        assert_eq!(skin_vertex_count_vse(&e), 5);
    }

    #[test]
    fn test_skin_export_size() {
        let e = export_vertex_skin(vec![[0; 4]], vec![[1.0, 0.0, 0.0, 0.0]], 4);
        assert_eq!(skin_export_size(&e), 24);
    }

    #[test]
    fn test_validate_ok() {
        let e = export_vertex_skin(
            vec![[0, 1, 0, 0]],
            vec![[0.5, 0.5, 0.0, 0.0]],
            4,
        );
        assert!(validate_vertex_skin(&e));
    }

    #[test]
    fn test_validate_mismatch() {
        let e = export_vertex_skin(vec![[0; 4]; 2], vec![[0.25; 4]], 4);
        assert!(!validate_vertex_skin(&e));
    }

    #[test]
    fn test_validate_bad_weights() {
        let e = export_vertex_skin(vec![[0; 4]], vec![[0.5, 0.5, 0.5, 0.0]], 4);
        assert!(!validate_vertex_skin(&e));
    }
}
