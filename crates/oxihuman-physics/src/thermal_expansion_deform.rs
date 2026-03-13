// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Thermal expansion deformation stub.

/// Material thermal expansion parameters.
#[derive(Debug, Clone)]
pub struct ThermalExpansionMaterial {
    pub cte: f32,           /* coefficient of thermal expansion [1/K] */
    pub ref_temp: f32,      /* reference temperature [K] */
    pub young_modulus: f32, /* Young's modulus [Pa] */
    pub poisson: f32,       /* Poisson's ratio */
}

impl ThermalExpansionMaterial {
    pub fn new(cte: f32, ref_temp: f32, young_modulus: f32, poisson: f32) -> Self {
        ThermalExpansionMaterial {
            cte,
            ref_temp,
            young_modulus,
            poisson,
        }
    }

    pub fn steel() -> Self {
        ThermalExpansionMaterial::new(12e-6, 293.15, 200e9, 0.3)
    }

    pub fn aluminum() -> Self {
        ThermalExpansionMaterial::new(23e-6, 293.15, 70e9, 0.33)
    }
}

impl Default for ThermalExpansionMaterial {
    fn default() -> Self {
        Self::steel()
    }
}

/// Linear strain due to temperature change.
pub fn thermal_strain(mat: &ThermalExpansionMaterial, temperature: f32) -> f32 {
    mat.cte * (temperature - mat.ref_temp)
}

/// Volumetric strain (for isotropic material).
pub fn volumetric_strain(mat: &ThermalExpansionMaterial, temperature: f32) -> f32 {
    3.0 * thermal_strain(mat, temperature)
}

/// Thermal stress in a fully constrained bar (sigma = -E * alpha * dT).
pub fn constrained_thermal_stress(mat: &ThermalExpansionMaterial, temperature: f32) -> f32 {
    -mat.young_modulus * thermal_strain(mat, temperature)
}

/// Deformation of a rod of length `l0` at given temperature.
pub fn thermal_elongation(mat: &ThermalExpansionMaterial, l0: f32, temperature: f32) -> f32 {
    l0 * thermal_strain(mat, temperature)
}

/// New length of rod after thermal expansion.
pub fn new_length(mat: &ThermalExpansionMaterial, l0: f32, temperature: f32) -> f32 {
    l0 + thermal_elongation(mat, l0, temperature)
}

/// Thermal expansion of a 3D body — returns strain tensor diagonal [eps, eps, eps].
pub fn isotropic_strain_tensor(mat: &ThermalExpansionMaterial, temperature: f32) -> [f32; 3] {
    let eps = thermal_strain(mat, temperature);
    [eps, eps, eps]
}

/// Temperature required to produce a given strain.
pub fn temp_for_strain(mat: &ThermalExpansionMaterial, target_strain: f32) -> f32 {
    if mat.cte.abs() < 1e-30 {
        mat.ref_temp
    } else {
        mat.ref_temp + target_strain / mat.cte
    }
}

/// Biaxial thermal stress (plane stress state).
pub fn biaxial_thermal_stress(mat: &ThermalExpansionMaterial, temperature: f32) -> f32 {
    -mat.young_modulus / (1.0 - mat.poisson) * mat.cte * (temperature - mat.ref_temp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thermal_strain_at_ref_temp() {
        let m = ThermalExpansionMaterial::steel();
        assert_eq!(thermal_strain(&m, m.ref_temp), 0.0);
    }

    #[test]
    fn test_thermal_strain_positive_above_ref() {
        let m = ThermalExpansionMaterial::steel();
        assert!(thermal_strain(&m, m.ref_temp + 100.0) > 0.0);
    }

    #[test]
    fn test_thermal_strain_negative_below_ref() {
        let m = ThermalExpansionMaterial::steel();
        assert!(thermal_strain(&m, m.ref_temp - 50.0) < 0.0);
    }

    #[test]
    fn test_constrained_stress_sign() {
        let m = ThermalExpansionMaterial::steel();
        /* heating a constrained bar => compressive (negative) stress */
        let sigma = constrained_thermal_stress(&m, m.ref_temp + 100.0);
        assert!(sigma < 0.0);
    }

    #[test]
    fn test_thermal_elongation_positive_heat() {
        let m = ThermalExpansionMaterial::steel();
        let dl = thermal_elongation(&m, 1.0, m.ref_temp + 100.0);
        assert!(dl > 0.0);
    }

    #[test]
    fn test_new_length_greater_when_heated() {
        let m = ThermalExpansionMaterial::steel();
        let l = new_length(&m, 1.0, m.ref_temp + 100.0);
        assert!(l > 1.0);
    }

    #[test]
    fn test_volumetric_strain_is_3x_linear() {
        let m = ThermalExpansionMaterial::steel();
        let lin = thermal_strain(&m, 400.0);
        let vol = volumetric_strain(&m, 400.0);
        assert!((vol - 3.0 * lin).abs() < 1e-20);
    }

    #[test]
    fn test_isotropic_strain_tensor_equal_components() {
        let m = ThermalExpansionMaterial::aluminum();
        let t = isotropic_strain_tensor(&m, 373.15);
        assert!((t[0] - t[1]).abs() < 1e-20 && (t[1] - t[2]).abs() < 1e-20);
    }

    #[test]
    fn test_temp_for_strain_roundtrip() {
        let m = ThermalExpansionMaterial::steel();
        let target_strain = 1e-3;
        let t = temp_for_strain(&m, target_strain);
        let computed = thermal_strain(&m, t);
        /* f32 precision: use relative tolerance */
        assert!((computed - target_strain).abs() < 1e-6);
    }

    #[test]
    fn test_biaxial_thermal_stress_sign() {
        let m = ThermalExpansionMaterial::steel();
        let sigma = biaxial_thermal_stress(&m, m.ref_temp + 100.0);
        assert!(sigma < 0.0);
    }
}
