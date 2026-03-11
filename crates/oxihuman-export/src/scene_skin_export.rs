#![allow(dead_code)]
//! Scene skin export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SceneSkinExport { joint_count: u32, root_joint: String, bind_matrices: Vec<[f32;16]>, skeleton_ref: String }

#[allow(dead_code)]
pub fn export_scene_skin(root: &str, bind_matrices: &[[f32;16]]) -> SceneSkinExport {
    SceneSkinExport { joint_count: bind_matrices.len() as u32, root_joint: root.to_string(), bind_matrices: bind_matrices.to_vec(), skeleton_ref: String::new() }
}
#[allow(dead_code)]
pub fn skin_joint_count_sse(m: &SceneSkinExport) -> u32 { m.joint_count }
#[allow(dead_code)]
pub fn skin_root_joint(m: &SceneSkinExport) -> &str { &m.root_joint }
#[allow(dead_code)]
pub fn skin_bind_matrices(m: &SceneSkinExport) -> &[[f32;16]] { &m.bind_matrices }
#[allow(dead_code)]
pub fn skin_to_json_sse(m: &SceneSkinExport) -> String {
    format!("{{\"joint_count\":{},\"root\":\"{}\",\"skeleton_ref\":\"{}\"}}", m.joint_count, m.root_joint, m.skeleton_ref)
}
#[allow(dead_code)]
pub fn skin_skeleton_ref(m: &SceneSkinExport) -> &str { &m.skeleton_ref }
#[allow(dead_code)]
pub fn skin_export_size(m: &SceneSkinExport) -> usize { m.bind_matrices.len() * 64 + m.root_joint.len() }
#[allow(dead_code)]
pub fn validate_scene_skin(m: &SceneSkinExport) -> bool { !m.root_joint.is_empty() && m.joint_count > 0 }

#[cfg(test)]
mod tests {
    use super::*;
    fn identity() -> [f32;16] { let mut m = [0.0;16]; m[0]=1.0; m[5]=1.0; m[10]=1.0; m[15]=1.0; m }
    fn data() -> SceneSkinExport { export_scene_skin("hips", &[identity(), identity()]) }
    #[test] fn test_export() { let m = data(); assert_eq!(skin_joint_count_sse(&m), 2); }
    #[test] fn test_count() { let m = data(); assert_eq!(skin_joint_count_sse(&m), 2); }
    #[test] fn test_root() { let m = data(); assert_eq!(skin_root_joint(&m), "hips"); }
    #[test] fn test_bind() { let m = data(); assert_eq!(skin_bind_matrices(&m).len(), 2); }
    #[test] fn test_json() { let m = data(); assert!(skin_to_json_sse(&m).contains("hips")); }
    #[test] fn test_skel_ref() { let m = data(); assert!(skin_skeleton_ref(&m).is_empty()); }
    #[test] fn test_size() { let m = data(); assert!(skin_export_size(&m) > 0); }
    #[test] fn test_validate() { let m = data(); assert!(validate_scene_skin(&m)); }
    #[test] fn test_invalid_root() { let m = export_scene_skin("", &[identity()]); assert!(!validate_scene_skin(&m)); }
    #[test] fn test_invalid_empty() { let m = export_scene_skin("root", &[]); assert!(!validate_scene_skin(&m)); }
}
