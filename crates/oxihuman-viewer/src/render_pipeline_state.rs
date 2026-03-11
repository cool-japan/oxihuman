#![allow(dead_code)]

/// Render pipeline state configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderPipelineState {
    vertex_shader: Option<String>,
    fragment_shader: Option<String>,
    blend_enabled: bool,
    depth_test: bool,
}

#[allow(dead_code)]
pub fn new_pipeline_state() -> RenderPipelineState {
    RenderPipelineState { vertex_shader: None, fragment_shader: None, blend_enabled: false, depth_test: true }
}

#[allow(dead_code)]
pub fn set_vertex_shader(ps: &mut RenderPipelineState, name: &str) { ps.vertex_shader = Some(name.to_string()); }

#[allow(dead_code)]
pub fn set_fragment_shader(ps: &mut RenderPipelineState, name: &str) { ps.fragment_shader = Some(name.to_string()); }

#[allow(dead_code)]
pub fn set_blend_state(ps: &mut RenderPipelineState, enabled: bool) { ps.blend_enabled = enabled; }

#[allow(dead_code)]
pub fn set_depth_state_rps(ps: &mut RenderPipelineState, enabled: bool) { ps.depth_test = enabled; }

#[allow(dead_code)]
pub fn pipeline_to_json(ps: &RenderPipelineState) -> String {
    format!("{{\"vertex\":{},\"fragment\":{},\"blend\":{},\"depth\":{}}}", ps.vertex_shader.is_some(), ps.fragment_shader.is_some(), ps.blend_enabled, ps.depth_test)
}

#[allow(dead_code)]
pub fn pipeline_is_valid(ps: &RenderPipelineState) -> bool {
    ps.vertex_shader.is_some() && ps.fragment_shader.is_some()
}

#[allow(dead_code)]
pub fn pipeline_reset(ps: &mut RenderPipelineState) {
    ps.vertex_shader = None; ps.fragment_shader = None; ps.blend_enabled = false; ps.depth_test = true;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { assert!(!pipeline_is_valid(&new_pipeline_state())); }
    #[test] fn test_set_shaders() {
        let mut p = new_pipeline_state();
        set_vertex_shader(&mut p, "v"); set_fragment_shader(&mut p, "f");
        assert!(pipeline_is_valid(&p));
    }
    #[test] fn test_blend() {
        let mut p = new_pipeline_state();
        set_blend_state(&mut p, true);
        assert!(p.blend_enabled);
    }
    #[test] fn test_depth() {
        let mut p = new_pipeline_state();
        set_depth_state_rps(&mut p, false);
        assert!(!p.depth_test);
    }
    #[test] fn test_to_json() { assert!(pipeline_to_json(&new_pipeline_state()).contains("blend")); }
    #[test] fn test_reset() {
        let mut p = new_pipeline_state();
        set_vertex_shader(&mut p, "v");
        pipeline_reset(&mut p);
        assert!(!pipeline_is_valid(&p));
    }
    #[test] fn test_partial_valid() {
        let mut p = new_pipeline_state();
        set_vertex_shader(&mut p, "v");
        assert!(!pipeline_is_valid(&p));
    }
    #[test] fn test_default_depth() { assert!(new_pipeline_state().depth_test); }
    #[test] fn test_default_blend() { assert!(!new_pipeline_state().blend_enabled); }
    #[test] fn test_set_both_shaders() {
        let mut p = new_pipeline_state();
        set_vertex_shader(&mut p, "main_vs"); set_fragment_shader(&mut p, "main_fs");
        assert!(pipeline_is_valid(&p));
    }
}
