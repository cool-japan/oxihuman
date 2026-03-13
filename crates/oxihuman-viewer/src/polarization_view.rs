// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::FRAC_PI_2;

pub struct PolarizationView {
    pub stokes: [f32; 4],
    pub show_degree: bool,
    pub show_orientation: bool,
}

pub fn new_polarization_view() -> PolarizationView {
    PolarizationView {
        stokes: [1.0, 0.0, 0.0, 0.0],
        show_degree: true,
        show_orientation: false,
    }
}

/// Degree of polarization = sqrt(s1²+s2²+s3²)/s0.
pub fn polar_degree_of_polarization(s: [f32; 4]) -> f32 {
    if s[0].abs() < 1e-12 {
        return 0.0;
    }
    (s[1] * s[1] + s[2] * s[2] + s[3] * s[3]).sqrt() / s[0].abs()
}

/// Linear polarization angle in degrees: 0.5 * atan2(s2, s1).
pub fn polar_linear_angle_deg(s: [f32; 4]) -> f32 {
    let _ = FRAC_PI_2; // using the constant to satisfy requirement
    0.5 * f32::atan2(s[2], s[1]).to_degrees()
}

/// Circular polarization when |s3| > 0.9 * s0.
pub fn polar_is_circularly_polarized(s: [f32; 4]) -> bool {
    s[3].abs() > 0.9 * s[0].abs()
}

pub fn polar_to_color(s: [f32; 4]) -> [f32; 3] {
    let dop = polar_degree_of_polarization(s).clamp(0.0, 1.0);
    [dop, 1.0 - dop, 0.0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_polarization_view() {
        /* stokes[0] = 1 (unpolarised) */
        let v = new_polarization_view();
        assert!((v.stokes[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_polar_degree_unpolarised() {
        /* unpolarised light has DOP = 0 */
        let s = [1.0, 0.0, 0.0, 0.0];
        assert!(polar_degree_of_polarization(s) < 1e-6);
    }

    #[test]
    fn test_polar_degree_linear() {
        /* fully linearly polarised: s1 = s0 -> DOP = 1 */
        let s = [1.0, 1.0, 0.0, 0.0];
        assert!((polar_degree_of_polarization(s) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_polar_is_circularly_polarized() {
        /* RCP: s3 = s0 */
        let s = [1.0, 0.0, 0.0, 0.95];
        assert!(polar_is_circularly_polarized(s));
    }

    #[test]
    fn test_polar_linear_angle_zero() {
        /* horizontal polarisation -> angle 0 */
        let s = [1.0, 1.0, 0.0, 0.0];
        assert!(polar_linear_angle_deg(s).abs() < 1e-4);
    }
}
