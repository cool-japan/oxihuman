// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Polymer gel swelling and elastic response model.

/// Flory-Rehner parameters for a polymer gel.
#[derive(Debug, Clone)]
pub struct GelParams {
    /// Polymer-solvent interaction parameter (chi).
    pub chi: f32,
    /// Number of repeat units between crosslinks.
    pub n_crosslink: f32,
    /// Dry polymer volume fraction (reference state).
    pub phi0: f32,
    /// Shear modulus of the dry network (Pa).
    pub shear_modulus_dry: f32,
}

/// State of a swelling gel.
#[derive(Debug, Clone)]
pub struct GelState {
    pub params: GelParams,
    /// Current polymer volume fraction (phi < phi0 means swollen).
    pub phi: f32,
}

/// Construct default GelParams.
pub fn new_gel_params() -> GelParams {
    GelParams {
        chi: 0.5,
        n_crosslink: 100.0,
        phi0: 1.0,
        shear_modulus_dry: 1_000.0,
    }
}

/// Construct a GelState from params and initial polymer fraction.
pub fn new_gel_state(params: GelParams, phi: f32) -> GelState {
    let phi = phi.clamp(1e-6, 1.0);
    GelState { params, phi }
}

impl GelParams {
    /// Flory mixing free energy per unit volume (J/m³), simplified.
    /// f_mix = (RT/v1) * [phi*ln(phi) + (1-phi)*ln(1-phi) + chi*phi*(1-phi)]
    /// Returns dimensionless free-energy density (v1/RT normalised).
    pub fn mixing_free_energy(&self, phi: f32) -> f32 {
        let phi = phi.clamp(1e-6, 1.0 - 1e-6);
        phi * phi.ln() + (1.0 - phi) * (1.0 - phi).ln() + self.chi * phi * (1.0 - phi)
    }

    /// Osmotic pressure from mixing (dimensionless, v1/RT units).
    pub fn osmotic_pressure_mix(&self, phi: f32) -> f32 {
        let phi = phi.clamp(1e-6, 1.0 - 1e-6);
        -(phi.ln() + 1.0 - phi + self.chi * phi * phi)
    }

    /// Elastic contribution to osmotic pressure (Flory network elasticity).
    pub fn osmotic_pressure_elastic(&self, phi: f32) -> f32 {
        let phi = phi.clamp(1e-6, 1.0);
        let phi0 = self.phi0.clamp(1e-6, 1.0);
        (1.0 / self.n_crosslink) * (phi / phi0).powf(1.0 / 3.0)
    }

    /// Total osmotic pressure = mixing + elastic.
    pub fn osmotic_pressure_total(&self, phi: f32) -> f32 {
        self.osmotic_pressure_mix(phi) + self.osmotic_pressure_elastic(phi)
    }

    /// Equilibrium swelling ratio Q = V_swollen / V_dry (volume-based).
    /// Q = 1 / phi_eq where phi_eq is found by bisection.
    pub fn equilibrium_swelling_ratio(&self) -> f32 {
        let phi_eq = self.equilibrium_phi();
        1.0 / phi_eq
    }

    /// Find equilibrium phi by bisection (osmotic pressure = 0).
    pub fn equilibrium_phi(&self) -> f32 {
        let mut lo = 1e-4f32;
        let mut hi = 0.9999f32;
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if self.osmotic_pressure_total(mid) > 0.0 {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        0.5 * (lo + hi)
    }
}

impl GelState {
    /// Linear swelling ratio relative to dry state.
    pub fn linear_swell_ratio(&self) -> f32 {
        let q = 1.0 / self.phi;
        q.powf(1.0 / 3.0)
    }

    /// Elastic shear modulus in swollen state (Pa).
    /// G_swollen = G_dry * (phi / phi0)^(1/3)
    pub fn shear_modulus(&self) -> f32 {
        let phi0 = self.params.phi0.clamp(1e-6, 1.0);
        self.params.shear_modulus_dry * (self.phi / phi0).powf(1.0 / 3.0)
    }

    /// Young's modulus (assuming incompressible, nu=0.5): E = 3*G.
    pub fn youngs_modulus(&self) -> f32 {
        3.0 * self.shear_modulus()
    }

    /// Stress response to uniaxial stretch lambda (neo-Hookean network).
    /// sigma = G * (lambda - 1/lambda^2)
    pub fn uniaxial_stress(&self, lambda: f32) -> f32 {
        let lambda = lambda.max(1e-3);
        self.shear_modulus() * (lambda - 1.0 / (lambda * lambda))
    }

    /// Advance toward equilibrium by one explicit step (phi relaxation).
    pub fn relax_step(&mut self, dt: f32, mobility: f32) {
        let pi_total = self.params.osmotic_pressure_total(self.phi);
        let dphi = -mobility * pi_total * dt;
        self.phi = (self.phi + dphi).clamp(1e-6, 1.0);
    }

    /// Number of steps to approximately reach equilibrium.
    pub fn steps_to_equilibrium(&self, dt: f32, mobility: f32, tol: f32) -> usize {
        let mut state = self.clone();
        for i in 0..100_000 {
            let pi = state.params.osmotic_pressure_total(state.phi);
            if pi.abs() < tol {
                return i;
            }
            state.relax_step(dt, mobility);
        }
        100_000
    }
}

/// Compute the Flory-Huggins chi parameter from solubility parameters.
/// chi = V_r / (RT) * (delta1 - delta2)^2  (V_r = molar volume of solvent, simplified to 1).
pub fn chi_from_solubility(delta1: f32, delta2: f32) -> f32 {
    (delta1 - delta2) * (delta1 - delta2)
}

/// Estimate dry shear modulus from crosslink density (mol/m³) and temperature.
pub fn shear_modulus_from_crosslink_density(rho_x: f32, temp_k: f32) -> f32 {
    const R: f32 = 8.314;
    rho_x * R * temp_k
}

/// Compute degree of swelling DS = (W_swollen - W_dry) / W_dry = Q - 1.
pub fn degree_of_swelling(phi_eq: f32) -> f32 {
    1.0 / phi_eq - 1.0
}

/// Presets
pub fn hydrogel_params() -> GelParams {
    GelParams {
        chi: 0.45,
        n_crosslink: 200.0,
        phi0: 1.0,
        shear_modulus_dry: 500.0,
    }
}

pub fn pnipam_gel_params() -> GelParams {
    /* PNIPAM above LCST (collapsed, chi > 0.5) */
    GelParams {
        chi: 0.8,
        n_crosslink: 50.0,
        phi0: 1.0,
        shear_modulus_dry: 5_000.0,
    }
}

/// Approximate Flory exponent for a swollen gel in good solvent (nu ~ 3/5).
pub const FLORY_EXPONENT_GOOD: f32 = 0.6;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_gel_params() {
        /* new_gel_params returns valid defaults */
        let p = new_gel_params();
        assert!(p.chi > 0.0);
        assert!(p.n_crosslink > 0.0);
    }

    #[test]
    fn test_new_gel_state_clamps() {
        /* new_gel_state clamps phi to [1e-6, 1.0] */
        let p = new_gel_params();
        let s = new_gel_state(p, -5.0);
        assert!(s.phi >= 1e-6);
    }

    #[test]
    fn test_mixing_free_energy_finite() {
        /* mixing_free_energy returns a finite value */
        let p = new_gel_params();
        let fe = p.mixing_free_energy(0.3);
        assert!(fe.is_finite());
    }

    #[test]
    fn test_osmotic_pressure_mix_sign() {
        /* osmotic_pressure_mix is positive for low phi (swelling drive) */
        let p = new_gel_params();
        let pi = p.osmotic_pressure_mix(0.01);
        assert!(pi > 0.0);
    }

    #[test]
    fn test_equilibrium_phi_in_range() {
        /* equilibrium_phi returns a value in (0, 1) */
        let p = hydrogel_params();
        let phi_eq = p.equilibrium_phi();
        assert!((0.0..1.0).contains(&phi_eq));
    }

    #[test]
    fn test_shear_modulus_decreases_with_swelling() {
        /* shear modulus in swollen state is less than dry modulus */
        let p = new_gel_params();
        let s = new_gel_state(p, 0.2); /* swollen */
        assert!(s.shear_modulus() < s.params.shear_modulus_dry);
    }

    #[test]
    fn test_uniaxial_stress_zero_at_one() {
        /* uniaxial stress is zero at lambda=1 (no deformation) */
        let p = new_gel_params();
        let s = new_gel_state(p, 0.5);
        let sigma = s.uniaxial_stress(1.0);
        assert!(sigma.abs() < 1e-3);
    }

    #[test]
    fn test_relax_step_moves_toward_equilibrium() {
        /* relax_step changes phi when starting far from equilibrium */
        let p = hydrogel_params();
        let mut s = new_gel_state(p, 0.9); /* compressed state, will swell */
        let phi_before = s.phi;
        s.relax_step(0.01, 10.0);
        assert!((s.phi - phi_before).abs() > 1e-10);
    }

    #[test]
    fn test_degree_of_swelling_positive() {
        /* degree of swelling is positive for phi_eq < 1 */
        let ds = degree_of_swelling(0.2);
        assert!(ds > 0.0);
    }

    #[test]
    fn test_chi_from_solubility() {
        /* chi_from_solubility is zero for identical parameters */
        let chi = chi_from_solubility(20.0, 20.0);
        assert!(chi.abs() < 1e-6);
    }
}
