// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Thermal strain and stress due to temperature change.

/// A material with thermal expansion properties.
#[derive(Debug, Clone)]
pub struct ThermalMaterial {
    /// Young's modulus (Pa).
    pub young_modulus: f32,
    /// Poisson's ratio.
    pub poisson_ratio: f32,
    /// Linear coefficient of thermal expansion (1/K).
    pub alpha: f32,
    /// Reference temperature (K).
    pub t_ref: f32,
}

/// Construct a new ThermalMaterial.
pub fn new_thermal_material(e: f32, nu: f32, alpha: f32, t_ref: f32) -> ThermalMaterial {
    ThermalMaterial {
        young_modulus: e,
        poisson_ratio: nu,
        alpha,
        t_ref,
    }
}

/// Steel material (approx).
pub fn steel_thermal_material() -> ThermalMaterial {
    new_thermal_material(200e9, 0.3, 12e-6, 293.15)
}

/// Aluminum material (approx).
pub fn aluminum_thermal_material() -> ThermalMaterial {
    new_thermal_material(70e9, 0.33, 23e-6, 293.15)
}

impl ThermalMaterial {
    /// Linear thermal strain: ε_th = α (T - T_ref).
    pub fn thermal_strain(&self, temperature: f32) -> f32 {
        self.alpha * (temperature - self.t_ref)
    }

    /// Thermal stress for constrained member: σ = -E α ΔT.
    pub fn thermal_stress(&self, temperature: f32) -> f32 {
        -self.young_modulus * self.thermal_strain(temperature)
    }

    /// Thermal elongation for a member of length L.
    pub fn thermal_elongation(&self, length: f32, temperature: f32) -> f32 {
        length * self.thermal_strain(temperature)
    }

    /// Volumetric thermal strain: ε_v = 3 α ΔT (isotropic).
    pub fn volumetric_strain(&self, temperature: f32) -> f32 {
        3.0 * self.thermal_strain(temperature)
    }

    /// Biaxial thermal stress (plane stress): σ = -E α ΔT / (1 - ν).
    pub fn biaxial_thermal_stress(&self, temperature: f32) -> f32 {
        -self.young_modulus * self.thermal_strain(temperature) / (1.0 - self.poisson_ratio + 1e-15)
    }

    /// Shear modulus G = E / (2(1+ν)).
    pub fn shear_modulus(&self) -> f32 {
        self.young_modulus / (2.0 * (1.0 + self.poisson_ratio))
    }
}

/// 1D thermal expansion bar simulation.
pub struct ThermalBar {
    pub material: ThermalMaterial,
    pub length: f32,
    pub temperature: f32,
}

/// Construct a new ThermalBar.
pub fn new_thermal_bar(material: ThermalMaterial, length: f32) -> ThermalBar {
    let t_ref = material.t_ref;
    ThermalBar {
        material,
        length,
        temperature: t_ref,
    }
}

impl ThermalBar {
    /// Set the current temperature.
    pub fn set_temperature(&mut self, t: f32) {
        self.temperature = t;
    }

    /// Current length under thermal expansion.
    pub fn current_length(&self) -> f32 {
        self.length
            + self
                .material
                .thermal_elongation(self.length, self.temperature)
    }

    /// Thermal stress if bar is fully constrained.
    pub fn constrained_stress(&self) -> f32 {
        self.material.thermal_stress(self.temperature)
    }

    /// Change in length from reference.
    pub fn delta_length(&self) -> f32 {
        self.current_length() - self.length
    }
}

/// Compute thermal stress for a 3D isotropic body (hydrostatic component).
pub fn hydrostatic_thermal_stress(mat: &ThermalMaterial, temperature: f32) -> f32 {
    let bulk = mat.young_modulus / (3.0 * (1.0 - 2.0 * mat.poisson_ratio + 1e-15));
    -3.0 * bulk * mat.alpha * (temperature - mat.t_ref)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_strain_at_ref_temp() {
        /* thermal strain is zero at reference temperature */
        let m = steel_thermal_material();
        assert!(m.thermal_strain(m.t_ref).abs() < 1e-12);
    }

    #[test]
    fn test_positive_strain_above_ref() {
        /* strain is positive above reference temperature */
        let m = steel_thermal_material();
        assert!(m.thermal_strain(m.t_ref + 100.0) > 0.0);
    }

    #[test]
    fn test_thermal_stress_sign() {
        /* constrained material heated => compressive stress (negative) */
        let m = steel_thermal_material();
        assert!(m.thermal_stress(m.t_ref + 100.0) < 0.0);
    }

    #[test]
    fn test_thermal_elongation() {
        /* elongation = L * alpha * dT for steel 1m heated 100K ~ 1.2mm */
        let m = steel_thermal_material();
        let el = m.thermal_elongation(1.0, m.t_ref + 100.0);
        assert!((el - 0.0012).abs() < 1e-4, "el={el}");
    }

    #[test]
    fn test_volumetric_strain() {
        /* volumetric strain = 3 * linear strain */
        let m = aluminum_thermal_material();
        let linear = m.thermal_strain(m.t_ref + 50.0);
        let vol = m.volumetric_strain(m.t_ref + 50.0);
        assert!((vol - 3.0 * linear).abs() < 1e-15);
    }

    #[test]
    fn test_thermal_bar_expands() {
        /* thermal bar gets longer when heated */
        let m = steel_thermal_material();
        let mut bar = new_thermal_bar(m, 1.0);
        let l0 = bar.current_length();
        bar.set_temperature(bar.material.t_ref + 200.0);
        assert!(bar.current_length() > l0);
    }

    #[test]
    fn test_constrained_stress_negative() {
        /* constrained heated bar has compressive stress */
        let m = steel_thermal_material();
        let mut bar = new_thermal_bar(m, 1.0);
        bar.set_temperature(bar.material.t_ref + 50.0);
        assert!(bar.constrained_stress() < 0.0);
    }

    #[test]
    fn test_shear_modulus() {
        /* shear modulus of steel ~77 GPa */
        let m = steel_thermal_material();
        let g = m.shear_modulus();
        assert!(g > 70e9 && g < 85e9, "g={g}");
    }

    #[test]
    fn test_hydrostatic_thermal_stress() {
        /* hydrostatic thermal stress is negative when heated */
        let m = steel_thermal_material();
        let s = hydrostatic_thermal_stress(&m, m.t_ref + 100.0);
        assert!(s < 0.0, "s={s}");
    }
}
