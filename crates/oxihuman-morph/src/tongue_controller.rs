//! Tongue position and shape morph control (tip position, curl, width).
//! Provides a simple morph-weight model for tongue extension, elevation,
//! curl and width — useful for phoneme-driven lip-sync or expression rigs.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Configuration for the tongue controller.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TongueCtrlConfig {
    /// Maximum extension weight (0 = fully retracted, 1 = fully out).
    pub max_out: f32,
    /// Maximum elevation weight (0 = neutral, 1 = fully up).
    pub max_up: f32,
    /// Maximum curl weight (0 = flat, 1 = fully curled).
    pub max_curl: f32,
    /// Maximum width weight (0 = narrow, 1 = fully wide).
    pub max_width: f32,
}

/// Runtime state of the tongue controller.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TongueCtrlState {
    /// Configuration used for clamping.
    pub config: TongueCtrlConfig,
    /// Current extension [0..1].
    pub out_amount: f32,
    /// Current upward elevation [0..1].
    pub up_amount: f32,
    /// Current curl amount [0..1].
    pub curl_amount: f32,
    /// Current width spread [0..1].
    pub width_amount: f32,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return a sensible default `TongueCtrlConfig`.
#[allow(dead_code)]
pub fn default_tongue_config() -> TongueCtrlConfig {
    TongueCtrlConfig {
        max_out: 1.0,
        max_up: 1.0,
        max_curl: 1.0,
        max_width: 1.0,
    }
}

/// Construct a fresh `TongueCtrlState` from the given config.
#[allow(dead_code)]
pub fn new_tongue_state(cfg: &TongueCtrlConfig) -> TongueCtrlState {
    TongueCtrlState {
        config: cfg.clone(),
        out_amount: 0.0,
        up_amount: 0.0,
        curl_amount: 0.0,
        width_amount: 0.0,
    }
}

/// Set tongue extension amount (how far out the tongue protrudes).
/// `amount` is clamped to `[0, config.max_out]`.
#[allow(dead_code)]
pub fn set_tongue_out(state: &mut TongueCtrlState, amount: f32) {
    state.out_amount = amount.clamp(0.0, state.config.max_out);
}

/// Set tongue elevation (how high the tongue tip is raised).
/// `amount` is clamped to `[0, config.max_up]`.
#[allow(dead_code)]
pub fn set_tongue_up(state: &mut TongueCtrlState, amount: f32) {
    state.up_amount = amount.clamp(0.0, state.config.max_up);
}

/// Set tongue curl amount (tip curling backward).
/// `amount` is clamped to `[0, config.max_curl]`.
#[allow(dead_code)]
pub fn set_tongue_curl(state: &mut TongueCtrlState, amount: f32) {
    state.curl_amount = amount.clamp(0.0, state.config.max_curl);
}

/// Set tongue width spread.
/// `amount` is clamped to `[0, config.max_width]`.
#[allow(dead_code)]
pub fn set_tongue_width(state: &mut TongueCtrlState, amount: f32) {
    state.width_amount = amount.clamp(0.0, state.config.max_width);
}

/// Return the four morph weights as `[out, up, curl, width]`.
#[allow(dead_code)]
pub fn tongue_morph_weights(state: &TongueCtrlState) -> [f32; 4] {
    [
        state.out_amount,
        state.up_amount,
        state.curl_amount,
        state.width_amount,
    ]
}

/// Reset the tongue to the neutral retracted position.
#[allow(dead_code)]
pub fn reset_tongue(state: &mut TongueCtrlState) {
    state.out_amount = 0.0;
    state.up_amount = 0.0;
    state.curl_amount = 0.0;
    state.width_amount = 0.0;
}

/// Linearly blend between two tongue states.  `t = 0` returns `a`, `t = 1` returns `b`.
#[allow(dead_code)]
pub fn blend_tongue_states(a: &TongueCtrlState, b: &TongueCtrlState, t: f32) -> TongueCtrlState {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    TongueCtrlState {
        config: a.config.clone(),
        out_amount: lerp(a.out_amount, b.out_amount),
        up_amount: lerp(a.up_amount, b.up_amount),
        curl_amount: lerp(a.curl_amount, b.curl_amount),
        width_amount: lerp(a.width_amount, b.width_amount),
    }
}

/// Returns `true` when the tongue is fully retracted (all weights near zero).
#[allow(dead_code)]
pub fn tongue_is_retracted(state: &TongueCtrlState) -> bool {
    state.out_amount < 1e-4
        && state.up_amount < 1e-4
        && state.curl_amount < 1e-4
        && state.width_amount < 1e-4
}

// ── Extended API (TongueConfig / TongueState) ─────────────────────────────────

/// High-level configuration for tongue morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TongueConfig {
    pub max_extension: f32,
    pub max_elevation: f32,
}

/// High-level tongue morph state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TongueState {
    pub extension: f32,
    pub elevation: f32,
    pub curl: f32,
    pub width: f32,
}

#[allow(dead_code)]
pub fn default_tongue_ext_config() -> TongueConfig {
    TongueConfig { max_extension: 1.0, max_elevation: 1.0 }
}

#[allow(dead_code)]
pub fn new_tongue_ext_state(cfg: &TongueConfig) -> TongueState {
    let _ = cfg;
    TongueState { extension: 0.0, elevation: 0.0, curl: 0.0, width: 0.5 }
}

#[allow(dead_code)]
pub fn tongue_extend(state: &mut TongueState, cfg: &TongueConfig, value: f32) {
    state.extension = value.clamp(0.0, cfg.max_extension);
}

#[allow(dead_code)]
pub fn tongue_elevate(state: &mut TongueState, cfg: &TongueConfig, value: f32) {
    state.elevation = value.clamp(0.0, cfg.max_elevation);
}

#[allow(dead_code)]
pub fn tongue_curl(state: &mut TongueState, value: f32) {
    state.curl = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn tongue_reset(state: &mut TongueState) {
    state.extension = 0.0;
    state.elevation = 0.0;
    state.curl = 0.0;
    state.width = 0.5;
}

#[allow(dead_code)]
pub fn tongue_to_weights(state: &TongueState) -> Vec<(String, f32)> {
    vec![
        ("tongue_extension".to_string(), state.extension),
        ("tongue_elevation".to_string(), state.elevation),
        ("tongue_curl".to_string(), state.curl),
        ("tongue_width".to_string(), state.width),
    ]
}

#[allow(dead_code)]
pub fn tongue_to_json(state: &TongueState) -> String {
    format!(
        r#"{{"extension":{:.4},"elevation":{:.4},"curl":{:.4},"width":{:.4}}}"#,
        state.extension, state.elevation, state.curl, state.width
    )
}

#[allow(dead_code)]
pub fn tongue_clamp(state: &mut TongueState, cfg: &TongueConfig) {
    state.extension = state.extension.clamp(0.0, cfg.max_extension);
    state.elevation = state.elevation.clamp(0.0, cfg.max_elevation);
    state.curl = state.curl.clamp(0.0, 1.0);
    state.width = state.width.clamp(0.0, 1.0);
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> TongueCtrlState {
        new_tongue_state(&default_tongue_config())
    }

    #[test]
    fn test_default_config_max_one() {
        let cfg = default_tongue_config();
        assert_eq!(cfg.max_out, 1.0);
        assert_eq!(cfg.max_up, 1.0);
        assert_eq!(cfg.max_curl, 1.0);
        assert_eq!(cfg.max_width, 1.0);
    }

    #[test]
    fn test_new_state_all_zeros() {
        let s = make_state();
        assert_eq!(s.out_amount, 0.0);
        assert_eq!(s.up_amount, 0.0);
        assert_eq!(s.curl_amount, 0.0);
        assert_eq!(s.width_amount, 0.0);
    }

    #[test]
    fn test_is_retracted_initially() {
        let s = make_state();
        assert!(tongue_is_retracted(&s));
    }

    #[test]
    fn test_set_tongue_out_clamps() {
        let mut s = make_state();
        set_tongue_out(&mut s, 2.0);
        assert_eq!(s.out_amount, 1.0);
        set_tongue_out(&mut s, -1.0);
        assert_eq!(s.out_amount, 0.0);
    }

    #[test]
    fn test_set_tongue_up_clamps() {
        let mut s = make_state();
        set_tongue_up(&mut s, 0.5);
        assert!((s.up_amount - 0.5).abs() < 1e-6);
        set_tongue_up(&mut s, 9.9);
        assert_eq!(s.up_amount, 1.0);
    }

    #[test]
    fn test_set_tongue_curl_clamps() {
        let mut s = make_state();
        set_tongue_curl(&mut s, 0.8);
        assert!((s.curl_amount - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_tongue_width_clamps() {
        let mut s = make_state();
        set_tongue_width(&mut s, 0.3);
        assert!((s.width_amount - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_morph_weights_returns_four() {
        let mut s = make_state();
        set_tongue_out(&mut s, 0.6);
        set_tongue_up(&mut s, 0.4);
        set_tongue_curl(&mut s, 0.2);
        set_tongue_width(&mut s, 0.1);
        let w = tongue_morph_weights(&s);
        assert_eq!(w.len(), 4);
        assert!((w[0] - 0.6).abs() < 1e-6);
        assert!((w[1] - 0.4).abs() < 1e-6);
        assert!((w[2] - 0.2).abs() < 1e-6);
        assert!((w[3] - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_reset_tongue() {
        let mut s = make_state();
        set_tongue_out(&mut s, 1.0);
        set_tongue_up(&mut s, 1.0);
        reset_tongue(&mut s);
        assert!(tongue_is_retracted(&s));
    }

    #[test]
    fn test_blend_tongue_states_midpoint() {
        let mut a = make_state();
        let mut b = make_state();
        set_tongue_out(&mut a, 0.0);
        set_tongue_out(&mut b, 1.0);
        let mid = blend_tongue_states(&a, &b, 0.5);
        assert!((mid.out_amount - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_tongue_states_t0_is_a() {
        let mut a = make_state();
        let b = make_state();
        set_tongue_out(&mut a, 0.7);
        let result = blend_tongue_states(&a, &b, 0.0);
        assert!((result.out_amount - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_blend_tongue_states_t1_is_b() {
        let a = make_state();
        let mut b = make_state();
        set_tongue_out(&mut b, 0.9);
        let result = blend_tongue_states(&a, &b, 1.0);
        assert!((result.out_amount - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_not_retracted_when_out_set() {
        let mut s = make_state();
        set_tongue_out(&mut s, 0.5);
        assert!(!tongue_is_retracted(&s));
    }
}
