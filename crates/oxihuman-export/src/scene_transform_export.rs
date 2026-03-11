#![allow(dead_code)]
//! Scene transform export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SceneTransformExport { position: [f32;3], rotation: [f32;4], scale: [f32;3] }

#[allow(dead_code)]
pub fn export_scene_transform(pos: [f32;3], rot: [f32;4], scale: [f32;3]) -> SceneTransformExport {
    SceneTransformExport { position: pos, rotation: rot, scale }
}
#[allow(dead_code)]
pub fn transform_position_ste(m: &SceneTransformExport) -> [f32;3] { m.position }
#[allow(dead_code)]
pub fn transform_rotation_ste(m: &SceneTransformExport) -> [f32;4] { m.rotation }
#[allow(dead_code)]
pub fn transform_scale_ste(m: &SceneTransformExport) -> [f32;3] { m.scale }
#[allow(dead_code)]
pub fn transform_to_matrix_ste(m: &SceneTransformExport) -> [f32;16] {
    let mut mat = [0.0f32;16];
    mat[0]=m.scale[0]; mat[5]=m.scale[1]; mat[10]=m.scale[2]; mat[15]=1.0;
    mat[12]=m.position[0]; mat[13]=m.position[1]; mat[14]=m.position[2];
    mat
}
#[allow(dead_code)]
pub fn transform_to_json_ste(m: &SceneTransformExport) -> String {
    format!("{{\"position\":[{:.4},{:.4},{:.4}],\"rotation\":[{:.4},{:.4},{:.4},{:.4}],\"scale\":[{:.4},{:.4},{:.4}]}}",
        m.position[0],m.position[1],m.position[2],m.rotation[0],m.rotation[1],m.rotation[2],m.rotation[3],m.scale[0],m.scale[1],m.scale[2])
}
#[allow(dead_code)]
pub fn transform_export_size(_m: &SceneTransformExport) -> usize { 40 }
#[allow(dead_code)]
pub fn validate_scene_transform(m: &SceneTransformExport) -> bool {
    m.position.iter().all(|v| v.is_finite()) && m.rotation.iter().all(|v| v.is_finite()) && m.scale.iter().all(|v| v.is_finite())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> SceneTransformExport { export_scene_transform([1.0,2.0,3.0],[0.0,0.0,0.0,1.0],[1.0,1.0,1.0]) }
    #[test] fn test_export() { let m = data(); assert!((transform_position_ste(&m)[0] - 1.0).abs() < 1e-6); }
    #[test] fn test_pos() { let m = data(); assert!((transform_position_ste(&m)[1] - 2.0).abs() < 1e-6); }
    #[test] fn test_rot() { let m = data(); assert!((transform_rotation_ste(&m)[3] - 1.0).abs() < 1e-6); }
    #[test] fn test_scale() { let m = data(); assert!((transform_scale_ste(&m)[0] - 1.0).abs() < 1e-6); }
    #[test] fn test_matrix() { let m = data(); let mat = transform_to_matrix_ste(&m); assert!((mat[15] - 1.0).abs() < 1e-6); }
    #[test] fn test_json() { let m = data(); assert!(transform_to_json_ste(&m).contains("position")); }
    #[test] fn test_size() { let m = data(); assert_eq!(transform_export_size(&m), 40); }
    #[test] fn test_validate() { let m = data(); assert!(validate_scene_transform(&m)); }
    #[test] fn test_zero() { let m = export_scene_transform([0.0;3],[0.0;4],[0.0;3]); assert!(validate_scene_transform(&m)); }
    #[test] fn test_matrix_translate() { let m = data(); let mat = transform_to_matrix_ste(&m); assert!((mat[12] - 1.0).abs() < 1e-6); }
}
