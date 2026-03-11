// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Reynolds lubrication theory for thin-film forces.

/// Fluid properties for lubrication.
#[derive(Debug, Clone)]
pub struct LubricantFluid {
    /// Dynamic viscosity (Pa·s).
    pub viscosity: f32,
}

/// Construct a lubrication fluid (water default: ~0.001 Pa·s).
pub fn new_lubricant_fluid(viscosity: f32) -> LubricantFluid {
    LubricantFluid {
        viscosity: viscosity.max(1e-12),
    }
}

impl Default for LubricantFluid {
    fn default() -> Self {
        LubricantFluid { viscosity: 0.001 }
    }
}

/// Squeeze-film lubrication force between two parallel circular discs.
/// Formula: F = (3π η R^4) / (2 h^3) * (dh/dt)
/// where h is gap thickness and dh/dt is approach rate.
pub fn squeeze_film_force(
    fluid: &LubricantFluid,
    radius: f32,
    gap: f32,
    approach_rate: f32,
) -> f32 {
    use std::f32::consts::PI;
    let h = gap.max(1e-9);
    (3.0 * PI * fluid.viscosity * radius.powi(4)) / (2.0 * h.powi(3)) * approach_rate
}

/// Couette flow shear stress: τ = η * (dv/dy) = η * v / h.
pub fn couette_shear_stress(fluid: &LubricantFluid, velocity: f32, gap: f32) -> f32 {
    fluid.viscosity * velocity / gap.max(1e-9)
}

/// Hydrodynamic load capacity for a slider bearing (1D Reynolds equation, simplified).
/// L = (6 η U L²) / h² * (h2/h1 - 1) where h1, h2 are entry/exit gap heights.
pub fn slider_bearing_load(
    fluid: &LubricantFluid,
    velocity: f32,
    length: f32,
    h1: f32,
    h2: f32,
) -> f32 {
    let h1 = h1.max(1e-9);
    let h2 = h2.max(1e-9);
    let ratio = h2 / h1;
    if (ratio - 1.0).abs() < 1e-6 {
        return 0.0;
    }
    6.0 * fluid.viscosity * velocity * length * length / (h1 * h1) * (ratio - 1.0)
        / (ratio * ratio - 1.0)
}

/// Sommerfeld number: dimensionless load parameter for journal bearings.
pub fn sommerfeld_number(
    fluid: &LubricantFluid,
    omega: f32,
    r: f32,
    l: f32,
    c: f32,
    w: f32,
) -> f32 {
    if w.abs() < 1e-15 {
        return f32::INFINITY;
    }
    fluid.viscosity * omega * r * l * (r / c).powi(2) / w
}

/// Film thickness from elastohydrodynamic (EHD) line contact (Dowson-Higginson).
/// h_min = 2.65 (u η0)^0.7 (E')^(-0.2) (R')^0.43 (W')^(-0.13)
/// Returns a dimensionless film thickness approximation.
pub fn ehd_min_film_thickness(u_eta: f32, e_prime: f32, r_prime: f32, w_prime: f32) -> f32 {
    2.65 * u_eta.powf(0.7) * e_prime.powf(-0.2) * r_prime.powf(0.43) * w_prime.powf(-0.13)
}

/// Lubrication film state.
pub struct LubricationFilm {
    pub fluid: LubricantFluid,
    pub gap: f32,
    pub velocity: f32,
}

/// Construct a lubrication film model.
pub fn new_lubrication_film(fluid: LubricantFluid, gap: f32, velocity: f32) -> LubricationFilm {
    LubricationFilm {
        fluid,
        gap: gap.max(1e-12),
        velocity,
    }
}

impl LubricationFilm {
    /// Shear stress in the film.
    pub fn shear_stress(&self) -> f32 {
        couette_shear_stress(&self.fluid, self.velocity, self.gap)
    }

    /// Film Reynolds number: Re = ρ v h / η.
    pub fn reynolds_number(&self, density: f32) -> f32 {
        density * self.velocity * self.gap / self.fluid.viscosity
    }

    /// Check if flow is laminar (Re < 1000).
    pub fn is_laminar(&self, density: f32) -> bool {
        self.reynolds_number(density) < 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_lubricant() {
        /* new lubricant fluid has correct viscosity */
        let f = new_lubricant_fluid(0.01);
        assert!((f.viscosity - 0.01).abs() < 1e-8);
    }

    #[test]
    fn test_squeeze_film_force_positive() {
        /* positive approach rate gives positive squeeze force */
        let f = LubricantFluid::default();
        let force = squeeze_film_force(&f, 0.01, 1e-4, 1.0);
        assert!(force > 0.0, "force={force}");
    }

    #[test]
    fn test_couette_shear_stress() {
        /* shear stress = viscosity * velocity / gap */
        let f = new_lubricant_fluid(0.001);
        let tau = couette_shear_stress(&f, 1.0, 0.001);
        assert!((tau - 1.0).abs() < 1e-6, "tau={tau}");
    }

    #[test]
    fn test_slider_bearing_same_gap_zero_load() {
        /* equal entry/exit gap gives near-zero load capacity */
        let f = LubricantFluid::default();
        let load = slider_bearing_load(&f, 1.0, 0.01, 0.001, 0.001);
        assert!(load.abs() < 1e-3, "load={load}");
    }

    #[test]
    fn test_sommerfeld_number_positive() {
        /* Sommerfeld number is positive for valid inputs */
        let f = new_lubricant_fluid(0.01);
        let s = sommerfeld_number(&f, 100.0, 0.05, 0.05, 0.0001, 1000.0);
        assert!(s > 0.0, "s={s}");
    }

    #[test]
    fn test_lubrication_film_shear() {
        /* film shear stress matches couette formula */
        let fluid = new_lubricant_fluid(0.001);
        let film = new_lubrication_film(fluid.clone(), 0.001, 1.0);
        assert!((film.shear_stress() - couette_shear_stress(&fluid, 1.0, 0.001)).abs() < 1e-8);
    }

    #[test]
    fn test_reynolds_number() {
        /* Reynolds number positive for moving film */
        let film = new_lubrication_film(LubricantFluid::default(), 1e-3, 1.0);
        assert!(film.reynolds_number(1000.0) > 0.0);
    }

    #[test]
    fn test_is_laminar_slow_flow() {
        /* slow flow is laminar */
        let film = new_lubrication_film(new_lubricant_fluid(1.0), 1e-3, 0.001);
        assert!(film.is_laminar(1000.0));
    }
}
