// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Depth pre-pass renderer configuration.

#![allow(dead_code)]

/// Configuration for the depth pre-pass.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DepthPrepassConfig {
    pub enabled: bool,
    pub write_depth: bool,
    pub test_depth: bool,
}

/// Runtime state for depth pre-pass.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DepthPrepassState {
    pub config: DepthPrepassConfig,
    pub draw_count: u32,
}

#[allow(dead_code)]
pub fn default_depth_prepass_config() -> DepthPrepassConfig {
    DepthPrepassConfig {
        enabled: true,
        write_depth: true,
        test_depth: true,
    }
}

#[allow(dead_code)]
pub fn new_depth_prepass_state() -> DepthPrepassState {
    DepthPrepassState {
        config: default_depth_prepass_config(),
        draw_count: 0,
    }
}

#[allow(dead_code)]
pub fn dp_set_enabled(state: &mut DepthPrepassState, enabled: bool) {
    state.config.enabled = enabled;
}

#[allow(dead_code)]
pub fn dp_is_enabled(state: &DepthPrepassState) -> bool {
    state.config.enabled
}

#[allow(dead_code)]
pub fn dp_increment_draw_count(state: &mut DepthPrepassState) {
    state.draw_count += 1;
}

#[allow(dead_code)]
pub fn dp_reset_draw_count(state: &mut DepthPrepassState) {
    state.draw_count = 0;
}

#[allow(dead_code)]
pub fn dp_draw_count(state: &DepthPrepassState) -> u32 {
    state.draw_count
}

#[allow(dead_code)]
pub fn dp_to_json(state: &DepthPrepassState) -> String {
    format!(
        r#"{{"enabled":{},"write_depth":{},"test_depth":{},"draw_count":{}}}"#,
        state.config.enabled,
        state.config.write_depth,
        state.config.test_depth,
        state.draw_count
    )
}

// ── New types required by task ─────────────────────────────────────────────

/// A full depth prepass pass descriptor.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DepthPrepass {
    pub width: u32,
    pub height: u32,
    pub near: f32,
    pub far: f32,
    pub enabled: bool,
}

/// CPU-side depth buffer (flat array of depth values, row-major).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DepthBuffer {
    pub width: u32,
    pub height: u32,
    pub data: Vec<f32>,
}

/// Create a new `DepthPrepass` with the given dimensions.
#[allow(dead_code)]
pub fn new_depth_prepass(width: u32, height: u32, near: f32, far: f32) -> DepthPrepass {
    DepthPrepass { width, height, near, far, enabled: true }
}

/// Fill a `DepthBuffer` with the far-plane value.
#[allow(dead_code)]
pub fn clear_depth(buf: &mut DepthBuffer, value: f32) {
    buf.data.fill(value);
}

/// Write a depth value at pixel (x, y).
#[allow(dead_code)]
pub fn write_depth(buf: &mut DepthBuffer, x: u32, y: u32, value: f32) {
    let idx = (y * buf.width + x) as usize;
    if idx < buf.data.len() {
        buf.data[idx] = value;
    }
}

/// Read the depth value at pixel (x, y).
#[allow(dead_code)]
pub fn read_depth(buf: &DepthBuffer, x: u32, y: u32) -> f32 {
    let idx = (y * buf.width + x) as usize;
    buf.data.get(idx).copied().unwrap_or(1.0)
}

/// Return true if `d` passes the depth test (less than stored depth).
#[allow(dead_code)]
pub fn depth_test(buf: &DepthBuffer, x: u32, y: u32, d: f32) -> bool {
    d < read_depth(buf, x, y)
}

/// Return the prepass width.
#[allow(dead_code)]
pub fn prepass_width(dp: &DepthPrepass) -> u32 {
    dp.width
}

/// Return the prepass height.
#[allow(dead_code)]
pub fn prepass_height(dp: &DepthPrepass) -> u32 {
    dp.height
}

/// Convert a non-linear depth value to a linear view-space depth.
#[allow(dead_code)]
pub fn depth_to_linear(dp: &DepthPrepass, ndc_depth: f32) -> f32 {
    let n = dp.near;
    let f = dp.far;
    (2.0 * n * f) / (f + n - ndc_depth * (f - n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_depth_prepass_config();
        assert!(cfg.enabled);
        assert!(cfg.write_depth);
        assert!(cfg.test_depth);
    }

    #[test]
    fn test_new_state_draw_count_zero() {
        let s = new_depth_prepass_state();
        assert_eq!(s.draw_count, 0);
    }

    #[test]
    fn test_set_enabled_false() {
        let mut s = new_depth_prepass_state();
        dp_set_enabled(&mut s, false);
        assert!(!dp_is_enabled(&s));
    }

    #[test]
    fn test_increment_draw_count() {
        let mut s = new_depth_prepass_state();
        dp_increment_draw_count(&mut s);
        dp_increment_draw_count(&mut s);
        assert_eq!(dp_draw_count(&s), 2);
    }

    #[test]
    fn test_reset_draw_count() {
        let mut s = new_depth_prepass_state();
        dp_increment_draw_count(&mut s);
        dp_reset_draw_count(&mut s);
        assert_eq!(dp_draw_count(&s), 0);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_depth_prepass_state();
        let j = dp_to_json(&s);
        assert!(j.contains("enabled"));
        assert!(j.contains("draw_count"));
    }

    #[test]
    fn test_set_enabled_true() {
        let mut s = new_depth_prepass_state();
        dp_set_enabled(&mut s, false);
        dp_set_enabled(&mut s, true);
        assert!(dp_is_enabled(&s));
    }

    #[test]
    fn test_multiple_increments() {
        let mut s = new_depth_prepass_state();
        for _ in 0..10 {
            dp_increment_draw_count(&mut s);
        }
        assert_eq!(dp_draw_count(&s), 10);
    }

    #[test]
    fn test_new_depth_prepass() {
        let dp = new_depth_prepass(800, 600, 0.1, 100.0);
        assert_eq!(prepass_width(&dp), 800);
        assert_eq!(prepass_height(&dp), 600);
        assert!(dp.enabled);
    }

    #[test]
    fn test_clear_depth() {
        let mut buf = DepthBuffer { width: 2, height: 2, data: vec![0.0; 4] };
        clear_depth(&mut buf, 1.0);
        assert!(buf.data.iter().all(|&v| (v - 1.0).abs() < 1e-6));
    }

    #[test]
    fn test_write_read_depth() {
        let mut buf = DepthBuffer { width: 4, height: 4, data: vec![1.0; 16] };
        write_depth(&mut buf, 1, 2, 0.5);
        assert!((read_depth(&buf, 1, 2) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_depth_test_pass() {
        let buf = DepthBuffer { width: 1, height: 1, data: vec![0.8] };
        assert!(depth_test(&buf, 0, 0, 0.5));
    }

    #[test]
    fn test_depth_test_fail() {
        let buf = DepthBuffer { width: 1, height: 1, data: vec![0.3] };
        assert!(!depth_test(&buf, 0, 0, 0.5));
    }

    #[test]
    fn test_depth_to_linear_near() {
        let dp = new_depth_prepass(1, 1, 0.1, 100.0);
        let lin = depth_to_linear(&dp, -1.0);
        // NDC depth -1 should give linear depth near the near plane
        assert!(lin > 0.0);
    }
}
