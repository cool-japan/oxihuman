// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! AO baking vertex data — stores per-vertex ambient occlusion values.

use std::f32::consts::PI;

/// Configuration for AO baking.
#[derive(Debug, Clone, Copy)]
pub struct AoBakeConfig {
    pub ray_count: u32,
    pub max_distance: f32,
    pub bias: f32,
}

impl Default for AoBakeConfig {
    fn default() -> Self {
        Self {
            ray_count: 64,
            max_distance: 1.0,
            bias: 0.001,
        }
    }
}

/// Per-vertex AO data.
#[derive(Debug, Clone, Copy)]
pub struct AoVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub ao: f32,
}

/// A mesh with baked ambient occlusion data.
#[derive(Debug, Default, Clone)]
pub struct AoMesh {
    pub vertices: Vec<AoVertex>,
    pub config: AoBakeConfig,
}

impl AoMesh {
    /// Creates a new AO mesh.
    pub fn new(config: AoBakeConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Returns the number of vertices.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the average AO value across all vertices.
    pub fn average_ao(&self) -> f32 {
        if self.vertices.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.vertices.iter().map(|v| v.ao).sum();
        sum / self.vertices.len() as f32
    }

    /// Sets the AO value for all vertices (for testing/reset).
    pub fn fill_ao(&mut self, value: f32) {
        let clamped = value.clamp(0.0, 1.0);
        for v in self.vertices.iter_mut() {
            v.ao = clamped;
        }
    }
}

/// Validates that all AO values are in [0, 1].
pub fn validate_ao_values(mesh: &AoMesh) -> bool {
    mesh.vertices.iter().all(|v| (0.0..=1.0).contains(&v.ao))
}

/// Generates hemisphere sample directions for AO rays.
pub fn hemisphere_samples(count: u32) -> Vec<[f32; 3]> {
    let mut samples = Vec::with_capacity(count as usize);
    for i in 0..count {
        let theta = (i as f32 / count as f32) * PI * 0.5;
        let phi = (i as f32 * 2.399_963) % (PI * 2.0); /* golden angle approximation */
        samples.push([
            theta.sin() * phi.cos(),
            theta.cos(),
            theta.sin() * phi.sin(),
        ]);
    }
    samples
}

/// Applies a contrast boost to AO values.
pub fn boost_ao_contrast(vertices: &mut [AoVertex], power: f32) {
    for v in vertices.iter_mut() {
        v.ao = v.ao.powf(power).clamp(0.0, 1.0);
    }
}

/// Smooths AO values by averaging with neighbors (simplified: global mean).
pub fn smooth_ao(vertices: &mut [AoVertex]) {
    if vertices.is_empty() {
        return;
    }
    let mean = vertices.iter().map(|v| v.ao).sum::<f32>() / vertices.len() as f32;
    for v in vertices.iter_mut() {
        v.ao = (v.ao + mean) * 0.5;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mesh_with_ao(n: usize, ao: f32) -> AoMesh {
        let mut mesh = AoMesh::new(AoBakeConfig::default());
        for _ in 0..n {
            mesh.vertices.push(AoVertex {
                position: [0.0; 3],
                normal: [0.0, 1.0, 0.0],
                ao,
            });
        }
        mesh
    }

    #[test]
    fn test_new_ao_mesh_empty() {
        /* New AO mesh should have zero vertices */
        assert_eq!(AoMesh::new(AoBakeConfig::default()).vertex_count(), 0);
    }

    #[test]
    fn test_average_ao_empty() {
        /* Empty mesh average should be 0 */
        assert_eq!(AoMesh::new(AoBakeConfig::default()).average_ao(), 0.0);
    }

    #[test]
    fn test_average_ao_uniform() {
        /* Uniform AO of 0.8 → average 0.8 */
        let mesh = make_mesh_with_ao(4, 0.8);
        assert!((mesh.average_ao() - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_validate_ao_valid() {
        /* AO in [0,1] should validate */
        let mesh = make_mesh_with_ao(3, 0.5);
        assert!(validate_ao_values(&mesh));
    }

    #[test]
    fn test_fill_ao_clamps() {
        /* fill_ao with value > 1 should clamp to 1 */
        let mut mesh = make_mesh_with_ao(2, 0.0);
        mesh.fill_ao(5.0);
        assert!(mesh.vertices.iter().all(|v| (0.0..=1.0).contains(&v.ao)));
    }

    #[test]
    fn test_hemisphere_samples_count() {
        /* Should return exactly the requested count */
        assert_eq!(hemisphere_samples(16).len(), 16);
    }

    #[test]
    fn test_boost_ao_contrast() {
        /* Boost with power 2 should reduce AO values < 1 */
        let mut verts = vec![AoVertex {
            position: [0.0; 3],
            normal: [0.0, 1.0, 0.0],
            ao: 0.5,
        }];
        boost_ao_contrast(&mut verts, 2.0);
        assert!(verts[0].ao < 0.5);
    }

    #[test]
    fn test_smooth_ao_changes_values() {
        /* Smoothing non-uniform AO should change at least one value */
        let mut verts = vec![
            AoVertex {
                position: [0.0; 3],
                normal: [0.0, 1.0, 0.0],
                ao: 0.0,
            },
            AoVertex {
                position: [0.0; 3],
                normal: [0.0, 1.0, 0.0],
                ao: 1.0,
            },
        ];
        let before = verts[0].ao;
        smooth_ao(&mut verts);
        assert!((verts[0].ao - before).abs() > f32::EPSILON);
    }

    #[test]
    fn test_smooth_ao_stays_in_range() {
        /* After smoothing all AO values must remain in [0,1] */
        let mut verts = vec![
            AoVertex {
                position: [0.0; 3],
                normal: [0.0, 1.0, 0.0],
                ao: 0.0,
            },
            AoVertex {
                position: [0.0; 3],
                normal: [0.0, 1.0, 0.0],
                ao: 1.0,
            },
        ];
        smooth_ao(&mut verts);
        assert!(verts.iter().all(|v| (0.0..=1.0).contains(&v.ao)));
    }

    #[test]
    fn test_default_config_ray_count() {
        /* Default should be 64 rays */
        assert_eq!(AoBakeConfig::default().ray_count, 64);
    }
}
