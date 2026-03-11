// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Open/closed cell foam compression model (Gibson-Ashby).

/// Foam cell type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FoamCellType {
    Open,
    Closed,
}

/// Gibson-Ashby foam parameters.
#[derive(Debug, Clone)]
pub struct FoamParams {
    /// Solid material Young's modulus (Pa).
    pub e_solid: f32,
    /// Solid material density (kg/m³).
    pub rho_solid: f32,
    /// Foam relative density (ρ_foam / ρ_solid), 0..1.
    pub relative_density: f32,
    /// Cell type.
    pub cell_type: FoamCellType,
}

impl FoamParams {
    /// Foam Young's modulus (Gibson-Ashby relation).
    pub fn foam_modulus(&self) -> f32 {
        let rd = self.relative_density;
        match self.cell_type {
            FoamCellType::Open => self.e_solid * rd * rd,
            FoamCellType::Closed => self.e_solid * (0.5 * rd * rd + 0.3 * rd),
        }
    }

    /// Foam density (kg/m³).
    pub fn foam_density(&self) -> f32 {
        self.relative_density * self.rho_solid
    }

    /// Elastic collapse stress (plateau stress).
    pub fn plateau_stress(&self) -> f32 {
        let rd = self.relative_density;
        match self.cell_type {
            FoamCellType::Open => 0.3 * self.e_solid * rd.powf(1.5),
            FoamCellType::Closed => 0.3 * self.e_solid * (0.5 * rd * rd + 0.3 * rd),
        }
    }

    /// Densification strain (approx: 1 - 1.4 * relative_density).
    pub fn densification_strain(&self) -> f32 {
        (1.0 - 1.4 * self.relative_density).max(0.0)
    }

    /// Energy absorbed per unit volume up to densification strain.
    pub fn energy_absorbed(&self) -> f32 {
        self.plateau_stress() * self.densification_strain()
    }
}

/// Construct foam parameters.
pub fn new_foam_params(
    e_solid: f32,
    rho_solid: f32,
    rel_density: f32,
    cell_type: FoamCellType,
) -> FoamParams {
    FoamParams {
        e_solid,
        rho_solid,
        relative_density: rel_density.clamp(1e-4, 1.0),
        cell_type,
    }
}

/// Common foam material: open-cell polyurethane (approx).
pub fn polyurethane_foam() -> FoamParams {
    new_foam_params(3e9, 1200.0, 0.05, FoamCellType::Open)
}

/// Common foam material: closed-cell polystyrene (approx).
pub fn polystyrene_foam() -> FoamParams {
    new_foam_params(3.4e9, 1050.0, 0.04, FoamCellType::Closed)
}

/// Foam compression state.
pub struct FoamCompression {
    pub params: FoamParams,
    pub strain: f32,
}

/// Construct a new FoamCompression.
pub fn new_foam_compression(params: FoamParams) -> FoamCompression {
    FoamCompression {
        params,
        strain: 0.0,
    }
}

impl FoamCompression {
    /// Apply compressive strain (0..1).
    pub fn compress(&mut self, strain: f32) {
        self.strain = strain.clamp(0.0, 1.0);
    }

    /// Current compressive stress.
    pub fn stress(&self) -> f32 {
        let eps = self.strain;
        let eps_d = self.params.densification_strain();
        let sigma_pl = self.params.plateau_stress();
        let e_f = self.params.foam_modulus();
        if eps < 0.05 {
            /* linear elastic region */
            e_f * eps
        } else if eps < eps_d {
            /* plateau region */
            sigma_pl
        } else {
            /* densification: exponential rise */
            sigma_pl * (1.0 + (eps - eps_d).powi(2) * 100.0)
        }
    }

    /// Is the foam in the densification regime?
    pub fn is_densified(&self) -> bool {
        self.strain >= self.params.densification_strain()
    }

    /// Specific energy absorption (J/kg) up to current strain.
    pub fn specific_energy(&self) -> f32 {
        let density = self.params.foam_density();
        if density < 1e-6 {
            return 0.0;
        }
        (self.stress() * self.strain) / density
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_cell_modulus() {
        /* open cell foam modulus = E_s * rd^2 */
        let p = new_foam_params(1e6, 1000.0, 0.1, FoamCellType::Open);
        let expected = 1e6 * 0.01;
        assert!(
            (p.foam_modulus() - expected).abs() < 1.0,
            "E={}",
            p.foam_modulus()
        );
    }

    #[test]
    fn test_plateau_stress_positive() {
        /* plateau stress is positive */
        let p = polyurethane_foam();
        assert!(p.plateau_stress() > 0.0);
    }

    #[test]
    fn test_densification_strain() {
        /* densification strain is in (0, 1) */
        let p = polystyrene_foam();
        let eps = p.densification_strain();
        assert!((0.0..=1.0).contains(&eps), "eps={eps}");
    }

    #[test]
    fn test_energy_absorbed_positive() {
        /* energy absorbed is positive */
        let p = polyurethane_foam();
        assert!(p.energy_absorbed() > 0.0);
    }

    #[test]
    fn test_foam_density() {
        /* foam density < solid density */
        let p = polyurethane_foam();
        assert!(p.foam_density() < p.rho_solid);
    }

    #[test]
    fn test_compression_linear_region() {
        /* stress in linear region = E * strain */
        let p = new_foam_params(1e6, 1000.0, 0.1, FoamCellType::Open);
        let e_f = p.foam_modulus();
        let mut fc = new_foam_compression(p);
        fc.compress(0.02);
        let expected = e_f * 0.02;
        assert!(
            (fc.stress() - expected).abs() < 1.0,
            "sigma={}",
            fc.stress()
        );
    }

    #[test]
    fn test_densification_flag() {
        /* is_densified returns true beyond densification strain */
        let p = polystyrene_foam();
        let eps_d = p.densification_strain();
        let mut fc = new_foam_compression(p);
        fc.compress(eps_d + 0.01);
        assert!(fc.is_densified());
    }

    #[test]
    fn test_specific_energy_positive() {
        /* specific energy is positive under compression */
        let mut fc = new_foam_compression(polyurethane_foam());
        fc.compress(0.3);
        assert!(fc.specific_energy() > 0.0);
    }

    #[test]
    fn test_closed_cell_higher_modulus() {
        /* closed cell has higher modulus than open cell at same rd */
        let open = new_foam_params(1e9, 1000.0, 0.2, FoamCellType::Open);
        let closed = new_foam_params(1e9, 1000.0, 0.2, FoamCellType::Closed);
        assert!(
            closed.foam_modulus() > open.foam_modulus(),
            "open={} closed={}",
            open.foam_modulus(),
            closed.foam_modulus()
        );
    }
}
