//! Chin shape and jaw-line morphology controls.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChinConfig {
    pub chin_width: f32,
    pub chin_height: f32,
    pub chin_projection: f32,
    pub jawline_sharpness: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChinState {
    pub width: f32,
    pub height: f32,
    pub projection: f32,
    pub cleft: f32,
    pub jawline: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChinMorphWeights {
    pub wide: f32,
    pub narrow: f32,
    pub tall: f32,
    pub short: f32,
    pub cleft: f32,
    pub pointy: f32,
}

#[allow(dead_code)]
pub fn default_chin_config() -> ChinConfig {
    ChinConfig {
        chin_width: 0.5,
        chin_height: 0.5,
        chin_projection: 0.5,
        jawline_sharpness: 0.5,
    }
}

#[allow(dead_code)]
pub fn new_chin_state() -> ChinState {
    ChinState {
        width: 0.5,
        height: 0.5,
        projection: 0.5,
        cleft: 0.0,
        jawline: 0.5,
    }
}

#[allow(dead_code)]
pub fn set_chin_width(state: &mut ChinState, width: f32) {
    state.width = width.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_chin_projection(state: &mut ChinState, proj: f32) {
    state.projection = proj.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_cleft(state: &mut ChinState, cleft: f32) {
    state.cleft = cleft.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_jawline(state: &mut ChinState, sharpness: f32) {
    state.jawline = sharpness.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_chin_weights(state: &ChinState, cfg: &ChinConfig) -> ChinMorphWeights {
    let w_bias = cfg.chin_width;
    let h_bias = cfg.chin_height;
    let wide = (state.width * w_bias).clamp(0.0, 1.0);
    let narrow = ((1.0 - state.width) * w_bias).clamp(0.0, 1.0);
    let tall = (state.height * h_bias).clamp(0.0, 1.0);
    let short = ((1.0 - state.height) * h_bias).clamp(0.0, 1.0);
    let cleft = state.cleft;
    let pointy = (state.projection * cfg.chin_projection).clamp(0.0, 1.0);
    ChinMorphWeights { wide, narrow, tall, short, cleft, pointy }
}

#[allow(dead_code)]
pub fn blend_chin(a: &ChinState, b: &ChinState, t: f32) -> ChinState {
    let t = t.clamp(0.0, 1.0);
    let u = 1.0 - t;
    ChinState {
        width: a.width * u + b.width * t,
        height: a.height * u + b.height * t,
        projection: a.projection * u + b.projection * t,
        cleft: a.cleft * u + b.cleft * t,
        jawline: a.jawline * u + b.jawline * t,
    }
}

#[allow(dead_code)]
pub fn reset_chin(state: &mut ChinState) {
    *state = new_chin_state();
}

#[allow(dead_code)]
pub fn chin_state_to_json(state: &ChinState) -> String {
    format!(
        r#"{{"width":{:.4},"height":{:.4},"projection":{:.4},"cleft":{:.4},"jawline":{:.4}}}"#,
        state.width, state.height, state.projection, state.cleft, state.jawline
    )
}

#[allow(dead_code)]
pub fn chin_prominence(state: &ChinState) -> f32 {
    state.projection * state.height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_chin_config() {
        let cfg = default_chin_config();
        assert!((cfg.chin_width - 0.5).abs() < 1e-6);
        assert!((cfg.chin_projection - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_new_chin_state() {
        let s = new_chin_state();
        assert!((s.cleft).abs() < 1e-6);
        assert!((s.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_chin_width_clamp() {
        let mut s = new_chin_state();
        set_chin_width(&mut s, 2.0);
        assert!((s.width - 1.0).abs() < 1e-6);
        set_chin_width(&mut s, -1.0);
        assert!(s.width.abs() < 1e-6);
    }

    #[test]
    fn test_set_cleft() {
        let mut s = new_chin_state();
        set_cleft(&mut s, 0.7);
        assert!((s.cleft - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_blend_chin_midpoint() {
        let a = new_chin_state();
        let mut b = new_chin_state();
        b.width = 1.0;
        let mid = blend_chin(&a, &b, 0.5);
        assert!((mid.width - 0.75).abs() < 1e-5);
    }

    #[test]
    fn test_chin_prominence() {
        let mut s = new_chin_state();
        s.projection = 0.8;
        s.height = 0.5;
        let p = chin_prominence(&s);
        assert!((p - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_reset_chin() {
        let mut s = new_chin_state();
        s.cleft = 1.0;
        s.width = 0.0;
        reset_chin(&mut s);
        assert!((s.cleft).abs() < 1e-6);
        assert!((s.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_chin_state_to_json() {
        let s = new_chin_state();
        let j = chin_state_to_json(&s);
        assert!(j.contains("width"));
        assert!(j.contains("projection"));
    }

    #[test]
    fn test_compute_chin_weights() {
        let s = new_chin_state();
        let cfg = default_chin_config();
        let w = compute_chin_weights(&s, &cfg);
        assert!(w.wide >= 0.0 && w.wide <= 1.0);
        assert!(w.narrow >= 0.0 && w.narrow <= 1.0);
    }
}
