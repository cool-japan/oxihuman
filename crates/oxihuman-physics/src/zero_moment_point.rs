// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Zero Moment Point (ZMP) balance criterion stub.

/// ZMP result.
#[derive(Debug, Clone, PartialEq)]
pub struct ZmpResult {
    pub zmp_x: f32,
    pub zmp_y: f32,
    pub is_stable: bool,
}

/// Support polygon (convex hull of contact points, approximated as AABB).
#[derive(Debug, Clone)]
pub struct SupportPolygon {
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
}

impl SupportPolygon {
    pub fn new(x_min: f32, x_max: f32, y_min: f32, y_max: f32) -> Self {
        Self {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }

    /// Return whether a point is inside this support polygon.
    pub fn contains(&self, x: f32, y: f32) -> bool {
        (self.x_min..=self.x_max).contains(&x) && (self.y_min..=self.y_max).contains(&y)
    }
}

/// Compute the ZMP from CoM position, velocity, and acceleration.
/// Uses the standard linear inverted pendulum ZMP formula: `zmp = com - h/g * com_acc`.
pub fn compute_zmp(
    com_x: f32,
    com_y: f32,
    com_acc_x: f32,
    com_acc_y: f32,
    height: f32,
    gravity: f32,
) -> ZmpResult {
    /* ZMP formula: p = c - h/g * c_ddot */
    let g = gravity.abs().max(1e-6);
    let factor = height / g;
    let zmp_x = com_x - factor * com_acc_x;
    let zmp_y = com_y - factor * com_acc_y;
    ZmpResult {
        zmp_x,
        zmp_y,
        is_stable: false,
    }
}

/// Evaluate ZMP stability given a support polygon.
pub fn evaluate_zmp_stability(zmp: ZmpResult, support: &SupportPolygon) -> ZmpResult {
    let is_stable = support.contains(zmp.zmp_x, zmp.zmp_y);
    ZmpResult { is_stable, ..zmp }
}

/// Compute the ZMP margin (minimum distance to polygon edge, stub).
pub fn zmp_margin(zmp: &ZmpResult, support: &SupportPolygon) -> f32 {
    let dx = (zmp.zmp_x - support.x_min).min(support.x_max - zmp.zmp_x);
    let dy = (zmp.zmp_y - support.y_min).min(support.y_max - zmp.zmp_y);
    dx.min(dy)
}

/// Return a simple stability label.
pub fn zmp_stability_label(margin: f32) -> &'static str {
    if margin > 0.05 {
        "stable"
    } else if margin > 0.0 {
        "marginal"
    } else {
        "unstable"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_support() -> SupportPolygon {
        SupportPolygon::new(-0.1, 0.1, -0.05, 0.05)
    }

    #[test]
    fn test_zmp_static_balance() {
        /* zero acceleration → ZMP equals CoM */
        let r = compute_zmp(0.0, 0.0, 0.0, 0.0, 1.0, 9.81);
        assert!((r.zmp_x).abs() < 1e-5);
        assert!((r.zmp_y).abs() < 1e-5);
    }

    #[test]
    fn test_zmp_with_acceleration() {
        /* forward acceleration shifts ZMP backwards */
        let r = compute_zmp(0.0, 0.0, 1.0, 0.0, 1.0, 9.81);
        assert!(r.zmp_x < 0.0);
    }

    #[test]
    fn test_support_polygon_contains() {
        /* point inside polygon */
        let s = default_support();
        assert!(s.contains(0.0, 0.0));
    }

    #[test]
    fn test_support_polygon_excludes() {
        /* point outside polygon */
        let s = default_support();
        assert!(!s.contains(1.0, 0.0));
    }

    #[test]
    fn test_evaluate_stability_true() {
        /* ZMP at centre is stable */
        let r = compute_zmp(0.0, 0.0, 0.0, 0.0, 1.0, 9.81);
        let r = evaluate_zmp_stability(r, &default_support());
        assert!(r.is_stable);
    }

    #[test]
    fn test_evaluate_stability_false() {
        /* ZMP far outside polygon is unstable */
        let r = ZmpResult {
            zmp_x: 5.0,
            zmp_y: 5.0,
            is_stable: false,
        };
        let r = evaluate_zmp_stability(r, &default_support());
        assert!(!r.is_stable);
    }

    #[test]
    fn test_zmp_margin_positive() {
        /* ZMP at centre has positive margin */
        let r = ZmpResult {
            zmp_x: 0.0,
            zmp_y: 0.0,
            is_stable: true,
        };
        assert!(zmp_margin(&r, &default_support()) > 0.0);
    }

    #[test]
    fn test_stability_label_stable() {
        /* large margin is stable */
        assert_eq!(zmp_stability_label(0.1), "stable");
    }

    #[test]
    fn test_stability_label_unstable() {
        /* negative margin is unstable */
        assert_eq!(zmp_stability_label(-0.01), "unstable");
    }
}
