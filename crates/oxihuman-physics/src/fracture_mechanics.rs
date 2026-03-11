// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Brittle fracture stress model stub.
//!
//! Computes stress intensity factors and critical conditions for Mode I, II, III
//! fracture mechanics using Linear Elastic Fracture Mechanics (LEFM).

/// Fracture mode.
#[derive(Debug, Clone, PartialEq)]
pub enum FractureMode {
    /// Mode I — opening (tension perpendicular to crack).
    ModeI,
    /// Mode II — sliding (shear parallel to crack).
    ModeII,
    /// Mode III — tearing (anti-plane shear).
    ModeIII,
}

/// Material fracture properties.
#[derive(Debug, Clone)]
pub struct FractureMaterial {
    /// Fracture toughness KIc [MPa·√m].
    pub toughness_k1c: f64,
    /// Young's modulus `[GPa]`.
    pub young_modulus: f64,
    /// Poisson ratio.
    pub poisson_ratio: f64,
    /// Critical stress for brittle fracture `[MPa]`.
    pub critical_stress: f64,
}

impl Default for FractureMaterial {
    fn default() -> Self {
        Self {
            toughness_k1c: 1.0,
            young_modulus: 70.0,
            poisson_ratio: 0.33,
            critical_stress: 200.0,
        }
    }
}

/// Result of a fracture analysis.
#[derive(Debug, Clone)]
pub struct FractureResult {
    pub stress_intensity: f64,
    pub is_critical: bool,
    pub safety_factor: f64,
    pub mode: FractureMode,
}

/// Compute the Mode I stress intensity factor K_I.
///
/// K_I = sigma * sqrt(pi * a) * F
///
/// where `sigma` is the applied stress `[MPa]`, `a` is the crack half-length `[m]`,
/// and `F` is a geometry correction factor (≈ 1 for an infinite plate).
pub fn stress_intensity_mode1(sigma: f64, crack_half_len: f64, geometry_factor: f64) -> f64 {
    sigma * (std::f64::consts::PI * crack_half_len).sqrt() * geometry_factor
}

/// Compute Mode II stress intensity factor (shear mode).
pub fn stress_intensity_mode2(tau: f64, crack_half_len: f64, geometry_factor: f64) -> f64 {
    tau * (std::f64::consts::PI * crack_half_len).sqrt() * geometry_factor
}

/// Determine if fracture occurs under Mode I loading.
pub fn analyze_fracture(sigma: f64, crack_half_len: f64, mat: &FractureMaterial) -> FractureResult {
    let ki = stress_intensity_mode1(sigma, crack_half_len, 1.0);
    let is_critical = ki >= mat.toughness_k1c;
    let safety_factor = if ki > 0.0 {
        mat.toughness_k1c / ki
    } else {
        f64::INFINITY
    };
    FractureResult {
        stress_intensity: ki,
        is_critical,
        safety_factor,
        mode: FractureMode::ModeI,
    }
}

/// Compute the critical crack size for a given applied stress.
///
/// a_c = (K_Ic / (sigma * sqrt(pi)))^2
pub fn critical_crack_size(sigma: f64, mat: &FractureMaterial) -> f64 {
    if sigma <= 0.0 || mat.toughness_k1c <= 0.0 {
        return f64::INFINITY;
    }
    let ratio = mat.toughness_k1c / (sigma * std::f64::consts::PI.sqrt());
    ratio * ratio
}

/// Compute the critical applied stress for a given crack size.
pub fn critical_stress(crack_half_len: f64, mat: &FractureMaterial) -> f64 {
    if crack_half_len <= 0.0 {
        return f64::INFINITY;
    }
    mat.toughness_k1c / (std::f64::consts::PI * crack_half_len).sqrt()
}

/// Compute the energy release rate G [J/m²] for Mode I (plane stress).
pub fn energy_release_rate_mode1(ki: f64, mat: &FractureMaterial) -> f64 {
    ki * ki / (mat.young_modulus * 1e9)
}

/// Check if a crack will propagate given current conditions.
pub fn will_propagate(ki: f64, mat: &FractureMaterial) -> bool {
    ki >= mat.toughness_k1c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stress_intensity_mode1_positive() {
        let ki = stress_intensity_mode1(100.0, 0.01, 1.0);
        assert!(ki > 0.0);
    }

    #[test]
    fn test_stress_intensity_mode2_positive() {
        let ki = stress_intensity_mode2(50.0, 0.01, 1.0);
        assert!(ki > 0.0);
    }

    #[test]
    fn test_analyze_fracture_not_critical_small_crack() {
        let mat = FractureMaterial::default();
        let r = analyze_fracture(10.0, 1e-6, &mat);
        assert!(!r.is_critical);
    }

    #[test]
    fn test_analyze_fracture_critical_large_crack() {
        let mat = FractureMaterial {
            toughness_k1c: 0.001,
            ..Default::default()
        };
        let r = analyze_fracture(200.0, 1.0, &mat);
        assert!(r.is_critical);
    }

    #[test]
    fn test_critical_crack_size_decreases_with_stress() {
        let mat = FractureMaterial::default();
        let a1 = critical_crack_size(100.0, &mat);
        let a2 = critical_crack_size(200.0, &mat);
        assert!(a1 > a2);
    }

    #[test]
    fn test_critical_stress_decreases_with_crack_size() {
        let mat = FractureMaterial::default();
        let s1 = critical_stress(0.001, &mat);
        let s2 = critical_stress(0.01, &mat);
        assert!(s1 > s2);
    }

    #[test]
    fn test_energy_release_rate_positive() {
        let mat = FractureMaterial::default();
        let g = energy_release_rate_mode1(1.0, &mat);
        assert!(g > 0.0);
    }

    #[test]
    fn test_will_propagate_false_for_small_ki() {
        let mat = FractureMaterial::default();
        assert!(!will_propagate(0.001, &mat));
    }

    #[test]
    fn test_safety_factor_greater_than_one_safe() {
        let mat = FractureMaterial::default();
        let r = analyze_fracture(1.0, 1e-8, &mat);
        assert!(r.safety_factor > 1.0);
    }
}

// ── Wave 151A simple f32 fracture API ──────────────────────────────────────

use std::f32::consts::PI as PI_F32;

/// Simple fracture model with f32 parameters.
#[derive(Debug, Clone)]
pub struct FractureModel {
    pub fracture_toughness: f32,
    pub geometry_factor: f32,
}

/// Create a new FractureModel.
pub fn new_fracture_model(k_ic: f32, f: f32) -> FractureModel {
    FractureModel {
        fracture_toughness: k_ic,
        geometry_factor: f,
    }
}

/// Critical crack length: a_c = (1/π) * (K_IC / (F * σ))².
pub fn critical_crack_length(m: &FractureModel, applied_stress: f32) -> f32 {
    let denom = m.geometry_factor * applied_stress;
    if denom.abs() < 1e-12 {
        return f32::INFINITY;
    }
    (1.0 / PI_F32) * (m.fracture_toughness / denom).powi(2)
}

/// Stress intensity factor: K = F * σ * sqrt(π * a).
pub fn stress_intensity(m: &FractureModel, stress: f32, crack_len: f32) -> f32 {
    m.geometry_factor * stress * (PI_F32 * crack_len).sqrt()
}

/// Returns true when the stress intensity exceeds fracture toughness.
pub fn will_fracture(m: &FractureModel, stress: f32, crack_len: f32) -> bool {
    stress_intensity(m, stress, crack_len) >= m.fracture_toughness
}
