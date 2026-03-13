// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Gravitational potential energy.
#[allow(dead_code)]
pub fn gravitational_pe(mass: f32, height: f32, g: f32) -> f32 {
    mass * g * height
}

/// Elastic spring potential energy (1D).
#[allow(dead_code)]
pub fn spring_pe(k: f32, extension: f32) -> f32 {
    0.5 * k * extension * extension
}

/// Torsional spring potential energy.
#[allow(dead_code)]
pub fn torsional_pe(k_torsion: f32, angle_rad: f32) -> f32 {
    0.5 * k_torsion * angle_rad * angle_rad
}

/// Gravitational potential between two masses (Newton's law).
#[allow(dead_code)]
pub fn newtonian_grav_pe(m1: f32, m2: f32, r: f32, big_g: f32) -> f32 {
    if r < 1e-10 {
        return f32::NEG_INFINITY;
    }
    -big_g * m1 * m2 / r
}

/// Centrifugal potential in rotating frame.
#[allow(dead_code)]
pub fn centrifugal_pe(mass: f32, omega: f32, r: f32) -> f32 {
    -0.5 * mass * omega * omega * r * r
}

/// Lennard-Jones potential.
#[allow(dead_code)]
pub fn lennard_jones_pe(epsilon: f32, sigma: f32, r: f32) -> f32 {
    let sr = sigma / r.max(1e-10);
    let sr6 = sr.powi(6);
    4.0 * epsilon * (sr6 * sr6 - sr6)
}

/// Coulomb electrostatic potential energy.
#[allow(dead_code)]
pub fn coulomb_pe(q1: f32, q2: f32, r: f32, k_e: f32) -> f32 {
    if r < 1e-10 {
        return f32::INFINITY;
    }
    k_e * q1 * q2 / r
}

/// Bending potential of an elastic rod element.
#[allow(dead_code)]
pub fn bending_pe(ei: f32, curvature: f32, length: f32) -> f32 {
    0.5 * ei * curvature * curvature * length
}

/// Total potential energy from a collection of spring extensions.
#[allow(dead_code)]
pub fn total_spring_pe(k: f32, extensions: &[f32]) -> f32 {
    extensions.iter().map(|&e| spring_pe(k, e)).sum()
}

/// Height of free-fall after time t.
#[allow(dead_code)]
pub fn free_fall_height(h0: f32, v0: f32, t: f32, g: f32) -> f32 {
    h0 + v0 * t - 0.5 * g * t * t
}

/// Period of vertical harmonic oscillator from spring constant.
#[allow(dead_code)]
pub fn spring_oscillator_period(mass: f32, k: f32) -> f32 {
    2.0 * PI * (mass / k.max(1e-10)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_gravitational_pe() {
        let pe = gravitational_pe(2.0, 10.0, 9.81);
        assert!((pe - 196.2).abs() < 0.01);
    }
    #[test]
    fn test_spring_pe_zero_at_rest() {
        assert_eq!(spring_pe(100.0, 0.0), 0.0);
    }
    #[test]
    fn test_spring_pe_positive() {
        assert!(spring_pe(10.0, 0.5) > 0.0);
    }
    #[test]
    fn test_torsional_pe() {
        let pe = torsional_pe(1.0, std::f32::consts::FRAC_PI_4);
        assert!(pe > 0.0);
    }
    #[test]
    fn test_newtonian_grav_pe_negative() {
        let pe = newtonian_grav_pe(1.0, 1.0, 1.0, 6.674e-11);
        assert!(pe < 0.0);
    }
    #[test]
    fn test_coulomb_pe_like_sign_positive() {
        let pe = coulomb_pe(1.0, 1.0, 1.0, 8.99e9);
        assert!(pe > 0.0);
    }
    #[test]
    fn test_total_spring_pe() {
        let exts = [0.1f32, 0.2, 0.3];
        let total = total_spring_pe(10.0, &exts);
        let expected: f32 = exts.iter().map(|&e| spring_pe(10.0, e)).sum();
        assert!((total - expected).abs() < 1e-5);
    }
    #[test]
    fn test_free_fall_height() {
        let h = free_fall_height(100.0, 0.0, 1.0, 9.81);
        assert!((h - 95.095).abs() < 0.01);
    }
    #[test]
    fn test_spring_oscillator_period() {
        let t = spring_oscillator_period(1.0, (2.0 * PI).powi(2));
        assert!((t - 1.0).abs() < 0.01);
    }
    #[test]
    fn test_lennard_jones_minimum() {
        let sigma = 1.0f32;
        let r_min = 2.0f32.powf(1.0 / 6.0) * sigma;
        let pe_min = lennard_jones_pe(1.0, sigma, r_min);
        let pe_far = lennard_jones_pe(1.0, sigma, 10.0 * sigma);
        assert!(pe_min < pe_far);
    }
}
