// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Mean curvature flow and Laplacian surface smoothing variants.
//!
//! Implements Laplacian, mean-curvature, and Taubin smoothing flows
//! with optional volume preservation.

use super::mesh::MeshBuffers;

#[allow(dead_code)]
pub struct FlowConfig {
    pub dt: f32,
    pub steps: u32,
    pub method: FlowMethod,
    pub preserve_volume: bool,
}

impl Default for FlowConfig {
    fn default() -> Self {
        FlowConfig {
            dt: 0.001,
            steps: 10,
            method: FlowMethod::Laplacian,
            preserve_volume: false,
        }
    }
}

#[allow(dead_code)]
pub enum FlowMethod {
    MeanCurvature,
    Laplacian,
    Taubin,
}

#[allow(dead_code)]
pub struct FlowResult {
    pub positions: Vec<[f32; 3]>,
    pub initial_volume: f32,
    pub final_volume: f32,
    pub steps_run: u32,
}

// ---------- small vec3 helpers ----------

fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn centroid(positions: &[[f32; 3]]) -> [f32; 3] {
    if positions.is_empty() {
        return [0.0; 3];
    }
    let mut c = [0.0f32; 3];
    for p in positions {
        c = vec3_add(c, *p);
    }
    vec3_scale(c, 1.0 / positions.len() as f32)
}

// ---------- adjacency ----------

/// Build an adjacency list from triangle indices (undirected, no duplicates).
#[allow(dead_code)]
pub fn build_adjacency_list(indices: &[u32], n: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        for &(i, j) in &[(a, b), (b, a), (b, c), (c, b), (a, c), (c, a)] {
            if i < n && j < n && !adj[i].contains(&j) {
                adj[i].push(j);
            }
        }
    }
    adj
}

// ---------- Laplacian ----------

/// Umbrella-operator Laplacian: L(v_i) = mean_neighbors - v_i.
#[allow(dead_code)]
pub fn vertex_laplacian(positions: &[[f32; 3]], adj: &[Vec<usize>]) -> Vec<[f32; 3]> {
    positions
        .iter()
        .enumerate()
        .map(|(i, &p)| {
            let neighbors = &adj[i];
            if neighbors.is_empty() {
                return [0.0; 3];
            }
            let mut mean = [0.0f32; 3];
            for &j in neighbors {
                mean = vec3_add(mean, positions[j]);
            }
            mean = vec3_scale(mean, 1.0 / neighbors.len() as f32);
            vec3_sub(mean, p)
        })
        .collect()
}

/// Laplacian smoothing step: p_new = p + dt * L(p).
#[allow(dead_code)]
pub fn laplacian_step(positions: &[[f32; 3]], indices: &[u32], dt: f32) -> Vec<[f32; 3]> {
    let n = positions.len();
    let adj = build_adjacency_list(indices, n);
    let lap = vertex_laplacian(positions, &adj);
    positions
        .iter()
        .zip(lap.iter())
        .map(|(&p, &l)| vec3_add(p, vec3_scale(l, dt)))
        .collect()
}

/// Mean-curvature flow step: move vertices along the (approximate) mean curvature normal.
/// Approximation: mean curvature normal ≈ Laplacian vector (umbrella operator).
#[allow(dead_code)]
pub fn mean_curvature_step(
    positions: &[[f32; 3]],
    indices: &[u32],
    _normals: &[[f32; 3]],
    dt: f32,
) -> Vec<[f32; 3]> {
    // For a uniform Laplacian, ΔP ≈ 2*H*N, so moving along Laplacian is proportional
    // to moving along mean curvature normal.
    laplacian_step(positions, indices, dt)
}

/// One Taubin smoothing iteration: forward Laplacian(λ) then backward Laplacian(−μ).
/// λ = 0.5, μ = 0.53 are standard values that reduce shrinkage.
#[allow(dead_code)]
pub fn taubin_step(positions: &[[f32; 3]], indices: &[u32], lambda: f32, mu: f32) -> Vec<[f32; 3]> {
    let after_lambda = laplacian_step(positions, indices, lambda);
    laplacian_step(&after_lambda, indices, -mu)
}

// ---------- volume ----------

/// Signed volume of a mesh (via divergence theorem).
#[allow(dead_code)]
pub fn mesh_volume_from_positions(positions: &[[f32; 3]], indices: &[u32]) -> f32 {
    let mut vol = 0.0f32;
    for tri in indices.chunks_exact(3) {
        let a = positions[tri[0] as usize];
        let b = positions[tri[1] as usize];
        let c = positions[tri[2] as usize];
        // Signed volume contribution: (a · (b × c)) / 6
        let bxc = [
            b[1] * c[2] - b[2] * c[1],
            b[2] * c[0] - b[0] * c[2],
            b[0] * c[1] - b[1] * c[0],
        ];
        vol += vec3_dot(a, bxc);
    }
    vol / 6.0
}

/// Rescale all positions from the centroid so that the new volume equals target_volume.
#[allow(dead_code)]
pub fn rescale_to_volume(positions: &mut [[f32; 3]], target_volume: f32, current_volume: f32) {
    let ratio = target_volume / current_volume;
    if ratio <= 0.0 || !ratio.is_finite() {
        return;
    }
    let scale = ratio.abs().cbrt();
    let c = centroid(positions);
    for p in positions.iter_mut() {
        let d = vec3_sub(*p, c);
        *p = vec3_add(c, vec3_scale(d, scale));
    }
}

// ---------- flow_mesh ----------

/// Run mean-curvature / Laplacian / Taubin flow on a MeshBuffers.
#[allow(dead_code)]
pub fn flow_mesh(mesh: &MeshBuffers, cfg: &FlowConfig) -> FlowResult {
    let initial_volume = mesh_volume_from_positions(&mesh.positions, &mesh.indices);
    let mut pos: Vec<[f32; 3]> = mesh.positions.clone();

    for _ in 0..cfg.steps {
        pos = match cfg.method {
            FlowMethod::MeanCurvature => {
                mean_curvature_step(&pos, &mesh.indices, &mesh.normals, cfg.dt)
            }
            FlowMethod::Laplacian => laplacian_step(&pos, &mesh.indices, cfg.dt),
            FlowMethod::Taubin => taubin_step(&pos, &mesh.indices, 0.5, 0.53),
        };

        if cfg.preserve_volume {
            let cur_vol = mesh_volume_from_positions(&pos, &mesh.indices);
            if cur_vol.abs() > 1e-12 && initial_volume.abs() > 1e-12 {
                rescale_to_volume(&mut pos, initial_volume, cur_vol);
            }
        }
    }

    let final_volume = mesh_volume_from_positions(&pos, &mesh.indices);
    FlowResult {
        positions: pos,
        initial_volume,
        final_volume,
        steps_run: cfg.steps,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn tetra_mesh() -> MeshBuffers {
        // Simple tetrahedron
        let src = MB {
            positions: vec![
                [0.0f32, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0]; 4],
            indices: vec![0, 1, 2, 0, 1, 3, 1, 2, 3, 0, 2, 3],
            has_suit: false,
        };
        MeshBuffers::from_morph(src)
    }

    fn flat_quad_positions() -> Vec<[f32; 3]> {
        vec![
            [-1.0f32, 0.5, 0.0],
            [1.0, 0.5, 0.0],
            [1.0, -0.5, 0.0],
            [-1.0, -0.5, 0.0],
            // center vertex displaced
            [0.0, 0.0, 0.5],
        ]
    }

    fn flat_quad_indices() -> Vec<u32> {
        vec![0, 1, 4, 1, 2, 4, 2, 3, 4, 3, 0, 4]
    }

    #[test]
    fn test_build_adjacency_list_count() {
        let indices = vec![0u32, 1, 2, 0, 2, 3];
        let adj = build_adjacency_list(&indices, 4);
        assert_eq!(adj.len(), 4);
        // vertex 0 should be adjacent to 1, 2, 3
        assert_eq!(adj[0].len(), 3, "vertex 0 neighbors: {:?}", adj[0]);
    }

    #[test]
    fn test_build_adjacency_list_no_duplicates() {
        let indices = vec![0u32, 1, 2, 0, 1, 3];
        let adj = build_adjacency_list(&indices, 4);
        // Edge 0-1 appears in both triangles, must not be duplicated
        let count_1_in_0 = adj[0].iter().filter(|&&j| j == 1).count();
        assert_eq!(count_1_in_0, 1, "no duplicate neighbors");
    }

    #[test]
    fn test_vertex_laplacian_at_centroid_is_zero() {
        // All vertices in a regular pattern where the centroid is at each vertex
        let positions = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [-1.0, 0.0, 0.0]];
        // Make vertex 0 connected to 1 and 2 (which are symmetric)
        let adj = vec![vec![1usize, 2], vec![0, 2], vec![0, 1]];
        let lap = vertex_laplacian(&positions, &adj);
        // Vertex 0 mean_neighbors = (1 + -1) / 2 = 0 → Laplacian = 0 - 0 = 0
        assert!(
            lap[0][0].abs() < 1e-5,
            "Laplacian x should be 0, got {}",
            lap[0][0]
        );
        assert!(
            lap[0][1].abs() < 1e-5,
            "Laplacian y should be 0, got {}",
            lap[0][1]
        );
        assert!(
            lap[0][2].abs() < 1e-5,
            "Laplacian z should be 0, got {}",
            lap[0][2]
        );
    }

    #[test]
    fn test_laplacian_step_reduces_roughness() {
        let pos = flat_quad_positions();
        let idx = flat_quad_indices();
        // Center vertex (index 4) is displaced upward; smoothing should bring it toward 0
        let smoothed = laplacian_step(&pos, &idx, 0.5);
        // The displaced center should move toward z=0
        assert!(
            smoothed[4][2].abs() < pos[4][2].abs(),
            "center z should decrease: before={}, after={}",
            pos[4][2],
            smoothed[4][2]
        );
    }

    #[test]
    fn test_taubin_preserves_better_than_plain_laplacian() {
        let pos = flat_quad_positions();
        let idx = flat_quad_indices();

        // Plain Laplacian shrinks the mesh
        let mut lap_pos = pos.clone();
        for _ in 0..20 {
            lap_pos = laplacian_step(&lap_pos, &idx, 0.5);
        }

        // Taubin resists shrinkage more
        let mut tau_pos = pos.clone();
        for _ in 0..20 {
            tau_pos = taubin_step(&tau_pos, &idx, 0.5, 0.53);
        }

        // Compute bounding box diagonal for each — Taubin should remain larger
        let lap_max_z = lap_pos.iter().map(|p| p[2].abs()).fold(0.0f32, f32::max);
        let tau_max_z = tau_pos.iter().map(|p| p[2].abs()).fold(0.0f32, f32::max);

        // Taubin should keep more volume (max z ≥ laplacian's)
        assert!(
            tau_max_z >= lap_max_z - 1e-4,
            "Taubin max_z={tau_max_z} should be >= Laplacian max_z={lap_max_z}"
        );
    }

    #[test]
    fn test_flow_mesh_no_nan() {
        let mesh = tetra_mesh();
        let cfg = FlowConfig {
            dt: 0.01,
            steps: 5,
            method: FlowMethod::Laplacian,
            preserve_volume: false,
        };
        let result = flow_mesh(&mesh, &cfg);
        for p in &result.positions {
            for &c in p {
                assert!(!c.is_nan(), "NaN in flow result");
            }
        }
    }

    #[test]
    fn test_flow_mesh_steps_run() {
        let mesh = tetra_mesh();
        let cfg = FlowConfig {
            dt: 0.001,
            steps: 7,
            method: FlowMethod::Taubin,
            preserve_volume: false,
        };
        let result = flow_mesh(&mesh, &cfg);
        assert_eq!(result.steps_run, 7);
    }

    #[test]
    fn test_flow_mesh_initial_volume_matches() {
        let mesh = tetra_mesh();
        let cfg = FlowConfig::default();
        let result = flow_mesh(&mesh, &cfg);
        let manual_vol = mesh_volume_from_positions(&mesh.positions, &mesh.indices);
        assert!(
            (result.initial_volume - manual_vol).abs() < 1e-5,
            "initial_volume mismatch: {} vs {}",
            result.initial_volume,
            manual_vol
        );
    }

    #[test]
    fn test_volume_preserved_with_preserve_volume_flag() {
        let mesh = tetra_mesh();
        let cfg = FlowConfig {
            dt: 0.01,
            steps: 10,
            method: FlowMethod::Laplacian,
            preserve_volume: true,
        };
        let result = flow_mesh(&mesh, &cfg);
        // With preserve_volume the rescale step should keep volume within 5%
        let vol_ratio = (result.final_volume / result.initial_volume).abs();
        assert!(
            (vol_ratio - 1.0).abs() < 0.05,
            "volume not preserved: initial={}, final={}, ratio={}",
            result.initial_volume,
            result.final_volume,
            vol_ratio
        );
    }

    #[test]
    fn test_mesh_volume_tetrahedron() {
        // Regular tetrahedron with edge ~1 should have volume ~ 1/12
        let positions = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, 0.5, 1.0],
        ];
        let indices = vec![0u32, 1, 2, 0, 1, 3, 1, 2, 3, 0, 2, 3];
        let vol = mesh_volume_from_positions(&positions, &indices).abs();
        assert!(vol > 0.0, "volume should be positive");
        assert!(vol < 1.0, "volume of unit tetrahedron should be < 1");
    }

    #[test]
    fn test_rescale_to_volume_scales_correctly() {
        let mut positions = vec![
            [1.0f32, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
        ];
        rescale_to_volume(&mut positions, 2.0, 1.0);
        // All vertices should be scaled by 2^(1/3) from centroid (which is at origin)
        let expected_scale = 2.0f32.cbrt();
        assert!(
            (positions[0][0] - expected_scale).abs() < 1e-5,
            "scaled x should be {expected_scale}, got {}",
            positions[0][0]
        );
    }

    #[test]
    fn test_laplacian_step_result_length() {
        let pos = flat_quad_positions();
        let idx = flat_quad_indices();
        let result = laplacian_step(&pos, &idx, 0.1);
        assert_eq!(result.len(), pos.len(), "output length must match input");
    }

    #[test]
    fn test_mean_curvature_step_no_nan() {
        let pos = flat_quad_positions();
        let idx = flat_quad_indices();
        let normals = vec![[0.0f32, 0.0, 1.0]; pos.len()];
        let result = mean_curvature_step(&pos, &idx, &normals, 0.01);
        for p in &result {
            for &c in p {
                assert!(!c.is_nan(), "NaN in mean_curvature_step result");
            }
        }
    }
}
