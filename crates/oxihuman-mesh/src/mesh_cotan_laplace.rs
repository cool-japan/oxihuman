// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cotangent-weight Laplace-Beltrami operator for triangle meshes.

/// One cotangent weight entry (vertex i, vertex j, weight w).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CotanEntry {
    pub i: usize,
    pub j: usize,
    pub w: f32,
}

/// Assembled cotangent Laplacian.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CotanLaplacian {
    pub entries: Vec<CotanEntry>,
    pub vertex_count: usize,
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    dot3(v, v).sqrt()
}

/// Compute cotangent weight for angle opposite edge (i,j) across triangle (i,k,j).
#[allow(dead_code)]
pub fn cotan_weight(pi: [f32; 3], pj: [f32; 3], pk: [f32; 3]) -> f32 {
    let u = sub3(pi, pk);
    let v = sub3(pj, pk);
    let c = dot3(u, v);
    let s = len3(cross3(u, v));
    if s.abs() < 1e-10 {
        0.0
    } else {
        c / s
    }
}

/// Build cotangent Laplacian from a triangle mesh.
#[allow(dead_code)]
pub fn build_cotan_laplacian(positions: &[[f32; 3]], indices: &[u32]) -> CotanLaplacian {
    let nv = positions.len();
    let nf = indices.len() / 3;
    let mut entries = Vec::with_capacity(nf * 6);
    for f in 0..nf {
        let ia = indices[f * 3] as usize;
        let ib = indices[f * 3 + 1] as usize;
        let ic = indices[f * 3 + 2] as usize;
        if ia >= nv || ib >= nv || ic >= nv {
            continue;
        }
        let pa = positions[ia];
        let pb = positions[ib];
        let pc = positions[ic];
        // cot at corner a (opposite edge b-c)
        let wa = cotan_weight(pb, pc, pa) * 0.5;
        // cot at corner b (opposite edge a-c)
        let wb = cotan_weight(pa, pc, pb) * 0.5;
        // cot at corner c (opposite edge a-b)
        let wc = cotan_weight(pa, pb, pc) * 0.5;
        entries.push(CotanEntry {
            i: ib,
            j: ic,
            w: wa,
        });
        entries.push(CotanEntry {
            i: ic,
            j: ib,
            w: wa,
        });
        entries.push(CotanEntry {
            i: ia,
            j: ic,
            w: wb,
        });
        entries.push(CotanEntry {
            i: ic,
            j: ia,
            w: wb,
        });
        entries.push(CotanEntry {
            i: ia,
            j: ib,
            w: wc,
        });
        entries.push(CotanEntry {
            i: ib,
            j: ia,
            w: wc,
        });
    }
    CotanLaplacian {
        entries,
        vertex_count: nv,
    }
}

/// Apply Laplacian smoothing using cotangent weights (one step).
#[allow(dead_code)]
pub fn cotan_smooth_step(
    positions: &[[f32; 3]],
    lap: &CotanLaplacian,
    lambda: f32,
) -> Vec<[f32; 3]> {
    let nv = positions.len();
    let mut num = vec![[0.0_f32; 3]; nv];
    let mut den = vec![0.0_f32; nv];
    for e in &lap.entries {
        if e.i < nv && e.j < nv {
            let w = e.w.max(0.0);
            num[e.i][0] += w * positions[e.j][0];
            num[e.i][1] += w * positions[e.j][1];
            num[e.i][2] += w * positions[e.j][2];
            den[e.i] += w;
        }
    }
    (0..nv)
        .map(|i| {
            if den[i] > 1e-12 {
                let cx = num[i][0] / den[i];
                let cy = num[i][1] / den[i];
                let cz = num[i][2] / den[i];
                [
                    positions[i][0] + lambda * (cx - positions[i][0]),
                    positions[i][1] + lambda * (cy - positions[i][1]),
                    positions[i][2] + lambda * (cz - positions[i][2]),
                ]
            } else {
                positions[i]
            }
        })
        .collect()
}

/// Entry count in the Laplacian.
#[allow(dead_code)]
pub fn entry_count(lap: &CotanLaplacian) -> usize {
    lap.entries.len()
}

/// Sum of all weights.
#[allow(dead_code)]
pub fn total_weight(lap: &CotanLaplacian) -> f32 {
    lap.entries.iter().map(|e| e.w).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single_tri() -> (Vec<[f32; 3]>, Vec<u32>) {
        (
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![0, 1, 2],
        )
    }

    #[test]
    fn entry_count_single_tri() {
        let (pos, idx) = single_tri();
        let lap = build_cotan_laplacian(&pos, &idx);
        assert_eq!(entry_count(&lap), 6);
    }

    #[test]
    fn cotan_weight_right_angle() {
        // right angle at pk: u and v are orthogonal -> cot = 0
        let pi = [1.0, 0.0, 0.0_f32];
        let pj = [0.0, 1.0, 0.0];
        let pk = [0.0, 0.0, 0.0];
        let w = cotan_weight(pi, pj, pk);
        assert!(w.abs() < 1e-4);
    }

    #[test]
    fn smooth_preserves_vertex_count() {
        let (pos, idx) = single_tri();
        let lap = build_cotan_laplacian(&pos, &idx);
        let out = cotan_smooth_step(&pos, &lap, 0.5);
        assert_eq!(out.len(), pos.len());
    }

    #[test]
    fn zero_lambda_no_change() {
        let (pos, idx) = single_tri();
        let lap = build_cotan_laplacian(&pos, &idx);
        let out = cotan_smooth_step(&pos, &lap, 0.0);
        for (a, b) in pos.iter().zip(out.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-5);
        }
    }

    #[test]
    fn vertex_count_stored() {
        let (pos, idx) = single_tri();
        let lap = build_cotan_laplacian(&pos, &idx);
        assert_eq!(lap.vertex_count, 3);
    }

    #[test]
    fn total_weight_finite() {
        let (pos, idx) = single_tri();
        let lap = build_cotan_laplacian(&pos, &idx);
        assert!(total_weight(&lap).is_finite());
    }

    #[test]
    fn cotan_degenerate_returns_zero() {
        // Degenerate: same point
        let p = [0.0_f32; 3];
        let w = cotan_weight(p, p, p);
        assert_eq!(w, 0.0);
    }

    #[test]
    fn empty_mesh() {
        let lap = build_cotan_laplacian(&[], &[]);
        assert_eq!(entry_count(&lap), 0);
    }

    #[test]
    fn contains_range() {
        let v = 0.3_f32;
        assert!((0.0..=1.0).contains(&v));
    }
}
