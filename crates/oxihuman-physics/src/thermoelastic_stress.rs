// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Thermoelastic stress computation stub.

/// Thermoelastic material properties.
#[derive(Debug, Clone)]
pub struct ThermoelasticMaterial {
    pub young_modulus: f32,
    pub poisson: f32,
    pub cte: f32, /* coefficient of thermal expansion */
    pub density: f32,
    pub specific_heat: f32,
    pub conductivity: f32,
}

impl ThermoelasticMaterial {
    pub fn new(
        young_modulus: f32,
        poisson: f32,
        cte: f32,
        density: f32,
        specific_heat: f32,
        conductivity: f32,
    ) -> Self {
        ThermoelasticMaterial {
            young_modulus,
            poisson,
            cte,
            density,
            specific_heat,
            conductivity,
        }
    }

    pub fn steel() -> Self {
        ThermoelasticMaterial::new(200e9, 0.3, 12e-6, 7850.0, 500.0, 50.0)
    }

    /// Lame parameter lambda.
    pub fn lambda(&self) -> f32 {
        self.young_modulus * self.poisson / ((1.0 + self.poisson) * (1.0 - 2.0 * self.poisson))
    }

    /// Shear modulus G.
    pub fn shear_modulus(&self) -> f32 {
        self.young_modulus / (2.0 * (1.0 + self.poisson))
    }

    /// Bulk modulus K.
    pub fn bulk_modulus(&self) -> f32 {
        self.young_modulus / (3.0 * (1.0 - 2.0 * self.poisson))
    }

    /// Thermoelastic coupling coefficient beta = (3*lambda + 2*G)*CTE.
    pub fn beta(&self) -> f32 {
        (3.0 * self.lambda() + 2.0 * self.shear_modulus()) * self.cte
    }
}

impl Default for ThermoelasticMaterial {
    fn default() -> Self {
        Self::steel()
    }
}

/// Hydrostatic thermal stress: sigma_th = -beta * dT.
pub fn hydrostatic_thermal_stress(mat: &ThermoelasticMaterial, delta_temp: f32) -> f32 {
    -mat.beta() * delta_temp
}

/// Thermoelastic temperature change from a volumetric strain.
pub fn temp_from_volumetric_strain(mat: &ThermoelasticMaterial, vol_strain: f32) -> f32 {
    if mat.beta().abs() < 1e-20 {
        return 0.0;
    }
    -mat.bulk_modulus() * vol_strain / mat.beta()
}

/// Thermal diffusivity kappa = k / (rho * cp).
pub fn thermal_diffusivity(mat: &ThermoelasticMaterial) -> f32 {
    if mat.density * mat.specific_heat <= 0.0 {
        return 0.0;
    }
    mat.conductivity / (mat.density * mat.specific_heat)
}

/// Thermoelastic stress xx in plane strain.
pub fn plane_strain_stress_xx(
    mat: &ThermoelasticMaterial,
    eps_xx: f32,
    eps_yy: f32,
    delta_temp: f32,
) -> f32 {
    let l = mat.lambda();
    let g = mat.shear_modulus();
    let beta = mat.beta();
    (l + 2.0 * g) * eps_xx + l * eps_yy - beta * delta_temp
}

/// Estimate thermoelastic damping ratio (Zener model).
pub fn thermoelastic_damping(mat: &ThermoelasticMaterial, frequency: f32) -> f32 {
    let alpha = mat.cte;
    let e = mat.young_modulus;
    let t0 = 293.15f32;
    let kappa = thermal_diffusivity(mat);
    let tau = kappa.recip() * 1e-6; /* characteristic time, stub */
    let omega = 2.0 * std::f32::consts::PI * frequency.max(0.0);
    let num = alpha * alpha * e * t0 / (mat.density * mat.specific_heat);
    num * omega * tau / (1.0 + (omega * tau) * (omega * tau))
}

/// Inelastic heat fraction (Taylor-Quinney like — stub).
pub fn inelastic_heat_fraction(plastic_work: f32, beta_tq: f32) -> f32 {
    beta_tq.clamp(0.0, 1.0) * plastic_work.max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shear_modulus_steel() {
        let m = ThermoelasticMaterial::steel();
        let g = m.shear_modulus();
        assert!(g > 70e9 && g < 90e9); /* ~77 GPa for steel */
    }

    #[test]
    fn test_bulk_modulus_positive() {
        let m = ThermoelasticMaterial::steel();
        assert!(m.bulk_modulus() > 0.0);
    }

    #[test]
    fn test_beta_positive() {
        let m = ThermoelasticMaterial::steel();
        assert!(m.beta() > 0.0);
    }

    #[test]
    fn test_hydrostatic_thermal_stress_sign() {
        let m = ThermoelasticMaterial::steel();
        /* positive dT => compressive (negative) stress */
        assert!(hydrostatic_thermal_stress(&m, 100.0) < 0.0);
    }

    #[test]
    fn test_hydrostatic_stress_zero_dt() {
        let m = ThermoelasticMaterial::steel();
        assert_eq!(hydrostatic_thermal_stress(&m, 0.0), 0.0);
    }

    #[test]
    fn test_thermal_diffusivity_positive() {
        let m = ThermoelasticMaterial::steel();
        assert!(thermal_diffusivity(&m) > 0.0);
    }

    #[test]
    fn test_plane_strain_stress_zero() {
        let m = ThermoelasticMaterial::steel();
        /* zero strain, zero dT => stress depends on lambda coupling: should be 0 */
        let s = plane_strain_stress_xx(&m, 0.0, 0.0, 0.0);
        assert_eq!(s, 0.0);
    }

    #[test]
    fn test_thermoelastic_damping_positive() {
        let m = ThermoelasticMaterial::steel();
        let d = thermoelastic_damping(&m, 1000.0);
        assert!(d >= 0.0);
    }

    #[test]
    fn test_inelastic_heat_fraction() {
        let q = inelastic_heat_fraction(100.0, 0.9);
        assert!((q - 90.0).abs() < 0.001);
    }

    #[test]
    fn test_inelastic_heat_fraction_clamp() {
        let q = inelastic_heat_fraction(100.0, 1.5);
        assert!((q - 100.0).abs() < 0.001);
    }
}
