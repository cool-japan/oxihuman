//! Fine-grained brow shape and arch controls beyond basic raise/lower.

#[allow(dead_code)]
pub struct BrowShapeConfig {
    pub arch_height: f32,
    pub arch_position: f32,
    pub tail_angle: f32,
    pub thickness: f32,
}

#[allow(dead_code)]
pub struct BrowShapeState {
    pub arch_l: f32,
    pub arch_r: f32,
    pub tail_l: f32,
    pub tail_r: f32,
    pub thickness_l: f32,
    pub thickness_r: f32,
    pub peak_l: f32,
    pub peak_r: f32,
}

#[allow(dead_code)]
pub struct BrowShapeWeights {
    pub arch_high_l: f32,
    pub arch_high_r: f32,
    pub arch_low_l: f32,
    pub arch_low_r: f32,
    pub tail_up_l: f32,
    pub tail_up_r: f32,
}

#[allow(dead_code)]
pub fn default_brow_shape_config() -> BrowShapeConfig {
    BrowShapeConfig {
        arch_height: 0.5,
        arch_position: 0.5,
        tail_angle: 0.0,
        thickness: 0.5,
    }
}

#[allow(dead_code)]
pub fn new_brow_shape_state() -> BrowShapeState {
    BrowShapeState {
        arch_l: 0.0,
        arch_r: 0.0,
        tail_l: 0.0,
        tail_r: 0.0,
        thickness_l: 0.5,
        thickness_r: 0.5,
        peak_l: 0.0,
        peak_r: 0.0,
    }
}

#[allow(dead_code)]
pub fn set_brow_shape_arch(state: &mut BrowShapeState, left: f32, right: f32) {
    state.arch_l = left.clamp(-1.0, 1.0);
    state.arch_r = right.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn set_brow_shape_tail(state: &mut BrowShapeState, left: f32, right: f32) {
    state.tail_l = left.clamp(-1.0, 1.0);
    state.tail_r = right.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn set_brow_shape_thickness(state: &mut BrowShapeState, left: f32, right: f32) {
    state.thickness_l = left.clamp(0.0, 1.0);
    state.thickness_r = right.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_brow_shape_peak(state: &mut BrowShapeState, left: f32, right: f32) {
    state.peak_l = left.clamp(-1.0, 1.0);
    state.peak_r = right.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_brow_shape_weights(
    state: &BrowShapeState,
    cfg: &BrowShapeConfig,
) -> BrowShapeWeights {
    let scale = cfg.arch_height.clamp(0.01, 2.0);
    let arch_high_l = (state.arch_l * scale).clamp(0.0, 1.0);
    let arch_high_r = (state.arch_r * scale).clamp(0.0, 1.0);
    let arch_low_l = ((-state.arch_l) * scale).clamp(0.0, 1.0);
    let arch_low_r = ((-state.arch_r) * scale).clamp(0.0, 1.0);
    let tail_scale = cfg.tail_angle.abs().clamp(0.0, 1.0) + 0.5;
    let tail_up_l = (state.tail_l * tail_scale).clamp(0.0, 1.0);
    let tail_up_r = (state.tail_r * tail_scale).clamp(0.0, 1.0);
    BrowShapeWeights {
        arch_high_l,
        arch_high_r,
        arch_low_l,
        arch_low_r,
        tail_up_l,
        tail_up_r,
    }
}

#[allow(dead_code)]
pub fn blend_brow_shapes(a: &BrowShapeState, b: &BrowShapeState, t: f32) -> BrowShapeState {
    let t = t.clamp(0.0, 1.0);
    let u = 1.0 - t;
    BrowShapeState {
        arch_l: a.arch_l * u + b.arch_l * t,
        arch_r: a.arch_r * u + b.arch_r * t,
        tail_l: a.tail_l * u + b.tail_l * t,
        tail_r: a.tail_r * u + b.tail_r * t,
        thickness_l: a.thickness_l * u + b.thickness_l * t,
        thickness_r: a.thickness_r * u + b.thickness_r * t,
        peak_l: a.peak_l * u + b.peak_l * t,
        peak_r: a.peak_r * u + b.peak_r * t,
    }
}

#[allow(dead_code)]
pub fn reset_brow_shape(state: &mut BrowShapeState) {
    state.arch_l = 0.0;
    state.arch_r = 0.0;
    state.tail_l = 0.0;
    state.tail_r = 0.0;
    state.thickness_l = 0.5;
    state.thickness_r = 0.5;
    state.peak_l = 0.0;
    state.peak_r = 0.0;
}

#[allow(dead_code)]
pub fn symmetrize_brow_shape(state: &mut BrowShapeState) {
    let arch_avg = (state.arch_l + state.arch_r) * 0.5;
    let tail_avg = (state.tail_l + state.tail_r) * 0.5;
    let thick_avg = (state.thickness_l + state.thickness_r) * 0.5;
    let peak_avg = (state.peak_l + state.peak_r) * 0.5;
    state.arch_l = arch_avg;
    state.arch_r = arch_avg;
    state.tail_l = tail_avg;
    state.tail_r = tail_avg;
    state.thickness_l = thick_avg;
    state.thickness_r = thick_avg;
    state.peak_l = peak_avg;
    state.peak_r = peak_avg;
}

#[allow(dead_code)]
pub fn brow_shape_to_json(state: &BrowShapeState) -> String {
    format!(
        r#"{{"arch_l":{:.4},"arch_r":{:.4},"tail_l":{:.4},"tail_r":{:.4},"thickness_l":{:.4},"thickness_r":{:.4},"peak_l":{:.4},"peak_r":{:.4}}}"#,
        state.arch_l,
        state.arch_r,
        state.tail_l,
        state.tail_r,
        state.thickness_l,
        state.thickness_r,
        state.peak_l,
        state.peak_r,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_brow_shape_config();
        assert!((cfg.arch_height - 0.5).abs() < 1e-5);
        assert!((cfg.arch_position - 0.5).abs() < 1e-5);
        assert!(cfg.tail_angle.abs() < 1e-5);
        assert!((cfg.thickness - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_new_state() {
        let s = new_brow_shape_state();
        assert!(s.arch_l.abs() < 1e-5);
        assert!(s.arch_r.abs() < 1e-5);
        assert!((s.thickness_l - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_set_brow_shape_arch() {
        let mut s = new_brow_shape_state();
        set_brow_shape_arch(&mut s, 0.8, -0.3);
        assert!((s.arch_l - 0.8).abs() < 1e-5);
        assert!((s.arch_r - (-0.3)).abs() < 1e-5);
    }

    #[test]
    fn test_set_brow_shape_arch_clamp() {
        let mut s = new_brow_shape_state();
        set_brow_shape_arch(&mut s, 2.0, -5.0);
        assert!((s.arch_l - 1.0).abs() < 1e-5);
        assert!((s.arch_r - (-1.0)).abs() < 1e-5);
    }

    #[test]
    fn test_set_brow_shape_tail() {
        let mut s = new_brow_shape_state();
        set_brow_shape_tail(&mut s, 0.5, 0.5);
        assert!((s.tail_l - 0.5).abs() < 1e-5);
        assert!((s.tail_r - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_set_brow_shape_thickness() {
        let mut s = new_brow_shape_state();
        set_brow_shape_thickness(&mut s, 0.9, 0.1);
        assert!((s.thickness_l - 0.9).abs() < 1e-5);
        assert!((s.thickness_r - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_set_brow_shape_peak() {
        let mut s = new_brow_shape_state();
        set_brow_shape_peak(&mut s, 0.6, -0.6);
        assert!((s.peak_l - 0.6).abs() < 1e-5);
        assert!((s.peak_r - (-0.6)).abs() < 1e-5);
    }

    #[test]
    fn test_compute_brow_shape_weights() {
        let mut s = new_brow_shape_state();
        let cfg = default_brow_shape_config();
        set_brow_shape_arch(&mut s, 0.8, 0.8);
        let w = compute_brow_shape_weights(&s, &cfg);
        assert!(w.arch_high_l > 0.0);
        assert!(w.arch_high_r > 0.0);
        assert!(w.arch_low_l < 1e-5);
    }

    #[test]
    fn test_blend_brow_shapes() {
        let a = new_brow_shape_state();
        let mut b = new_brow_shape_state();
        b.arch_l = 1.0;
        b.arch_r = 1.0;
        let mid = blend_brow_shapes(&a, &b, 0.5);
        assert!((mid.arch_l - 0.5).abs() < 1e-5);
        assert!((mid.arch_r - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_reset_brow_shape() {
        let mut s = new_brow_shape_state();
        set_brow_shape_arch(&mut s, 1.0, 1.0);
        reset_brow_shape(&mut s);
        assert!(s.arch_l.abs() < 1e-5);
        assert!(s.arch_r.abs() < 1e-5);
        assert!((s.thickness_l - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_symmetrize_brow_shape() {
        let mut s = new_brow_shape_state();
        s.arch_l = 0.8;
        s.arch_r = 0.4;
        symmetrize_brow_shape(&mut s);
        assert!((s.arch_l - 0.6).abs() < 1e-5);
        assert!((s.arch_r - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_brow_shape_to_json() {
        let s = new_brow_shape_state();
        let json = brow_shape_to_json(&s);
        assert!(json.contains("arch_l"));
        assert!(json.contains("thickness_l"));
    }
}
