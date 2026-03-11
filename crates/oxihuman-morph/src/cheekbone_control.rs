//! Cheekbone prominence and facial width controls.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekboneConfig {
    pub prominence: f32,
    pub width: f32,
    pub height_position: f32,
    pub hollowness: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekboneState {
    pub prominence_l: f32,
    pub prominence_r: f32,
    pub width_l: f32,
    pub width_r: f32,
    pub hollow_l: f32,
    pub hollow_r: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekboneMorphWeights {
    pub prominent_l: f32,
    pub prominent_r: f32,
    pub flat_l: f32,
    pub flat_r: f32,
    pub hollow_l: f32,
    pub hollow_r: f32,
}

#[allow(dead_code)]
pub fn default_cheekbone_config() -> CheekboneConfig {
    CheekboneConfig {
        prominence: 0.5,
        width: 1.0,
        height_position: 0.5,
        hollowness: 0.0,
    }
}

#[allow(dead_code)]
pub fn new_cheekbone_state() -> CheekboneState {
    CheekboneState {
        prominence_l: 0.0,
        prominence_r: 0.0,
        width_l: 0.0,
        width_r: 0.0,
        hollow_l: 0.0,
        hollow_r: 0.0,
    }
}

#[allow(dead_code)]
pub fn set_prominence(state: &mut CheekboneState, left: f32, right: f32) {
    state.prominence_l = left.clamp(-1.0, 1.0);
    state.prominence_r = right.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn set_cheekbone_width(state: &mut CheekboneState, left: f32, right: f32) {
    state.width_l = left.clamp(0.0, 1.0);
    state.width_r = right.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_hollowness(state: &mut CheekboneState, left: f32, right: f32) {
    state.hollow_l = left.clamp(0.0, 1.0);
    state.hollow_r = right.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_cheekbone_weights(state: &CheekboneState, cfg: &CheekboneConfig) -> CheekboneMorphWeights {
    let p_scale = cfg.prominence;
    let h_scale = cfg.hollowness.clamp(0.0, 1.0);
    CheekboneMorphWeights {
        prominent_l: (state.prominence_l.max(0.0) * p_scale).clamp(0.0, 1.0),
        prominent_r: (state.prominence_r.max(0.0) * p_scale).clamp(0.0, 1.0),
        flat_l: ((-state.prominence_l).max(0.0) * cfg.width).clamp(0.0, 1.0),
        flat_r: ((-state.prominence_r).max(0.0) * cfg.width).clamp(0.0, 1.0),
        hollow_l: (state.hollow_l * (1.0 + h_scale)).clamp(0.0, 1.0),
        hollow_r: (state.hollow_r * (1.0 + h_scale)).clamp(0.0, 1.0),
    }
}

#[allow(dead_code)]
pub fn blend_cheekbones(a: &CheekboneState, b: &CheekboneState, t: f32) -> CheekboneState {
    let t = t.clamp(0.0, 1.0);
    let s = 1.0 - t;
    CheekboneState {
        prominence_l: a.prominence_l * s + b.prominence_l * t,
        prominence_r: a.prominence_r * s + b.prominence_r * t,
        width_l: a.width_l * s + b.width_l * t,
        width_r: a.width_r * s + b.width_r * t,
        hollow_l: a.hollow_l * s + b.hollow_l * t,
        hollow_r: a.hollow_r * s + b.hollow_r * t,
    }
}

#[allow(dead_code)]
pub fn reset_cheekbones(state: &mut CheekboneState) {
    state.prominence_l = 0.0;
    state.prominence_r = 0.0;
    state.width_l = 0.0;
    state.width_r = 0.0;
    state.hollow_l = 0.0;
    state.hollow_r = 0.0;
}

#[allow(dead_code)]
pub fn symmetrize_cheekbones(state: &mut CheekboneState) {
    let prom_avg = (state.prominence_l + state.prominence_r) * 0.5;
    state.prominence_l = prom_avg;
    state.prominence_r = prom_avg;
    let width_avg = (state.width_l + state.width_r) * 0.5;
    state.width_l = width_avg;
    state.width_r = width_avg;
    let hollow_avg = (state.hollow_l + state.hollow_r) * 0.5;
    state.hollow_l = hollow_avg;
    state.hollow_r = hollow_avg;
}

#[allow(dead_code)]
pub fn cheekbone_state_to_json(state: &CheekboneState) -> String {
    format!(
        "{{\"prominence_l\":{},\"prominence_r\":{},\"width_l\":{},\"width_r\":{},\"hollow_l\":{},\"hollow_r\":{}}}",
        state.prominence_l, state.prominence_r,
        state.width_l, state.width_r,
        state.hollow_l, state.hollow_r
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_cheekbone_config();
        assert!((cfg.prominence - 0.5).abs() < 1e-6);
        assert!((cfg.width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_zeros() {
        let s = new_cheekbone_state();
        assert_eq!(s.prominence_l, 0.0);
        assert_eq!(s.prominence_r, 0.0);
        assert_eq!(s.hollow_l, 0.0);
    }

    #[test]
    fn test_set_prominence_clamped() {
        let mut s = new_cheekbone_state();
        set_prominence(&mut s, 2.0, -2.0);
        assert!((s.prominence_l - 1.0).abs() < 1e-6);
        assert!((s.prominence_r - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_blend_cheekbones() {
        let a = new_cheekbone_state();
        let mut b = new_cheekbone_state();
        set_prominence(&mut b, 1.0, 1.0);
        let blended = blend_cheekbones(&a, &b, 0.5);
        assert!((blended.prominence_l - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_symmetrize() {
        let mut s = new_cheekbone_state();
        s.hollow_l = 0.8;
        s.hollow_r = 0.0;
        symmetrize_cheekbones(&mut s);
        assert!((s.hollow_l - 0.4).abs() < 1e-6);
        assert!((s.hollow_r - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let mut s = new_cheekbone_state();
        set_prominence(&mut s, 1.0, 1.0);
        set_hollowness(&mut s, 0.5, 0.5);
        reset_cheekbones(&mut s);
        assert_eq!(s.prominence_l, 0.0);
        assert_eq!(s.hollow_l, 0.0);
    }

    #[test]
    fn test_compute_weights() {
        let mut s = new_cheekbone_state();
        set_prominence(&mut s, 1.0, 0.0);
        set_hollowness(&mut s, 0.5, 0.5);
        let cfg = default_cheekbone_config();
        let w = compute_cheekbone_weights(&s, &cfg);
        assert!(w.prominent_l > 0.0);
        assert_eq!(w.prominent_r, 0.0);
        assert!(w.hollow_l > 0.0);
    }

    #[test]
    fn test_to_json() {
        let s = new_cheekbone_state();
        let json = cheekbone_state_to_json(&s);
        assert!(json.contains("prominence_l"));
        assert!(json.contains("hollow_r"));
    }
}
