#![allow(dead_code)]

//! Mesh skin bind matrix export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshSkinBindExport {
    pub joints: Vec<SkinBindJoint>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SkinBindJoint {
    pub name: String,
    pub bind_matrix: [f32; 16],
}

#[allow(dead_code)]
pub fn export_skin_bind(joints: Vec<SkinBindJoint>) -> MeshSkinBindExport {
    MeshSkinBindExport { joints }
}

#[allow(dead_code)]
pub fn bind_joint_count(exp: &MeshSkinBindExport) -> usize { exp.joints.len() }

#[allow(dead_code)]
pub fn bind_matrix_at(exp: &MeshSkinBindExport, idx: usize) -> Option<&[f32; 16]> {
    exp.joints.get(idx).map(|j| &j.bind_matrix)
}

#[allow(dead_code)]
pub fn bind_to_json(exp: &MeshSkinBindExport) -> String {
    let items: Vec<String> = exp.joints.iter().map(|j|
        format!("{{\"name\":\"{}\"}}", j.name)
    ).collect();
    format!("{{\"joint_count\":{},\"joints\":[{}]}}", exp.joints.len(), items.join(","))
}

#[allow(dead_code)]
pub fn bind_to_bytes(exp: &MeshSkinBindExport) -> Vec<u8> {
    let mut bytes = Vec::new();
    for j in &exp.joints {
        for &v in &j.bind_matrix { bytes.extend_from_slice(&v.to_le_bytes()); }
    }
    bytes
}

#[allow(dead_code)]
pub fn bind_inverse_matrix(mat: &[f32; 16]) -> [f32; 16] {
    // Simple transpose for orthogonal matrices (approximation)
    let mut result = [0.0f32; 16];
    for r in 0..4 { for c in 0..4 { result[r*4+c] = mat[c*4+r]; } }
    result
}

#[allow(dead_code)]
pub fn bind_export_size(exp: &MeshSkinBindExport) -> usize { exp.joints.len() * 64 }

#[allow(dead_code)]
pub fn validate_skin_bind(exp: &MeshSkinBindExport) -> bool {
    !exp.joints.is_empty() && exp.joints.iter().all(|j| !j.name.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn jnt(n: &str) -> SkinBindJoint {
        let mut m = [0.0f32; 16]; m[0]=1.0; m[5]=1.0; m[10]=1.0; m[15]=1.0;
        SkinBindJoint { name: n.into(), bind_matrix: m }
    }

    #[test]
    fn test_export() { let e = export_skin_bind(vec![jnt("hip")]); assert_eq!(bind_joint_count(&e), 1); }
    #[test]
    fn test_matrix_at() { let e = export_skin_bind(vec![jnt("a")]); assert!(bind_matrix_at(&e, 0).is_some()); }
    #[test]
    fn test_matrix_none() { let e = export_skin_bind(vec![]); assert!(bind_matrix_at(&e, 0).is_none()); }
    #[test]
    fn test_to_json() { let e = export_skin_bind(vec![jnt("a")]); assert!(bind_to_json(&e).contains("\"joint_count\":1")); }
    #[test]
    fn test_to_bytes() { let e = export_skin_bind(vec![jnt("a")]); assert_eq!(bind_to_bytes(&e).len(), 64); }
    #[test]
    fn test_inverse() { let m = [1.0,0.0,0.0,0.0, 0.0,1.0,0.0,0.0, 0.0,0.0,1.0,0.0, 0.0,0.0,0.0,1.0]; let inv = bind_inverse_matrix(&m); assert!((inv[0]-1.0).abs()<1e-6); }
    #[test]
    fn test_export_size() { let e = export_skin_bind(vec![jnt("a")]); assert_eq!(bind_export_size(&e), 64); }
    #[test]
    fn test_validate() { assert!(validate_skin_bind(&export_skin_bind(vec![jnt("a")]))); }
    #[test]
    fn test_validate_empty() { assert!(!validate_skin_bind(&export_skin_bind(vec![]))); }
    #[test]
    fn test_multi() { let e = export_skin_bind(vec![jnt("a"),jnt("b")]); assert_eq!(bind_joint_count(&e), 2); }
}
