// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Darcy flow in porous media stub.
//!
//! Models fluid flow through a porous medium using Darcy's law:
//! q = -K/μ * ∇p, where K is permeability, μ is dynamic viscosity, ∇p is pressure gradient.

/// Properties of a porous medium.
#[derive(Debug, Clone)]
pub struct PorousParams {
    /// Intrinsic permeability `[m²]`.
    pub permeability: f64,
    /// Porosity φ ∈ (0, 1).
    pub porosity: f64,
    /// Fluid dynamic viscosity [Pa·s].
    pub viscosity: f64,
    /// Fluid density [kg/m³].
    pub fluid_density: f64,
}

impl Default for PorousParams {
    fn default() -> Self {
        Self {
            permeability: 1e-12,
            porosity: 0.3,
            viscosity: 1e-3,
            fluid_density: 1000.0,
        }
    }
}

/// State of a porous medium cell.
#[derive(Debug, Clone, Default)]
pub struct PorousCell {
    pub pressure: f64,
    pub saturation: f64,
    pub velocity: [f64; 3],
}

impl PorousCell {
    pub fn new(p: f64, sat: f64) -> Self {
        Self {
            pressure: p,
            saturation: sat.clamp(0.0, 1.0),
            velocity: [0.0; 3],
        }
    }
}

/// Compute the Darcy flux q [m/s] given a 1D pressure gradient.
pub fn darcy_flux(dp_dx: f64, params: &PorousParams) -> f64 {
    -(params.permeability / params.viscosity) * dp_dx
}

/// Compute the Darcy flux vector in 3D.
pub fn darcy_flux_3d(grad_p: &[f64; 3], params: &PorousParams) -> [f64; 3] {
    let k_over_mu = params.permeability / params.viscosity;
    [
        -k_over_mu * grad_p[0],
        -k_over_mu * grad_p[1],
        -k_over_mu * grad_p[2],
    ]
}

/// Compute the seepage velocity (average interstitial velocity).
pub fn seepage_velocity(darcy_q: f64, porosity: f64) -> f64 {
    if porosity <= 0.0 {
        return 0.0;
    }
    darcy_q / porosity
}

/// Compute the Reynolds number for porous flow (pore-scale).
pub fn porous_reynolds(darcy_q: f64, grain_diameter: f64, params: &PorousParams) -> f64 {
    params.fluid_density * darcy_q * grain_diameter / params.viscosity
}

/// Compute the Kozeny-Carman permeability from porosity and grain size.
pub fn kozeny_carman_permeability(porosity: f64, grain_diameter: f64) -> f64 {
    let phi3 = porosity.powi(3);
    let omp2 = (1.0 - porosity).powi(2);
    grain_diameter * grain_diameter * phi3 / (180.0 * omp2)
}

/// Compute compressibility-driven storage coefficient.
pub fn storage_coefficient(
    porosity: f64,
    fluid_compressibility: f64,
    solid_compressibility: f64,
) -> f64 {
    porosity * fluid_compressibility + (1.0 - porosity) * solid_compressibility
}

/// Advance pressure in a 1D porous column (explicit finite difference).
pub fn advance_pressure_1d(cells: &mut [PorousCell], dx: f64, dt: f64, params: &PorousParams) {
    if cells.len() < 2 {
        return;
    }
    let n = cells.len();
    let mut new_pressure: Vec<f64> = cells.iter().map(|c| c.pressure).collect();
    for i in 1..n.saturating_sub(1) {
        let grad_right = (cells[i + 1].pressure - cells[i].pressure) / dx;
        let grad_left = (cells[i].pressure - cells[i - 1].pressure) / dx;
        let flux_right = darcy_flux(grad_right, params);
        let flux_left = darcy_flux(grad_left, params);
        new_pressure[i] += dt * (flux_right - flux_left) / (dx * params.porosity);
    }
    for (c, p) in cells.iter_mut().zip(new_pressure.iter()) {
        c.pressure = *p;
    }
}

/// Check if flow is in the Darcy regime (Re < 1).
pub fn is_darcy_regime(re: f64) -> bool {
    re < 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params() -> PorousParams {
        PorousParams::default()
    }

    #[test]
    fn test_darcy_flux_direction() {
        /* Positive gradient → negative flux (flow from high to low pressure) */
        let q = darcy_flux(1000.0, &default_params());
        assert!(q < 0.0);
    }

    #[test]
    fn test_darcy_flux_zero_at_zero_gradient() {
        let q = darcy_flux(0.0, &default_params());
        assert_eq!(q, 0.0);
    }

    #[test]
    fn test_darcy_flux_3d_proportional() {
        let p = default_params();
        let g = [1000.0, 0.0, 0.0];
        let q = darcy_flux_3d(&g, &p);
        assert!(q[0] < 0.0);
        assert_eq!(q[1], 0.0);
    }

    #[test]
    fn test_seepage_velocity_gt_darcy() {
        let q = 1e-5;
        let v = seepage_velocity(q, 0.3);
        assert!(v > q);
    }

    #[test]
    fn test_kozeny_carman_positive() {
        let k = kozeny_carman_permeability(0.3, 1e-3);
        assert!(k > 0.0);
    }

    #[test]
    fn test_kozeny_carman_increases_with_porosity() {
        let k1 = kozeny_carman_permeability(0.2, 1e-3);
        let k2 = kozeny_carman_permeability(0.4, 1e-3);
        assert!(k2 > k1);
    }

    #[test]
    fn test_reynolds_positive() {
        let re = porous_reynolds(1e-5, 1e-3, &default_params());
        assert!(re > 0.0);
    }

    #[test]
    fn test_is_darcy_regime_true() {
        assert!(is_darcy_regime(0.5));
        assert!(!is_darcy_regime(2.0));
    }

    #[test]
    fn test_advance_pressure_no_panic() {
        let p = default_params();
        let mut cells = vec![
            PorousCell::new(100.0, 1.0),
            PorousCell::new(50.0, 1.0),
            PorousCell::new(0.0, 1.0),
        ];
        advance_pressure_1d(&mut cells, 0.1, 0.001, &p);
    }
}
