// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Inverted triangle (V-shape) body figure morph.

/// Configuration for the inverted triangle morph.
#[derive(Debug, Clone)]
pub struct InvertedTriangleConfig {
    pub shoulder_broadness: f32,
    pub chest_width: f32,
    pub hip_narrowness: f32,
}

impl Default for InvertedTriangleConfig {
    fn default() -> Self {
        InvertedTriangleConfig {
            shoulder_broadness: 0.85,
            chest_width: 0.7,
            hip_narrowness: 0.6,
        }
    }
}

/// State for the inverted triangle morph.
#[derive(Debug, Clone)]
pub struct InvertedTriangleMorph {
    pub intensity: f32,
    pub config: InvertedTriangleConfig,
    pub enabled: bool,
}

/// Create a new inverted triangle morph.
pub fn new_inverted_triangle_morph() -> InvertedTriangleMorph {
    InvertedTriangleMorph {
        intensity: 0.0,
        config: InvertedTriangleConfig::default(),
        enabled: true,
    }
}

/// Set intensity [0, 1].
pub fn inv_set_intensity(m: &mut InvertedTriangleMorph, v: f32) {
    m.intensity = v.clamp(0.0, 1.0);
}

/// Shoulder broadness weight.
pub fn inv_shoulder_broad(m: &InvertedTriangleMorph) -> f32 {
    m.intensity * m.config.shoulder_broadness
}

/// Chest width weight.
pub fn inv_chest_width(m: &InvertedTriangleMorph) -> f32 {
    m.intensity * m.config.chest_width
}

/// Hip narrowness weight.
pub fn inv_hip_narrow(m: &InvertedTriangleMorph) -> f32 {
    m.intensity * m.config.hip_narrowness
}

/// Shoulder-to-hip ratio estimate.
pub fn inv_shoulder_hip_ratio(m: &InvertedTriangleMorph) -> f32 {
    1.0 + 0.4 * m.intensity
}

/// Serialise to JSON.
pub fn inv_to_json(m: &InvertedTriangleMorph) -> String {
    format!(
        r#"{{"intensity":{:.3},"shoulder_broad":{:.3},"enabled":{}}}"#,
        m.intensity,
        inv_shoulder_broad(m),
        m.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_zero() {
        let m = new_inverted_triangle_morph();
        assert!((m.intensity - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn clamp() {
        let mut m = new_inverted_triangle_morph();
        inv_set_intensity(&mut m, 5.0);
        assert!((m.intensity - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn shoulder_broad_at_full() {
        let mut m = new_inverted_triangle_morph();
        inv_set_intensity(&mut m, 1.0);
        assert!((inv_shoulder_broad(&m) - m.config.shoulder_broadness).abs() < 1e-6 /* correct */);
    }

    #[test]
    fn hip_narrow_zero_at_zero() {
        let m = new_inverted_triangle_morph();
        assert!((inv_hip_narrow(&m) - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn shoulder_hip_ratio_increases() {
        let mut m = new_inverted_triangle_morph();
        inv_set_intensity(&mut m, 0.0);
        let r0 = inv_shoulder_hip_ratio(&m);
        inv_set_intensity(&mut m, 1.0);
        let r1 = inv_shoulder_hip_ratio(&m);
        assert!(r1 > r0 /* more inverted triangle at higher intensity */);
    }

    #[test]
    fn json_has_shoulder() {
        let m = new_inverted_triangle_morph();
        assert!(inv_to_json(&m).contains("shoulder") /* json has field */);
    }

    #[test]
    fn enabled_default() {
        let m = new_inverted_triangle_morph();
        assert!(m.enabled /* enabled */);
    }

    #[test]
    fn config_positive() {
        let m = new_inverted_triangle_morph();
        assert!(m.config.chest_width > 0.0 /* positive */);
    }
}
