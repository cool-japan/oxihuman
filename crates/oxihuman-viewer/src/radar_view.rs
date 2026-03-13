// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Radar sweep visualization stub.

/// A blip (detected object) on the radar display.
#[derive(Debug, Clone)]
pub struct RadarBlip {
    pub angle_rad: f32,
    pub distance: f32,
    pub intensity: f32,
}

/// Radar view configuration.
#[derive(Debug, Clone)]
pub struct RadarView {
    pub sweep_angle_rad: f32,
    pub sweep_speed_rad_per_sec: f32,
    pub range: f32,
    pub blips: Vec<RadarBlip>,
    pub sweep_decay: f32,
    pub enabled: bool,
}

impl RadarView {
    pub fn new() -> Self {
        RadarView {
            sweep_angle_rad: 0.0,
            sweep_speed_rad_per_sec: 1.0,
            range: 100.0,
            blips: Vec::new(),
            sweep_decay: 0.95,
            enabled: true,
        }
    }
}

impl Default for RadarView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new radar view.
pub fn new_radar_view() -> RadarView {
    RadarView::new()
}

/// Advance sweep angle.
pub fn rdv_tick(rdv: &mut RadarView, dt: f32) {
    /* Stub: advances sweep angle, wraps to [0, 2pi) */
    rdv.sweep_angle_rad += rdv.sweep_speed_rad_per_sec * dt.max(0.0);
    let two_pi = std::f32::consts::TAU;
    while rdv.sweep_angle_rad >= two_pi {
        rdv.sweep_angle_rad -= two_pi;
    }
}

/// Add a blip.
pub fn rdv_add_blip(rdv: &mut RadarView, blip: RadarBlip) {
    rdv.blips.push(blip);
}

/// Return blip count.
pub fn rdv_blip_count(rdv: &RadarView) -> usize {
    rdv.blips.len()
}

/// Set sweep speed.
pub fn rdv_set_speed(rdv: &mut RadarView, speed: f32) {
    rdv.sweep_speed_rad_per_sec = speed.max(0.0);
}

/// Enable or disable.
pub fn rdv_set_enabled(rdv: &mut RadarView, enabled: bool) {
    rdv.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn rdv_to_json(rdv: &RadarView) -> String {
    format!(
        r#"{{"sweep_angle":{:.4},"range":{},"blip_count":{},"enabled":{}}}"#,
        rdv.sweep_angle_rad,
        rdv.range,
        rdv.blips.len(),
        rdv.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_sweep_zero() {
        let r = new_radar_view();
        assert!((r.sweep_angle_rad).abs() < 1e-6, /* initial sweep must be zero */);
    }

    #[test]
    fn test_tick_advances_sweep() {
        let mut r = new_radar_view();
        rdv_tick(&mut r, 1.0);
        assert!(r.sweep_angle_rad > 0.0, /* sweep must advance after tick */);
    }

    #[test]
    fn test_tick_wraps_sweep() {
        let mut r = new_radar_view();
        rdv_tick(&mut r, 1000.0);
        assert!(r.sweep_angle_rad < std::f32::consts::TAU, /* sweep must wrap */);
    }

    #[test]
    fn test_no_blips_initially() {
        let r = new_radar_view();
        assert_eq!(rdv_blip_count(&r), 0 /* no blips initially */,);
    }

    #[test]
    fn test_add_blip() {
        let mut r = new_radar_view();
        rdv_add_blip(
            &mut r,
            RadarBlip {
                angle_rad: 0.5,
                distance: 20.0,
                intensity: 1.0,
            },
        );
        assert_eq!(rdv_blip_count(&r), 1 /* one blip after add */,);
    }

    #[test]
    fn test_set_speed() {
        let mut r = new_radar_view();
        rdv_set_speed(&mut r, 2.0);
        assert!((r.sweep_speed_rad_per_sec - 2.0).abs() < 1e-5, /* speed must be set */);
    }

    #[test]
    fn test_set_enabled() {
        let mut r = new_radar_view();
        rdv_set_enabled(&mut r, false);
        assert!(!r.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_sweep_angle() {
        let r = new_radar_view();
        let j = rdv_to_json(&r);
        assert!(j.contains("\"sweep_angle\""), /* json must contain sweep_angle */);
    }

    #[test]
    fn test_enabled_default() {
        let r = new_radar_view();
        assert!(r.enabled /* must be enabled by default */,);
    }

    #[test]
    fn test_range_default() {
        let r = new_radar_view();
        assert!((r.range - 100.0).abs() < 1e-5, /* default range must be 100 */);
    }
}
