#![allow(dead_code)]
//! Scene animation export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SceneAnimationExport { name: String, duration: f32, channel_count: u32, loop_mode: String }

#[allow(dead_code)]
pub fn export_scene_animation(name: &str, duration: f32, channels: u32) -> SceneAnimationExport {
    SceneAnimationExport { name: name.to_string(), duration, channel_count: channels, loop_mode: "once".to_string() }
}
#[allow(dead_code)]
pub fn animation_name_sae(m: &SceneAnimationExport) -> &str { &m.name }
#[allow(dead_code)]
pub fn animation_duration_sae(m: &SceneAnimationExport) -> f32 { m.duration }
#[allow(dead_code)]
pub fn animation_channel_count_sae(m: &SceneAnimationExport) -> u32 { m.channel_count }
#[allow(dead_code)]
pub fn animation_to_json(m: &SceneAnimationExport) -> String {
    format!("{{\"name\":\"{}\",\"duration\":{:.4},\"channels\":{},\"loop\":\"{}\"}}", m.name, m.duration, m.channel_count, m.loop_mode)
}
#[allow(dead_code)]
pub fn animation_loop_mode(m: &SceneAnimationExport) -> &str { &m.loop_mode }
#[allow(dead_code)]
pub fn animation_export_size(m: &SceneAnimationExport) -> usize { m.name.len() + 12 }
#[allow(dead_code)]
pub fn validate_scene_animation(m: &SceneAnimationExport) -> bool { !m.name.is_empty() && m.duration >= 0.0 }

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> SceneAnimationExport { export_scene_animation("walk", 2.0, 3) }
    #[test] fn test_export() { let m = data(); assert_eq!(animation_name_sae(&m), "walk"); }
    #[test] fn test_name() { let m = data(); assert_eq!(animation_name_sae(&m), "walk"); }
    #[test] fn test_duration() { let m = data(); assert!((animation_duration_sae(&m) - 2.0).abs() < 1e-6); }
    #[test] fn test_channels() { let m = data(); assert_eq!(animation_channel_count_sae(&m), 3); }
    #[test] fn test_json() { let m = data(); assert!(animation_to_json(&m).contains("walk")); }
    #[test] fn test_loop() { let m = data(); assert_eq!(animation_loop_mode(&m), "once"); }
    #[test] fn test_size() { let m = data(); assert!(animation_export_size(&m) > 0); }
    #[test] fn test_validate() { let m = data(); assert!(validate_scene_animation(&m)); }
    #[test] fn test_invalid_name() { let m = export_scene_animation("", 1.0, 1); assert!(!validate_scene_animation(&m)); }
    #[test] fn test_invalid_dur() { let m = export_scene_animation("x", -1.0, 1); assert!(!validate_scene_animation(&m)); }
}
