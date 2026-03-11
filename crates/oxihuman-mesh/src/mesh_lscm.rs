// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Least Squares Conformal Maps (LSCM) UV parameterization.

#[allow(dead_code)]
pub struct LscmConfig {
    pub pin_vertex_a: usize,
    pub pin_uv_a: [f32; 2],
    pub pin_vertex_b: usize,
    pub pin_uv_b: [f32; 2],
    pub iterations: u32,
}

impl Default for LscmConfig {
    fn default() -> Self {
        Self {
            pin_vertex_a: 0,
            pin_uv_a: [0.0, 0.0],
            pin_vertex_b: 1,
            pin_uv_b: [1.0, 0.0],
            iterations: 50,
        }
    }
}

#[allow(dead_code)]
pub struct LscmResult {
    pub uvs: Vec<[f32; 2]>,
    pub conformal_energy: f32,
    pub iterations_run: u32,
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn norm3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l < 1e-12 {
        [0.0, 0.0, 0.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

/// Compute tangent and bitangent vectors for a triangle.
#[allow(dead_code)]
pub fn compute_local_frame(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let e1 = sub3(p1, p0);
    let e2 = sub3(p2, p0);
    let tangent = norm3(e1);
    let normal = norm3(cross3(e1, e2));
    let bitangent = norm3(cross3(normal, tangent));
    (tangent, bitangent)
}

/// Project a 3D point onto a UV coordinate using a local frame.
#[allow(dead_code)]
pub fn project_to_uv(
    p: [f32; 3],
    origin: [f32; 3],
    tangent: [f32; 3],
    bitangent: [f32; 3],
) -> [f32; 2] {
    let v = sub3(p, origin);
    [dot3(v, tangent), dot3(v, bitangent)]
}

/// Signed area of a UV triangle.
#[allow(dead_code)]
pub fn uv_area(uv0: [f32; 2], uv1: [f32; 2], uv2: [f32; 2]) -> f32 {
    0.5 * ((uv1[0] - uv0[0]) * (uv2[1] - uv0[1]) - (uv2[0] - uv0[0]) * (uv1[1] - uv0[1]))
}

/// Conformal energy for a triangle: measures angle distortion between 3D and UV.
#[allow(dead_code)]
pub fn triangle_conformal_energy(
    uv0: [f32; 2],
    uv1: [f32; 2],
    uv2: [f32; 2],
    p0: [f32; 3],
    p1: [f32; 3],
    p2: [f32; 3],
) -> f32 {
    let area_3d = {
        let e1 = sub3(p1, p0);
        let e2 = sub3(p2, p0);
        len3(cross3(e1, e2)) * 0.5
    };
    let area_uv = uv_area(uv0, uv1, uv2).abs();
    if area_3d < 1e-12 || area_uv < 1e-12 {
        return 0.0;
    }
    // Scale factor between 3D and UV
    let scale = (area_uv / area_3d).sqrt();
    // Jacobian stretch: difference from identity (angle-preserving = conformal)
    let du1 = uv1[0] - uv0[0];
    let dv1 = uv1[1] - uv0[1];
    let du2 = uv2[0] - uv0[0];
    let dv2 = uv2[1] - uv0[1];
    // Frobenius norm of (J - scale*I) gives angle distortion measure
    let j11 = du1 / scale;
    let j12 = du2 / scale;
    let j21 = dv1 / scale;
    let j22 = dv2 / scale;
    let diff = (j11 - 1.0).powi(2) + j12.powi(2) + j21.powi(2) + (j22 - 1.0).powi(2);
    diff * area_3d
}

/// Normalize UVs to fit within [0, 1]² range.
#[allow(dead_code)]
pub fn normalize_uvs_lscm(uvs: &mut [[f32; 2]]) {
    if uvs.is_empty() {
        return;
    }
    let mut min = [f32::MAX; 2];
    let mut max = [f32::MIN; 2];
    for uv in uvs.iter() {
        min[0] = min[0].min(uv[0]);
        min[1] = min[1].min(uv[1]);
        max[0] = max[0].max(uv[0]);
        max[1] = max[1].max(uv[1]);
    }
    let range = [(max[0] - min[0]).max(1e-12), (max[1] - min[1]).max(1e-12)];
    for uv in uvs.iter_mut() {
        uv[0] = (uv[0] - min[0]) / range[0];
        uv[1] = (uv[1] - min[1]) / range[1];
    }
}

/// Mean UV stretch metric: average ratio of UV area to 3D area.
#[allow(dead_code)]
pub fn uv_stretch_metric(uvs: &[[f32; 2]], positions: &[[f32; 3]], indices: &[u32]) -> f32 {
    let n_tri = indices.len() / 3;
    if n_tri == 0 {
        return 0.0;
    }
    let mut total = 0.0f32;
    let mut count = 0u32;
    for ti in 0..n_tri {
        let ia = indices[ti * 3] as usize;
        let ib = indices[ti * 3 + 1] as usize;
        let ic = indices[ti * 3 + 2] as usize;
        if ia >= uvs.len() || ib >= uvs.len() || ic >= uvs.len() {
            continue;
        }
        if ia >= positions.len() || ib >= positions.len() || ic >= positions.len() {
            continue;
        }
        let area_3d = {
            let e1 = sub3(positions[ib], positions[ia]);
            let e2 = sub3(positions[ic], positions[ia]);
            len3(cross3(e1, e2)) * 0.5
        };
        let area_uv = uv_area(uvs[ia], uvs[ib], uvs[ic]).abs();
        if area_3d > 1e-12 {
            total += area_uv / area_3d;
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
}

/// Build adjacency list: for each vertex, which triangles contain it.
fn build_vertex_triangles(n_verts: usize, indices: &[u32]) -> Vec<Vec<usize>> {
    let mut vt: Vec<Vec<usize>> = vec![Vec::new(); n_verts];
    let n_tri = indices.len() / 3;
    for ti in 0..n_tri {
        for k in 0..3 {
            let vi = indices[ti * 3 + k] as usize;
            if vi < n_verts {
                vt[vi].push(ti);
            }
        }
    }
    vt
}

/// LSCM parameterization using iterative Gauss-Seidel approach.
#[allow(dead_code)]
pub fn lscm_parameterize(positions: &[[f32; 3]], indices: &[u32], cfg: &LscmConfig) -> LscmResult {
    let n_verts = positions.len();
    let n_tri = indices.len() / 3;
    if n_verts == 0 || n_tri == 0 {
        return LscmResult {
            uvs: vec![],
            conformal_energy: 0.0,
            iterations_run: 0,
        };
    }

    // Initialize UVs: pin the two pinned vertices, initialize others by projection.
    let mut uvs: Vec<[f32; 2]> = vec![[0.0, 0.0]; n_verts];

    // Use a global frame from the pinned vertices to initialize.
    let pa = positions[cfg.pin_vertex_a.min(n_verts - 1)];
    let pb = positions[cfg.pin_vertex_b.min(n_verts - 1)];
    let global_tangent = {
        let d = sub3(pb, pa);
        let l = len3(d);
        if l < 1e-12 {
            [1.0, 0.0, 0.0]
        } else {
            [d[0] / l, d[1] / l, d[2] / l]
        }
    };
    let up = if global_tangent[2].abs() < 0.9 {
        [0.0, 0.0, 1.0f32]
    } else {
        [1.0, 0.0, 0.0]
    };
    let global_bitangent = norm3(cross3(global_tangent, up));

    for (i, p) in positions.iter().enumerate() {
        uvs[i] = project_to_uv(*p, pa, global_tangent, global_bitangent);
    }

    // Override pinned vertices.
    let pin_a = cfg.pin_vertex_a.min(n_verts - 1);
    let pin_b = cfg.pin_vertex_b.min(n_verts - 1);
    uvs[pin_a] = cfg.pin_uv_a;
    uvs[pin_b] = cfg.pin_uv_b;

    let vertex_triangles = build_vertex_triangles(n_verts, indices);

    // Gauss-Seidel iterations: for each free vertex, set UV to weighted average
    // of projected UVs from its neighboring triangles' local frames.
    for _ in 0..cfg.iterations {
        for vi in 0..n_verts {
            if vi == pin_a || vi == pin_b {
                continue;
            }
            let mut sum_u = 0.0f32;
            let mut sum_v = 0.0f32;
            let mut weight_sum = 0.0f32;

            for &ti in &vertex_triangles[vi] {
                let ia = indices[ti * 3] as usize;
                let ib = indices[ti * 3 + 1] as usize;
                let ic = indices[ti * 3 + 2] as usize;

                let (tangent, bitangent) =
                    compute_local_frame(positions[ia], positions[ib], positions[ic]);

                // Compute UV of vi by projecting into this triangle's local frame.
                // Use the opposite vertices' current UVs as anchors.
                let other_verts: Vec<usize> =
                    [ia, ib, ic].iter().filter(|&&v| v != vi).copied().collect();

                if other_verts.len() >= 2 {
                    let origin_3d = positions[other_verts[0]];
                    let origin_uv = uvs[other_verts[0]];
                    let predicted = project_to_uv(positions[vi], origin_3d, tangent, bitangent);
                    let w = 1.0f32;
                    sum_u += w * (origin_uv[0] + predicted[0]);
                    sum_v += w * (origin_uv[1] + predicted[1]);
                    weight_sum += w;
                }
            }

            if weight_sum > 1e-12 {
                uvs[vi] = [sum_u / weight_sum, sum_v / weight_sum];
            }
        }
        // Re-apply pin constraints.
        uvs[pin_a] = cfg.pin_uv_a;
        uvs[pin_b] = cfg.pin_uv_b;
    }

    // Compute conformal energy.
    let conformal_energy: f32 = (0..n_tri)
        .map(|ti| {
            let ia = indices[ti * 3] as usize;
            let ib = indices[ti * 3 + 1] as usize;
            let ic = indices[ti * 3 + 2] as usize;
            if ia < n_verts && ib < n_verts && ic < n_verts {
                triangle_conformal_energy(
                    uvs[ia],
                    uvs[ib],
                    uvs[ic],
                    positions[ia],
                    positions[ib],
                    positions[ic],
                )
            } else {
                0.0
            }
        })
        .sum();

    LscmResult {
        uvs,
        conformal_energy: conformal_energy.max(0.0),
        iterations_run: cfg.iterations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_quad_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let positions = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let indices = vec![0u32, 1, 2, 0, 2, 3];
        (positions, indices)
    }

    #[test]
    fn compute_local_frame_orthogonal() {
        let p0 = [0.0f32, 0.0, 0.0];
        let p1 = [1.0, 0.0, 0.0];
        let p2 = [0.0, 1.0, 0.0];
        let (t, b) = compute_local_frame(p0, p1, p2);
        let dot = t[0] * b[0] + t[1] * b[1] + t[2] * b[2];
        assert!(
            dot.abs() < 1e-5,
            "tangent and bitangent should be orthogonal, dot={dot}"
        );
    }

    #[test]
    fn compute_local_frame_unit_length() {
        let p0 = [0.0f32, 0.0, 0.0];
        let p1 = [1.0, 0.0, 0.0];
        let p2 = [0.0, 1.0, 0.0];
        let (t, b) = compute_local_frame(p0, p1, p2);
        let lt = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt();
        let lb = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
        assert!((lt - 1.0).abs() < 1e-5, "tangent not unit length: {lt}");
        assert!((lb - 1.0).abs() < 1e-5, "bitangent not unit length: {lb}");
    }

    #[test]
    fn project_to_uv_vertex0_is_origin() {
        let p0 = [0.0f32, 0.0, 0.0];
        let p1 = [1.0, 0.0, 0.0];
        let p2 = [0.0, 1.0, 0.0];
        let (t, b) = compute_local_frame(p0, p1, p2);
        let uv = project_to_uv(p0, p0, t, b);
        assert!(uv[0].abs() < 1e-5, "u should be 0 at origin");
        assert!(uv[1].abs() < 1e-5, "v should be 0 at origin");
    }

    #[test]
    fn project_to_uv_vertex1() {
        let p0 = [0.0f32, 0.0, 0.0];
        let p1 = [1.0, 0.0, 0.0];
        let p2 = [0.0, 1.0, 0.0];
        let (t, b) = compute_local_frame(p0, p1, p2);
        let uv = project_to_uv(p1, p0, t, b);
        assert!((uv[0] - 1.0).abs() < 1e-5, "u should be 1.0 at p1");
        assert!(uv[1].abs() < 1e-5, "v should be ~0 at p1");
    }

    #[test]
    fn uv_area_triangle() {
        let uv0 = [0.0f32, 0.0];
        let uv1 = [1.0, 0.0];
        let uv2 = [0.0, 1.0];
        let area = uv_area(uv0, uv1, uv2);
        assert!((area - 0.5).abs() < 1e-6, "area should be 0.5");
    }

    #[test]
    fn uv_area_degenerate_zero() {
        let uv0 = [0.0f32, 0.0];
        let uv1 = [1.0, 1.0];
        let uv2 = [2.0, 2.0];
        let area = uv_area(uv0, uv1, uv2);
        assert!(area.abs() < 1e-6, "collinear UVs should give ~0 area");
    }

    #[test]
    fn uv_area_negative_for_flipped() {
        let uv0 = [0.0f32, 0.0];
        let uv1 = [0.0, 1.0];
        let uv2 = [1.0, 0.0];
        let area = uv_area(uv0, uv1, uv2);
        assert!(area < 0.0, "flipped winding should give negative area");
    }

    #[test]
    fn lscm_produces_n_vertex_uvs() {
        let (pos, idx) = flat_quad_mesh();
        let cfg = LscmConfig::default();
        let result = lscm_parameterize(&pos, &idx, &cfg);
        assert_eq!(
            result.uvs.len(),
            pos.len(),
            "should produce one UV per vertex"
        );
    }

    #[test]
    fn lscm_pinned_vertices_match_config() {
        let (pos, idx) = flat_quad_mesh();
        let cfg = LscmConfig {
            pin_vertex_a: 0,
            pin_uv_a: [0.1, 0.2],
            pin_vertex_b: 1,
            pin_uv_b: [0.9, 0.8],
            iterations: 20,
        };
        let result = lscm_parameterize(&pos, &idx, &cfg);
        assert!((result.uvs[0][0] - 0.1).abs() < 1e-5, "pin_a u mismatch");
        assert!((result.uvs[0][1] - 0.2).abs() < 1e-5, "pin_a v mismatch");
        assert!((result.uvs[1][0] - 0.9).abs() < 1e-5, "pin_b u mismatch");
        assert!((result.uvs[1][1] - 0.8).abs() < 1e-5, "pin_b v mismatch");
    }

    #[test]
    fn lscm_uvs_are_finite() {
        let (pos, idx) = flat_quad_mesh();
        let cfg = LscmConfig::default();
        let result = lscm_parameterize(&pos, &idx, &cfg);
        for uv in &result.uvs {
            assert!(uv[0].is_finite(), "UV u component has NaN/inf");
            assert!(uv[1].is_finite(), "UV v component has NaN/inf");
        }
    }

    #[test]
    fn normalize_uvs_in_range() {
        let mut uvs = vec![[-2.0f32, -3.0], [1.0, 5.0], [0.0, 2.0]];
        normalize_uvs_lscm(&mut uvs);
        for uv in &uvs {
            assert!(uv[0] >= 0.0 && uv[0] <= 1.0, "u out of [0,1]: {}", uv[0]);
            assert!(uv[1] >= 0.0 && uv[1] <= 1.0, "v out of [0,1]: {}", uv[1]);
        }
    }

    #[test]
    fn normalize_uvs_min_max() {
        let mut uvs = vec![[2.0f32, 5.0], [4.0, 10.0]];
        normalize_uvs_lscm(&mut uvs);
        assert!((uvs[0][0] - 0.0).abs() < 1e-6, "min should map to 0");
        assert!((uvs[1][0] - 1.0).abs() < 1e-6, "max should map to 1");
    }

    #[test]
    fn conformal_energy_non_negative() {
        let (pos, idx) = flat_quad_mesh();
        let cfg = LscmConfig::default();
        let result = lscm_parameterize(&pos, &idx, &cfg);
        assert!(
            result.conformal_energy >= 0.0,
            "conformal energy must be non-negative"
        );
    }

    #[test]
    fn uv_stretch_metric_positive() {
        let (pos, idx) = flat_quad_mesh();
        let cfg = LscmConfig::default();
        let result = lscm_parameterize(&pos, &idx, &cfg);
        let stretch = uv_stretch_metric(&result.uvs, &pos, &idx);
        assert!(stretch >= 0.0, "stretch metric should be non-negative");
    }

    #[test]
    fn triangle_conformal_energy_zero_degenerate() {
        // Degenerate UV should return 0
        let uv0 = [0.0f32, 0.0];
        let uv1 = [0.0, 0.0];
        let uv2 = [0.0, 0.0];
        let p0 = [0.0f32, 0.0, 0.0];
        let p1 = [1.0, 0.0, 0.0];
        let p2 = [0.0, 1.0, 0.0];
        let energy = triangle_conformal_energy(uv0, uv1, uv2, p0, p1, p2);
        assert!(energy.abs() < 1e-6, "degenerate UV should have 0 energy");
    }
}
