// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Worm-like chain (WLC) model — semi-flexible polymer model with bending
//! persistence length `lp`. Provides force-extension and thermal fluctuation
//! approximations.

/// WLC model parameters.
pub struct WormLikeChain {
    pub contour_length: f64,     /* L — total contour length */
    pub persistence_length: f64, /* lp — bending stiffness / kT */
    pub kbt: f64,                /* thermal energy (kB * T) */
}

impl WormLikeChain {
    /// Create a new WLC model.
    pub fn new(contour_length: f64, persistence_length: f64, kbt: f64) -> Self {
        Self {
            contour_length,
            persistence_length,
            kbt,
        }
    }

    /// Marko-Siggia interpolation formula: force f for extension x ∈ [0, L).
    /// Returns force in units of kBT / lp.
    pub fn force_at_extension(&self, x: f64) -> f64 {
        let x = x.clamp(0.0, self.contour_length * 0.9999);
        let z = x / self.contour_length;
        let lp = self.persistence_length;
        (self.kbt / lp) * (0.25 / (1.0 - z).powi(2) - 0.25 + z)
    }

    /// Extension at a given force using Newton iteration on Marko-Siggia.
    pub fn extension_at_force(&self, force: f64) -> f64 {
        if force <= 0.0 {
            return 0.0;
        }
        /* binary search: find x such that force_at_extension(x) ≈ force */
        let mut lo = 0.0f64;
        let mut hi = self.contour_length * 0.9999;
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if self.force_at_extension(mid) < force {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    }

    /// RMS end-to-end distance for a 3-D WLC: sqrt(2*lp*L*(1 - lp/L*(1-exp(-L/lp)))).
    pub fn rms_end_to_end(&self) -> f64 {
        let lp = self.persistence_length;
        let l = self.contour_length;
        let inner = 2.0 * lp * l - 2.0 * lp * lp * (1.0 - (-l / lp).exp());
        inner.max(0.0).sqrt()
    }

    /// Tangent-tangent correlation at arc separation `s`: exp(-s/(2*lp)) in 3D.
    pub fn tangent_correlation(&self, s: f64) -> f64 {
        (-s / (2.0 * self.persistence_length)).exp()
    }

    /// Contour length accessor.
    pub fn contour_length(&self) -> f64 {
        self.contour_length
    }

    /// Persistence length accessor.
    pub fn persistence_length(&self) -> f64 {
        self.persistence_length
    }

    /// Bending stiffness κ = kBT * lp.
    pub fn bending_stiffness(&self) -> f64 {
        self.kbt * self.persistence_length
    }
}

/// Create a new WLC model.
pub fn new_worm_like_chain(l: f64, lp: f64, kbt: f64) -> WormLikeChain {
    WormLikeChain::new(l, lp, kbt)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dna_chain() -> WormLikeChain {
        /* DNA: L=10 μm, lp=50 nm, kT=4.1 pN·nm (use normalised units) */
        WormLikeChain::new(200.0, 1.0, 1.0)
    }

    #[test]
    fn test_force_increases_with_extension() {
        let wlc = dna_chain();
        let f1 = wlc.force_at_extension(50.0);
        let f2 = wlc.force_at_extension(150.0);
        assert!(f2 > f1); /* force increases with extension */
    }

    #[test]
    fn test_force_near_full_extension_is_large() {
        let wlc = dna_chain();
        let f = wlc.force_at_extension(199.0);
        assert!(f > 1.0); /* large force near full extension */
    }

    #[test]
    fn test_extension_at_force_increases_with_force() {
        let wlc = dna_chain();
        let x1 = wlc.extension_at_force(0.1);
        let x2 = wlc.extension_at_force(1.0);
        assert!(x2 > x1); /* larger force gives larger extension */
    }

    #[test]
    fn test_extension_at_zero_force() {
        let wlc = dna_chain();
        assert_eq!(wlc.extension_at_force(0.0), 0.0); /* no force, no extension */
    }

    #[test]
    fn test_rms_end_to_end_positive() {
        let wlc = dna_chain();
        assert!(wlc.rms_end_to_end() > 0.0); /* positive distance */
    }

    #[test]
    fn test_tangent_correlation_at_zero() {
        let wlc = dna_chain();
        assert!((wlc.tangent_correlation(0.0) - 1.0).abs() < 1e-10); /* 1 at s=0 */
    }

    #[test]
    fn test_tangent_correlation_decays() {
        let wlc = dna_chain();
        let c1 = wlc.tangent_correlation(1.0);
        let c2 = wlc.tangent_correlation(5.0);
        assert!(c2 < c1); /* exponential decay */
    }

    #[test]
    fn test_bending_stiffness() {
        let wlc = WormLikeChain::new(10.0, 3.0, 2.0);
        assert!((wlc.bending_stiffness() - 6.0).abs() < 1e-10); /* kBT * lp */
    }

    #[test]
    fn test_new_helper() {
        let wlc = new_worm_like_chain(50.0, 2.0, 1.0);
        assert!((wlc.contour_length() - 50.0).abs() < 1e-10); /* helper works */
    }

    #[test]
    fn test_rms_shorter_than_contour() {
        let wlc = dna_chain();
        assert!(wlc.rms_end_to_end() < wlc.contour_length()); /* always true */
    }
}
