#![allow(dead_code)]
//! Render state export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderStateExport { blend_mode: String, depth_test: bool, cull_mode: String }

#[allow(dead_code)]
pub fn export_render_state(blend: &str, depth: bool, cull: &str) -> RenderStateExport {
    RenderStateExport { blend_mode: blend.to_string(), depth_test: depth, cull_mode: cull.to_string() }
}
#[allow(dead_code)]
pub fn blend_mode_rse(m: &RenderStateExport) -> &str { &m.blend_mode }
#[allow(dead_code)]
pub fn depth_test_rse(m: &RenderStateExport) -> bool { m.depth_test }
#[allow(dead_code)]
pub fn cull_mode_rse(m: &RenderStateExport) -> &str { &m.cull_mode }
#[allow(dead_code)]
pub fn state_to_json_rse(m: &RenderStateExport) -> String {
    format!("{{\"blend\":\"{}\",\"depth_test\":{},\"cull\":\"{}\"}}", m.blend_mode, m.depth_test, m.cull_mode)
}
#[allow(dead_code)]
pub fn state_is_default_rse(m: &RenderStateExport) -> bool { m.blend_mode == "opaque" && m.depth_test && m.cull_mode == "back" }
#[allow(dead_code)]
pub fn state_export_size(m: &RenderStateExport) -> usize { m.blend_mode.len() + m.cull_mode.len() + 1 }
#[allow(dead_code)]
pub fn validate_render_state(m: &RenderStateExport) -> bool { !m.blend_mode.is_empty() && !m.cull_mode.is_empty() }

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> RenderStateExport { export_render_state("opaque", true, "back") }
    #[test] fn test_export() { let m = data(); assert_eq!(blend_mode_rse(&m), "opaque"); }
    #[test] fn test_blend() { let m = data(); assert_eq!(blend_mode_rse(&m), "opaque"); }
    #[test] fn test_depth() { let m = data(); assert!(depth_test_rse(&m)); }
    #[test] fn test_cull() { let m = data(); assert_eq!(cull_mode_rse(&m), "back"); }
    #[test] fn test_json() { let m = data(); assert!(state_to_json_rse(&m).contains("blend")); }
    #[test] fn test_default() { let m = data(); assert!(state_is_default_rse(&m)); }
    #[test] fn test_not_default() { let m = export_render_state("alpha", false, "none"); assert!(!state_is_default_rse(&m)); }
    #[test] fn test_size() { let m = data(); assert!(state_export_size(&m) > 0); }
    #[test] fn test_validate() { let m = data(); assert!(validate_render_state(&m)); }
    #[test] fn test_invalid() { let m = export_render_state("", true, "back"); assert!(!validate_render_state(&m)); }
}
