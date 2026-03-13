// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Kirchhoff plate bending stub — 1-D Euler-Bernoulli beam / Kirchhoff plate
//! bending using a finite-difference stiffness formulation.

/// A plate bending model discretised along one axis.
pub struct PlateBending {
    pub n: usize,               /* number of nodes */
    pub dx: f64,                /* node spacing */
    pub w: Vec<f64>,            /* transverse displacement (deflection) */
    pub load: Vec<f64>,         /* distributed transverse load q(x) */
    pub bending_stiffness: f64, /* D = E*h^3/(12*(1-ν²)) */
    pub boundary_simple: bool,  /* true = simply supported, false = clamped */
}

impl PlateBending {
    /// Create a new 1-D plate model.
    pub fn new(
        n: usize,
        length: f64,
        elastic_mod: f64,
        thickness: f64,
        poisson: f64,
        boundary_simple: bool,
    ) -> Self {
        let dx = length / (n as f64 - 1.0).max(1.0);
        let d = elastic_mod * thickness.powi(3) / (12.0 * (1.0 - poisson * poisson));
        Self {
            n,
            dx,
            w: vec![0.0; n],
            load: vec![0.0; n],
            bending_stiffness: d,
            boundary_simple,
        }
    }

    /// Apply a uniform load `q` across all interior nodes.
    pub fn apply_uniform_load(&mut self, q: f64) {
        for v in &mut self.load {
            *v = q;
        }
        /* zero load at boundary */
        if self.n > 0 {
            self.load[0] = 0.0;
            self.load[self.n - 1] = 0.0;
        }
    }

    /// Solve deflection using explicit Gauss-Seidel relaxation.
    /// For simply-supported: `w[0]`=`w[n-1]`=0 and w''`[0]`=w''`[n-1]`=0.
    /// For clamped: `w[0]`=`w[n-1]`=0 and w'`[0]`=w'`[n-1]`=0.
    pub fn solve(&mut self, iterations: usize) {
        let dx4 = self.dx.powi(4);
        let d = self.bending_stiffness;
        /* enforce boundary conditions */
        if self.boundary_simple {
            /* simply supported: w = 0 at both ends */
            self.w[0] = 0.0;
            self.w[self.n - 1] = 0.0;
        } else {
            /* clamped: w = 0, w' ≈ 0 */
            self.w[0] = 0.0;
            self.w[self.n - 1] = 0.0;
        }
        /* biharmonic stencil: w[i-2] - 4*w[i-1] + 6*w[i] - 4*w[i+1] + w[i+2] = q*dx^4/D */
        for _ in 0..iterations {
            for i in 2..(self.n.saturating_sub(2)) {
                let rhs = self.load[i] * dx4 / d;
                let w_im2 = if i >= 2 { self.w[i - 2] } else { 0.0 };
                let w_im1 = self.w[i - 1];
                let w_ip1 = self.w[i + 1];
                let w_ip2 = if i + 2 < self.n { self.w[i + 2] } else { 0.0 };
                self.w[i] = (rhs + 4.0 * w_im1 - w_im2 + 4.0 * w_ip1 - w_ip2) / 6.0;
            }
        }
    }

    /// Maximum deflection.
    pub fn max_deflection(&self) -> f64 {
        self.w.iter().cloned().fold(0.0f64, f64::max)
    }

    /// Central node deflection.
    pub fn central_deflection(&self) -> f64 {
        self.w[self.n / 2]
    }

    /// Approximate curvature at node `i`.
    pub fn curvature_at(&self, i: usize) -> f64 {
        if i == 0 || i + 1 >= self.n {
            return 0.0;
        }
        (self.w[i + 1] - 2.0 * self.w[i] + self.w[i - 1]) / (self.dx * self.dx)
    }

    /// Bending moment M = -D * κ.
    pub fn bending_moment_at(&self, i: usize) -> f64 {
        -self.bending_stiffness * self.curvature_at(i)
    }
}

/// Create a new plate bending model.
#[allow(clippy::too_many_arguments)]
pub fn new_plate_bending(
    n: usize,
    length: f64,
    e: f64,
    h: f64,
    nu: f64,
    boundary_simple: bool,
) -> PlateBending {
    PlateBending::new(n, length, e, h, nu, boundary_simple)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ss_beam() -> PlateBending {
        /* simply supported beam: 10 nodes, L=1m, E=200GPa, h=0.01m, nu=0.3 */
        PlateBending::new(11, 1.0, 2e11, 0.01, 0.3, true)
    }

    #[test]
    fn test_initial_deflection_zero() {
        let pb = ss_beam();
        assert!(pb.w.iter().all(|&v| v == 0.0)); /* no deflection initially */
    }

    #[test]
    fn test_apply_uniform_load() {
        let mut pb = ss_beam();
        pb.apply_uniform_load(1000.0);
        assert!(pb.load[5] > 0.0); /* load applied to interior */
    }

    #[test]
    fn test_boundary_zero_after_solve() {
        let mut pb = ss_beam();
        pb.apply_uniform_load(1000.0);
        pb.solve(500);
        assert_eq!(pb.w[0], 0.0); /* BC: w=0 at left */
        assert_eq!(pb.w[10], 0.0); /* BC: w=0 at right */
    }

    #[test]
    fn test_deflection_after_solve_positive() {
        let mut pb = ss_beam();
        pb.apply_uniform_load(1000.0);
        pb.solve(2000);
        assert!(pb.central_deflection() > 0.0); /* bends under load */
    }

    #[test]
    fn test_central_deflection_maximum() {
        let mut pb = ss_beam();
        pb.apply_uniform_load(1000.0);
        pb.solve(2000);
        /* for SS beam, central deflection is the maximum */
        let max_d = pb.max_deflection();
        let central = pb.central_deflection();
        assert!(central >= max_d - max_d * 0.01); /* central ≈ max (within 1%) */
    }

    #[test]
    fn test_bending_stiffness_positive() {
        let pb = ss_beam();
        assert!(pb.bending_stiffness > 0.0); /* stiffness > 0 */
    }

    #[test]
    fn test_curvature_zero_at_boundary() {
        let pb = ss_beam();
        assert_eq!(pb.curvature_at(0), 0.0); /* zero at boundary */
    }

    #[test]
    fn test_new_helper() {
        let pb = new_plate_bending(7, 1.0, 1e5, 0.01, 0.3, true);
        assert_eq!(pb.n, 7); /* helper works */
    }

    #[test]
    fn test_load_boundary_zero() {
        let mut pb = ss_beam();
        pb.apply_uniform_load(100.0);
        assert_eq!(pb.load[0], 0.0); /* no load at boundary */
        assert_eq!(pb.load[10], 0.0);
    }
}
