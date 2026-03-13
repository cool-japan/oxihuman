// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::TAU;

/// Single-degree-of-freedom vibration model.
#[derive(Debug, Clone)]
pub struct VibrationModel {
    pub mass: f32,
    pub stiffness: f32,
    pub damping: f32,
}

/// Create a new VibrationModel.
pub fn new_vibration_model(m: f32, k: f32, c: f32) -> VibrationModel {
    VibrationModel {
        mass: m,
        stiffness: k,
        damping: c,
    }
}

/// Natural frequency in Hz: fn = sqrt(k/m) / (2π).
pub fn natural_frequency(v: &VibrationModel) -> f32 {
    if v.mass < 1e-12 {
        return 0.0;
    }
    (v.stiffness / v.mass).sqrt() / TAU
}

/// Damping ratio: zeta = c / (2 * sqrt(k * m)).
pub fn damping_ratio(v: &VibrationModel) -> f32 {
    let denom = 2.0 * (v.stiffness * v.mass).sqrt();
    if denom < 1e-12 {
        return 0.0;
    }
    v.damping / denom
}

/// Returns true if the system is overdamped (zeta >= 1.0).
pub fn is_overdamped(v: &VibrationModel) -> bool {
    damping_ratio(v) >= 1.0
}

/// Resonance amplitude: X = F / (k * sqrt((1 - r²)² + (2*zeta*r)²))
/// evaluated at resonance r = 1.
pub fn resonance_amplitude(v: &VibrationModel, force: f32) -> f32 {
    let zeta = damping_ratio(v);
    if v.stiffness < 1e-12 {
        return 0.0;
    }
    let denominator = v.stiffness * 2.0 * zeta;
    if denominator.abs() < 1e-12 {
        return f32::INFINITY;
    }
    force / denominator
}

/// Damped natural frequency in Hz.
pub fn damped_natural_frequency(v: &VibrationModel) -> f32 {
    let zeta = damping_ratio(v);
    if zeta >= 1.0 {
        return 0.0;
    }
    natural_frequency(v) * (1.0 - zeta * zeta).sqrt()
}

/// Logarithmic decrement: delta = 2*pi*zeta / sqrt(1 - zeta^2).
pub fn logarithmic_decrement(v: &VibrationModel) -> f32 {
    let zeta = damping_ratio(v);
    if zeta >= 1.0 {
        return f32::INFINITY;
    }
    use std::f32::consts::PI;
    2.0 * PI * zeta / (1.0 - zeta * zeta).sqrt()
}

/// Static deflection: delta_st = F / k.
pub fn static_deflection(v: &VibrationModel, force: f32) -> f32 {
    if v.stiffness < 1e-12 {
        return 0.0;
    }
    force / v.stiffness
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_vibration_model() {
        /* constructor */
        let v = new_vibration_model(1.0, 100.0, 5.0);
        assert!((v.mass - 1.0).abs() < 1e-9);
        assert!((v.stiffness - 100.0).abs() < 1e-9);
        assert!((v.damping - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_natural_frequency() {
        /* undamped: fn = sqrt(k/m)/(2pi) */
        use std::f32::consts::PI;
        let v = new_vibration_model(1.0, TAU * TAU, 0.0);
        let fn_ = natural_frequency(&v);
        assert!((fn_ - 1.0).abs() < 1e-4);
        let _ = PI;
    }

    #[test]
    fn test_damping_ratio_undamped() {
        let v = new_vibration_model(1.0, 100.0, 0.0);
        assert!(damping_ratio(&v).abs() < 1e-9);
    }

    #[test]
    fn test_damping_ratio_critical() {
        /* c = 2*sqrt(k*m) -> zeta = 1.0 */
        let k = 100.0f32;
        let m = 1.0f32;
        let c = 2.0 * (k * m).sqrt();
        let v = new_vibration_model(m, k, c);
        assert!((damping_ratio(&v) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_is_overdamped_true() {
        let v = new_vibration_model(1.0, 1.0, 100.0);
        assert!(is_overdamped(&v));
    }

    #[test]
    fn test_is_overdamped_false() {
        let v = new_vibration_model(1.0, 100.0, 1.0);
        assert!(!is_overdamped(&v));
    }

    #[test]
    fn test_resonance_amplitude_positive() {
        let v = new_vibration_model(1.0, 100.0, 5.0);
        let x = resonance_amplitude(&v, 10.0);
        assert!(x > 0.0);
    }

    #[test]
    fn test_static_deflection() {
        /* delta = F/k */
        let v = new_vibration_model(1.0, 100.0, 5.0);
        let d = static_deflection(&v, 50.0);
        assert!((d - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_damped_freq_underdamped() {
        let v = new_vibration_model(1.0, 100.0, 1.0);
        let fd = damped_natural_frequency(&v);
        let fn_ = natural_frequency(&v);
        assert!(fd < fn_);
        assert!(fd > 0.0);
    }

    #[test]
    fn test_logarithmic_decrement() {
        let v = new_vibration_model(1.0, 100.0, 1.0);
        let ld = logarithmic_decrement(&v);
        assert!(ld > 0.0);
    }
}
