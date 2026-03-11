//! Mouth shape and lip morphology controls for expressive facial animation.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MouthShapeConfig {
    pub mouth_width: f32,
    pub lip_thickness: f32,
    pub mouth_depth: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MouthShapeState {
    pub width: f32,
    pub open: f32,
    pub upper_lip: f32,
    pub lower_lip: f32,
    pub pout: f32,
    pub dimple: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MouthShapeMorphWeights {
    pub wide: f32,
    pub narrow: f32,
    pub open: f32,
    pub closed: f32,
    pub upper_thick: f32,
    pub lower_thick: f32,
}

#[allow(dead_code)]
pub fn default_mouth_shape_config() -> MouthShapeConfig {
    MouthShapeConfig {
        mouth_width: 1.0,
        lip_thickness: 1.0,
        mouth_depth: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_mouth_shape_state() -> MouthShapeState {
    MouthShapeState {
        width: 0.0,
        open: 0.0,
        upper_lip: 0.0,
        lower_lip: 0.0,
        pout: 0.0,
        dimple: 0.0,
    }
}

#[allow(dead_code)]
pub fn set_mouth_width(state: &mut MouthShapeState, width: f32) {
    state.width = width.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn set_mouth_open(state: &mut MouthShapeState, open: f32) {
    state.open = open.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_upper_lip(state: &mut MouthShapeState, thickness: f32) {
    state.upper_lip = thickness.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn set_lower_lip(state: &mut MouthShapeState, thickness: f32) {
    state.lower_lip = thickness.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn set_pout(state: &mut MouthShapeState, pout: f32) {
    state.pout = pout.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_mouth_shape_weights(
    state: &MouthShapeState,
    cfg: &MouthShapeConfig,
) -> MouthShapeMorphWeights {
    let width_factor = cfg.mouth_width.max(0.001);
    let thick_factor = cfg.lip_thickness.max(0.001);

    let wide = (state.width * width_factor).clamp(0.0, 1.0);
    let narrow = ((-state.width) * width_factor).clamp(0.0, 1.0);
    let open = (state.open * cfg.mouth_depth.max(0.001)).clamp(0.0, 1.0);
    let closed = 1.0 - open;
    let upper_thick = (state.upper_lip * thick_factor).clamp(0.0, 1.0);
    let lower_thick = (state.lower_lip * thick_factor).clamp(0.0, 1.0);

    MouthShapeMorphWeights {
        wide,
        narrow,
        open,
        closed,
        upper_thick,
        lower_thick,
    }
}

#[allow(dead_code)]
pub fn blend_mouth_shapes(
    a: &MouthShapeState,
    b: &MouthShapeState,
    t: f32,
) -> MouthShapeState {
    let t = t.clamp(0.0, 1.0);
    let s = 1.0 - t;
    MouthShapeState {
        width: a.width * s + b.width * t,
        open: a.open * s + b.open * t,
        upper_lip: a.upper_lip * s + b.upper_lip * t,
        lower_lip: a.lower_lip * s + b.lower_lip * t,
        pout: a.pout * s + b.pout * t,
        dimple: a.dimple * s + b.dimple * t,
    }
}

#[allow(dead_code)]
pub fn reset_mouth_shape(state: &mut MouthShapeState) {
    *state = new_mouth_shape_state();
}

#[allow(dead_code)]
pub fn mouth_shape_to_json(state: &MouthShapeState) -> String {
    format!(
        r#"{{"width":{:.4},"open":{:.4},"upper_lip":{:.4},"lower_lip":{:.4},"pout":{:.4},"dimple":{:.4}}}"#,
        state.width, state.open, state.upper_lip, state.lower_lip, state.pout, state.dimple
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_mouth_shape_config();
        assert!((cfg.mouth_width - 1.0).abs() < 1e-6);
        assert!((cfg.lip_thickness - 1.0).abs() < 1e-6);
        assert!((cfg.mouth_depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_zeroed() {
        let s = new_mouth_shape_state();
        assert_eq!(s.width, 0.0);
        assert_eq!(s.open, 0.0);
        assert_eq!(s.pout, 0.0);
    }

    #[test]
    fn test_set_clamping() {
        let mut s = new_mouth_shape_state();
        set_mouth_width(&mut s, 5.0);
        assert!((s.width - 1.0).abs() < 1e-6);
        set_mouth_open(&mut s, -2.0);
        assert_eq!(s.open, 0.0);
        set_pout(&mut s, 2.0);
        assert!((s.pout - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let cfg = default_mouth_shape_config();
        let mut s = new_mouth_shape_state();
        set_mouth_width(&mut s, 1.0);
        set_mouth_open(&mut s, 0.5);
        let w = compute_mouth_shape_weights(&s, &cfg);
        assert!((w.wide - 1.0).abs() < 1e-6);
        assert_eq!(w.narrow, 0.0);
        assert!((w.open - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend() {
        let mut a = new_mouth_shape_state();
        set_mouth_open(&mut a, 0.0);
        let mut b = new_mouth_shape_state();
        set_mouth_open(&mut b, 1.0);
        let mid = blend_mouth_shapes(&a, &b, 0.5);
        assert!((mid.open - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let mut s = new_mouth_shape_state();
        set_mouth_open(&mut s, 1.0);
        set_pout(&mut s, 1.0);
        reset_mouth_shape(&mut s);
        assert_eq!(s.open, 0.0);
        assert_eq!(s.pout, 0.0);
    }

    #[test]
    fn test_to_json() {
        let s = new_mouth_shape_state();
        let j = mouth_shape_to_json(&s);
        assert!(j.contains("\"width\""));
        assert!(j.contains("\"open\""));
        assert!(j.contains("\"pout\""));
    }
}
