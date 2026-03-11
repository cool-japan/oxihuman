// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Fiber composite material micro-mechanics (rule of mixtures and beyond).

/// A fiber composite layer definition.
#[derive(Debug, Clone)]
pub struct FiberComposite {
    /// Fiber volume fraction `[0, 1]`.
    pub vf: f32,
    /// Fiber Young's modulus `[GPa]`.
    pub ef: f32,
    /// Matrix Young's modulus `[GPa]`.
    pub em: f32,
    /// Fiber Poisson's ratio.
    pub nuf: f32,
    /// Matrix Poisson's ratio.
    pub num: f32,
}

/// Create a new FiberComposite.
pub fn new_fiber_composite(vf: f32, ef: f32, em: f32, nuf: f32, num: f32) -> FiberComposite {
    FiberComposite {
        vf: vf.clamp(0.0, 1.0),
        ef,
        em,
        nuf,
        num,
    }
}

/// Longitudinal modulus E1 by rule of mixtures.
pub fn composite_e1(c: &FiberComposite) -> f32 {
    c.vf * c.ef + (1.0 - c.vf) * c.em
}

/// Transverse modulus E2 by inverse rule of mixtures.
pub fn composite_e2(c: &FiberComposite) -> f32 {
    let vm = 1.0 - c.vf;
    if c.vf * c.em + vm * c.ef < 1e-12 {
        return 0.0;
    }
    c.ef * c.em / (c.vf * c.em + vm * c.ef)
}

/// Longitudinal Poisson's ratio nu12 = vf*nu_f + vm*nu_m.
pub fn composite_nu12(c: &FiberComposite) -> f32 {
    c.vf * c.nuf + (1.0 - c.vf) * c.num
}

/// In-plane shear modulus G12 (inverse rule of mixtures).
/// Assumes G = E / (2*(1+nu)).
pub fn composite_g12(c: &FiberComposite) -> f32 {
    let gf = c.ef / (2.0 * (1.0 + c.nuf));
    let gm = c.em / (2.0 * (1.0 + c.num));
    let vm = 1.0 - c.vf;
    if c.vf * gm + vm * gf < 1e-12 {
        return 0.0;
    }
    gf * gm / (c.vf * gm + vm * gf)
}

/// Fiber volume fraction from weight fraction assuming densities.
pub fn vf_from_weight_fraction(wf: f32, rho_f: f32, rho_m: f32) -> f32 {
    if wf >= 1.0 || rho_f < 1e-12 || rho_m < 1e-12 {
        return 0.0;
    }
    (wf / rho_f) / (wf / rho_f + (1.0 - wf) / rho_m)
}

/// Composite density: rho = vf*rho_f + vm*rho_m.
pub fn composite_density(vf: f32, rho_f: f32, rho_m: f32) -> f32 {
    vf * rho_f + (1.0 - vf) * rho_m
}

/// Longitudinal tensile strength (rule of mixtures): sigma_1u = vf*sigma_fu + vm*sigma_mu.
pub fn composite_longitudinal_strength(vf: f32, sigma_fu: f32, sigma_mu: f32) -> f32 {
    vf * sigma_fu + (1.0 - vf) * sigma_mu
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_fiber_composite() {
        /* constructor clamps vf */
        let c = new_fiber_composite(1.5, 70.0, 3.0, 0.2, 0.35);
        assert!(c.vf <= 1.0);
    }

    #[test]
    fn test_e1_rule_of_mixtures() {
        /* E1 = vf*Ef + vm*Em */
        let c = new_fiber_composite(0.5, 70.0, 3.0, 0.2, 0.35);
        let e1 = composite_e1(&c);
        assert!((e1 - 36.5).abs() < 1e-3);
    }

    #[test]
    fn test_e2_less_than_e1() {
        /* E2 < E1 for high stiffness fiber */
        let c = new_fiber_composite(0.6, 230.0, 3.5, 0.2, 0.35);
        assert!(composite_e2(&c) < composite_e1(&c));
    }

    #[test]
    fn test_nu12_between_fiber_and_matrix() {
        /* nu12 should be between nuf and num */
        let c = new_fiber_composite(0.5, 70.0, 3.0, 0.2, 0.35);
        let nu = composite_nu12(&c);
        assert!((0.2..=0.35).contains(&nu));
    }

    #[test]
    fn test_g12_positive() {
        /* shear modulus > 0 */
        let c = new_fiber_composite(0.5, 70.0, 3.5, 0.2, 0.35);
        assert!(composite_g12(&c) > 0.0);
    }

    #[test]
    fn test_composite_density() {
        /* density = vf*rho_f + vm*rho_m */
        let d = composite_density(0.6, 1800.0, 1200.0);
        /* 0.6*1800 + 0.4*1200 = 1080 + 480 = 1560 */
        assert!((d - 1560.0).abs() < 1.0);
    }

    #[test]
    fn test_vf_from_weight_fraction() {
        /* result in [0, 1] */
        let vf = vf_from_weight_fraction(0.6, 1780.0, 1200.0);
        assert!((0.0..=1.0).contains(&vf));
    }

    #[test]
    fn test_longitudinal_strength() {
        /* strength > 0 */
        let s = composite_longitudinal_strength(0.6, 3500.0, 80.0);
        assert!(s > 0.0);
    }
}
