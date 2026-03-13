// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Freely-jointed polymer chain stub — models a polymer as N rigid segments
//! of length `b` with uncorrelated orientations (Kuhn model).

use std::f64::consts::PI;

/// A freely-jointed chain segment with direction (theta, phi).
#[derive(Debug, Clone)]
pub struct ChainSegment {
    pub theta: f64, /* polar angle */
    pub phi: f64,   /* azimuthal angle */
    pub length: f64,
}

impl ChainSegment {
    pub fn new(theta: f64, phi: f64, length: f64) -> Self {
        Self { theta, phi, length }
    }

    /// Cartesian end-to-end vector contribution.
    pub fn vector(&self) -> [f64; 3] {
        let st = self.theta.sin();
        [
            self.length * st * self.phi.cos(),
            self.length * st * self.phi.sin(),
            self.length * self.theta.cos(),
        ]
    }
}

/// Freely-jointed polymer chain.
pub struct PolymerChain {
    pub segments: Vec<ChainSegment>,
    pub kuhn_length: f64,
}

impl PolymerChain {
    /// Create a new chain with `n` segments of Kuhn length `b`.
    pub fn new(n: usize, b: f64) -> Self {
        /* initialise all segments pointing along z */
        let segments = (0..n).map(|_| ChainSegment::new(0.0, 0.0, b)).collect();
        Self {
            segments,
            kuhn_length: b,
        }
    }

    /// Generate a random-walk conformation using a simple LCG seed.
    pub fn randomise(&mut self, seed: u64) {
        let mut r = seed;
        let lcg = |rng: &mut u64| -> f64 {
            *rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (*rng >> 33) as f64 / (u32::MAX as f64)
        };
        for seg in &mut self.segments {
            seg.theta = lcg(&mut r) * PI;
            seg.phi = lcg(&mut r) * 2.0 * PI;
        }
    }

    /// End-to-end vector of the chain.
    pub fn end_to_end_vector(&self) -> [f64; 3] {
        let mut v = [0.0f64; 3];
        for seg in &self.segments {
            let dv = seg.vector();
            v[0] += dv[0];
            v[1] += dv[1];
            v[2] += dv[2];
        }
        v
    }

    /// End-to-end distance.
    pub fn end_to_end_distance(&self) -> f64 {
        let v = self.end_to_end_vector();
        (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
    }

    /// Contour length (N * b).
    pub fn contour_length(&self) -> f64 {
        self.segments.iter().map(|s| s.length).sum()
    }

    /// Theoretical RMS end-to-end distance sqrt(N) * b.
    pub fn rms_end_to_end(&self) -> f64 {
        (self.segments.len() as f64).sqrt() * self.kuhn_length
    }

    /// Number of segments.
    pub fn n_segments(&self) -> usize {
        self.segments.len()
    }

    /// Radius of gyration (simplified: Rg = R_ee / sqrt(6) for ideal chain).
    pub fn radius_of_gyration(&self) -> f64 {
        self.rms_end_to_end() / 6.0f64.sqrt()
    }
}

/// Create a new polymer chain.
pub fn new_polymer_chain(n: usize, b: f64) -> PolymerChain {
    PolymerChain::new(n, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contour_length() {
        let chain = PolymerChain::new(10, 1.5);
        assert!((chain.contour_length() - 15.0).abs() < 1e-10); /* N*b */
    }

    #[test]
    fn test_rms_formula() {
        let chain = PolymerChain::new(100, 1.0);
        assert!((chain.rms_end_to_end() - 10.0).abs() < 1e-10); /* sqrt(100)*1 */
    }

    #[test]
    fn test_n_segments() {
        let chain = PolymerChain::new(50, 1.0);
        assert_eq!(chain.n_segments(), 50); /* correct count */
    }

    #[test]
    fn test_initial_all_along_z() {
        let chain = PolymerChain::new(5, 1.0);
        let ete = chain.end_to_end_distance();
        assert!((ete - 5.0).abs() < 1e-10); /* all along z, R = N*b */
    }

    #[test]
    fn test_randomise_changes_conformation() {
        let mut chain = PolymerChain::new(100, 1.0);
        let ete_before = chain.end_to_end_distance();
        chain.randomise(42);
        let ete_after = chain.end_to_end_distance();
        assert!(ete_after != ete_before); /* conformation changed */
    }

    #[test]
    fn test_end_to_end_after_randomise_reasonable() {
        let mut chain = PolymerChain::new(100, 1.0);
        chain.randomise(123);
        let ete = chain.end_to_end_distance();
        /* should be well below contour length */
        assert!(ete < chain.contour_length()); /* not fully extended */
    }

    #[test]
    fn test_radius_of_gyration() {
        let chain = PolymerChain::new(100, 1.0);
        let rg = chain.radius_of_gyration();
        assert!(rg > 0.0); /* positive Rg */
    }

    #[test]
    fn test_segment_vector_along_z() {
        let seg = ChainSegment::new(0.0, 0.0, 2.0);
        let v = seg.vector();
        assert!(v[2].abs() - 2.0 < 1e-10); /* along z */
    }

    #[test]
    fn test_new_helper() {
        let chain = new_polymer_chain(20, 0.5);
        assert_eq!(chain.n_segments(), 20); /* helper works */
    }

    #[test]
    fn test_end_to_end_vector_initial() {
        let chain = PolymerChain::new(3, 1.0);
        let v = chain.end_to_end_vector();
        /* all along z: sum of z components = N */
        assert!((v[2] - 3.0).abs() < 1e-10); /* z component correct */
    }
}
