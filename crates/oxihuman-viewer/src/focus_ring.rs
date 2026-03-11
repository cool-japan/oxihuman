// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Animated focus ring / selection highlight drawn around the selected object's
//! screen-space bounding box.

#![allow(dead_code)]

/// Configuration for the focus ring.
#[derive(Debug, Clone)]
pub struct FocusRingConfig {
    /// Base radius of the ring in pixels.
    pub base_radius: f32,
    /// Animation speed (radians per second for rotation, or expansion per second).
    pub anim_speed: f32,
    /// Ring line width in pixels.
    pub line_width: f32,
    /// Default ring color [r, g, b, a] in 0.0–1.0.
    pub color: [f32; 4],
}

/// State for the focus ring overlay.
#[derive(Debug, Clone)]
pub struct FocusRingState {
    /// The id of the currently targeted object, if any.
    pub target_id: Option<u32>,
    /// Screen-space center of the target bounding box.
    pub center: [f32; 2],
    /// Half-size of the target bounding box (width, height).
    pub half_size: [f32; 2],
    /// Current animation phase (accumulated time in seconds).
    pub phase: f32,
    /// Current radius (may pulse around `base_radius`).
    pub radius: f32,
    /// Current ring color [r, g, b, a].
    pub color: [f32; 4],
    /// Whether the ring is visible.
    pub visible: bool,
}

/// Returns the default [`FocusRingConfig`].
pub fn default_focus_ring_config() -> FocusRingConfig {
    FocusRingConfig {
        base_radius: 32.0,
        anim_speed: 2.0,
        line_width: 2.0,
        color: [1.0, 0.8, 0.0, 1.0],
    }
}

/// Creates a new [`FocusRingState`] with no target.
pub fn new_focus_ring(cfg: &FocusRingConfig) -> FocusRingState {
    FocusRingState {
        target_id: None,
        center: [0.0, 0.0],
        half_size: [0.0, 0.0],
        phase: 0.0,
        radius: cfg.base_radius,
        color: cfg.color,
        visible: false,
    }
}

/// Sets the focus ring target.
/// `center` is screen-space center in pixels.
/// `half_size` is (half_width, half_height) of the bounding box.
pub fn focus_ring_set_target(
    state: &mut FocusRingState,
    cfg: &FocusRingConfig,
    id: u32,
    center: [f32; 2],
    half_size: [f32; 2],
) {
    state.target_id = Some(id);
    state.center = center;
    state.half_size = half_size;
    state.radius = cfg.base_radius;
    state.visible = true;
}

/// Clears the current focus ring target, hiding the ring.
pub fn focus_ring_clear_target(state: &mut FocusRingState) {
    state.target_id = None;
    state.visible = false;
    state.phase = 0.0;
}

/// Returns `true` if a target is currently set.
pub fn focus_ring_has_target(state: &FocusRingState) -> bool {
    state.target_id.is_some()
}

/// Advances the animation by `dt` seconds.
/// Uses a sine-wave pulse to expand/contract the radius.
pub fn focus_ring_animate(state: &mut FocusRingState, cfg: &FocusRingConfig, dt: f32) {
    if !state.visible {
        return;
    }
    state.phase += dt * cfg.anim_speed;
    // Keep phase in [0, 2π] using modulo-like arithmetic
    let two_pi = 2.0 * core::f32::consts::PI;
    if state.phase > two_pi {
        state.phase -= two_pi;
    }
    // Pulse: base_radius ± 8 pixels
    let pulse = 8.0 * sin_approx(state.phase);
    state.radius = (cfg.base_radius + pulse).max(1.0);
}

/// Returns the current ring color.
pub fn focus_ring_color(state: &FocusRingState) -> [f32; 4] {
    state.color
}

/// Returns the current ring radius.
pub fn focus_ring_radius(state: &FocusRingState) -> f32 {
    state.radius
}

/// Serialises the current focus ring state as JSON.
pub fn focus_ring_to_json(state: &FocusRingState, cfg: &FocusRingConfig) -> String {
    let target_str = match state.target_id {
        Some(id) => format!("{id}"),
        None => "null".to_string(),
    };
    format!(
        "{{\"target_id\":{},\"center\":[{:.2},{:.2}],\
         \"half_size\":[{:.2},{:.2}],\"radius\":{:.4},\
         \"phase\":{:.4},\"visible\":{},\
         \"color\":[{:.4},{:.4},{:.4},{:.4}],\
         \"base_radius\":{:.4},\"line_width\":{:.4}}}",
        target_str,
        state.center[0], state.center[1],
        state.half_size[0], state.half_size[1],
        state.radius,
        state.phase,
        state.visible,
        state.color[0], state.color[1], state.color[2], state.color[3],
        cfg.base_radius,
        cfg.line_width
    )
}

/// Resets the focus ring to initial state (no target, default radius).
pub fn focus_ring_reset(state: &mut FocusRingState, cfg: &FocusRingConfig) {
    state.target_id = None;
    state.center = [0.0, 0.0];
    state.half_size = [0.0, 0.0];
    state.phase = 0.0;
    state.radius = cfg.base_radius;
    state.color = cfg.color;
    state.visible = false;
}

// ── internal helpers ───────────────────────────────────────────────────────────

/// Fast sine approximation using a polynomial (no `std::f32::sin` dependency in no_std).
/// Accurate to within ~0.001 over [0, 2π].
fn sin_approx(x: f32) -> f32 {
    // Reduce to [-π, π]
    let pi = core::f32::consts::PI;
    let two_pi = 2.0 * pi;
    let mut v = x;
    while v > pi { v -= two_pi; }
    while v < -pi { v += two_pi; }
    // Bhaskara I approximation
    let abs_v = if v < 0.0 { -v } else { v };
    let sign = if v < 0.0 { -1.0_f32 } else { 1.0_f32 };
    sign * (16.0 * abs_v * (pi - abs_v)) / (5.0 * pi * pi - 4.0 * abs_v * (pi - abs_v))
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_focus_ring_config();
        assert!((cfg.base_radius - 32.0).abs() < 1e-6);
        assert!((cfg.anim_speed - 2.0).abs() < 1e-6);
        assert!((cfg.line_width - 2.0).abs() < 1e-6);
    }

    #[test]
    fn new_ring_has_no_target() {
        let cfg = default_focus_ring_config();
        let s = new_focus_ring(&cfg);
        assert!(!focus_ring_has_target(&s));
        assert!(!s.visible);
    }

    #[test]
    fn set_target_enables_visibility() {
        let cfg = default_focus_ring_config();
        let mut s = new_focus_ring(&cfg);
        focus_ring_set_target(&mut s, &cfg, 42, [320.0, 240.0], [50.0, 80.0]);
        assert!(focus_ring_has_target(&s));
        assert!(s.visible);
        assert_eq!(s.target_id, Some(42));
    }

    #[test]
    fn clear_target_hides_ring() {
        let cfg = default_focus_ring_config();
        let mut s = new_focus_ring(&cfg);
        focus_ring_set_target(&mut s, &cfg, 1, [0.0, 0.0], [10.0, 10.0]);
        focus_ring_clear_target(&mut s);
        assert!(!focus_ring_has_target(&s));
        assert!(!s.visible);
    }

    #[test]
    fn animate_advances_phase() {
        let cfg = default_focus_ring_config();
        let mut s = new_focus_ring(&cfg);
        focus_ring_set_target(&mut s, &cfg, 1, [0.0, 0.0], [10.0, 10.0]);
        let phase_before = s.phase;
        focus_ring_animate(&mut s, &cfg, 0.1);
        assert!(s.phase > phase_before);
    }

    #[test]
    fn animate_does_nothing_when_invisible() {
        let cfg = default_focus_ring_config();
        let mut s = new_focus_ring(&cfg);
        let phase_before = s.phase;
        focus_ring_animate(&mut s, &cfg, 1.0);
        assert!((s.phase - phase_before).abs() < 1e-6);
    }

    #[test]
    fn focus_ring_color_returns_current() {
        let cfg = default_focus_ring_config();
        let s = new_focus_ring(&cfg);
        let c = focus_ring_color(&s);
        assert!((c[0] - 1.0).abs() < 1e-6);
        assert!((c[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn focus_ring_radius_within_reasonable_bounds() {
        let cfg = default_focus_ring_config();
        let mut s = new_focus_ring(&cfg);
        focus_ring_set_target(&mut s, &cfg, 5, [100.0, 100.0], [30.0, 30.0]);
        for i in 0..20 {
            focus_ring_animate(&mut s, &cfg, 0.1 * i as f32);
        }
        let r = focus_ring_radius(&s);
        assert!(r >= 1.0);
        assert!(r < 200.0);
    }

    #[test]
    fn json_contains_target_id() {
        let cfg = default_focus_ring_config();
        let mut s = new_focus_ring(&cfg);
        focus_ring_set_target(&mut s, &cfg, 77, [0.0, 0.0], [10.0, 10.0]);
        let json = focus_ring_to_json(&s, &cfg);
        assert!(json.contains("77"));
        assert!(json.contains("radius"));
        assert!(json.contains("visible"));
    }

    #[test]
    fn reset_clears_target() {
        let cfg = default_focus_ring_config();
        let mut s = new_focus_ring(&cfg);
        focus_ring_set_target(&mut s, &cfg, 3, [10.0, 10.0], [5.0, 5.0]);
        focus_ring_reset(&mut s, &cfg);
        assert!(!focus_ring_has_target(&s));
        assert!((s.phase - 0.0).abs() < 1e-6);
    }

    #[test]
    fn sin_approx_zero_is_zero() {
        assert!(sin_approx(0.0).abs() < 0.01);
    }

    #[test]
    fn sin_approx_half_pi_is_approx_one() {
        let v = sin_approx(core::f32::consts::PI / 2.0);
        assert!((v - 1.0).abs() < 0.01);
    }
}
