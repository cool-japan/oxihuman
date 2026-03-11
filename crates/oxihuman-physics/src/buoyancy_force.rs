// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Compute buoyancy force for a given submerged volume.
/// F_b = rho_fluid * g * submerged_vol  (upward)
#[allow(dead_code)]
pub fn buoyancy_force_simple(submerged_vol: f32, rho_fluid: f32, g: f32) -> f32 {
    rho_fluid * g * submerged_vol
}

/// Compute the volume of a sphere submerged to depth `depth` below the fluid surface.
/// Uses the spherical cap formula: V = pi * h^2 * (3r - h) / 3, clamped to [0, 4/3*pi*r^3].
#[allow(dead_code)]
pub fn submerged_sphere_vol(radius: f32, depth: f32) -> f32 {
    if depth <= 0.0 {
        return 0.0;
    }
    let h = depth.min(2.0 * radius);
    let full = (4.0 / 3.0) * PI * radius * radius * radius;
    let cap = PI * h * h * (3.0 * radius - h) / 3.0;
    cap.min(full).max(0.0)
}

/// Compute the geometric center of buoyancy for a partially submerged sphere.
/// Returns the center of the submerged spherical cap as a 3D point (y = submerged centroid height).
#[allow(dead_code)]
pub fn buoyancy_center_sphere(radius: f32, depth: f32) -> [f32; 3] {
    if depth <= 0.0 {
        return [0.0, -radius, 0.0];
    }
    let h = depth.min(2.0 * radius);
    // The centroid of a spherical cap of height h on a sphere of radius r
    // measured from the bottom of the cap is: y_c = (3*(2r-h)^2) / (4*(3r-h))
    let denom = 4.0 * (3.0 * radius - h);
    let centroid_from_bottom = if denom.abs() < 1e-9 {
        0.0
    } else {
        3.0 * (2.0 * radius - h) * (2.0 * radius - h) / denom
    };
    // The bottom of the cap is at y = sphere_center_y - radius
    let y_bottom = -radius;
    [0.0, y_bottom + centroid_from_bottom, 0.0]
}

/// Archimedes principle: buoyancy = rho * g * volume.
#[allow(dead_code)]
pub fn archimedes_principle(volume: f32, rho: f32, g: f32) -> f32 {
    rho * g * volume
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buoyancy_force_simple_basic() {
        let f = buoyancy_force_simple(1.0, 1000.0, 9.81);
        assert!((f - 9810.0).abs() < 0.1);
    }

    #[test]
    fn buoyancy_force_zero_volume() {
        let f = buoyancy_force_simple(0.0, 1000.0, 9.81);
        assert_eq!(f, 0.0);
    }

    #[test]
    fn submerged_sphere_vol_not_submerged() {
        let v = submerged_sphere_vol(1.0, 0.0);
        assert_eq!(v, 0.0);
    }

    #[test]
    fn submerged_sphere_vol_fully_submerged() {
        let r = 1.0f32;
        let v = submerged_sphere_vol(r, 2.0 * r);
        let full = (4.0 / 3.0) * PI * r * r * r;
        assert!((v - full).abs() < 1e-4, "v={v}, full={full}");
    }

    #[test]
    fn submerged_sphere_vol_half() {
        let r = 1.0f32;
        let v = submerged_sphere_vol(r, r);
        let full = (4.0 / 3.0) * PI * r * r * r;
        // Half submerged should be approximately half volume
        assert!(v > 0.0 && v < full);
    }

    #[test]
    fn submerged_sphere_vol_negative_depth() {
        let v = submerged_sphere_vol(1.0, -1.0);
        assert_eq!(v, 0.0);
    }

    #[test]
    fn buoyancy_center_sphere_not_submerged() {
        let c = buoyancy_center_sphere(1.0, 0.0);
        assert!((c[1] - (-1.0)).abs() < 1e-5);
    }

    #[test]
    fn buoyancy_center_sphere_fully_submerged() {
        let r = 1.0f32;
        let c = buoyancy_center_sphere(r, 2.0 * r);
        // center of buoyancy must be within the sphere bounds: y in [-r, r]
        assert!(c[1] >= -r && c[1] <= r, "center y={} outside sphere", c[1]);
    }

    #[test]
    fn archimedes_principle_matches_buoyancy_force() {
        let f1 = buoyancy_force_simple(0.5, 1000.0, 9.81);
        let f2 = archimedes_principle(0.5, 1000.0, 9.81);
        assert!((f1 - f2).abs() < 1e-6);
    }

    #[test]
    fn archimedes_zero_volume() {
        let f = archimedes_principle(0.0, 1000.0, 9.81);
        assert_eq!(f, 0.0);
    }
}
