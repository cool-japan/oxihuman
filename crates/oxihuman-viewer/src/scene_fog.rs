#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FogMode { Linear, Exponential, ExponentialSquared }

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SceneFog {
    mode: FogMode,
    density: f32,
    color: [f32; 3],
    start: f32,
    end: f32,
    enabled: bool,
}

#[allow(dead_code)]
pub fn new_scene_fog() -> SceneFog {
    SceneFog { mode: FogMode::Linear, density: 0.01, color: [0.7, 0.7, 0.8], start: 10.0, end: 100.0, enabled: true }
}

#[allow(dead_code)]
pub fn set_fog_mode(fog: &mut SceneFog, mode: FogMode) { fog.mode = mode; }

#[allow(dead_code)]
pub fn fog_density(fog: &SceneFog) -> f32 { fog.density }

#[allow(dead_code)]
pub fn fog_color_sf(fog: &SceneFog) -> [f32; 3] { fog.color }

#[allow(dead_code)]
pub fn fog_start(fog: &SceneFog) -> f32 { fog.start }

#[allow(dead_code)]
pub fn fog_end(fog: &SceneFog) -> f32 { fog.end }

#[allow(dead_code)]
pub fn fog_to_json(fog: &SceneFog) -> String {
    format!("{{\"density\":{:.4},\"start\":{:.2},\"end\":{:.2},\"enabled\":{}}}", fog.density, fog.start, fog.end, fog.enabled)
}

#[allow(dead_code)]
pub fn fog_is_enabled(fog: &SceneFog) -> bool { fog.enabled }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let f = new_scene_fog(); assert!(fog_is_enabled(&f)); }
    #[test] fn test_mode() { let mut f = new_scene_fog(); set_fog_mode(&mut f, FogMode::Exponential); assert_eq!(f.mode, FogMode::Exponential); }
    #[test] fn test_density() { let f = new_scene_fog(); assert!((fog_density(&f) - 0.01).abs() < 1e-6); }
    #[test] fn test_color() { let f = new_scene_fog(); assert!((fog_color_sf(&f)[0] - 0.7).abs() < 1e-6); }
    #[test] fn test_start() { let f = new_scene_fog(); assert!((fog_start(&f) - 10.0).abs() < 1e-6); }
    #[test] fn test_end() { let f = new_scene_fog(); assert!((fog_end(&f) - 100.0).abs() < 1e-6); }
    #[test] fn test_json() { let f = new_scene_fog(); assert!(fog_to_json(&f).contains("density")); }
    #[test] fn test_enabled() { let f = new_scene_fog(); assert!(fog_is_enabled(&f)); }
    #[test] fn test_exp_sq() { let mut f = new_scene_fog(); set_fog_mode(&mut f, FogMode::ExponentialSquared); assert_eq!(f.mode, FogMode::ExponentialSquared); }
    #[test] fn test_linear() { let f = new_scene_fog(); assert_eq!(f.mode, FogMode::Linear); }
}
