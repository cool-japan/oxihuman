// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Thin shell kinematics stub — implements the Kirchhoff-Love shell
//! kinematic relations: membrane strains and curvature changes from
//! mid-surface displacements.

/// A 3-D mid-surface node of the shell.
#[derive(Debug, Clone)]
pub struct ShellNode {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub u: f64, /* displacement x */
    pub v: f64, /* displacement y */
    pub w: f64, /* displacement z (out-of-plane) */
}

impl ShellNode {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            x,
            y,
            z,
            u: 0.0,
            v: 0.0,
            w: 0.0,
        }
    }
}

/// Membrane strain tensor components for a 2-D flat shell element
/// given node displacements.  Uses simplified finite-difference for 1D strip.
#[derive(Debug, Clone, Copy)]
pub struct MembraneStrain {
    pub eps_xx: f64,
    pub eps_yy: f64,
    pub eps_xy: f64,
}

/// Curvature (bending) strain components.
#[derive(Debug, Clone, Copy)]
pub struct Curvature {
    pub kappa_xx: f64,
    pub kappa_yy: f64,
    pub kappa_xy: f64,
}

/// Thin shell kinematics helper.
pub struct ShellKinematics {
    pub nodes: Vec<ShellNode>,
    pub thickness: f64,
    pub elastic_mod: f64,
    pub poisson: f64,
}

impl ShellKinematics {
    /// Create a shell kinematic model.
    pub fn new(thickness: f64, elastic_mod: f64, poisson: f64) -> Self {
        Self {
            nodes: Vec::new(),
            thickness,
            elastic_mod,
            poisson,
        }
    }

    /// Add a node.
    pub fn add_node(&mut self, x: f64, y: f64, z: f64) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(ShellNode::new(x, y, z));
        idx
    }

    /// Compute membrane strain between two nodes `i` and `j` (1-D strip).
    pub fn membrane_strain_1d(&self, i: usize, j: usize) -> f64 {
        let dx = self.nodes[j].x - self.nodes[i].x;
        let du = self.nodes[j].u - self.nodes[i].u;
        let dist = dx.abs().max(1e-14);
        du / dist
    }

    /// Compute curvature κ = d²w/dx² between three equally-spaced nodes i,j,k.
    pub fn curvature_1d(&self, i: usize, j: usize, k: usize) -> f64 {
        let dx = self.nodes[j].x - self.nodes[i].x;
        if dx.abs() < 1e-14 {
            return 0.0;
        }
        (self.nodes[k].w - 2.0 * self.nodes[j].w + self.nodes[i].w) / (dx * dx)
    }

    /// Bending stiffness D = E*h^3 / (12*(1-ν^2)).
    pub fn bending_stiffness(&self) -> f64 {
        self.elastic_mod * self.thickness.powi(3) / (12.0 * (1.0 - self.poisson * self.poisson))
    }

    /// Membrane stiffness per unit width K = E*h / (1-ν^2).
    pub fn membrane_stiffness(&self) -> f64 {
        self.elastic_mod * self.thickness / (1.0 - self.poisson * self.poisson)
    }

    /// Bending moment M = D * κ.
    pub fn bending_moment(&self, kappa: f64) -> f64 {
        self.bending_stiffness() * kappa
    }

    /// Number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

/// Create a new shell kinematics model.
pub fn new_shell_kinematics(t: f64, e: f64, nu: f64) -> ShellKinematics {
    ShellKinematics::new(t, e, nu)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_strip() -> ShellKinematics {
        let mut sk = ShellKinematics::new(0.01, 2.0e5, 0.3);
        sk.add_node(0.0, 0.0, 0.0);
        sk.add_node(1.0, 0.0, 0.0);
        sk.add_node(2.0, 0.0, 0.0);
        sk
    }

    #[test]
    fn test_membrane_strain_zero_at_rest() {
        let sk = flat_strip();
        let eps = sk.membrane_strain_1d(0, 1);
        assert_eq!(eps, 0.0); /* no displacement, no strain */
    }

    #[test]
    fn test_membrane_strain_nonzero() {
        let mut sk = flat_strip();
        sk.nodes[1].u = 0.01; /* elongate */
        let eps = sk.membrane_strain_1d(0, 1);
        assert!(eps > 0.0); /* tensile strain */
    }

    #[test]
    fn test_curvature_zero_flat() {
        let sk = flat_strip();
        let kappa = sk.curvature_1d(0, 1, 2);
        assert_eq!(kappa, 0.0); /* flat, no curvature */
    }

    #[test]
    fn test_curvature_nonzero_bent() {
        let mut sk = flat_strip();
        sk.nodes[1].w = 0.05; /* bent upward */
        let kappa = sk.curvature_1d(0, 1, 2);
        assert!(kappa.abs() > 0.0); /* nonzero curvature */
    }

    #[test]
    fn test_bending_stiffness_positive() {
        let sk = flat_strip();
        assert!(sk.bending_stiffness() > 0.0); /* positive stiffness */
    }

    #[test]
    fn test_membrane_stiffness_positive() {
        let sk = flat_strip();
        assert!(sk.membrane_stiffness() > 0.0); /* positive stiffness */
    }

    #[test]
    fn test_bending_moment() {
        let sk = flat_strip();
        let kappa = 0.01;
        let m = sk.bending_moment(kappa);
        assert!((m - sk.bending_stiffness() * kappa).abs() < 1e-10); /* M = D*κ */
    }

    #[test]
    fn test_node_count() {
        let sk = flat_strip();
        assert_eq!(sk.node_count(), 3); /* three nodes */
    }

    #[test]
    fn test_new_helper() {
        let sk = new_shell_kinematics(0.005, 1e5, 0.25);
        assert_eq!(sk.node_count(), 0); /* empty model */
    }

    #[test]
    fn test_add_node() {
        let mut sk = ShellKinematics::new(0.01, 1e5, 0.3);
        let idx = sk.add_node(0.0, 0.0, 0.0);
        assert_eq!(idx, 0); /* first node index */
    }
}
