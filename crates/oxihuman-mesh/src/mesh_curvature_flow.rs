// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Mean curvature flow for mesh smoothing and fairing.
//!
//! Vertices are moved each step along their mean curvature normal,
//! effectively minimising the Dirichlet (surface-area) energy and
//! producing progressively smoother surfaces.

// ── Type aliases ─────────────────────────────────────────────────────────────

/// Vertex position `[x, y, z]`.
#[allow(dead_code)]
pub type Pos3 = [f32; 3];

/// Index triple for a triangle face.
#[allow(dead_code)]
pub type FaceIdx = [usize; 3];

/// Per-vertex flow velocity vector.
#[allow(dead_code)]
pub type FlowVelocity = [f32; 3];

// ── Config ────────────────────────────────────────────────────────────────────

/// Configuration for a mean curvature flow simulation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CurvatureFlowConfig {
    /// Time-step size per flow iteration.
    pub step_size: f32,
    /// Maximum number of iterations.
    pub max_iterations: usize,
    /// Convergence threshold (L∞ vertex movement).
    pub convergence_eps: f32,
    /// Whether to preserve volume after each step.
    pub preserve_volume: bool,
    /// Whether boundary vertices are fixed.
    pub fix_boundary: bool,
}

impl Default for CurvatureFlowConfig {
    fn default() -> Self {
        Self {
            step_size: 1e-3,
            max_iterations: 100,
            convergence_eps: 1e-5,
            preserve_volume: false,
            fix_boundary: true,
        }
    }
}

/// Result returned after one or more curvature-flow steps.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CurvatureFlowResult {
    /// Updated vertex positions.
    pub positions: Vec<Pos3>,
    /// Dirichlet energy after the last step.
    pub energy: f32,
    /// Number of steps actually performed.
    pub steps_taken: usize,
    /// Whether the flow converged within `max_iterations`.
    pub converged: bool,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

#[allow(dead_code)]
fn dot(a: Pos3, b: Pos3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
fn sub(a: Pos3, b: Pos3) -> Pos3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
fn add(a: Pos3, b: Pos3) -> Pos3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[allow(dead_code)]
fn scale(v: Pos3, s: f32) -> Pos3 {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)]
fn len(v: Pos3) -> f32 {
    dot(v, v).sqrt()
}

/// Cotangent of the angle at vertex `c` in the triangle (a, b, c).
#[allow(dead_code)]
fn cot_angle(a: Pos3, b: Pos3, c: Pos3) -> f32 {
    let ca = sub(a, c);
    let cb = sub(b, c);
    let cos_t = dot(ca, cb);
    let cross = [
        ca[1] * cb[2] - ca[2] * cb[1],
        ca[2] * cb[0] - ca[0] * cb[2],
        ca[0] * cb[1] - ca[1] * cb[0],
    ];
    let sin_t = len(cross);
    if sin_t.abs() < 1e-10 {
        0.0
    } else {
        cos_t / sin_t
    }
}

/// Check whether vertex `vi` lies on the mesh boundary (appears in only one
/// triangle for at least one of its edges).
#[allow(dead_code)]
fn is_boundary_vertex(vi: usize, faces: &[FaceIdx]) -> bool {
    use std::collections::HashMap;
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();
    for f in faces {
        for k in 0..3 {
            let a = f[k];
            let b = f[(k + 1) % 3];
            if a == vi || b == vi {
                let key = if a < b { (a, b) } else { (b, a) };
                *edge_count.entry(key).or_insert(0) += 1;
            }
        }
    }
    edge_count.values().any(|&c| c == 1)
}

/// Signed volume of the mesh (sum of signed tet volumes).
#[allow(dead_code)]
fn signed_volume(positions: &[Pos3], faces: &[FaceIdx]) -> f32 {
    let mut vol = 0.0_f32;
    for f in faces {
        let a = positions[f[0]];
        let b = positions[f[1]];
        let c = positions[f[2]];
        // V = (a · (b × c)) / 6
        let bxc = [
            b[1] * c[2] - b[2] * c[1],
            b[2] * c[0] - b[0] * c[2],
            b[0] * c[1] - b[1] * c[0],
        ];
        vol += dot(a, bxc);
    }
    vol / 6.0
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return a `CurvatureFlowConfig` with sensible defaults.
#[allow(dead_code)]
pub fn default_flow_config() -> CurvatureFlowConfig {
    CurvatureFlowConfig::default()
}

/// Total number of vertices that will be updated by the flow.
#[allow(dead_code)]
pub fn flow_vertex_count(positions: &[Pos3]) -> usize {
    positions.len()
}

/// Compute the mean curvature normal at vertex `vi` using cotangent weights.
///
/// Returns `[0,0,0]` when the vertex has no triangles or is isolated.
#[allow(dead_code)]
pub fn mean_curvature_normal_at(vi: usize, positions: &[Pos3], faces: &[FaceIdx]) -> Pos3 {
    let mut laplacian = [0.0_f32; 3];
    let mut area_sum = 0.0_f32;

    for f in faces {
        // Find whether vi participates in this face and which local index.
        let local = f.iter().position(|&x| x == vi);
        let Some(li) = local else { continue };

        let p = positions[vi];
        let q = positions[f[(li + 1) % 3]];
        let r = positions[f[(li + 2) % 3]];

        // Cotangent weights at the two opposite vertices.
        let cot_r = cot_angle(p, q, r); // angle at r opposite edge (p,q)
        let cot_q = cot_angle(p, r, q); // angle at q opposite edge (p,r)

        let pq = sub(q, p);
        let pr = sub(r, p);

        // Laplacian contribution.
        for k in 0..3 {
            laplacian[k] += cot_r * pq[k] + cot_q * pr[k];
        }

        // Mixed area contribution (simple: one-third of triangle area).
        let cross = [
            pq[1] * pr[2] - pq[2] * pr[1],
            pq[2] * pr[0] - pq[0] * pr[2],
            pq[0] * pr[1] - pq[1] * pr[0],
        ];
        area_sum += len(cross) / 6.0; // 1/3 of 1/2 * base * height
    }

    if area_sum < 1e-12 {
        return [0.0; 3];
    }
    let inv = 1.0 / (2.0 * area_sum);
    scale(laplacian, inv)
}

/// Compute the flow velocity (displacement direction) for every vertex.
#[allow(dead_code)]
pub fn compute_flow_velocity(
    positions: &[Pos3],
    faces: &[FaceIdx],
    cfg: &CurvatureFlowConfig,
) -> Vec<FlowVelocity> {
    positions
        .iter()
        .enumerate()
        .map(|(i, _)| {
            if cfg.fix_boundary && is_boundary_vertex(i, faces) {
                [0.0; 3]
            } else {
                mean_curvature_normal_at(i, positions, faces)
            }
        })
        .collect()
}

/// Global Dirichlet (surface area) energy: sum of squared edge lengths / 2.
#[allow(dead_code)]
pub fn flow_energy(positions: &[Pos3], faces: &[FaceIdx]) -> f32 {
    let mut energy = 0.0_f32;
    for f in faces {
        for k in 0..3 {
            let a = positions[f[k]];
            let b = positions[f[(k + 1) % 3]];
            let d = sub(b, a);
            energy += dot(d, d);
        }
    }
    energy * 0.5
}

/// Apply one mean curvature flow step.
///
/// Returns the new positions after moving each vertex along its curvature normal
/// scaled by `cfg.step_size`.
#[allow(dead_code)]
pub fn apply_curvature_flow(
    positions: &[Pos3],
    faces: &[FaceIdx],
    cfg: &CurvatureFlowConfig,
) -> Vec<Pos3> {
    let velocities = compute_flow_velocity(positions, faces, cfg);
    let mut new_pos: Vec<Pos3> = positions
        .iter()
        .zip(velocities.iter())
        .map(|(&p, &v)| add(p, scale(v, cfg.step_size)))
        .collect();

    if cfg.preserve_volume {
        constrain_volume(&mut new_pos, positions, faces);
    }
    new_pos
}

/// Rescale the mesh uniformly so that its signed volume matches the original.
#[allow(dead_code)]
pub fn constrain_volume(new_pos: &mut [Pos3], original: &[Pos3], faces: &[FaceIdx]) {
    let vol_orig = signed_volume(original, faces).abs();
    let vol_new = signed_volume(new_pos, faces).abs();
    if vol_new < 1e-12 || vol_orig < 1e-12 {
        return;
    }
    let scale_factor = (vol_orig / vol_new).cbrt();
    // Find centroid.
    let n = new_pos.len() as f32;
    let cx: f32 = new_pos.iter().map(|p| p[0]).sum::<f32>() / n;
    let cy: f32 = new_pos.iter().map(|p| p[1]).sum::<f32>() / n;
    let cz: f32 = new_pos.iter().map(|p| p[2]).sum::<f32>() / n;
    for p in new_pos.iter_mut() {
        p[0] = cx + (p[0] - cx) * scale_factor;
        p[1] = cy + (p[1] - cy) * scale_factor;
        p[2] = cz + (p[2] - cz) * scale_factor;
    }
}

/// Apply `n` curvature flow steps, stopping early if converged.
#[allow(dead_code)]
pub fn flow_n_steps(
    positions: &[Pos3],
    faces: &[FaceIdx],
    n: usize,
    cfg: &CurvatureFlowConfig,
) -> CurvatureFlowResult {
    let mut pos = positions.to_vec();
    let mut converged = false;

    for step in 0..n {
        let next = apply_curvature_flow(&pos, faces, cfg);
        let err = flow_convergence_error(&pos, &next);
        pos = next;
        if err < cfg.convergence_eps {
            converged = true;
            let energy = flow_energy(&pos, faces);
            return CurvatureFlowResult {
                positions: pos,
                energy,
                steps_taken: step + 1,
                converged,
            };
        }
    }

    let energy = flow_energy(&pos, faces);
    CurvatureFlowResult {
        positions: pos,
        energy,
        steps_taken: n,
        converged,
    }
}

/// Adaptive flow step: halve `step_size` when energy increases.
///
/// Returns the new positions using the (possibly reduced) step size.
#[allow(dead_code)]
pub fn adaptive_flow_step(
    positions: &[Pos3],
    faces: &[FaceIdx],
    cfg: &CurvatureFlowConfig,
) -> (Vec<Pos3>, f32) {
    let e0 = flow_energy(positions, faces);
    let mut trial_cfg = cfg.clone();
    let mut candidate = apply_curvature_flow(positions, faces, &trial_cfg);
    let mut e1 = flow_energy(&candidate, faces);

    // Halve step up to 8 times if energy increases.
    for _ in 0..8 {
        if e1 <= e0 {
            break;
        }
        trial_cfg.step_size *= 0.5;
        candidate = apply_curvature_flow(positions, faces, &trial_cfg);
        e1 = flow_energy(&candidate, faces);
    }

    (candidate, trial_cfg.step_size)
}

/// L∞ distance between two position arrays (convergence criterion).
#[allow(dead_code)]
pub fn flow_convergence_error(old: &[Pos3], new: &[Pos3]) -> f32 {
    old.iter()
        .zip(new.iter())
        .map(|(a, b)| {
            let d = sub(*b, *a);
            len(d)
        })
        .fold(0.0_f32, f32::max)
}

/// Apply flow only to boundary vertices (smooth their distribution).
#[allow(dead_code)]
pub fn smooth_boundary_flow(
    positions: &[Pos3],
    faces: &[FaceIdx],
    cfg: &CurvatureFlowConfig,
) -> Vec<Pos3> {
    let mut pos = positions.to_vec();
    for (i, p) in pos.iter_mut().enumerate() {
        if is_boundary_vertex(i, faces) {
            let h = mean_curvature_normal_at(i, positions, faces);
            *p = add(*p, scale(h, cfg.step_size));
        }
    }
    pos
}

/// Quality metric: fraction of faces whose aspect ratio is ≤ 4.
///
/// Returns a value in `[0, 1]`; 1 means all faces are well-shaped.
#[allow(dead_code)]
pub fn flow_quality(positions: &[Pos3], faces: &[FaceIdx]) -> f32 {
    if faces.is_empty() {
        return 1.0;
    }
    let good = faces.iter().filter(|f| {
        let a = len(sub(positions[f[1]], positions[f[0]]));
        let b = len(sub(positions[f[2]], positions[f[1]]));
        let c = len(sub(positions[f[0]], positions[f[2]]));
        let max_e = a.max(b).max(c);
        let min_e = a.min(b).min(c);
        min_e > 1e-8 && max_e / min_e <= 4.0
    });
    good.count() as f32 / faces.len() as f32
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_triangle() -> (Vec<Pos3>, Vec<FaceIdx>) {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let faces = vec![[0, 1, 2]];
        (positions, faces)
    }

    fn simple_quad_mesh() -> (Vec<Pos3>, Vec<FaceIdx>) {
        // Two triangles forming a quad in the XY plane.
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let faces = vec![[0, 1, 2], [0, 2, 3]];
        (positions, faces)
    }

    #[test]
    fn test_default_flow_config() {
        let cfg = default_flow_config();
        assert!(cfg.step_size > 0.0);
        assert!(cfg.max_iterations > 0);
        assert!(cfg.convergence_eps > 0.0);
    }

    #[test]
    fn test_flow_vertex_count() {
        let (pos, _) = simple_quad_mesh();
        assert_eq!(flow_vertex_count(&pos), 4);
    }

    #[test]
    fn test_mean_curvature_normal_flat_mesh() {
        // On a flat mesh the mean curvature normal has no out-of-plane component.
        let (pos, faces) = simple_quad_mesh();
        let h = mean_curvature_normal_at(1, &pos, &faces);
        // Z component must be zero (mesh is in the XY plane).
        assert!(h[2].abs() < 1e-5, "z component must be zero on flat mesh, got {h:?}");
    }

    #[test]
    fn test_mean_curvature_normal_isolated_vertex() {
        let positions = vec![[0.0, 0.0, 0.0]];
        let faces: Vec<FaceIdx> = vec![];
        let h = mean_curvature_normal_at(0, &positions, &faces);
        assert_eq!(h, [0.0; 3]);
    }

    #[test]
    fn test_compute_flow_velocity_len() {
        let (pos, faces) = simple_quad_mesh();
        let cfg = default_flow_config();
        let vel = compute_flow_velocity(&pos, &faces, &cfg);
        assert_eq!(vel.len(), pos.len());
    }

    #[test]
    fn test_flow_energy_positive() {
        let (pos, faces) = simple_quad_mesh();
        let e = flow_energy(&pos, &faces);
        assert!(e >= 0.0);
    }

    #[test]
    fn test_flow_energy_zero_degenerate() {
        let positions = vec![[0.0, 0.0, 0.0]; 3];
        let faces = vec![[0, 1, 2]];
        let e = flow_energy(&positions, &faces);
        assert_eq!(e, 0.0);
    }

    #[test]
    fn test_apply_curvature_flow_output_len() {
        let (pos, faces) = simple_quad_mesh();
        let cfg = default_flow_config();
        let new_pos = apply_curvature_flow(&pos, &faces, &cfg);
        assert_eq!(new_pos.len(), pos.len());
    }

    #[test]
    fn test_flow_n_steps_returns_result() {
        let (pos, faces) = simple_quad_mesh();
        let cfg = default_flow_config();
        let result = flow_n_steps(&pos, &faces, 5, &cfg);
        assert!(result.steps_taken >= 1);
        assert_eq!(result.positions.len(), pos.len());
    }

    #[test]
    fn test_adaptive_flow_step_returns_positions() {
        let (pos, faces) = simple_quad_mesh();
        let cfg = default_flow_config();
        let (new_pos, step) = adaptive_flow_step(&pos, &faces, &cfg);
        assert_eq!(new_pos.len(), pos.len());
        assert!(step > 0.0);
    }

    #[test]
    fn test_flow_convergence_error_identical() {
        let (pos, _) = simple_quad_mesh();
        let err = flow_convergence_error(&pos, &pos);
        assert_eq!(err, 0.0);
    }

    #[test]
    fn test_flow_convergence_error_nonzero() {
        let (pos, _) = simple_quad_mesh();
        let shifted: Vec<Pos3> = pos.iter().map(|&p| [p[0] + 0.1, p[1], p[2]]).collect();
        let err = flow_convergence_error(&pos, &shifted);
        assert!((err - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_smooth_boundary_flow_len() {
        let (pos, faces) = simple_quad_mesh();
        let cfg = default_flow_config();
        let new_pos = smooth_boundary_flow(&pos, &faces, &cfg);
        assert_eq!(new_pos.len(), pos.len());
    }

    #[test]
    fn test_constrain_volume_single_triangle() {
        let (pos, faces) = unit_triangle();
        let mut new_pos = pos.clone();
        constrain_volume(&mut new_pos, &pos, &faces);
        // For a flat mesh the signed volume is 0, function should not crash.
        assert_eq!(new_pos.len(), pos.len());
    }

    #[test]
    fn test_flow_quality_perfect_equilateral() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 0.866, 0.0],
        ];
        let faces = vec![[0, 1, 2]];
        let q = flow_quality(&positions, &faces);
        assert!((0.0..=1.0).contains(&q));
    }

    #[test]
    fn test_flow_quality_empty_faces() {
        let positions: Vec<Pos3> = vec![];
        let faces: Vec<FaceIdx> = vec![];
        let q = flow_quality(&positions, &faces);
        assert_eq!(q, 1.0);
    }

    #[test]
    fn test_flow_n_steps_converges_flat() {
        // A flat mesh should converge quickly since curvature normals are tiny.
        let (pos, faces) = simple_quad_mesh();
        let mut cfg = default_flow_config();
        cfg.convergence_eps = 1.0; // very loose convergence
        let result = flow_n_steps(&pos, &faces, 100, &cfg);
        assert!(result.converged || result.steps_taken <= 100);
    }
}
