#![allow(dead_code)]
//! GPU render state: blend mode, depth test, culling, wireframe.

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum BlendMode {
    Opaque,
    Alpha,
    Additive,
    Multiply,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct RenderState {
    pub blend_mode: BlendMode,
    pub depth_test: bool,
    pub cull_face: bool,
    pub wireframe: bool,
}

#[allow(dead_code)]
pub fn new_render_state() -> RenderState {
    RenderState {
        blend_mode: BlendMode::Opaque,
        depth_test: true,
        cull_face: true,
        wireframe: false,
    }
}

#[allow(dead_code)]
pub fn set_blend_mode(s: &mut RenderState, mode: BlendMode) {
    s.blend_mode = mode;
}

#[allow(dead_code)]
pub fn set_depth_test(s: &mut RenderState, enabled: bool) {
    s.depth_test = enabled;
}

#[allow(dead_code)]
pub fn set_cull_face(s: &mut RenderState, enabled: bool) {
    s.cull_face = enabled;
}

#[allow(dead_code)]
pub fn set_wireframe(s: &mut RenderState, enabled: bool) {
    s.wireframe = enabled;
}

#[allow(dead_code)]
pub fn state_to_json(s: &RenderState) -> String {
    let blend = match &s.blend_mode {
        BlendMode::Opaque => "opaque",
        BlendMode::Alpha => "alpha",
        BlendMode::Additive => "additive",
        BlendMode::Multiply => "multiply",
    };
    format!(
        "{{\"blend\":\"{}\",\"depth\":{},\"cull\":{},\"wireframe\":{}}}",
        blend, s.depth_test, s.cull_face, s.wireframe
    )
}

#[allow(dead_code)]
pub fn state_is_default(s: &RenderState) -> bool {
    s.blend_mode == BlendMode::Opaque && s.depth_test && s.cull_face && !s.wireframe
}

#[allow(dead_code)]
pub fn reset_render_state(s: &mut RenderState) {
    s.blend_mode = BlendMode::Opaque;
    s.depth_test = true;
    s.cull_face = true;
    s.wireframe = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_render_state() {
        let s = new_render_state();
        assert!(state_is_default(&s));
    }

    #[test]
    fn test_set_blend_mode() {
        let mut s = new_render_state();
        set_blend_mode(&mut s, BlendMode::Alpha);
        assert_eq!(s.blend_mode, BlendMode::Alpha);
    }

    #[test]
    fn test_set_depth_test() {
        let mut s = new_render_state();
        set_depth_test(&mut s, false);
        assert!(!s.depth_test);
    }

    #[test]
    fn test_set_cull_face() {
        let mut s = new_render_state();
        set_cull_face(&mut s, false);
        assert!(!s.cull_face);
    }

    #[test]
    fn test_set_wireframe() {
        let mut s = new_render_state();
        set_wireframe(&mut s, true);
        assert!(s.wireframe);
    }

    #[test]
    fn test_state_to_json() {
        let s = new_render_state();
        let json = state_to_json(&s);
        assert!(json.contains("\"blend\":\"opaque\""));
    }

    #[test]
    fn test_state_is_default() {
        let mut s = new_render_state();
        assert!(state_is_default(&s));
        set_wireframe(&mut s, true);
        assert!(!state_is_default(&s));
    }

    #[test]
    fn test_reset_render_state() {
        let mut s = new_render_state();
        set_blend_mode(&mut s, BlendMode::Additive);
        set_wireframe(&mut s, true);
        reset_render_state(&mut s);
        assert!(state_is_default(&s));
    }

    #[test]
    fn test_additive_blend() {
        let mut s = new_render_state();
        set_blend_mode(&mut s, BlendMode::Additive);
        let json = state_to_json(&s);
        assert!(json.contains("\"blend\":\"additive\""));
    }

    #[test]
    fn test_multiply_blend() {
        let mut s = new_render_state();
        set_blend_mode(&mut s, BlendMode::Multiply);
        assert_eq!(s.blend_mode, BlendMode::Multiply);
    }
}
