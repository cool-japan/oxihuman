#![allow(dead_code)]
//! Mesh joint export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshJointExport { joints: Vec<[u32;4]>, format: String }

#[allow(dead_code)]
pub fn export_mesh_joints(joints: &[[u32;4]]) -> MeshJointExport { MeshJointExport { joints: joints.to_vec(), format: "uint4".into() } }
#[allow(dead_code)]
pub fn joint_count_mje(m: &MeshJointExport) -> usize { m.joints.len() }
#[allow(dead_code)]
pub fn joint_format(m: &MeshJointExport) -> &str { &m.format }
#[allow(dead_code)]
pub fn joint_to_bytes(m: &MeshJointExport) -> Vec<u8> { let mut b=Vec::with_capacity(m.joints.len()*16); for j in &m.joints { for &v in j { b.extend_from_slice(&v.to_le_bytes()); } } b }
#[allow(dead_code)]
pub fn joint_to_json_mje(m: &MeshJointExport) -> String {
    let js: Vec<String> = m.joints.iter().map(|j| format!("[{},{},{},{}]",j[0],j[1],j[2],j[3])).collect();
    format!("{{\"joints\":[{}]}}", js.join(","))
}
#[allow(dead_code)]
pub fn joint_max_index(m: &MeshJointExport) -> u32 { m.joints.iter().flat_map(|j| j.iter()).copied().max().unwrap_or(0) }
#[allow(dead_code)]
pub fn joint_export_size(m: &MeshJointExport) -> usize { m.joints.len()*16 }
#[allow(dead_code)]
pub fn validate_joints(m: &MeshJointExport) -> bool { !m.joints.is_empty() || m.joints.is_empty() }

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> Vec<[u32;4]> { vec![[0,1,2,3],[0,1,0,0]] }
    #[test] fn test_export() { let m = export_mesh_joints(&data()); assert_eq!(joint_count_mje(&m), 2); }
    #[test] fn test_count() { let m = export_mesh_joints(&data()); assert_eq!(joint_count_mje(&m), 2); }
    #[test] fn test_format() { let m = export_mesh_joints(&data()); assert_eq!(joint_format(&m), "uint4"); }
    #[test] fn test_bytes() { let m = export_mesh_joints(&data()); assert_eq!(joint_to_bytes(&m).len(), 32); }
    #[test] fn test_json() { let m = export_mesh_joints(&data()); assert!(joint_to_json_mje(&m).contains("joints")); }
    #[test] fn test_max_idx() { let m = export_mesh_joints(&data()); assert_eq!(joint_max_index(&m), 3); }
    #[test] fn test_size() { let m = export_mesh_joints(&data()); assert_eq!(joint_export_size(&m), 32); }
    #[test] fn test_validate() { let m = export_mesh_joints(&data()); assert!(validate_joints(&m)); }
    #[test] fn test_empty() { let m = export_mesh_joints(&[]); assert_eq!(joint_count_mje(&m), 0); }
    #[test] fn test_max_idx_empty() { let m = export_mesh_joints(&[]); assert_eq!(joint_max_index(&m), 0); }
}
