// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Euler-Bernoulli beam FEM stub — 1-D beam elements with axial and bending
//! degrees of freedom. Builds local stiffness matrices and assembles them.

/// Beam cross-section properties.
#[derive(Debug, Clone, Copy)]
pub struct BeamSection {
    pub area: f64,              /* cross-sectional area */
    pub moment_of_inertia: f64, /* second moment of area Iz */
}

impl BeamSection {
    pub fn rectangular(width: f64, height: f64) -> Self {
        Self {
            area: width * height,
            moment_of_inertia: width * height.powi(3) / 12.0,
        }
    }
}

/// A beam element connecting two nodes.
#[derive(Debug, Clone)]
pub struct BeamElem {
    pub node_i: usize,
    pub node_j: usize,
    pub length: f64,
    pub section: BeamSection,
    pub elastic_mod: f64,
}

impl BeamElem {
    /// Axial stiffness EA/L.
    pub fn axial_stiffness(&self) -> f64 {
        self.elastic_mod * self.section.area / self.length
    }

    /// Bending stiffness 12EI/L³.
    pub fn bending_stiffness(&self) -> f64 {
        12.0 * self.elastic_mod * self.section.moment_of_inertia / self.length.powi(3)
    }

    /// Build the 4x4 Euler-Bernoulli local stiffness matrix
    /// (2 translational + 2 rotational DOF per element in 2D bending).
    /// DOF order: [v_i, θ_i, v_j, θ_j]
    pub fn local_bending_stiffness(&self) -> [[f64; 4]; 4] {
        let l = self.length;
        let ei = self.elastic_mod * self.section.moment_of_inertia;
        let c = ei / (l * l * l);
        [
            [12.0 * c, 6.0 * l * c, -12.0 * c, 6.0 * l * c],
            [6.0 * l * c, 4.0 * l * l * c, -6.0 * l * c, 2.0 * l * l * c],
            [-12.0 * c, -6.0 * l * c, 12.0 * c, -6.0 * l * c],
            [6.0 * l * c, 2.0 * l * l * c, -6.0 * l * c, 4.0 * l * l * c],
        ]
    }
}

/// Beam FEM model.
pub struct BeamElement {
    pub node_positions: Vec<f64>, /* x-coordinates of nodes */
    pub elements: Vec<BeamElem>,
    pub displacements: Vec<f64>, /* transverse DOF w per node */
    pub rotations: Vec<f64>,     /* rotation DOF θ per node */
    pub loads: Vec<f64>,         /* transverse load per node */
}

impl BeamElement {
    /// Create a beam FEM model with evenly-spaced nodes.
    pub fn new_uniform(n_nodes: usize, total_length: f64, section: BeamSection, e: f64) -> Self {
        let dx = total_length / (n_nodes as f64 - 1.0).max(1.0);
        let positions: Vec<f64> = (0..n_nodes).map(|i| i as f64 * dx).collect();
        let mut elements = Vec::new();
        for i in 0..(n_nodes.saturating_sub(1)) {
            elements.push(BeamElem {
                node_i: i,
                node_j: i + 1,
                length: dx,
                section,
                elastic_mod: e,
            });
        }
        let m = n_nodes;
        Self {
            node_positions: positions,
            elements,
            displacements: vec![0.0; m],
            rotations: vec![0.0; m],
            loads: vec![0.0; m],
        }
    }

    /// Apply a point load at node `idx`.
    pub fn apply_load(&mut self, idx: usize, load: f64) {
        if idx < self.loads.len() {
            self.loads[idx] = load;
        }
    }

    /// Number of nodes.
    pub fn node_count(&self) -> usize {
        self.node_positions.len()
    }

    /// Number of elements.
    pub fn element_count(&self) -> usize {
        self.elements.len()
    }

    /// Total axial stiffness (for a single-element beam, EA/L).
    pub fn total_axial_stiffness(&self) -> f64 {
        self.elements.iter().map(|e| e.axial_stiffness()).sum()
    }

    /// Maximum bending stiffness among elements.
    pub fn max_bending_stiffness(&self) -> f64 {
        self.elements
            .iter()
            .map(|e| e.bending_stiffness())
            .fold(0.0f64, f64::max)
    }

    /// Simple cantilever deflection at tip for uniform load using analytical formula.
    /// δ = q*L^4 / (8*E*I) (cantilever, distributed load).
    pub fn cantilever_tip_deflection(&self, q: f64) -> f64 {
        if self.elements.is_empty() {
            return 0.0;
        }
        let e = self.elements[0].elastic_mod;
        let i = self.elements[0].section.moment_of_inertia;
        let l: f64 = self.node_positions.last().copied().unwrap_or(0.0);
        q * l.powi(4) / (8.0 * e * i)
    }
}

/// Create a uniform beam FEM model.
pub fn new_beam_element(n: usize, l: f64, section: BeamSection, e: f64) -> BeamElement {
    BeamElement::new_uniform(n, l, section, e)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn steel_beam() -> BeamElement {
        let sec = BeamSection::rectangular(0.1, 0.2);
        BeamElement::new_uniform(5, 1.0, sec, 2e11)
    }

    #[test]
    fn test_node_count() {
        let b = steel_beam();
        assert_eq!(b.node_count(), 5); /* 5 nodes */
    }

    #[test]
    fn test_element_count() {
        let b = steel_beam();
        assert_eq!(b.element_count(), 4); /* 4 elements for 5 nodes */
    }

    #[test]
    fn test_axial_stiffness_positive() {
        let sec = BeamSection::rectangular(0.1, 0.2);
        let elem = BeamElem {
            node_i: 0,
            node_j: 1,
            length: 1.0,
            section: sec,
            elastic_mod: 2e11,
        };
        assert!(elem.axial_stiffness() > 0.0); /* positive EA/L */
    }

    #[test]
    fn test_bending_stiffness_positive() {
        let sec = BeamSection::rectangular(0.1, 0.2);
        let elem = BeamElem {
            node_i: 0,
            node_j: 1,
            length: 1.0,
            section: sec,
            elastic_mod: 2e11,
        };
        assert!(elem.bending_stiffness() > 0.0); /* positive 12EI/L³ */
    }

    #[test]
    #[allow(clippy::needless_range_loop)]
    fn test_local_stiffness_symmetry() {
        let sec = BeamSection::rectangular(0.1, 0.2);
        let elem = BeamElem {
            node_i: 0,
            node_j: 1,
            length: 1.0,
            section: sec,
            elastic_mod: 2e11,
        };
        let k = elem.local_bending_stiffness();
        for i in 0..4 {
            for j in 0..4 {
                assert!((k[i][j] - k[j][i]).abs() < 1e-6); /* symmetric */
            }
        }
    }

    #[test]
    fn test_section_area() {
        let sec = BeamSection::rectangular(0.1, 0.2);
        assert!((sec.area - 0.02).abs() < 1e-10); /* 0.1 * 0.2 */
    }

    #[test]
    fn test_section_moment_of_inertia() {
        let sec = BeamSection::rectangular(0.1, 0.2);
        let expected = 0.1 * 0.2f64.powi(3) / 12.0;
        assert!((sec.moment_of_inertia - expected).abs() < 1e-15); /* bh³/12 */
    }

    #[test]
    fn test_apply_load() {
        let mut b = steel_beam();
        b.apply_load(2, -1000.0);
        assert!((b.loads[2] + 1000.0).abs() < 1e-10); /* load stored */
    }

    #[test]
    fn test_cantilever_deflection_positive() {
        let b = steel_beam();
        let delta = b.cantilever_tip_deflection(1000.0);
        assert!(delta > 0.0); /* positive deflection */
    }

    #[test]
    fn test_new_helper() {
        let sec = BeamSection::rectangular(0.05, 0.1);
        let b = new_beam_element(3, 0.5, sec, 1e11);
        assert_eq!(b.node_count(), 3); /* helper works */
    }
}
