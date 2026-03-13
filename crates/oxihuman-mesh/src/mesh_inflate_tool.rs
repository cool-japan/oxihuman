// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Mesh inflation along normals.

/// Result of an inflate operation.
#[derive(Debug, Clone)]
pub struct InflateResult {
    pub positions: Vec<[f32; 3]>,
    pub affected_count: usize,
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        return [0.0, 0.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Inflate positions along their normals by `amount` (with optional soft-falloff
/// centred at `centre` within `radius`).
pub fn inflate_mesh(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    amount: f32,
    centre: Option<[f32; 3]>,
    radius: f32,
) -> InflateResult {
    let n = positions.len().min(normals.len());
    let mut out = positions.to_vec();
    let mut affected_count = 0usize;
    for i in 0..n {
        let weight = if let Some(c) = centre {
            let d = dist3(positions[i], c);
            if d >= radius {
                0.0
            } else {
                let t = 1.0 - d / radius;
                t * t * (3.0 - 2.0 * t)
            }
        } else {
            1.0
        };
        if weight < 1e-8 {
            continue;
        }
        let nrm = normalize3(normals[i]);
        out[i][0] += nrm[0] * amount * weight;
        out[i][1] += nrm[1] * amount * weight;
        out[i][2] += nrm[2] * amount * weight;
        affected_count += 1;
    }
    InflateResult {
        positions: out,
        affected_count,
    }
}

/// Inflate uniformly (no falloff).
pub fn inflate_uniform(positions: &[[f32; 3]], normals: &[[f32; 3]], amount: f32) -> InflateResult {
    inflate_mesh(positions, normals, amount, None, 1.0)
}

/// Deflate (negative inflation).
pub fn deflate_mesh(positions: &[[f32; 3]], normals: &[[f32; 3]], amount: f32) -> InflateResult {
    inflate_uniform(positions, normals, -amount)
}

/// Compute vertex normals from triangle soup (area-weighted).
pub fn compute_vertex_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut accum = vec![[0.0f32; 3]; positions.len()];
    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            continue;
        }
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if a >= positions.len() || b >= positions.len() || c >= positions.len() {
            continue;
        }
        let pa = positions[a];
        let pb = positions[b];
        let pc = positions[c];
        let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let ac = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let n = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        for idx in [a, b, c] {
            accum[idx][0] += n[0];
            accum[idx][1] += n[1];
            accum[idx][2] += n[2];
        }
    }
    accum.iter().map(|&v| normalize3(v)).collect()
}

/// Return inflate amount required to achieve a target surface displacement per vertex.
pub fn inflate_to_target_offset(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    target: [f32; 3],
    vertex_index: usize,
) -> f32 {
    if vertex_index >= normals.len() {
        return 0.0;
    }
    let nrm = normalize3(normals[vertex_index]);
    let dot = nrm[0] * target[0] + nrm[1] * target[1] + nrm[2] * target[2];
    let _ = positions;
    dot
}

/// Clamp inflate amount to prevent self-intersection (simple heuristic).
pub fn clamp_inflate_amount(amount: f32, min_edge_length: f32) -> f32 {
    amount.clamp(-min_edge_length * 0.5, min_edge_length * 0.5)
}

/// Compute average normal of a mesh.
pub fn average_normal(normals: &[[f32; 3]]) -> [f32; 3] {
    if normals.is_empty() {
        return [0.0, 0.0, 1.0];
    }
    let n = normals.len() as f32;
    let sum = normals
        .iter()
        .fold([0.0f32; 3], |a, &v| [a[0] + v[0], a[1] + v[1], a[2] + v[2]]);
    normalize3([sum[0] / n, sum[1] / n, sum[2] / n])
}

/// Scale the inflate amount by a per-vertex map.
pub fn inflate_with_weight_map(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    base_amount: f32,
    weights: &[f32],
) -> InflateResult {
    let n = positions.len().min(normals.len()).min(weights.len());
    let mut out = positions.to_vec();
    let mut affected_count = 0usize;
    for i in 0..n {
        let w = weights[i];
        if w < 1e-8 {
            continue;
        }
        let nrm = normalize3(normals[i]);
        let amt = base_amount * w;
        out[i][0] += nrm[0] * amt;
        out[i][1] += nrm[1] * amt;
        out[i][2] += nrm[2] * amt;
        affected_count += 1;
    }
    InflateResult {
        positions: out,
        affected_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_quad() -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
        let pts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let nrm = vec![[0.0, 0.0, 1.0]; 4];
        (pts, nrm)
    }

    /* inflate_uniform moves along Z */
    #[test]
    fn test_inflate_uniform_z() {
        let (pts, nrm) = flat_quad();
        let res = inflate_uniform(&pts, &nrm, 0.5);
        assert!(res.positions[0][2] > 0.4);
    }

    /* deflate */
    #[test]
    fn test_deflate() {
        let (pts, nrm) = flat_quad();
        let res = deflate_mesh(&pts, &nrm, 0.5);
        assert!(res.positions[0][2] < -0.4);
    }

    /* affected_count matches vertex count for uniform */
    #[test]
    fn test_inflate_affected_count() {
        let (pts, nrm) = flat_quad();
        let res = inflate_uniform(&pts, &nrm, 1.0);
        assert_eq!(res.affected_count, 4);
    }

    /* compute_vertex_normals from triangle */
    #[test]
    fn test_compute_vertex_normals() {
        let pts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let nrm = compute_vertex_normals(&pts, &idx);
        assert_eq!(nrm.len(), 3);
        /* should point in +Z or -Z */
        assert!(nrm[0][2].abs() > 0.5);
    }

    /* average_normal */
    #[test]
    fn test_average_normal() {
        let nrm = vec![[0.0, 0.0, 1.0f32]; 5];
        let avg = average_normal(&nrm);
        assert!((avg[2] - 1.0).abs() < 1e-5);
    }

    /* clamp_inflate_amount */
    #[test]
    fn test_clamp_inflate_amount() {
        let clamped = clamp_inflate_amount(10.0, 1.0);
        assert!(clamped <= 0.5 + 1e-6);
    }

    /* inflate_with_weight_map */
    #[test]
    fn test_inflate_with_weight_map() {
        let (pts, nrm) = flat_quad();
        let weights = vec![1.0, 0.0, 1.0, 0.0];
        let res = inflate_with_weight_map(&pts, &nrm, 1.0, &weights);
        assert_eq!(res.affected_count, 2);
        assert!((res.positions[0][2] - 1.0).abs() < 1e-5);
        assert!((res.positions[1][2]).abs() < 1e-6);
    }

    /* inflate_mesh with falloff */
    #[test]
    fn test_inflate_mesh_falloff() {
        let (pts, nrm) = flat_quad();
        let res = inflate_mesh(&pts, &nrm, 1.0, Some([0.0, 0.0, 0.0]), 0.5);
        /* only vertex 0 within radius 0.5 */
        assert!(res.positions[0][2] > 0.0);
    }
}
