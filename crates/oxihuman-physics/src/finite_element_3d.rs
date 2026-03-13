// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! 3D tetrahedral FEM stub — linear tetrahedral elements for deformable body
//! simulation. Computes element volume and a simplified stress proxy.

/// A 3D node position.
#[derive(Debug, Clone, Copy)]
pub struct Node3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Node3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

/// A tetrahedral element defined by four node indices.
#[derive(Debug, Clone, Copy)]
pub struct Tetrahedron {
    pub nodes: [usize; 4],
}

/// FEM mesh of tetrahedra.
pub struct FiniteElement3D {
    pub nodes: Vec<Node3D>,
    pub elements: Vec<Tetrahedron>,
    /// Young's modulus.
    pub young: f64,
    /// Poisson's ratio.
    pub poisson: f64,
}

impl FiniteElement3D {
    /// Create a new FEM mesh.
    pub fn new(young: f64, poisson: f64) -> Self {
        Self {
            nodes: Vec::new(),
            elements: Vec::new(),
            young,
            poisson,
        }
    }

    /// Add a node, return its index.
    pub fn add_node(&mut self, x: f64, y: f64, z: f64) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(Node3D::new(x, y, z));
        idx
    }

    /// Add a tetrahedral element by node indices.
    pub fn add_element(&mut self, a: usize, b: usize, c: usize, d: usize) {
        self.elements.push(Tetrahedron {
            nodes: [a, b, c, d],
        });
    }

    /// Compute the signed volume of a tetrahedron (should be positive for correct winding).
    pub fn element_volume(&self, elem_idx: usize) -> f64 {
        let e = &self.elements[elem_idx];
        let n = &self.nodes;
        let [a, b, c, d] = e.nodes;
        let ab = [n[b].x - n[a].x, n[b].y - n[a].y, n[b].z - n[a].z];
        let ac = [n[c].x - n[a].x, n[c].y - n[a].y, n[c].z - n[a].z];
        let ad = [n[d].x - n[a].x, n[d].y - n[a].y, n[d].z - n[a].z];
        /* scalar triple product / 6 */
        (ab[0] * (ac[1] * ad[2] - ac[2] * ad[1]) - ab[1] * (ac[0] * ad[2] - ac[2] * ad[0])
            + ab[2] * (ac[0] * ad[1] - ac[1] * ad[0]))
            / 6.0
    }

    /// Total mesh volume.
    pub fn total_volume(&self) -> f64 {
        (0..self.elements.len())
            .map(|i| self.element_volume(i).abs())
            .sum()
    }

    /// Lamé parameter λ.
    pub fn lame_lambda(&self) -> f64 {
        self.young * self.poisson / ((1.0 + self.poisson) * (1.0 - 2.0 * self.poisson))
    }

    /// Lamé parameter μ (shear modulus).
    pub fn lame_mu(&self) -> f64 {
        self.young / (2.0 * (1.0 + self.poisson))
    }

    /// Number of degrees of freedom (3 per node).
    pub fn dof_count(&self) -> usize {
        self.nodes.len() * 3
    }
}

/// Create a new 3D FEM mesh.
pub fn new_finite_element_3d(young: f64, poisson: f64) -> FiniteElement3D {
    FiniteElement3D::new(young, poisson)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_tet() -> FiniteElement3D {
        let mut fem = FiniteElement3D::new(1e6, 0.3);
        fem.add_node(0.0, 0.0, 0.0);
        fem.add_node(1.0, 0.0, 0.0);
        fem.add_node(0.0, 1.0, 0.0);
        fem.add_node(0.0, 0.0, 1.0);
        fem.add_element(0, 1, 2, 3);
        fem
    }

    #[test]
    fn test_unit_tet_volume() {
        let fem = unit_tet();
        let vol = fem.element_volume(0).abs();
        assert!((vol - 1.0 / 6.0).abs() < 1e-10); /* unit tet volume = 1/6 */
    }

    #[test]
    fn test_total_volume() {
        let fem = unit_tet();
        assert!((fem.total_volume() - 1.0 / 6.0).abs() < 1e-10); /* single element */
    }

    #[test]
    fn test_lame_lambda() {
        let fem = FiniteElement3D::new(1.0, 0.25);
        let lambda = fem.lame_lambda();
        assert!(lambda > 0.0); /* positive Lamé lambda */
    }

    #[test]
    fn test_lame_mu() {
        let fem = FiniteElement3D::new(2.0, 0.25);
        let mu = fem.lame_mu();
        assert!((mu - 0.8).abs() < 1e-10); /* E/(2(1+nu)) = 2/2.5 = 0.8 */
    }

    #[test]
    fn test_dof_count() {
        let fem = unit_tet();
        assert_eq!(fem.dof_count(), 12); /* 4 nodes * 3 dof */
    }

    #[test]
    fn test_add_node() {
        let mut fem = FiniteElement3D::new(1e6, 0.3);
        let idx = fem.add_node(1.0, 2.0, 3.0);
        assert_eq!(idx, 0); /* first node index */
    }

    #[test]
    fn test_element_count() {
        let fem = unit_tet();
        assert_eq!(fem.elements.len(), 1); /* one element added */
    }

    #[test]
    fn test_new_helper() {
        let fem = new_finite_element_3d(1e5, 0.4);
        assert_eq!(fem.dof_count(), 0); /* empty mesh */
    }

    #[test]
    fn test_two_elements_volume() {
        let mut fem = FiniteElement3D::new(1e6, 0.3);
        /* two unit tets sharing face */
        fem.add_node(0.0, 0.0, 0.0);
        fem.add_node(1.0, 0.0, 0.0);
        fem.add_node(0.0, 1.0, 0.0);
        fem.add_node(0.0, 0.0, 1.0);
        fem.add_node(1.0, 1.0, 1.0);
        fem.add_element(0, 1, 2, 3);
        fem.add_element(1, 2, 3, 4);
        let v = fem.total_volume();
        assert!(v > 1.0 / 6.0); /* two tets have more volume */
    }
}
