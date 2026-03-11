#![allow(dead_code)]

//! Scene camera export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SceneCameraExport {
    pub camera_type: String,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

#[allow(dead_code)]
pub fn export_scene_camera(camera_type: &str, fov: f32, near: f32, far: f32) -> SceneCameraExport {
    SceneCameraExport { camera_type: camera_type.to_string(), fov, near, far }
}

#[allow(dead_code)]
pub fn camera_type_sce(cam: &SceneCameraExport) -> &str { &cam.camera_type }

#[allow(dead_code)]
pub fn camera_fov_sce(cam: &SceneCameraExport) -> f32 { cam.fov }

#[allow(dead_code)]
pub fn camera_near_sce(cam: &SceneCameraExport) -> f32 { cam.near }

#[allow(dead_code)]
pub fn camera_far_sce(cam: &SceneCameraExport) -> f32 { cam.far }

#[allow(dead_code)]
pub fn camera_to_json_sce(cam: &SceneCameraExport) -> String {
    format!("{{\"type\":\"{}\",\"fov\":{:.4},\"near\":{:.4},\"far\":{:.4}}}", cam.camera_type, cam.fov, cam.near, cam.far)
}

#[allow(dead_code)]
pub fn camera_export_size(cam: &SceneCameraExport) -> usize {
    cam.camera_type.len() + 12
}

#[allow(dead_code)]
pub fn validate_scene_camera(cam: &SceneCameraExport) -> bool {
    cam.fov > 0.0 && cam.near > 0.0 && cam.far > cam.near
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export() { let c = export_scene_camera("perspective", 60.0, 0.1, 1000.0); assert_eq!(camera_type_sce(&c), "perspective"); }
    #[test]
    fn test_fov() { let c = export_scene_camera("p", 45.0, 0.1, 100.0); assert!((camera_fov_sce(&c) - 45.0).abs() < 1e-6); }
    #[test]
    fn test_near() { let c = export_scene_camera("p", 60.0, 0.5, 100.0); assert!((camera_near_sce(&c) - 0.5).abs() < 1e-6); }
    #[test]
    fn test_far() { let c = export_scene_camera("p", 60.0, 0.1, 500.0); assert!((camera_far_sce(&c) - 500.0).abs() < 1e-6); }
    #[test]
    fn test_to_json() { let c = export_scene_camera("ortho", 60.0, 0.1, 100.0); assert!(camera_to_json_sce(&c).contains("\"type\":\"ortho\"")); }
    #[test]
    fn test_export_size() { let c = export_scene_camera("p", 60.0, 0.1, 100.0); assert!(camera_export_size(&c) > 0); }
    #[test]
    fn test_validate() { assert!(validate_scene_camera(&export_scene_camera("p", 60.0, 0.1, 100.0))); }
    #[test]
    fn test_validate_bad_fov() { assert!(!validate_scene_camera(&export_scene_camera("p", 0.0, 0.1, 100.0))); }
    #[test]
    fn test_validate_bad_range() { assert!(!validate_scene_camera(&export_scene_camera("p", 60.0, 100.0, 0.1))); }
    #[test]
    fn test_type() { let c = export_scene_camera("perspective", 60.0, 0.1, 100.0); assert_eq!(camera_type_sce(&c), "perspective"); }
}
