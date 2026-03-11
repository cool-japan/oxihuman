// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Damping ratio utilities for spring-damper systems.

use std::f32::consts::PI;

/// Damping category.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DampingCategory {
    Underdamped,
    CriticallyDamped,
    Overdamped,
}

/// Classify damping ratio.
#[allow(dead_code)]
pub fn classify_damping(zeta: f32) -> DampingCategory {
    if zeta < 1.0 {
        DampingCategory::Underdamped
    } else if (zeta - 1.0).abs() < 1e-5 {
        DampingCategory::CriticallyDamped
    } else {
        DampingCategory::Overdamped
    }
}

/// Natural frequency from spring stiffness and mass.
#[allow(dead_code)]
pub fn natural_frequency(stiffness: f32, mass: f32) -> f32 {
    (stiffness / mass.max(1e-12)).sqrt()
}

/// Damping ratio from damping coefficient, mass, and stiffness.
#[allow(dead_code)]
pub fn damping_ratio(c: f32, mass: f32, stiffness: f32) -> f32 {
    let cc = 2.0 * (mass * stiffness).sqrt();
    if cc < 1e-12 {
        return 0.0;
    }
    c / cc
}

/// Critical damping coefficient.
#[allow(dead_code)]
pub fn critical_damping_coeff(mass: f32, stiffness: f32) -> f32 {
    2.0 * (mass * stiffness).sqrt()
}

/// Damped natural frequency.
#[allow(dead_code)]
pub fn damped_frequency(omega_n: f32, zeta: f32) -> f32 {
    omega_n * (1.0 - zeta * zeta).max(0.0).sqrt()
}

/// Settling time (to within 2% for underdamped system).
#[allow(dead_code)]
pub fn settling_time_2pct(zeta: f32, omega_n: f32) -> f32 {
    if omega_n < 1e-12 {
        return f32::MAX;
    }
    4.0 / (zeta * omega_n)
}

/// Peak overshoot fraction for underdamped system.
#[allow(dead_code)]
pub fn peak_overshoot(zeta: f32) -> f32 {
    if zeta >= 1.0 {
        return 0.0;
    }
    (-PI * zeta / (1.0 - zeta * zeta).sqrt()).exp()
}

/// Period of damped oscillation.
#[allow(dead_code)]
pub fn damped_period(omega_n: f32, zeta: f32) -> Option<f32> {
    let wd = damped_frequency(omega_n, zeta);
    if wd < 1e-12 {
        return None;
    }
    Some(2.0 * PI / wd)
}

/// Response amplitude at frequency omega for forced oscillation (normalised).
#[allow(dead_code)]
pub fn frequency_response(omega_n: f32, zeta: f32, omega: f32) -> f32 {
    let r = omega / omega_n.max(1e-12);
    let denom = ((1.0 - r * r) * (1.0 - r * r) + 4.0 * zeta * zeta * r * r).sqrt();
    if denom < 1e-12 {
        return f32::MAX;
    }
    1.0 / denom
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_underdamped() {
        assert_eq!(classify_damping(0.5), DampingCategory::Underdamped);
    }

    #[test]
    fn test_classify_critical() {
        assert_eq!(classify_damping(1.0), DampingCategory::CriticallyDamped);
    }

    #[test]
    fn test_classify_overdamped() {
        assert_eq!(classify_damping(1.5), DampingCategory::Overdamped);
    }

    #[test]
    fn test_natural_frequency() {
        let omega = natural_frequency(100.0, 1.0);
        assert!((omega - 10.0_f32).abs() < 1e-4);
    }

    #[test]
    fn test_damping_ratio_critical() {
        let cc = critical_damping_coeff(1.0, 100.0);
        let zeta = damping_ratio(cc, 1.0, 100.0);
        assert!((zeta - 1.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_damped_frequency_underdamped() {
        let wd = damped_frequency(10.0, 0.5);
        assert!((wd - 10.0 * (0.75_f32).sqrt()).abs() < 1e-4);
    }

    #[test]
    fn test_peak_overshoot_zero_at_critical() {
        assert_eq!(peak_overshoot(1.0), 0.0);
    }

    #[test]
    fn test_peak_overshoot_positive_underdamped() {
        assert!(peak_overshoot(0.3) > 0.0);
    }

    #[test]
    fn test_settling_time() {
        let t = settling_time_2pct(1.0, 10.0);
        assert!((t - 0.4_f32).abs() < 1e-4);
    }

    #[test]
    fn test_frequency_response_resonance() {
        // At resonance (omega=omega_n), response should be large for low zeta.
        let resp = frequency_response(10.0, 0.01, 10.0);
        assert!(resp > 10.0);
    }
}
