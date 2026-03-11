#![allow(dead_code)]

//! Scene environment export (skybox, fog, ambient).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SceneEnvironmentExport {
    pub skybox: String,
    pub ambient_color: [f32; 3],
    pub fog_density: f32,
    pub exposure: f32,
}

#[allow(dead_code)]
pub fn export_scene_environment(skybox: &str, ambient: [f32; 3], fog: f32, exposure: f32) -> SceneEnvironmentExport {
    SceneEnvironmentExport { skybox: skybox.into(), ambient_color: ambient, fog_density: fog, exposure }
}

#[allow(dead_code)]
pub fn env_skybox(env: &SceneEnvironmentExport) -> &str { &env.skybox }

#[allow(dead_code)]
pub fn env_ambient_color(env: &SceneEnvironmentExport) -> [f32; 3] { env.ambient_color }

#[allow(dead_code)]
pub fn env_fog_density(env: &SceneEnvironmentExport) -> f32 { env.fog_density }

#[allow(dead_code)]
pub fn env_to_json(env: &SceneEnvironmentExport) -> String {
    format!("{{\"skybox\":\"{}\",\"ambient\":[{:.3},{:.3},{:.3}],\"fog\":{:.4},\"exposure\":{:.4}}}",
        env.skybox, env.ambient_color[0], env.ambient_color[1], env.ambient_color[2], env.fog_density, env.exposure)
}

#[allow(dead_code)]
pub fn env_exposure_sce(env: &SceneEnvironmentExport) -> f32 { env.exposure }

#[allow(dead_code)]
pub fn env_export_size(env: &SceneEnvironmentExport) -> usize {
    env.skybox.len() + 20
}

#[allow(dead_code)]
pub fn validate_environment(env: &SceneEnvironmentExport) -> bool {
    env.fog_density >= 0.0 && env.exposure > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export() { let e = export_scene_environment("sky.hdr", [0.1;3], 0.01, 1.0); assert_eq!(env_skybox(&e), "sky.hdr"); }
    #[test]
    fn test_ambient() { let e = export_scene_environment("", [0.5,0.5,0.5], 0.0, 1.0); assert!((env_ambient_color(&e)[0]-0.5).abs()<1e-6); }
    #[test]
    fn test_fog() { let e = export_scene_environment("", [0.0;3], 0.05, 1.0); assert!((env_fog_density(&e)-0.05).abs()<1e-6); }
    #[test]
    fn test_exposure() { let e = export_scene_environment("", [0.0;3], 0.0, 2.5); assert!((env_exposure_sce(&e)-2.5).abs()<1e-6); }
    #[test]
    fn test_to_json() { let e = export_scene_environment("sky", [0.0;3], 0.0, 1.0); assert!(env_to_json(&e).contains("\"skybox\":\"sky\"")); }
    #[test]
    fn test_export_size() { let e = export_scene_environment("a", [0.0;3], 0.0, 1.0); assert!(env_export_size(&e) > 0); }
    #[test]
    fn test_validate() { assert!(validate_environment(&export_scene_environment("", [0.0;3], 0.0, 1.0))); }
    #[test]
    fn test_validate_bad_exposure() { assert!(!validate_environment(&export_scene_environment("", [0.0;3], 0.0, 0.0))); }
    #[test]
    fn test_validate_bad_fog() { assert!(!validate_environment(&export_scene_environment("", [0.0;3], -1.0, 1.0))); }
    #[test]
    fn test_skybox_empty() { let e = export_scene_environment("", [0.0;3], 0.0, 1.0); assert_eq!(env_skybox(&e), ""); }
}
