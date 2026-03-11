// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Snap-to settings for transform operations.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Snap settings for transform operations.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SnapSettings {
    pub enabled: bool,
    /// Snap target: 0 = increment, 1 = vertex, 2 = edge, 3 = face, 4 = grid.
    pub snap_to: u8,
    pub increment: f32,
    pub snap_rotation: f32,
    pub snap_scale: f32,
}

/// Returns default snap settings (disabled, 1.0 increment, 15° rotation).
#[allow(dead_code)]
pub fn default_snap_settings() -> SnapSettings {
    SnapSettings {
        enabled: false,
        snap_to: 0,
        increment: 1.0,
        snap_rotation: PI / 12.0, // 15 degrees
        snap_scale: 0.1,
    }
}

/// Snaps a value to the nearest increment if snap is enabled.
#[allow(dead_code)]
pub fn snap_value(settings: &SnapSettings, val: f32) -> f32 {
    if !settings.enabled || settings.increment.abs() < f32::EPSILON {
        return val;
    }
    (val / settings.increment).round() * settings.increment
}

/// Snaps an angle to the nearest snap_rotation increment if enabled.
#[allow(dead_code)]
pub fn snap_angle(settings: &SnapSettings, angle: f32) -> f32 {
    if !settings.enabled || settings.snap_rotation.abs() < f32::EPSILON {
        return angle;
    }
    (angle / settings.snap_rotation).round() * settings.snap_rotation
}

/// Cycles to the next snap target (0..=4).
#[allow(dead_code)]
pub fn next_snap_target(settings: &mut SnapSettings) {
    settings.snap_to = (settings.snap_to + 1) % 5;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_snap_disabled() {
        let s = default_snap_settings();
        assert!(!s.enabled);
    }

    #[test]
    fn test_default_increment() {
        let s = default_snap_settings();
        assert!((s.increment - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_snap_value_disabled() {
        let s = default_snap_settings();
        let v = snap_value(&s, 1.7);
        assert!((v - 1.7).abs() < 1e-6);
    }

    #[test]
    fn test_snap_value_enabled() {
        let mut s = default_snap_settings();
        s.enabled = true;
        let v = snap_value(&s, 1.7);
        assert!((v - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_snap_value_exact() {
        let mut s = default_snap_settings();
        s.enabled = true;
        let v = snap_value(&s, 3.0);
        assert!((v - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_snap_angle_disabled() {
        let s = default_snap_settings();
        let a = snap_angle(&s, 0.3);
        assert!((a - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_snap_angle_enabled() {
        let mut s = default_snap_settings();
        s.enabled = true;
        // 15 degrees = PI/12; snap PI/11 to nearest PI/12
        let input = PI / 11.0;
        let snapped = snap_angle(&s, input);
        // just check it's a multiple of snap_rotation
        let ratio = snapped / (PI / 12.0);
        assert!((ratio - ratio.round()).abs() < 1e-4);
    }

    #[test]
    fn test_next_snap_target_cycles() {
        let mut s = default_snap_settings();
        assert_eq!(s.snap_to, 0);
        next_snap_target(&mut s);
        assert_eq!(s.snap_to, 1);
        next_snap_target(&mut s);
        assert_eq!(s.snap_to, 2);
    }

    #[test]
    fn test_next_snap_target_wraps() {
        let mut s = default_snap_settings();
        s.snap_to = 4;
        next_snap_target(&mut s);
        assert_eq!(s.snap_to, 0);
    }

    #[test]
    fn test_snap_rotation_uses_pi() {
        let s = default_snap_settings();
        // 15 degrees = PI/12
        assert!((s.snap_rotation - PI / 12.0).abs() < 1e-6);
    }
}
