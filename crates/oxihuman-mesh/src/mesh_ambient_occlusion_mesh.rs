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
    /// Triangle index list (3 indices per triangle). When present, enables
    /// topology-aware 1-ring Laplacian smoothing; an empty list is valid and
    /// falls back to proximity-graph smoothing.
    pub indices: Vec<u32>,
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

    /// Smooth baked AO. Uses topological 1-ring Laplacian when `indices` are
    /// present, otherwise falls back to proximity-graph smoothing.
    pub fn smooth(&mut self, iterations: usize) {
        if self.indices.is_empty() {
            smooth_ao(&mut self.vertices);
        } else {
            let idx = self.indices.clone();
            smooth_ao_laplacian(&mut self.vertices, &idx, iterations);
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

/// Smooth AO values with one pass of geometric Laplacian smoothing over a
/// proximity graph: each vertex is averaged with neighbours within an adaptive
/// radius (derived from the mean nearest-neighbour distance). This is a genuine
/// local Laplacian (unlike a global-mean blend). Coincident points are treated
/// as mutual neighbours. Result stays in [0, 1].
pub fn smooth_ao(vertices: &mut [AoVertex]) {
    let n = vertices.len();
    if n < 2 {
        return;
    }

    // Adaptive radius: 2 × mean nearest-neighbour distance (floor to keep
    // coincident/degenerate inputs well-defined).
    let mut nn_sum = 0.0f32;
    #[allow(clippy::needless_range_loop)]
    for i in 0..n {
        let pi = vertices[i].position;
        let mut best = f32::INFINITY;
        for j in 0..n {
            if i == j {
                continue;
            }
            let pj = vertices[j].position;
            let dx = pi[0] - pj[0];
            let dy = pi[1] - pj[1];
            let dz = pi[2] - pj[2];
            let d2 = dx * dx + dy * dy + dz * dz;
            if d2 < best {
                best = d2;
            }
        }
        if best.is_finite() {
            nn_sum += best.sqrt();
        }
    }
    let mean_nn = nn_sum / n as f32;
    let radius = (2.0 * mean_nn).max(1e-4);
    let radius2 = radius * radius;

    let mut smoothed = vec![0.0f32; n];
    #[allow(clippy::needless_range_loop)]
    for i in 0..n {
        let pi = vertices[i].position;
        let mut acc = 0.0f32;
        let mut count = 0.0f32;
        for j in 0..n {
            if i == j {
                continue;
            }
            let pj = vertices[j].position;
            let dx = pi[0] - pj[0];
            let dy = pi[1] - pj[1];
            let dz = pi[2] - pj[2];
            if dx * dx + dy * dy + dz * dz <= radius2 {
                acc += vertices[j].ao;
                count += 1.0;
            }
        }
        smoothed[i] = if count > 0.0 {
            0.5 * vertices[i].ao + 0.5 * (acc / count)
        } else {
            vertices[i].ao
        };
    }
    for (v, &s) in vertices.iter_mut().zip(smoothed.iter()) {
        v.ao = s.clamp(0.0, 1.0);
    }
}

/// Topological 1-ring Laplacian AO smoothing using a triangle index buffer.
/// Builds vertex adjacency from `indices` (triples) and runs `iterations`
/// umbrella-operator passes: ao' = 0.5·ao + 0.5·mean(1-ring neighbours).
/// Falls back to leaving a vertex unchanged if it has no incident edges.
pub fn smooth_ao_laplacian(vertices: &mut [AoVertex], indices: &[u32], iterations: usize) {
    let n = vertices.len();
    if n == 0 || indices.len() < 3 {
        return;
    }
    // Build 1-ring adjacency (dedup via sorted Vec per vertex).
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for tri in indices.chunks_exact(3) {
        let a = tri[0] as usize;
        let b = tri[1] as usize;
        let c = tri[2] as usize;
        if a >= n || b >= n || c >= n {
            continue;
        }
        for &(u, v) in &[(a, b), (b, c), (c, a)] {
            if !adj[u].contains(&v) {
                adj[u].push(v);
            }
            if !adj[v].contains(&u) {
                adj[v].push(u);
            }
        }
    }
    for _ in 0..iterations {
        let mut next = vec![0.0f32; n];
        #[allow(clippy::needless_range_loop)]
        for i in 0..n {
            if adj[i].is_empty() {
                next[i] = vertices[i].ao;
            } else {
                let sum: f32 = adj[i].iter().map(|&j| vertices[j].ao).sum();
                let mean = sum / adj[i].len() as f32;
                next[i] = 0.5 * vertices[i].ao + 0.5 * mean;
            }
        }
        for (v, &s) in vertices.iter_mut().zip(next.iter()) {
            v.ao = s.clamp(0.0, 1.0);
        }
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

    #[test]
    fn test_smooth_ao_laplacian_diffuses_along_topology() {
        // Two triangles sharing an edge: a quad (0,1,2,3) with one hot vertex.
        let mut verts = vec![
            AoVertex { position: [0.0, 0.0, 0.0], normal: [0.0, 1.0, 0.0], ao: 1.0 },
            AoVertex { position: [1.0, 0.0, 0.0], normal: [0.0, 1.0, 0.0], ao: 0.0 },
            AoVertex { position: [1.0, 1.0, 0.0], normal: [0.0, 1.0, 0.0], ao: 0.0 },
            AoVertex { position: [0.0, 1.0, 0.0], normal: [0.0, 1.0, 0.0], ao: 0.0 },
        ];
        let indices = [0u32, 1, 2, 0, 2, 3];
        let before = verts[1].ao;
        smooth_ao_laplacian(&mut verts, &indices, 1);
        // Vertex 1 is adjacent to the hot vertex 0 → its AO must rise.
        assert!(verts[1].ao > before, "neighbour of hot vertex should increase");
        assert!(verts.iter().all(|v| (0.0..=1.0).contains(&v.ao)));
    }

    #[test]
    fn test_ao_mesh_smooth_uses_indices() {
        let mut mesh = AoMesh::new(AoBakeConfig::default());
        mesh.vertices = vec![
            AoVertex { position: [0.0, 0.0, 0.0], normal: [0.0, 1.0, 0.0], ao: 1.0 },
            AoVertex { position: [1.0, 0.0, 0.0], normal: [0.0, 1.0, 0.0], ao: 0.0 },
            AoVertex { position: [0.0, 1.0, 0.0], normal: [0.0, 1.0, 0.0], ao: 0.0 },
        ];
        mesh.indices = vec![0, 1, 2];
        mesh.smooth(2);
        assert!(mesh.vertices.iter().all(|v| (0.0..=1.0).contains(&v.ao)));
        // Topology present → vertex 1 (adjacent to hot vertex 0) increased from 0.
        assert!(mesh.vertices[1].ao > 0.0);
    }
}
