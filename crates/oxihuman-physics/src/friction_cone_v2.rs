// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Friction cone v2 — Coulomb friction cone constraint for rigid body contacts.

/// A friction cone at a contact point.
#[derive(Debug, Clone)]
pub struct FrictionConeV2 {
    pub normal: [f64; 3],
    pub mu: f64, /* coefficient of friction */
    pub normal_force: f64,
}

impl FrictionConeV2 {
    /// Create a new friction cone.
    pub fn new(normal: [f64; 3], mu: f64, normal_force: f64) -> Self {
        FrictionConeV2 {
            normal,
            mu: mu.max(0.0),
            normal_force: normal_force.max(0.0),
        }
    }

    /// Maximum tangential force magnitude (Coulomb limit).
    pub fn max_tangential_force(&self) -> f64 {
        self.mu * self.normal_force
    }

    /// Clamp a tangential force vector to the friction cone.
    pub fn clamp_tangential(&self, ft: [f64; 3]) -> [f64; 3] {
        let limit = self.max_tangential_force();
        let len = (ft[0] * ft[0] + ft[1] * ft[1] + ft[2] * ft[2]).sqrt();
        if len <= limit || len < 1e-12 {
            ft
        } else {
            let scale = limit / len;
            [ft[0] * scale, ft[1] * scale, ft[2] * scale]
        }
    }

    /// Check if a force vector (3D) lies inside the friction cone.
    pub fn is_inside_cone(&self, force: [f64; 3]) -> bool {
        let fn_val = dot3(force, self.normal);
        if fn_val <= 0.0 {
            return false;
        }
        let ft = sub3(force, scale3(self.normal, fn_val));
        let ft_len = len3(ft);
        ft_len <= self.mu * fn_val
    }
}

fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn sub3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale3(a: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn len3(a: [f64; 3]) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

/// Project a 3D force onto the tangential plane of the contact normal.
pub fn tangential_component(force: [f64; 3], normal: [f64; 3]) -> [f64; 3] {
    let fn_val = dot3(force, normal);
    sub3(force, scale3(normal, fn_val))
}

/// Compute the impulse needed to enforce the friction cone given a proposed tangential impulse.
pub fn friction_clamp(proposed: [f64; 3], mu: f64, normal_impulse: f64) -> [f64; 3] {
    let limit = mu * normal_impulse.max(0.0);
    let len = len3(proposed);
    if len <= limit || len < 1e-12 {
        proposed
    } else {
        scale3(proposed, limit / len)
    }
}

/// Sliding direction: unit vector of tangential velocity, or zero if negligible.
pub fn sliding_direction(rel_vel: [f64; 3], normal: [f64; 3]) -> [f64; 3] {
    let vt = tangential_component(rel_vel, normal);
    let len = len3(vt);
    if len < 1e-12 {
        [0.0; 3]
    } else {
        scale3(vt, 1.0 / len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_tangential_force() {
        let cone = FrictionConeV2::new([0.0, 1.0, 0.0], 0.5, 10.0);
        assert!((cone.max_tangential_force() - 5.0).abs() < 1e-10 /* mu * N = 5 */);
    }

    #[test]
    fn test_clamp_within_limit() {
        let cone = FrictionConeV2::new([0.0, 1.0, 0.0], 0.5, 10.0);
        let ft = [2.0, 0.0, 0.0];
        let clamped = cone.clamp_tangential(ft);
        assert_eq!(clamped, ft /* within limit, no clamping */);
    }

    #[test]
    fn test_clamp_exceeds_limit() {
        let cone = FrictionConeV2::new([0.0, 1.0, 0.0], 0.5, 10.0);
        let ft = [100.0, 0.0, 0.0];
        let clamped = cone.clamp_tangential(ft);
        let len = len3(clamped);
        assert!((len - 5.0).abs() < 1e-8 /* clamped to max limit */);
    }

    #[test]
    fn test_is_inside_cone_true() {
        let cone = FrictionConeV2::new([0.0, 1.0, 0.0], 0.5, 1.0);
        let force = [0.1, 1.0, 0.0]; /* small tangential, large normal */
        assert!(cone.is_inside_cone(force) /* force inside cone */);
    }

    #[test]
    fn test_is_inside_cone_false() {
        let cone = FrictionConeV2::new([0.0, 1.0, 0.0], 0.1, 1.0);
        let force = [10.0, 1.0, 0.0]; /* large tangential, small normal */
        assert!(!cone.is_inside_cone(force) /* force outside cone */);
    }

    #[test]
    fn test_tangential_component_perpendicular() {
        let force = [1.0, 5.0, 0.0];
        let normal = [0.0, 1.0, 0.0];
        let vt = tangential_component(force, normal);
        assert!((vt[1]).abs() < 1e-10 /* no normal component in tangential */);
    }

    #[test]
    fn test_friction_clamp_within() {
        let result = friction_clamp([1.0, 0.0, 0.0], 0.5, 10.0);
        assert_eq!(result, [1.0, 0.0, 0.0] /* within limit */);
    }

    #[test]
    fn test_sliding_direction() {
        let rel_vel = [3.0, 1.0, 0.0];
        let normal = [0.0, 1.0, 0.0];
        let dir = sliding_direction(rel_vel, normal);
        let len = len3(dir);
        assert!((len - 1.0).abs() < 1e-10 /* unit direction */);
    }

    #[test]
    fn test_mu_clamped_to_zero() {
        let cone = FrictionConeV2::new([0.0, 1.0, 0.0], -1.0, 10.0);
        assert!(cone.mu >= 0.0 /* mu clamped to zero */);
    }
}
