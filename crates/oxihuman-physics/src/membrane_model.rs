// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Lipid membrane tension and bending model (Helfrich Hamiltonian).

use std::f32::consts::PI;

/// Helfrich membrane parameters.
#[derive(Debug, Clone)]
pub struct HelfrichMembrane {
    /// Bending rigidity κ (J).
    pub kappa: f32,
    /// Gaussian bending rigidity κ_G (J).
    pub kappa_g: f32,
    /// Surface tension σ (N/m).
    pub sigma: f32,
    /// Spontaneous curvature c0 (m^-1).
    pub c0: f32,
    /// Membrane area (m^2).
    pub area: f32,
}

impl HelfrichMembrane {
    pub fn new(kappa: f32, kappa_g: f32, sigma: f32, c0: f32, area: f32) -> Self {
        HelfrichMembrane {
            kappa,
            kappa_g,
            sigma,
            c0,
            area,
        }
    }

    /// Typical DPPC lipid bilayer.
    pub fn lipid_bilayer() -> Self {
        HelfrichMembrane::new(
            20.0 * 4.114e-21, /* ~20 kBT bending rigidity */
            -10.0 * 4.114e-21,
            1e-5, /* low tension (nearly tensionless) */
            0.0,
            4.0 * PI * (10e-6_f32).powi(2), /* 10 μm vesicle */
        )
    }

    /// Helfrich bending energy for a surface element with principal curvatures c1, c2.
    /// F_bend = 0.5 * κ * (c1 + c2 - c0)^2 + κ_G * c1 * c2
    pub fn bending_energy_density(&self, c1: f32, c2: f32) -> f32 {
        let mean_curv = c1 + c2;
        let gauss_curv = c1 * c2;
        0.5 * self.kappa * (mean_curv - self.c0).powi(2) + self.kappa_g * gauss_curv
    }

    /// Total bending energy for a sphere of radius R.
    /// Sphere: c1 = c2 = 1/R, Gaussian curvature = 1/R^2, area = 4πR^2.
    /// F_total = 4π * (2κ + κ_G) for zero spontaneous curvature.
    pub fn sphere_bending_energy(&self, radius: f32) -> f32 {
        let inv_r = 1.0 / radius.max(1e-15);
        let f_density = self.bending_energy_density(inv_r, inv_r);
        f_density * 4.0 * PI * radius * radius
    }

    /// Tension energy: E_tension = σ * ΔA (excess area).
    pub fn tension_energy(&self, excess_area: f32) -> f32 {
        self.sigma * excess_area
    }

    /// Shape equation for a spherical vesicle (Ou-Yang and Helfrich).
    /// Equilibrium sphere radius from osmotic pressure.
    pub fn equilibrium_sphere_radius(&self, dp: f32) -> f32 {
        /* Laplace: dp = 2 * (2κ/R^2 * (1/R - c0) + σ/R) */
        /* Simplified: dp * R = 2σ for tensionless: dp = 4κ/R^3 */
        if self.sigma > 1e-10 {
            2.0 * self.sigma / dp.max(1e-20)
        } else {
            (4.0 * self.kappa / dp.max(1e-20)).cbrt()
        }
    }

    /// Effective spring constant for small deformations.
    pub fn effective_spring_constant(&self) -> f32 {
        self.kappa / self.area
    }

    /// Thermal undulation amplitude <h^2> for a flat membrane.
    /// <h^2> = kBT / (κ * q^4 * A) integrated → kBT * L^2 / (4π^2 * κ)
    pub fn undulation_amplitude_sq(&self, kbt: f32, side_length: f32) -> f32 {
        kbt * side_length * side_length / (4.0 * PI * PI * self.kappa)
    }
}

/// A discrete membrane patch for simulation.
pub struct MembranePatch {
    pub nodes: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub mean_curvatures: Vec<f32>,
    pub params: HelfrichMembrane,
}

impl MembranePatch {
    pub fn new(params: HelfrichMembrane) -> Self {
        MembranePatch {
            nodes: Vec::new(),
            normals: Vec::new(),
            mean_curvatures: Vec::new(),
            params,
        }
    }

    pub fn add_node(&mut self, pos: [f32; 3], normal: [f32; 3], mean_curv: f32) {
        self.nodes.push(pos);
        self.normals.push(normal);
        self.mean_curvatures.push(mean_curv);
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Mean curvature averaged over all nodes.
    pub fn average_mean_curvature(&self) -> f32 {
        if self.nodes.is_empty() {
            return 0.0;
        }
        self.mean_curvatures.iter().sum::<f32>() / self.nodes.len() as f32
    }

    /// Total bending energy estimate (assuming uniform patch area per node).
    pub fn total_bending_energy_estimate(&self, area_per_node: f32) -> f32 {
        self.mean_curvatures
            .iter()
            .map(|&h| 0.5 * self.params.kappa * (2.0 * h - self.params.c0).powi(2) * area_per_node)
            .sum()
    }
}

pub fn new_helfrich_membrane(kappa: f32, sigma: f32) -> HelfrichMembrane {
    HelfrichMembrane::new(kappa, -0.5 * kappa, sigma, 0.0, 1.0)
}

pub fn new_membrane_patch(kappa: f32, sigma: f32) -> MembranePatch {
    MembranePatch::new(new_helfrich_membrane(kappa, sigma))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lipid_bilayer() {
        let m = HelfrichMembrane::lipid_bilayer();
        assert!(m.kappa > 0.0);
    }

    #[test]
    fn test_bending_energy_sphere() {
        /* Sphere with c1=c2=1/R, spontaneous curvature 0 */
        let m = HelfrichMembrane::new(4.114e-21, 0.0, 0.0, 0.0, 1.0);
        let e = m.sphere_bending_energy(10e-6);
        /* Should equal 4π * 2κ ≈ 8π κ */
        let expected = 8.0 * PI * m.kappa;
        assert!((e - expected).abs() / expected < 0.01);
    }

    #[test]
    fn test_tension_energy() {
        let m = HelfrichMembrane::new(1.0, 0.0, 2.0, 0.0, 1.0);
        assert!((m.tension_energy(3.0) - 6.0).abs() < 1e-5);
    }

    #[test]
    fn test_undulation_amplitude() {
        let m = new_helfrich_membrane(4.114e-21, 1e-6);
        let amp = m.undulation_amplitude_sq(4.114e-21, 1e-6);
        assert!(amp > 0.0);
    }

    #[test]
    fn test_effective_spring_constant() {
        let m = HelfrichMembrane::new(1.0, 0.0, 0.0, 0.0, 1.0);
        assert!((m.effective_spring_constant() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_membrane_patch_add_node() {
        let mut patch = new_membrane_patch(1e-19, 1e-5);
        patch.add_node([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1e4);
        patch.add_node([1e-6, 0.0, 0.0], [0.0, 0.0, 1.0], 1e4);
        assert_eq!(patch.node_count(), 2);
    }

    #[test]
    fn test_average_mean_curvature() {
        let mut patch = new_membrane_patch(1e-19, 1e-5);
        patch.add_node([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 2.0);
        patch.add_node([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 4.0);
        assert!((patch.average_mean_curvature() - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_equilibrium_sphere_radius() {
        let m = HelfrichMembrane::new(1e-19, 0.0, 1e-5, 0.0, 1.0);
        let r = m.equilibrium_sphere_radius(1.0);
        assert!(r > 0.0);
    }
}
