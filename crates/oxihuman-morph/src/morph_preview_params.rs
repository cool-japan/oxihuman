#![allow(dead_code)]

//! Morph preview rendering parameters.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphPreviewRenderParams {
    pub show_deltas: bool,
    pub delta_scale: f32,
    pub highlight_color: [f32; 4],
    pub show_affected_only: bool,
    pub wireframe: bool,
    pub opacity: f32,
}

#[allow(dead_code)]
pub fn default_morph_preview_render_params() -> MorphPreviewRenderParams {
    MorphPreviewRenderParams {
        show_deltas: true,
        delta_scale: 1.0,
        highlight_color: [1.0, 0.5, 0.0, 1.0],
        show_affected_only: false,
        wireframe: false,
        opacity: 1.0,
    }
}

#[allow(dead_code)]
pub fn mpp_set_delta_scale(params: &mut MorphPreviewRenderParams, scale: f32) {
    params.delta_scale = scale.max(0.0);
}

#[allow(dead_code)]
pub fn mpp_set_highlight_color(params: &mut MorphPreviewRenderParams, color: [f32; 4]) {
    params.highlight_color = color;
}

#[allow(dead_code)]
pub fn mpp_toggle_wireframe(params: &mut MorphPreviewRenderParams) {
    params.wireframe = !params.wireframe;
}

#[allow(dead_code)]
pub fn mpp_set_opacity(params: &mut MorphPreviewRenderParams, opacity: f32) {
    params.opacity = opacity.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn mpp_set_show_affected_only(params: &mut MorphPreviewRenderParams, val: bool) {
    params.show_affected_only = val;
}

#[allow(dead_code)]
pub fn mpp_toggle_deltas(params: &mut MorphPreviewRenderParams) {
    params.show_deltas = !params.show_deltas;
}

#[allow(dead_code)]
pub fn mpp_is_visible(params: &MorphPreviewRenderParams) -> bool {
    params.opacity > 0.0
}

#[allow(dead_code)]
pub fn mpp_reset(params: &mut MorphPreviewRenderParams) {
    *params = default_morph_preview_render_params();
}

#[allow(dead_code)]
pub fn mpp_to_json(params: &MorphPreviewRenderParams) -> String {
    format!(
        "{{\"show_deltas\":{},\"delta_scale\":{},\"wireframe\":{},\"opacity\":{}}}",
        params.show_deltas, params.delta_scale, params.wireframe, params.opacity
    )
}

#[allow(dead_code)]
pub fn mpp_effective_alpha(params: &MorphPreviewRenderParams) -> f32 {
    params.highlight_color[3] * params.opacity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let p = default_morph_preview_render_params();
        assert!(p.show_deltas);
        assert!((p.delta_scale - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_delta_scale() {
        let mut p = default_morph_preview_render_params();
        mpp_set_delta_scale(&mut p, 2.5);
        assert!((p.delta_scale - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_delta_scale_clamps_negative() {
        let mut p = default_morph_preview_render_params();
        mpp_set_delta_scale(&mut p, -1.0);
        assert!((p.delta_scale - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_toggle_wireframe() {
        let mut p = default_morph_preview_render_params();
        mpp_toggle_wireframe(&mut p);
        assert!(p.wireframe);
        mpp_toggle_wireframe(&mut p);
        assert!(!p.wireframe);
    }

    #[test]
    fn test_set_opacity() {
        let mut p = default_morph_preview_render_params();
        mpp_set_opacity(&mut p, 0.5);
        assert!((p.opacity - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_is_visible() {
        let mut p = default_morph_preview_render_params();
        assert!(mpp_is_visible(&p));
        mpp_set_opacity(&mut p, 0.0);
        assert!(!mpp_is_visible(&p));
    }

    #[test]
    fn test_toggle_deltas() {
        let mut p = default_morph_preview_render_params();
        mpp_toggle_deltas(&mut p);
        assert!(!p.show_deltas);
    }

    #[test]
    fn test_set_show_affected_only() {
        let mut p = default_morph_preview_render_params();
        mpp_set_show_affected_only(&mut p, true);
        assert!(p.show_affected_only);
    }

    #[test]
    fn test_reset() {
        let mut p = default_morph_preview_render_params();
        mpp_set_opacity(&mut p, 0.0);
        mpp_reset(&mut p);
        assert!((p.opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let p = default_morph_preview_render_params();
        let json = mpp_to_json(&p);
        assert!(json.contains("delta_scale"));
    }
}
