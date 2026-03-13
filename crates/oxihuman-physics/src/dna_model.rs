// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Worm-like chain (WLC) DNA elasticity model.

/// WLC model parameters for DNA.
#[derive(Debug, Clone)]
pub struct WlcDnaModel {
    /// Persistence length (meters).
    pub lp: f32,
    /// Contour length (meters).
    pub l_contour: f32,
    /// Thermal energy kB * T (Joules).
    pub kbt: f32,
}

impl WlcDnaModel {
    /// Typical DNA parameters at room temperature (T=300K).
    pub fn new(lp: f32, l_contour: f32, kbt: f32) -> Self {
        WlcDnaModel { lp, l_contour, kbt }
    }

    /// Marko-Siggia WLC force-extension: F = (kBT/lp) * [1/(4*(1-x)^2) - 1/4 + x]
    /// where x = end_to_end / L_contour.
    pub fn force(&self, end_to_end: f32) -> f32 {
        let x = (end_to_end / self.l_contour).min(0.9999);
        (self.kbt / self.lp) * (0.25 / (1.0 - x).powi(2) - 0.25 + x)
    }

    /// End-to-end distance at thermal equilibrium: <R^2> = 2 * lp * L * (1 - lp/L * (1 - exp(-L/lp)))
    pub fn mean_sq_end_to_end(&self) -> f32 {
        let l = self.l_contour;
        let lp = self.lp;
        2.0 * lp * l * (1.0 - lp / l * (1.0 - (-l / lp).exp()))
    }

    /// RMS end-to-end distance.
    pub fn rms_end_to_end(&self) -> f32 {
        self.mean_sq_end_to_end().sqrt()
    }

    /// Bending stiffness: A = kBT * lp.
    pub fn bending_stiffness(&self) -> f32 {
        self.kbt * self.lp
    }

    /// Radius of gyration for WLC: R_g^2 = lp*L/3 * (1 - 3lp/L + 6*(lp/L)^2 - ...) (Kratky-Porod).
    pub fn radius_of_gyration_sq(&self) -> f32 {
        let l = self.l_contour;
        let lp = self.lp;
        let u = l / lp;
        lp * l / 3.0 - lp * lp + 2.0 * lp.powi(3) / l * (1.0 - (-u).exp())
    }

    pub fn radius_of_gyration(&self) -> f32 {
        self.radius_of_gyration_sq().max(0.0).sqrt()
    }

    /// Number of Kuhn segments: N = L / (2 * lp).
    pub fn kuhn_segments(&self) -> f32 {
        self.l_contour / (2.0 * self.lp)
    }

    /// Kuhn length: b = 2 * lp.
    pub fn kuhn_length(&self) -> f32 {
        2.0 * self.lp
    }

    /// Persistence length in base pairs (bp), given bp_spacing ~ 0.34 nm.
    pub fn persistence_length_bp(&self, bp_spacing: f32) -> f32 {
        self.lp / bp_spacing
    }

    /// Twist persistence length for DNA (approx 80 nm).
    pub fn twist_persistence_length() -> f32 {
        80e-9
    }
}

/// Standard B-DNA WLC model at T = 300 K.
pub fn bdna_model() -> WlcDnaModel {
    WlcDnaModel::new(
        50e-9,     /* lp = 50 nm */
        1.6e-6,    /* L = 1.6 μm (~ 4700 bp * 0.34 nm/bp) */
        4.114e-21, /* kBT at 300 K */
    )
}

/// Freely jointed chain (FJC) force: F = (kBT/b) * (coth(Fb/kBT) - kBT/(Fb))
/// Approximate Langevin function inversion for end_to_end / L.
pub fn fjc_force(end_to_end: f32, l_contour: f32, b: f32, kbt: f32) -> f32 {
    let x = (end_to_end / l_contour).min(0.99);
    /* Inverse Langevin approximation: L^{-1}(x) ≈ x*(3 - x^2)/(1 - x^2) */
    let inv_langevin = x * (3.0 - x * x) / (1.0 - x * x);
    kbt / b * inv_langevin
}

/// Odijk deflection length for DNA confined in a channel of width d.
pub fn odijk_deflection_length(lp: f32, d: f32) -> f32 {
    (lp * d * d).cbrt()
}

pub fn new_dna_model(lp: f32, l_contour: f32, kbt: f32) -> WlcDnaModel {
    WlcDnaModel::new(lp, l_contour, kbt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bdna_model() {
        let dna = bdna_model();
        assert!((dna.lp - 50e-9).abs() < 1e-12);
    }

    #[test]
    fn test_force_at_small_extension() {
        let dna = bdna_model();
        /* Small extension: force should be small but positive */
        let f = dna.force(0.1 * dna.l_contour);
        assert!(f > 0.0);
    }

    #[test]
    fn test_force_increases_with_extension() {
        let dna = bdna_model();
        let f1 = dna.force(0.3 * dna.l_contour);
        let f2 = dna.force(0.8 * dna.l_contour);
        assert!(f2 > f1);
    }

    #[test]
    fn test_rms_end_to_end_positive() {
        let dna = bdna_model();
        let rms = dna.rms_end_to_end();
        assert!(rms > 0.0 && rms < dna.l_contour);
    }

    #[test]
    fn test_bending_stiffness() {
        let dna = bdna_model();
        let a = dna.bending_stiffness();
        assert!(a > 0.0);
    }

    #[test]
    fn test_kuhn_length() {
        let dna = bdna_model();
        assert!((dna.kuhn_length() - 100e-9).abs() < 1e-12);
    }

    #[test]
    fn test_radius_of_gyration() {
        let dna = bdna_model();
        let rg = dna.radius_of_gyration();
        assert!(rg > 0.0);
    }

    #[test]
    fn test_odijk_deflection() {
        /* ld = (lp * d^2)^(1/3) ≈ (50e-9 * (100e-9)^2)^(1/3) ≈ 79 nm */
        let ld = odijk_deflection_length(50e-9, 100e-9);
        assert!(ld > 0.0 && ld < 500e-9);
    }
}
