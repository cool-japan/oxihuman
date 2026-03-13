// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Synovial joint fluid — Carreau shear-thinning model.
pub struct SynovialFluid {
    pub zero_shear_viscosity: f32,
    pub infinite_shear_viscosity: f32,
    pub relaxation_time: f32,
}

impl SynovialFluid {
    pub fn new() -> Self {
        SynovialFluid {
            zero_shear_viscosity: 0.5,       // Pa·s (resting)
            infinite_shear_viscosity: 0.001, // Pa·s (high shear)
            relaxation_time: 0.1,            // s
        }
    }
}

impl Default for SynovialFluid {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_synovial_fluid() -> SynovialFluid {
    SynovialFluid::new()
}

/// Carreau model: η = η_inf + (η_0 - η_inf) * (1 + (λγ)²)^((n-1)/2), n=0.5
pub fn synovial_viscosity(f: &SynovialFluid, shear_rate: f32) -> f32 {
    let n = 0.5_f32;
    let lambda_gamma = f.relaxation_time * shear_rate;
    let exponent = (n - 1.0) / 2.0;
    let factor = (1.0 + lambda_gamma * lambda_gamma).powf(exponent);
    f.infinite_shear_viscosity + (f.zero_shear_viscosity - f.infinite_shear_viscosity) * factor
}

pub fn synovial_shear_stress(f: &SynovialFluid, shear_rate: f32) -> f32 {
    synovial_viscosity(f, shear_rate) * shear_rate
}

/// True if viscosity at rate2 < viscosity at rate1 (shear thinning).
pub fn synovial_is_shear_thinning(f: &SynovialFluid, rate1: f32, rate2: f32) -> bool {
    synovial_viscosity(f, rate2) < synovial_viscosity(f, rate1)
}

/// Lubrication number: η * speed / load
pub fn synovial_lubrication_number(f: &SynovialFluid, speed: f32, load: f32) -> f32 {
    if load.abs() < 1e-12 {
        return f32::INFINITY;
    }
    synovial_viscosity(f, speed) * speed / load
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* new fluid has expected viscosities */
        let f = new_synovial_fluid();
        assert!(f.zero_shear_viscosity > f.infinite_shear_viscosity);
    }

    #[test]
    fn test_viscosity_at_zero_shear() {
        /* at zero shear rate, viscosity ≈ zero_shear_viscosity */
        let f = new_synovial_fluid();
        let v = synovial_viscosity(&f, 0.0);
        assert!((v - f.zero_shear_viscosity).abs() < 1e-4);
    }

    #[test]
    fn test_viscosity_decreases_with_rate() {
        /* higher shear rate gives lower viscosity */
        let f = new_synovial_fluid();
        let v_low = synovial_viscosity(&f, 1.0);
        let v_high = synovial_viscosity(&f, 1000.0);
        assert!(v_high < v_low);
    }

    #[test]
    fn test_shear_thinning() {
        /* fluid is shear thinning for increasing rates */
        let f = new_synovial_fluid();
        assert!(synovial_is_shear_thinning(&f, 1.0, 100.0));
    }

    #[test]
    fn test_shear_stress_positive() {
        /* shear stress is positive for positive shear rate */
        let f = new_synovial_fluid();
        assert!(synovial_shear_stress(&f, 10.0) > 0.0);
    }

    #[test]
    fn test_lubrication_number() {
        /* lubrication number is positive */
        let f = new_synovial_fluid();
        let ln = synovial_lubrication_number(&f, 0.1, 10.0);
        assert!(ln > 0.0);
    }

    #[test]
    fn test_default() {
        /* Default impl works */
        let f = SynovialFluid::default();
        assert!(f.zero_shear_viscosity > 0.0);
    }
}
