//! Lip compression and pursing morphs for expressive speech articulation.

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct LipCompressionConfig {
    pub compress_factor: f32,
    pub purse_factor: f32,
    pub spread_factor: f32,
    pub symmetrical: bool,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct LipCompressionState {
    pub upper_compress: f32,
    pub lower_compress: f32,
    pub purse: f32,
    pub spread: f32,
    pub corner_tension: f32,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct LipCompressionWeights {
    pub corner_l: f32,
    pub corner_r: f32,
    pub upper_mid: f32,
    pub lower_mid: f32,
    pub upper_thin: f32,
    pub lower_thin: f32,
}

#[allow(dead_code)]
pub fn default_lip_compression_config() -> LipCompressionConfig {
    LipCompressionConfig {
        compress_factor: 1.0,
        purse_factor: 1.0,
        spread_factor: 1.0,
        symmetrical: true,
    }
}

#[allow(dead_code)]
pub fn new_lip_compression_state() -> LipCompressionState {
    LipCompressionState {
        upper_compress: 0.0,
        lower_compress: 0.0,
        purse: 0.0,
        spread: 0.0,
        corner_tension: 0.0,
    }
}

#[allow(dead_code)]
pub fn set_compress(state: &mut LipCompressionState, upper: f32, lower: f32) {
    state.upper_compress = upper.clamp(0.0, 1.0);
    state.lower_compress = lower.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_purse(state: &mut LipCompressionState, amount: f32) {
    state.purse = amount.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_spread(state: &mut LipCompressionState, amount: f32) {
    state.spread = amount.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_compression_weights(
    state: &LipCompressionState,
    cfg: &LipCompressionConfig,
) -> LipCompressionWeights {
    let upper = state.upper_compress * cfg.compress_factor;
    let lower = state.lower_compress * cfg.compress_factor;
    let purse = state.purse * cfg.purse_factor;
    let spread = state.spread * cfg.spread_factor;
    let tension = state.corner_tension;
    LipCompressionWeights {
        corner_l: (tension + spread * 0.5).clamp(0.0, 1.0),
        corner_r: (tension + spread * 0.5).clamp(0.0, 1.0),
        upper_mid: (upper + purse * 0.5).clamp(0.0, 1.0),
        lower_mid: (lower + purse * 0.5).clamp(0.0, 1.0),
        upper_thin: upper.clamp(0.0, 1.0),
        lower_thin: lower.clamp(0.0, 1.0),
    }
}

#[allow(dead_code)]
pub fn blend_compression(
    a: &LipCompressionState,
    b: &LipCompressionState,
    t: f32,
) -> LipCompressionState {
    let t = t.clamp(0.0, 1.0);
    let s = 1.0 - t;
    LipCompressionState {
        upper_compress: a.upper_compress * s + b.upper_compress * t,
        lower_compress: a.lower_compress * s + b.lower_compress * t,
        purse: a.purse * s + b.purse * t,
        spread: a.spread * s + b.spread * t,
        corner_tension: a.corner_tension * s + b.corner_tension * t,
    }
}

#[allow(dead_code)]
pub fn reset_compression(state: &mut LipCompressionState) {
    state.upper_compress = 0.0;
    state.lower_compress = 0.0;
    state.purse = 0.0;
    state.spread = 0.0;
    state.corner_tension = 0.0;
}

#[allow(dead_code)]
pub fn lip_compression_to_json(state: &LipCompressionState) -> String {
    format!(
        "{{\"upper_compress\":{},\"lower_compress\":{},\"purse\":{},\"spread\":{},\"corner_tension\":{}}}",
        state.upper_compress,
        state.lower_compress,
        state.purse,
        state.spread,
        state.corner_tension
    )
}

#[allow(dead_code)]
pub fn symmetrize_compression(state: &mut LipCompressionState) {
    let avg = (state.upper_compress + state.lower_compress) * 0.5;
    state.upper_compress = avg;
    state.lower_compress = avg;
}

#[allow(dead_code)]
pub fn corner_tension(state: &LipCompressionState) -> f32 {
    state.corner_tension
}

#[allow(dead_code)]
pub fn total_compression(state: &LipCompressionState) -> f32 {
    (state.upper_compress + state.lower_compress) / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_lip_compression_config();
        assert!((cfg.compress_factor - 1.0).abs() < 1e-6);
        assert!(cfg.symmetrical);
    }

    #[test]
    fn test_new_state_zero() {
        let s = new_lip_compression_state();
        assert!((s.upper_compress).abs() < 1e-6);
        assert!((s.lower_compress).abs() < 1e-6);
        assert!((s.purse).abs() < 1e-6);
    }

    #[test]
    fn test_set_compress() {
        let mut s = new_lip_compression_state();
        set_compress(&mut s, 0.8, 0.6);
        assert!((s.upper_compress - 0.8).abs() < 1e-6);
        assert!((s.lower_compress - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_compress_clamps() {
        let mut s = new_lip_compression_state();
        set_compress(&mut s, 2.0, -1.0);
        assert!((s.upper_compress - 1.0).abs() < 1e-6);
        assert!((s.lower_compress).abs() < 1e-6);
    }

    #[test]
    fn test_set_purse() {
        let mut s = new_lip_compression_state();
        set_purse(&mut s, 0.5);
        assert!((s.purse - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_spread() {
        let mut s = new_lip_compression_state();
        set_spread(&mut s, 0.7);
        assert!((s.spread - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_total_compression() {
        let mut s = new_lip_compression_state();
        set_compress(&mut s, 0.8, 0.4);
        let total = total_compression(&s);
        assert!((total - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_blend_compression() {
        let mut a = new_lip_compression_state();
        set_compress(&mut a, 0.0, 0.0);
        let mut b = new_lip_compression_state();
        set_compress(&mut b, 1.0, 1.0);
        let mid = blend_compression(&a, &b, 0.5);
        assert!((mid.upper_compress - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_reset_compression() {
        let mut s = new_lip_compression_state();
        set_compress(&mut s, 0.9, 0.9);
        reset_compression(&mut s);
        assert!((total_compression(&s)).abs() < 1e-6);
    }

    #[test]
    fn test_lip_compression_to_json() {
        let s = new_lip_compression_state();
        let json = lip_compression_to_json(&s);
        assert!(json.contains("upper_compress"));
        assert!(json.contains("lower_compress"));
    }

    #[test]
    fn test_symmetrize_compression() {
        let mut s = new_lip_compression_state();
        set_compress(&mut s, 0.8, 0.4);
        symmetrize_compression(&mut s);
        assert!((s.upper_compress - s.lower_compress).abs() < 1e-6);
    }

    #[test]
    fn test_corner_tension() {
        let mut s = new_lip_compression_state();
        s.corner_tension = 0.3;
        assert!((corner_tension(&s) - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_compute_compression_weights() {
        let cfg = default_lip_compression_config();
        let mut s = new_lip_compression_state();
        set_compress(&mut s, 0.5, 0.5);
        set_purse(&mut s, 0.2);
        let w = compute_compression_weights(&s, &cfg);
        assert!(w.upper_mid >= 0.0);
        assert!(w.lower_mid >= 0.0);
    }
}
