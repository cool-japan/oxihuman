// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! HeadProportion — parametric head size and shape.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Head width, height, and depth in normalised units.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HeadProportion {
    pub width: f32,
    pub height: f32,
    pub depth: f32,
}

impl Default for HeadProportion {
    fn default() -> Self {
        HeadProportion { width: 1.0, height: 1.0, depth: 1.0 }
    }
}

/// Create a default `HeadProportion`.
#[allow(dead_code)]
pub fn new_head_proportion() -> HeadProportion {
    HeadProportion::default()
}

/// Set the head width parameter.
#[allow(dead_code)]
pub fn set_head_width(hp: &mut HeadProportion, w: f32) {
    hp.width = w;
}

/// Set the head height parameter.
#[allow(dead_code)]
pub fn set_head_height(hp: &mut HeadProportion, h: f32) {
    hp.height = h;
}

/// Set the head depth parameter.
#[allow(dead_code)]
pub fn set_head_depth(hp: &mut HeadProportion, d: f32) {
    hp.depth = d;
}

/// Approximate ellipsoidal volume: (4/3)·π·a·b·c.
#[allow(dead_code)]
pub fn head_volume_approx(hp: &HeadProportion) -> f32 {
    (4.0 / 3.0) * PI * (hp.width / 2.0) * (hp.height / 2.0) * (hp.depth / 2.0)
}

/// Return the width-to-height aspect ratio.
#[allow(dead_code)]
pub fn head_aspect_ratio(hp: &HeadProportion) -> f32 {
    hp.width / hp.height.max(f32::EPSILON)
}

/// Convert proportions to a `[width, height, depth]` parameter array.
#[allow(dead_code)]
pub fn proportion_to_params(hp: &HeadProportion) -> [f32; 3] {
    [hp.width, hp.height, hp.depth]
}

/// Approximate head circumference using the ellipse perimeter approximation:
/// π · (3(a+b) − √((3a+b)(a+3b))) where a = width/2, b = depth/2.
#[allow(dead_code)]
pub fn head_circumference_approx(hp: &HeadProportion) -> f32 {
    let a = hp.width / 2.0;
    let b = hp.depth / 2.0;
    PI * (3.0 * (a + b) - ((3.0 * a + b) * (a + 3.0 * b)).max(0.0).sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_head_proportion_defaults() {
        let hp = new_head_proportion();
        assert_eq!(hp.width, 1.0);
        assert_eq!(hp.height, 1.0);
        assert_eq!(hp.depth, 1.0);
    }

    #[test]
    fn test_set_head_width() {
        let mut hp = new_head_proportion();
        set_head_width(&mut hp, 1.2);
        assert!((hp.width - 1.2).abs() < 1e-6);
    }

    #[test]
    fn test_set_head_height() {
        let mut hp = new_head_proportion();
        set_head_height(&mut hp, 1.3);
        assert!((hp.height - 1.3).abs() < 1e-6);
    }

    #[test]
    fn test_set_head_depth() {
        let mut hp = new_head_proportion();
        set_head_depth(&mut hp, 0.9);
        assert!((hp.depth - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_head_volume_unit_sphere() {
        use std::f32::consts::PI;
        let hp = new_head_proportion();
        let v = head_volume_approx(&hp);
        // (4/3)π(0.5)(0.5)(0.5) = π/6
        let expected = PI / 6.0;
        assert!((v - expected).abs() < 1e-5);
    }

    #[test]
    fn test_head_aspect_ratio_equal() {
        let hp = new_head_proportion();
        assert!((head_aspect_ratio(&hp) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_proportion_to_params() {
        let mut hp = new_head_proportion();
        set_head_width(&mut hp, 1.1);
        set_head_height(&mut hp, 1.2);
        set_head_depth(&mut hp, 1.3);
        let p = proportion_to_params(&hp);
        assert!((p[0] - 1.1).abs() < 1e-5);
        assert!((p[1] - 1.2).abs() < 1e-5);
        assert!((p[2] - 1.3).abs() < 1e-5);
    }

    #[test]
    fn test_head_circumference_positive() {
        let hp = new_head_proportion();
        assert!(head_circumference_approx(&hp) > 0.0);
    }

    #[test]
    fn test_head_circumference_larger_head() {
        let mut hp = new_head_proportion();
        let c1 = head_circumference_approx(&hp);
        set_head_width(&mut hp, 2.0);
        set_head_depth(&mut hp, 2.0);
        let c2 = head_circumference_approx(&hp);
        assert!(c2 > c1);
    }
}
