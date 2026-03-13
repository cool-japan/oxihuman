// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Camera shake preview stub.

/// Shake profile type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShakeProfile {
    Handheld,
    Explosion,
    Earthquake,
    Subtle,
}

/// Camera shake view configuration.
#[derive(Debug, Clone)]
pub struct CameraShakeView {
    pub profile: ShakeProfile,
    pub amplitude: f32,
    pub frequency: f32,
    pub decay: f32,
    pub time: f32,
    pub enabled: bool,
}

impl CameraShakeView {
    pub fn new() -> Self {
        CameraShakeView {
            profile: ShakeProfile::Handheld,
            amplitude: 0.05,
            frequency: 10.0,
            decay: 0.9,
            time: 0.0,
            enabled: true,
        }
    }
}

impl Default for CameraShakeView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new camera shake view.
pub fn new_camera_shake_view() -> CameraShakeView {
    CameraShakeView::new()
}

/// Advance simulation time.
pub fn csh_tick(csh: &mut CameraShakeView, dt: f32) {
    csh.time += dt.max(0.0);
}

/// Sample shake offset at current time (stub: zeroed).
pub fn csh_sample_offset(csh: &CameraShakeView) -> [f32; 3] {
    /* Stub: returns zero offset */
    let _ = csh.time;
    [0.0; 3]
}

/// Set profile.
pub fn csh_set_profile(csh: &mut CameraShakeView, profile: ShakeProfile) {
    csh.profile = profile;
}

/// Set amplitude.
pub fn csh_set_amplitude(csh: &mut CameraShakeView, amplitude: f32) {
    csh.amplitude = amplitude.max(0.0);
}

/// Enable or disable.
pub fn csh_set_enabled(csh: &mut CameraShakeView, enabled: bool) {
    csh.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn csh_to_json(csh: &CameraShakeView) -> String {
    format!(
        r#"{{"amplitude":{},"frequency":{},"decay":{},"enabled":{}}}"#,
        csh.amplitude, csh.frequency, csh.decay, csh.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_amplitude() {
        let c = new_camera_shake_view();
        assert!((c.amplitude - 0.05).abs() < 1e-5, /* default amplitude must be 0.05 */);
    }

    #[test]
    fn test_tick_advances_time() {
        let mut c = new_camera_shake_view();
        csh_tick(&mut c, 0.016);
        assert!(c.time > 0.0 /* time must advance after tick */,);
    }

    #[test]
    fn test_negative_dt_ignored() {
        let mut c = new_camera_shake_view();
        csh_tick(&mut c, -1.0);
        assert!((c.time).abs() < 1e-6, /* negative dt must not advance time */);
    }

    #[test]
    fn test_sample_offset_length() {
        let c = new_camera_shake_view();
        let off = csh_sample_offset(&c);
        assert_eq!(off.len(), 3 /* offset must have 3 components */,);
    }

    #[test]
    fn test_set_profile() {
        let mut c = new_camera_shake_view();
        csh_set_profile(&mut c, ShakeProfile::Explosion);
        assert_eq!(
            c.profile,
            ShakeProfile::Explosion, /* profile must be set */
        );
    }

    #[test]
    fn test_set_amplitude() {
        let mut c = new_camera_shake_view();
        csh_set_amplitude(&mut c, 0.2);
        assert!((c.amplitude - 0.2).abs() < 1e-5, /* amplitude must be set */);
    }

    #[test]
    fn test_amplitude_clamped() {
        let mut c = new_camera_shake_view();
        csh_set_amplitude(&mut c, -1.0);
        assert!((c.amplitude).abs() < 1e-6, /* negative amplitude clamped to 0 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut c = new_camera_shake_view();
        csh_set_enabled(&mut c, false);
        assert!(!c.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_amplitude() {
        let c = new_camera_shake_view();
        let j = csh_to_json(&c);
        assert!(j.contains("\"amplitude\""), /* json must contain amplitude */);
    }

    #[test]
    fn test_enabled_default() {
        let c = new_camera_shake_view();
        assert!(c.enabled /* must be enabled by default */,);
    }
}
