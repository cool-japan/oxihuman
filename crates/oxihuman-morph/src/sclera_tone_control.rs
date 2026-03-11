// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Sclera yellowing and redness morph control.

/// Sclera tone parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ScleraToneParams {
    /// Yellowing (icterus) 0..=1.
    pub yellowing: f32,
    /// Redness (injection) 0..=1.
    pub redness: f32,
    /// Vein visibility 0..=1.
    pub vein_visibility: f32,
    /// Overall brightness offset -1..=1.
    pub brightness: f32,
}

impl Default for ScleraToneParams {
    fn default() -> Self {
        Self {
            yellowing: 0.0,
            redness: 0.0,
            vein_visibility: 0.0,
            brightness: 0.0,
        }
    }
}

/// Create default params.
#[allow(dead_code)]
pub fn default_sclera_tone_params() -> ScleraToneParams {
    ScleraToneParams::default()
}

/// Apply sclera tone to a base white color.
#[allow(dead_code)]
pub fn apply_sclera_tone(base: [f32; 3], params: &ScleraToneParams) -> [f32; 3] {
    let y = params.yellowing.clamp(0.0, 1.0);
    let r = params.redness.clamp(0.0, 1.0);
    let b = params.brightness.clamp(-1.0, 1.0);

    let red = (base[0] + b + r * 0.3).clamp(0.0, 1.0);
    let green = (base[1] + b + y * 0.15 - r * 0.1).clamp(0.0, 1.0);
    let blue = (base[2] + b - y * 0.3 - r * 0.2).clamp(0.0, 1.0);
    [red, green, blue]
}

/// Set yellowing.
#[allow(dead_code)]
pub fn set_sclera_yellowing(params: &mut ScleraToneParams, value: f32) {
    params.yellowing = value.clamp(0.0, 1.0);
}

/// Set redness.
#[allow(dead_code)]
pub fn set_sclera_redness(params: &mut ScleraToneParams, value: f32) {
    params.redness = value.clamp(0.0, 1.0);
}

/// Set vein visibility.
#[allow(dead_code)]
pub fn set_sclera_vein_visibility(params: &mut ScleraToneParams, value: f32) {
    params.vein_visibility = value.clamp(0.0, 1.0);
}

/// Set brightness.
#[allow(dead_code)]
pub fn set_sclera_brightness(params: &mut ScleraToneParams, value: f32) {
    params.brightness = value.clamp(-1.0, 1.0);
}

/// Blend two param sets.
#[allow(dead_code)]
pub fn blend_sclera_tone(a: &ScleraToneParams, b: &ScleraToneParams, t: f32) -> ScleraToneParams {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    ScleraToneParams {
        yellowing: a.yellowing * inv + b.yellowing * t,
        redness: a.redness * inv + b.redness * t,
        vein_visibility: a.vein_visibility * inv + b.vein_visibility * t,
        brightness: a.brightness * inv + b.brightness * t,
    }
}

/// Reset to default.
#[allow(dead_code)]
pub fn reset_sclera_tone(params: &mut ScleraToneParams) {
    *params = ScleraToneParams::default();
}

/// Check if healthy (near-default).
#[allow(dead_code)]
pub fn is_sclera_healthy(params: &ScleraToneParams) -> bool {
    params.yellowing < 0.1 && params.redness < 0.1
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn sclera_tone_to_json(params: &ScleraToneParams) -> String {
    format!(
        r#"{{"yellowing":{:.4},"redness":{:.4},"vein_visibility":{:.4},"brightness":{:.4}}}"#,
        params.yellowing, params.redness, params.vein_visibility, params.brightness
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_healthy() {
        let p = ScleraToneParams::default();
        assert!(is_sclera_healthy(&p));
    }

    #[test]
    fn test_apply_neutral() {
        let p = ScleraToneParams::default();
        let base = [1.0f32, 1.0, 1.0];
        let out = apply_sclera_tone(base, &p);
        assert!((out[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_apply_yellowing() {
        let p = ScleraToneParams {
            yellowing: 1.0,
            ..Default::default()
        };
        let base = [1.0f32, 1.0, 1.0];
        let out = apply_sclera_tone(base, &p);
        assert!(out[2] < out[0]);
    }

    #[test]
    fn test_set_yellowing_clamp() {
        let mut p = ScleraToneParams::default();
        set_sclera_yellowing(&mut p, 5.0);
        assert!((p.yellowing - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_redness() {
        let mut p = ScleraToneParams::default();
        set_sclera_redness(&mut p, 0.5);
        assert!((p.redness - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_brightness_clamp() {
        let mut p = ScleraToneParams::default();
        set_sclera_brightness(&mut p, -5.0);
        assert!((p.brightness + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_midpoint() {
        let a = ScleraToneParams {
            yellowing: 0.0,
            ..Default::default()
        };
        let b = ScleraToneParams {
            yellowing: 1.0,
            ..Default::default()
        };
        let r = blend_sclera_tone(&a, &b, 0.5);
        assert!((r.yellowing - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let mut p = ScleraToneParams {
            yellowing: 0.9,
            ..Default::default()
        };
        reset_sclera_tone(&mut p);
        assert!(p.yellowing.abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let j = sclera_tone_to_json(&ScleraToneParams::default());
        assert!(j.contains("yellowing"));
        assert!(j.contains("redness"));
    }
}
