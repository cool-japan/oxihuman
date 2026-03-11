// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! ShoulderWidth — shoulder width control parameter.

#![allow(dead_code)]

/// Shoulder width state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShoulderWidth {
    pub width: f32,
    pub symmetry: f32,
}

impl Default for ShoulderWidth {
    fn default() -> Self {
        ShoulderWidth { width: 1.0, symmetry: 1.0 }
    }
}

/// Create a default `ShoulderWidth`.
#[allow(dead_code)]
pub fn new_shoulder_width() -> ShoulderWidth {
    ShoulderWidth::default()
}

/// Set the shoulder width value.
#[allow(dead_code)]
pub fn set_shoulder_width(sw: &mut ShoulderWidth, w: f32) {
    sw.width = w;
}

/// Return the current shoulder width.
#[allow(dead_code)]
pub fn get_shoulder_width(sw: &ShoulderWidth) -> f32 {
    sw.width
}

/// Return the lateral shoulder offset from the centreline.
#[allow(dead_code)]
pub fn shoulder_offset(sw: &ShoulderWidth) -> f32 {
    sw.width / 2.0
}

/// Return the symmetry factor (1.0 = perfectly symmetric).
#[allow(dead_code)]
pub fn shoulder_symmetry(sw: &ShoulderWidth) -> f32 {
    sw.symmetry
}

/// Apply the shoulder width to a weight array (index 0 = width param).
#[allow(dead_code)]
pub fn apply_shoulder_width(sw: &ShoulderWidth, weights: &mut [f32]) {
    if !weights.is_empty() {
        weights[0] = sw.width;
    }
}

/// Convert the ShoulderWidth to a scalar parameter in [0, 1].
#[allow(dead_code)]
pub fn shoulder_to_param(sw: &ShoulderWidth, min_w: f32, max_w: f32) -> f32 {
    let range = (max_w - min_w).max(f32::EPSILON);
    ((sw.width - min_w) / range).clamp(0.0, 1.0)
}

/// Reconstruct a `ShoulderWidth` from a scalar parameter in [0, 1].
#[allow(dead_code)]
pub fn shoulder_from_param(param: f32, min_w: f32, max_w: f32) -> ShoulderWidth {
    ShoulderWidth { width: min_w + param.clamp(0.0, 1.0) * (max_w - min_w), symmetry: 1.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_width_one() {
        let sw = new_shoulder_width();
        assert_eq!(get_shoulder_width(&sw), 1.0);
    }

    #[test]
    fn test_set_get_shoulder_width() {
        let mut sw = new_shoulder_width();
        set_shoulder_width(&mut sw, 1.5);
        assert!((get_shoulder_width(&sw) - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_shoulder_offset_half_width() {
        let sw = ShoulderWidth { width: 2.0, symmetry: 1.0 };
        assert!((shoulder_offset(&sw) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_shoulder_symmetry() {
        let sw = new_shoulder_width();
        assert_eq!(shoulder_symmetry(&sw), 1.0);
    }

    #[test]
    fn test_apply_shoulder_width() {
        let sw = ShoulderWidth { width: 1.8, symmetry: 1.0 };
        let mut w = vec![0.0_f32];
        apply_shoulder_width(&sw, &mut w);
        assert!((w[0] - 1.8).abs() < 1e-6);
    }

    #[test]
    fn test_shoulder_to_param_midpoint() {
        let sw = ShoulderWidth { width: 1.5, symmetry: 1.0 };
        let p = shoulder_to_param(&sw, 1.0, 2.0);
        assert!((p - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_shoulder_from_param_midpoint() {
        let sw = shoulder_from_param(0.5, 1.0, 2.0);
        assert!((sw.width - 1.5).abs() < 1e-5);
    }

    #[test]
    fn test_shoulder_to_param_roundtrip() {
        let sw = ShoulderWidth { width: 1.3, symmetry: 1.0 };
        let p = shoulder_to_param(&sw, 1.0, 2.0);
        let sw2 = shoulder_from_param(p, 1.0, 2.0);
        assert!((sw2.width - sw.width).abs() < 1e-4);
    }
}
