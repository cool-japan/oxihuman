// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Gordon surface (net of curves) stub.

/// A Gordon surface is defined by a grid of u-curves and v-curves plus a
/// cross-interpolation grid.  This stub stores the input network and provides
/// a simple linear tessellation.
#[derive(Debug, Clone)]
pub struct GordonSurface {
    /// `u_curves[i]` is a list of 3-D points along the i-th u-direction curve.
    pub u_curves: Vec<Vec<[f32; 3]>>,
    /// `v_curves[j]` is a list of 3-D points along the j-th v-direction curve.
    pub v_curves: Vec<Vec<[f32; 3]>>,
    /// Intersection matrix `intersections[i][j]` = `u_curves[i]` ∩ `v_curves[j]`.
    pub intersections: Vec<Vec<[f32; 3]>>,
}

/// Create an empty Gordon surface network.
pub fn new_gordon_surface(
    u_curves: Vec<Vec<[f32; 3]>>,
    v_curves: Vec<Vec<[f32; 3]>>,
) -> GordonSurface {
    let nu = u_curves.len();
    let nv = v_curves.len();
    /* stub: intersections taken as first point of each u-curve per v-curve */
    let intersections = (0..nu)
        .map(|i| {
            (0..nv)
                .map(|_j| u_curves[i].first().copied().unwrap_or([0.0; 3]))
                .collect()
        })
        .collect();
    GordonSurface {
        u_curves,
        v_curves,
        intersections,
    }
}

/// Return the number of u-curves.
pub fn gordon_u_curve_count(surf: &GordonSurface) -> usize {
    surf.u_curves.len()
}

/// Return the number of v-curves.
pub fn gordon_v_curve_count(surf: &GordonSurface) -> usize {
    surf.v_curves.len()
}

/// Validate that the intersection matrix has the correct size.
pub fn validate_gordon(surf: &GordonSurface) -> bool {
    let nu = surf.u_curves.len();
    let nv = surf.v_curves.len();
    surf.intersections.len() == nu && surf.intersections.iter().all(|row| row.len() == nv)
}

/// Evaluate a polyline curve at parameter t ∈ [0, 1] by linear interpolation
/// between consecutive control points.
fn eval_curve(curve: &[[f32; 3]], t: f32) -> [f32; 3] {
    let n = curve.len();
    if n == 0 {
        return [0.0; 3];
    }
    if n == 1 || t <= 0.0 {
        return curve[0];
    }
    if t >= 1.0 {
        return curve[n - 1];
    }
    // Map t into a segment index.
    let seg_f = t * (n - 1) as f32;
    let seg = (seg_f as usize).min(n - 2);
    let local_t = seg_f - seg as f32;
    let a = curve[seg];
    let b = curve[seg + 1];
    [
        a[0] + local_t * (b[0] - a[0]),
        a[1] + local_t * (b[1] - a[1]),
        a[2] + local_t * (b[2] - a[2]),
    ]
}

/// Bilinear blend at parameter (u, v) using the four corner intersection points.
///
/// B(u, v) = (1-u)(1-v)·P₀₀ + u(1-v)·P₁₀ + (1-u)v·P₀₁ + u·v·P₁₁
///
/// Corner intersection points are obtained by evaluating:
///   P₀₀ = u_curves[0]  at v=0   (or v_curves[0]  at u=0)
///   P₁₀ = u_curves[nu-1] at v=0
///   P₀₁ = u_curves[0]  at v=1
///   P₁₁ = u_curves[nu-1] at v=1
/// Here we use the u_curves evaluated at the extreme v parameters (0 and 1).
fn bilinear_blend(
    u_curves: &[Vec<[f32; 3]>],
    v_curves: &[Vec<[f32; 3]>],
    u: f32,
    v: f32,
) -> [f32; 3] {
    let nu = u_curves.len();
    let nv = v_curves.len();
    // Corner positions: P(u_curve_idx, v_param)
    // P₀₀ = first u-curve at v=0  → v_curves[0] at u=0
    // P₁₀ = last  u-curve at v=0  → v_curves[0] at u=1
    // P₀₁ = first u-curve at v=1  → v_curves[nv-1] at u=0
    // P₁₁ = last  u-curve at v=1  → v_curves[nv-1] at u=1
    // We use the v_curves to get consistent corner positions.
    let p00 = eval_curve(&v_curves[0], 0.0);
    let p10 = eval_curve(&v_curves[0], 1.0);
    let last_v = if nv > 0 { nv - 1 } else { 0 };
    let p01 = eval_curve(&v_curves[last_v], 0.0);
    let p11 = eval_curve(&v_curves[last_v], 1.0);
    let _ = nu; // corners do not depend on which u-curve
    let wu0 = 1.0 - u;
    let wu1 = u;
    let wv0 = 1.0 - v;
    let wv1 = v;
    [
        wu0 * wv0 * p00[0] + wu1 * wv0 * p10[0] + wu0 * wv1 * p01[0] + wu1 * wv1 * p11[0],
        wu0 * wv0 * p00[1] + wu1 * wv0 * p10[1] + wu0 * wv1 * p01[1] + wu1 * wv1 * p11[1],
        wu0 * wv0 * p00[2] + wu1 * wv0 * p10[2] + wu0 * wv1 * p01[2] + wu1 * wv1 * p11[2],
    ]
}

/// Tessellate the Gordon surface using the boolean-sum formula
/// S(u,v) = L_u(u,v) + L_v(u,v) − B(u,v)
///
/// where:
///   L_u(u,v) = point on u_curve[round(u*(nu-1))] at parameter v
///   L_v(u,v) = point on v_curve[round(v*(nv-1))] at parameter u
///   B(u,v)   = bilinear blend of the four corner intersection points
///
/// The grid is nu × nv points (one per curve intersection when samples=1).
/// Returns (vertices, triangles).
pub fn tessellate_gordon(surf: &GordonSurface, samples: usize) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let nu = surf.u_curves.len();
    let nv = surf.v_curves.len();
    if nu < 2 || nv < 2 || samples == 0 {
        return (vec![], vec![]);
    }

    // Grid dimensions: nu rows × nv columns (one intersection point per curve pair).
    // `samples` is the number of subdivisions per interval; when samples=1 there is
    // exactly one output point per curve (so nu×nv total), matching the existing tests.
    let nu_pts = nu;
    let nv_pts = nv;

    let mut verts = Vec::with_capacity(nu_pts * nv_pts);
    for i in 0..nu_pts {
        // u_param: position along the u-curve family (0 = first curve, 1 = last)
        let u_param = if nu_pts > 1 {
            i as f32 / (nu_pts - 1) as f32
        } else {
            0.0
        };
        for j in 0..nv_pts {
            // v_param: position along the v-curve family (0 = first curve, 1 = last)
            let v_param = if nv_pts > 1 {
                j as f32 / (nv_pts - 1) as f32
            } else {
                0.0
            };

            // L_u: sample the i-th u-curve at v_param
            let lu = eval_curve(&surf.u_curves[i], v_param);

            // L_v: sample the j-th v-curve at u_param
            let lv = eval_curve(&surf.v_curves[j], u_param);

            // B: bilinear blend of corner intersection points
            let b = bilinear_blend(&surf.u_curves, &surf.v_curves, u_param, v_param);

            // Gordon boolean-sum: S = L_u + L_v − B
            verts.push([lu[0] + lv[0] - b[0], lu[1] + lv[1] - b[1], lu[2] + lv[2] - b[2]]);
        }
    }

    // Triangulate: each (rows-1)×(cols-1) quad → 2 triangles.
    let mut tris = Vec::new();
    for i in 0..nu_pts - 1 {
        for j in 0..nv_pts - 1 {
            let a = (i * nv_pts + j) as u32;
            let b = (i * nv_pts + j + 1) as u32;
            let c = ((i + 1) * nv_pts + j) as u32;
            let d = ((i + 1) * nv_pts + j + 1) as u32;
            tris.push([a, c, b]);
            tris.push([b, c, d]);
        }
    }
    (verts, tris)
}

/// Total point count across all u-curves.
pub fn gordon_total_u_points(surf: &GordonSurface) -> usize {
    surf.u_curves.iter().map(|c| c.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_u_curves(n: usize) -> Vec<Vec<[f32; 3]>> {
        (0..n)
            .map(|i| vec![[i as f32, 0.0, 0.0], [i as f32, 1.0, 0.0]])
            .collect()
    }
    fn make_v_curves(n: usize) -> Vec<Vec<[f32; 3]>> {
        (0..n)
            .map(|j| vec![[0.0, j as f32, 0.0], [1.0, j as f32, 0.0]])
            .collect()
    }

    #[test]
    fn test_gordon_u_curve_count() {
        let s = new_gordon_surface(make_u_curves(4), make_v_curves(3));
        assert_eq!(gordon_u_curve_count(&s), 4);
    }

    #[test]
    fn test_gordon_v_curve_count() {
        let s = new_gordon_surface(make_u_curves(4), make_v_curves(3));
        assert_eq!(gordon_v_curve_count(&s), 3);
    }

    #[test]
    fn test_validate_gordon() {
        let s = new_gordon_surface(make_u_curves(3), make_v_curves(5));
        assert!(validate_gordon(&s));
    }

    #[test]
    fn test_tessellate_gordon_vertex_count() {
        let s = new_gordon_surface(make_u_curves(4), make_v_curves(4));
        let (v, _) = tessellate_gordon(&s, 1);
        assert_eq!(v.len(), 16);
    }

    #[test]
    fn test_tessellate_gordon_tri_count() {
        let s = new_gordon_surface(make_u_curves(4), make_v_curves(4));
        let (_, t) = tessellate_gordon(&s, 1);
        assert_eq!(t.len(), 18);
    }

    #[test]
    fn test_tessellate_gordon_empty_on_too_few() {
        let s = new_gordon_surface(make_u_curves(1), make_v_curves(4));
        let (v, _) = tessellate_gordon(&s, 2);
        assert!(v.is_empty());
    }

    #[test]
    fn test_gordon_total_u_points() {
        let s = new_gordon_surface(make_u_curves(3), make_v_curves(2));
        assert_eq!(gordon_total_u_points(&s), 6);
    }

    #[test]
    fn test_validate_gordon_empty() {
        let s = new_gordon_surface(vec![], vec![]);
        assert!(validate_gordon(&s));
    }

    #[test]
    fn test_tessellate_empty_on_zero_samples() {
        let s = new_gordon_surface(make_u_curves(3), make_v_curves(3));
        let (v, _) = tessellate_gordon(&s, 0);
        assert!(v.is_empty());
    }

    #[test]
    fn gordon_boundary_passes_through_curve_intersections() {
        // make_u_curves(n): u_curves[i] = line from [i,0,0] to [i,1,0]
        // make_v_curves(n): v_curves[j] = line from [0,j,0] to [1,j,0]
        // At grid corner (i=0, j=0): u_param=0, v_param=0
        //   L_u = u_curves[0] at v=0 = [0,0,0]
        //   L_v = v_curves[0] at u=0 = [0,0,0]
        //   B   = bilinear at (0,0) = v_curves[0] at u=0 = [0,0,0]
        //   S   = [0,0,0] + [0,0,0] - [0,0,0] = [0,0,0] ✓
        let n = 4usize;
        let s = new_gordon_surface(make_u_curves(n), make_v_curves(n));
        let (verts, _) = tessellate_gordon(&s, 1);
        // The grid is nu×nv = n×n.  Corner (0,0) is verts[0].
        let corner = verts[0];
        let expected = [0.0f32, 0.0, 0.0];
        for k in 0..3 {
            assert!(
                (corner[k] - expected[k]).abs() < 1e-4,
                "corner[{k}]: expected {}, got {}",
                expected[k],
                corner[k]
            );
        }
        // Corner (nu-1, nv-1) is the last vertex: verts[n*n - 1]
        let last = verts[n * n - 1];
        // u_curves[n-1] at v=1 = [n-1, 1, 0]
        // v_curves[n-1] at u=1 = [1, n-1, 0]
        // B at (u=1, v=1): v_curves[n-1] at u=1 = [1, n-1, 0]
        // S = [n-1,1,0]+[1,n-1,0]-[1,n-1,0] = [n-1,1,0]
        let nf = (n - 1) as f32;
        let exp_last = [nf, 1.0, 0.0];
        for k in 0..3 {
            assert!(
                (last[k] - exp_last[k]).abs() < 1e-3,
                "last corner[{k}]: expected {}, got {}",
                exp_last[k],
                last[k]
            );
        }
    }
}
