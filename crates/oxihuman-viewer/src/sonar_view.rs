// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Sonar-style depth visualization stub.

/// Sonar pulse configuration.
#[derive(Debug, Clone)]
pub struct SonarView {
    pub pulse_radius: f32,
    pub max_radius: f32,
    pub pulse_speed: f32,
    pub color: [f32; 4],
    pub ring_count: usize,
    pub enabled: bool,
}

impl SonarView {
    pub fn new() -> Self {
        SonarView {
            pulse_radius: 0.0,
            max_radius: 100.0,
            pulse_speed: 10.0,
            color: [0.0, 1.0, 0.5, 0.8],
            ring_count: 3,
            enabled: true,
        }
    }
}

impl Default for SonarView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new sonar view.
pub fn new_sonar_view() -> SonarView {
    SonarView::new()
}

/// Advance the pulse simulation.
pub fn snv_tick(snv: &mut SonarView, dt: f32) {
    /* Stub: advances pulse radius, wraps around max_radius using remainder */
    snv.pulse_radius += snv.pulse_speed * dt.max(0.0);
    let max = snv.max_radius.max(1e-6);
    snv.pulse_radius %= max;
}

/// Compute pulse intensity at a given distance from center.
pub fn snv_intensity_at(snv: &SonarView, distance: f32) -> f32 {
    /* Stub: returns intensity based on distance from pulse ring */
    let diff = (distance - snv.pulse_radius).abs();
    let ring_width = snv.max_radius / (snv.ring_count.max(1) as f32 * 4.0);
    if diff < ring_width {
        1.0 - diff / ring_width
    } else {
        0.0
    }
}

/// Set pulse speed.
pub fn snv_set_speed(snv: &mut SonarView, speed: f32) {
    snv.pulse_speed = speed.max(0.0);
}

/// Set ring count.
pub fn snv_set_ring_count(snv: &mut SonarView, count: usize) {
    snv.ring_count = count.max(1);
}

/// Enable or disable.
pub fn snv_set_enabled(snv: &mut SonarView, enabled: bool) {
    snv.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn snv_to_json(snv: &SonarView) -> String {
    format!(
        r#"{{"pulse_radius":{},"max_radius":{},"pulse_speed":{},"ring_count":{},"enabled":{}}}"#,
        snv.pulse_radius, snv.max_radius, snv.pulse_speed, snv.ring_count, snv.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_pulse_radius_zero() {
        let s = new_sonar_view();
        assert!((s.pulse_radius).abs() < 1e-6, /* initial pulse radius must be zero */);
    }

    #[test]
    fn test_tick_advances_pulse() {
        let mut s = new_sonar_view();
        snv_tick(&mut s, 1.0);
        assert!(s.pulse_radius > 0.0, /* pulse must advance after tick */);
    }

    #[test]
    fn test_tick_wraps_around() {
        let mut s = new_sonar_view();
        snv_tick(&mut s, 1000.0);
        assert!(s.pulse_radius < s.max_radius, /* pulse must wrap within max_radius */);
    }

    #[test]
    fn test_intensity_at_pulse_ring() {
        let s = new_sonar_view();
        let intensity = snv_intensity_at(&s, 0.0);
        assert!(intensity >= 0.0 /* intensity must be non-negative */,);
    }

    #[test]
    fn test_set_speed() {
        let mut s = new_sonar_view();
        snv_set_speed(&mut s, 20.0);
        assert!((s.pulse_speed - 20.0).abs() < 1e-5, /* speed must be set */);
    }

    #[test]
    fn test_speed_clamped_zero() {
        let mut s = new_sonar_view();
        snv_set_speed(&mut s, -5.0);
        assert!((s.pulse_speed).abs() < 1e-6, /* negative speed clamped to 0 */);
    }

    #[test]
    fn test_set_ring_count() {
        let mut s = new_sonar_view();
        snv_set_ring_count(&mut s, 5);
        assert_eq!(s.ring_count, 5 /* ring count must be set */,);
    }

    #[test]
    fn test_ring_count_minimum_one() {
        let mut s = new_sonar_view();
        snv_set_ring_count(&mut s, 0);
        assert_eq!(s.ring_count, 1 /* ring count must be at least 1 */,);
    }

    #[test]
    fn test_set_enabled() {
        let mut s = new_sonar_view();
        snv_set_enabled(&mut s, false);
        assert!(!s.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_pulse_radius() {
        let s = new_sonar_view();
        let j = snv_to_json(&s);
        assert!(j.contains("\"pulse_radius\""), /* json must contain pulse_radius */);
    }
}
