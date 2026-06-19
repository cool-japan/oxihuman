// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! NURBS surface stub.

/// NURBS surface definition stub.
#[derive(Debug, Clone)]
pub struct NurbsSurface {
    /// Control points in a `(u_count × v_count)` grid, stored row-major.
    pub control_points: Vec<[f32; 3]>,
    /// Homogeneous weights for each control point.
    pub weights: Vec<f32>,
    pub u_count: usize,
    pub v_count: usize,
    pub u_degree: usize,
    pub v_degree: usize,
    pub u_knots: Vec<f32>,
    pub v_knots: Vec<f32>,
}

/// Clamp-open uniform knot vector for given count and degree.
pub fn uniform_knots(n: usize, degree: usize) -> Vec<f32> {
    let knot_count = n + degree + 1;
    (0..knot_count)
        .map(|i| {
            if i <= degree {
                0.0
            } else if i >= knot_count.saturating_sub(degree + 1) {
                1.0
            } else {
                (i - degree) as f32 / (n - degree) as f32
            }
        })
        .collect()
}

/// Create a NURBS surface stub with uniform weights.
pub fn new_nurbs_surface(
    control_points: Vec<[f32; 3]>,
    u_count: usize,
    v_count: usize,
    u_degree: usize,
    v_degree: usize,
) -> NurbsSurface {
    let n = control_points.len();
    let weights = vec![1.0f32; n];
    let u_knots = uniform_knots(u_count, u_degree);
    let v_knots = uniform_knots(v_count, v_degree);
    NurbsSurface {
        control_points,
        weights,
        u_count,
        v_count,
        u_degree,
        v_degree,
        u_knots,
        v_knots,
    }
}

/// Validate that the NURBS surface has consistent dimensions.
pub fn validate_nurbs(surf: &NurbsSurface) -> bool {
    surf.control_points.len() == surf.u_count * surf.v_count
        && surf.weights.len() == surf.control_points.len()
        && surf.u_knots.len() == surf.u_count + surf.u_degree + 1
        && surf.v_knots.len() == surf.v_count + surf.v_degree + 1
}

/// Return the total control point count.
pub fn nurbs_control_point_count(surf: &NurbsSurface) -> usize {
    surf.control_points.len()
}

/// Compute the bounding box of the control polygon (not the surface itself).
pub fn nurbs_control_bbox(surf: &NurbsSurface) -> ([f32; 3], [f32; 3]) {
    if surf.control_points.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = surf.control_points[0];
    let mut mx = surf.control_points[0];
    for &p in &surf.control_points {
        for k in 0..3 {
            mn[k] = mn[k].min(p[k]);
            mx[k] = mx[k].max(p[k]);
        }
    }
    (mn, mx)
}

/// Evaluate all B-spline basis functions N_{i,degree}(t) over the given knot vector
/// via the Cox–de Boor recursion.
///
/// Returns a vector of length `n = knots.len() - degree - 1`.
/// Each entry is the value of the i-th basis function at parameter `t`.
fn de_boor_basis(knots: &[f32], degree: usize, t: f32) -> Vec<f32> {
    let n = match knots.len().checked_sub(degree + 1) {
        Some(v) if v > 0 => v,
        _ => return vec![],
    };

    // Order-0 (degree-0) basis: 1 on the half-open interval [knot_i, knot_{i+1}).
    // We allocate n+degree slots because the recursion needs the extra entries
    // when lifting from degree 0 up to the target degree.
    let total = n + degree;
    let mut b: Vec<f32> = (0..total)
        .map(|i| {
            if knots[i] <= t && t < knots[i + 1] {
                1.0
            } else {
                0.0
            }
        })
        .collect();

    // Clamped knot vectors: at t == last_knot, assign the active span to the last
    // *non-degenerate* span (where knots[i] < knots[i+1]), not just total-1.
    // knots[len-2] may equal knots[len-1] for multiplicity > 1, so we search backward.
    let last = knots[knots.len() - 1];
    if (t - last).abs() < 1e-7 {
        let mut last_span = total.saturating_sub(1);
        while last_span > 0 && (knots[last_span + 1] - knots[last_span]).abs() < 1e-10 {
            last_span -= 1;
        }
        if last_span < b.len() {
            // Clear any half-open assignments that missed t=last_knot, then activate.
            for v in b.iter_mut() {
                *v = 0.0;
            }
            b[last_span] = 1.0;
        }
    }

    // Triangular table: lift degree from 1 up to `degree`.
    for d in 1..=degree {
        let count = n + degree - d; // active entries at this level
        for i in 0..count {
            let left = {
                let denom = knots[i + d] - knots[i];
                if denom.abs() > 1e-10 {
                    (t - knots[i]) / denom * b[i]
                } else {
                    0.0
                }
            };
            let right = {
                let denom = knots[i + d + 1] - knots[i + 1];
                if denom.abs() > 1e-10 {
                    (knots[i + d + 1] - t) / denom * b[i + 1]
                } else {
                    0.0
                }
            };
            b[i] = left + right;
        }
        b.truncate(count);
    }

    b.truncate(n);
    b
}

/// Evaluate the rational tensor-product NURBS surface S(u,v) at a single parameter pair.
///
/// S(u,v) = ΣΣ N_i(u) N_j(v) w_{ij} P_{ij}  /  ΣΣ N_i(u) N_j(v) w_{ij}
///
/// Returns the fallback first control point if the denominator is degenerate.
fn eval_nurbs_point(surf: &NurbsSurface, u_param: f32, v_param: f32) -> [f32; 3] {
    let basis_u = de_boor_basis(&surf.u_knots, surf.u_degree, u_param);
    let basis_v = de_boor_basis(&surf.v_knots, surf.v_degree, v_param);

    let mut num = [0.0f32; 3];
    let mut denom = 0.0f32;

    for i in 0..surf.u_count {
        let ni = if i < basis_u.len() { basis_u[i] } else { 0.0 };
        if ni.abs() < 1e-14 {
            continue; // skip zero contributions for efficiency
        }
        for j in 0..surf.v_count {
            let nj = if j < basis_v.len() { basis_v[j] } else { 0.0 };
            let w = surf.weights[i * surf.v_count + j];
            let p = surf.control_points[i * surf.v_count + j];
            let nw = ni * nj * w;
            num[0] += nw * p[0];
            num[1] += nw * p[1];
            num[2] += nw * p[2];
            denom += nw;
        }
    }

    if denom.abs() > 1e-10 {
        [num[0] / denom, num[1] / denom, num[2] / denom]
    } else {
        surf.control_points[0]
    }
}

/// Tessellate the NURBS surface into a flat grid mesh using rational tensor-product evaluation.
///
/// The lattice has `(u_divs+1) × (v_divs+1)` vertices; each quad cell is split into
/// two triangles, yielding `2 × u_divs × v_divs` triangles total.
pub fn tessellate_nurbs(
    surf: &NurbsSurface,
    u_divs: usize,
    v_divs: usize,
) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    if !validate_nurbs(surf) || u_divs == 0 || v_divs == 0 {
        return (vec![], vec![]);
    }

    let rows = u_divs + 1;
    let cols = v_divs + 1;

    // Determine the valid parameter domain from the clamped knot vectors.
    // For uniform clamped knots this is [0, 1] in both directions.
    let u_min = surf.u_knots[surf.u_degree];
    let u_max = surf.u_knots[surf.u_count]; // knot at index u_count (= last - u_degree)
    let v_min = surf.v_knots[surf.v_degree];
    let v_max = surf.v_knots[surf.v_count];

    let mut verts = Vec::with_capacity(rows * cols);
    for i in 0..rows {
        // Map lattice index i ∈ [0, rows-1] to u_param ∈ [u_min, u_max].
        let t_u = i as f32 / (rows - 1).max(1) as f32;
        let u_param = u_min + t_u * (u_max - u_min);
        for j in 0..cols {
            let t_v = j as f32 / (cols - 1).max(1) as f32;
            let v_param = v_min + t_v * (v_max - v_min);
            verts.push(eval_nurbs_point(surf, u_param, v_param));
        }
    }

    // Wire each quad (i,j)–(i,j+1)–(i+1,j)–(i+1,j+1) as two counter-clockwise triangles.
    let mut tris = Vec::with_capacity(2 * u_divs * v_divs);
    for i in 0..u_divs {
        for j in 0..v_divs {
            let a = (i * cols + j) as u32;
            let b = (i * cols + j + 1) as u32;
            let c = ((i + 1) * cols + j) as u32;
            let d = ((i + 1) * cols + j + 1) as u32;
            tris.push([a, c, b]);
            tris.push([b, c, d]);
        }
    }

    (verts, tris)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_grid(u: usize, v: usize) -> Vec<[f32; 3]> {
        (0..u)
            .flat_map(|i| (0..v).map(move |j| [i as f32, j as f32, 0.0]))
            .collect()
    }

    #[test]
    fn test_uniform_knots_length() {
        /* degree=2, count=4 → 7 knots */
        let k = uniform_knots(4, 2);
        assert_eq!(k.len(), 7);
    }

    #[test]
    fn test_uniform_knots_clamped() {
        let k = uniform_knots(5, 2);
        assert_eq!(k[0], 0.0);
        assert_eq!(*k.last().expect("should succeed"), 1.0);
    }

    #[test]
    fn test_new_nurbs_surface_valid() {
        let cp = flat_grid(4, 3);
        let surf = new_nurbs_surface(cp, 4, 3, 2, 2);
        assert!(validate_nurbs(&surf));
    }

    #[test]
    fn test_nurbs_control_point_count() {
        let cp = flat_grid(3, 4);
        let surf = new_nurbs_surface(cp, 3, 4, 2, 2);
        assert_eq!(nurbs_control_point_count(&surf), 12);
    }

    #[test]
    fn test_nurbs_bbox_flat() {
        let cp = flat_grid(3, 3);
        let surf = new_nurbs_surface(cp, 3, 3, 2, 2);
        let (mn, mx) = nurbs_control_bbox(&surf);
        assert!(mn[2].abs() < 1e-6);
        assert!(mx[2].abs() < 1e-6);
    }

    #[test]
    fn test_tessellate_nurbs_vertex_count() {
        /* 4x4 divs → 5*5 = 25 verts */
        let cp = flat_grid(4, 4);
        let surf = new_nurbs_surface(cp, 4, 4, 2, 2);
        let (verts, _) = tessellate_nurbs(&surf, 4, 4);
        assert_eq!(verts.len(), 25);
    }

    #[test]
    fn test_tessellate_nurbs_tri_count() {
        let cp = flat_grid(4, 4);
        let surf = new_nurbs_surface(cp, 4, 4, 2, 2);
        let (_, tris) = tessellate_nurbs(&surf, 4, 4);
        assert_eq!(tris.len(), 32);
    }

    #[test]
    fn test_tessellate_empty_on_zero_divs() {
        let cp = flat_grid(4, 4);
        let surf = new_nurbs_surface(cp, 4, 4, 2, 2);
        let (v, t) = tessellate_nurbs(&surf, 0, 4);
        assert!(v.is_empty());
        assert!(t.is_empty());
    }

    #[test]
    fn test_weights_all_one() {
        let cp = flat_grid(3, 3);
        let surf = new_nurbs_surface(cp, 3, 3, 2, 2);
        assert!(surf.weights.iter().all(|&w| (w - 1.0).abs() < 1e-6));
    }

    #[test]
    fn test_nurbs_bbox_empty() {
        let surf = new_nurbs_surface(vec![], 0, 0, 0, 0);
        let (mn, mx) = nurbs_control_bbox(&surf);
        assert_eq!(mn, [0.0; 3]);
        assert_eq!(mx, [0.0; 3]);
    }

    /// A clamped NURBS surface must interpolate its four corner control points exactly.
    /// The first tessellated vertex (u=0, v=0) must coincide with the first control point,
    /// and the last vertex (u=1, v=1) must coincide with the last control point.
    #[test]
    fn test_nurbs_corner_interpolation() {
        // Build a 4×4 control grid with recognisable corners so the assertion is meaningful.
        let mut cp: Vec<[f32; 3]> = flat_grid(4, 4);
        // Give distinct z values to each corner so they are distinguishable.
        cp[0] = [0.0, 0.0, 0.5];  // (i=0, j=0)
        cp[3] = [0.0, 3.0, 1.5];  // (i=0, j=3)
        cp[12] = [3.0, 0.0, 2.5]; // (i=3, j=0)
        cp[15] = [3.0, 3.0, 3.5]; // (i=3, j=3)
        let surf = new_nurbs_surface(cp.clone(), 4, 4, 2, 2);
        let (verts, _) = tessellate_nurbs(&surf, 4, 4);
        assert!(!verts.is_empty(), "tessellation must produce vertices");

        // First vertex: u=0, v=0 → should match cp[0]
        let v0 = verts[0];
        let p0 = cp[0];
        assert!(
            (v0[0] - p0[0]).abs() < 0.05,
            "corner (0,0) x: expected {}, got {}",
            p0[0],
            v0[0]
        );
        assert!(
            (v0[1] - p0[1]).abs() < 0.05,
            "corner (0,0) y: expected {}, got {}",
            p0[1],
            v0[1]
        );
        assert!(
            (v0[2] - p0[2]).abs() < 0.05,
            "corner (0,0) z: expected {}, got {}",
            p0[2],
            v0[2]
        );

        // Last vertex: u=1, v=1 → should match cp[15]
        let v_last = verts[verts.len() - 1];
        let p_last = cp[15];
        assert!(
            (v_last[0] - p_last[0]).abs() < 0.05,
            "corner (3,3) x: expected {}, got {}",
            p_last[0],
            v_last[0]
        );
        assert!(
            (v_last[1] - p_last[1]).abs() < 0.05,
            "corner (3,3) y: expected {}, got {}",
            p_last[1],
            v_last[1]
        );
        assert!(
            (v_last[2] - p_last[2]).abs() < 0.05,
            "corner (3,3) z: expected {}, got {}",
            p_last[2],
            v_last[2]
        );
    }

    /// Raising the weight of an interior control point (and setting a non-zero z on it)
    /// must pull the tessellated surface toward that point relative to the unit-weight case.
    #[test]
    fn test_nurbs_weight_sensitivity() {
        // Flat 4×4 grid, all z=0, uniform weights=1.
        let cp = flat_grid(4, 4);
        let s1 = new_nurbs_surface(cp.clone(), 4, 4, 2, 2);

        // Clone the surface and bump one interior control point's weight and z.
        let mut s2 = new_nurbs_surface(cp, 4, 4, 2, 2);
        // Centre of a 4×4 grid (0-indexed): use (i=1, j=1) → flat index v_count + 1.
        let mid = s2.v_count + 1;
        if mid < s2.weights.len() {
            s2.control_points[mid][2] = 1.0;
            s2.weights[mid] = 10.0;
        }

        let (verts1, _) = tessellate_nurbs(&s1, 4, 4);
        let (verts2, _) = tessellate_nurbs(&s2, 4, 4);

        assert_eq!(
            verts1.len(),
            verts2.len(),
            "vertex counts must match between the two surfaces"
        );
        if verts1.len() > 12 {
            // The vertex nearest the bumped control point should be pulled toward it in Z.
            // For a 5×5 vertex lattice the vertex closest to cp (1,1) is lattice (1,1) → idx 6.
            let probe = 6_usize.min(verts1.len() - 1);
            assert!(
                verts2[probe][2] > verts1[probe][2],
                "high-weight control point (z=1) should pull the surface upward in Z \
                 at probe vertex {}; z1={}, z2={}",
                probe,
                verts1[probe][2],
                verts2[probe][2]
            );
        }
    }
}
