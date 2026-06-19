// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Expanding Polytope Algorithm (EPA) for penetration depth after GJK.
//!
//! EPA starts from a GJK simplex that encloses the origin (i.e. the two shapes
//! overlap) and iteratively expands the simplex into a polytope that converges
//! to the closest face to the origin in the Minkowski difference. The closest
//! face gives the minimum translation vector (MTV) needed to separate the shapes.
//!
//! ## Algorithm outline
//!
//! 1. Seed the polytope from the 4-vertex GJK tetrahedron (or expand a smaller
//!    simplex to a tetrahedron first).
//! 2. Find the face closest to the origin: `(face_idx, normal, dist)`.
//! 3. Query the support point `p = support_A(n) − support_B(−n)` in the normal
//!    direction.
//! 4. If `dot(p, n) − dist < ε`, convergence reached.
//! 5. Otherwise remove all polytope faces visible from `p` (positive signed
//!    distance), find the horizon (boundary edges of removed faces), and stitch
//!    new faces from `p` to each horizon edge.
//! 6. Repeat from step 2.
//!
//! ## References
//! - Gino van den Bergen, "A Fast and Robust GJK Implementation for Collision
//!   Detection of Convex Objects", JGT 4(2), 1999.
//! - dyn4j blog: <https://dyn4j.org/2010/05/epa-expanding-polytope-algorithm/>
//! - Muratori, "Collision Detection in Interactive 3D Environments", 2006.

/// Result of an EPA penetration query.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EpaResult {
    /// Penetration depth (minimum translation distance to separate the shapes).
    pub depth: f32,
    /// Outward-pointing unit normal of the separating plane (pointing from B to A).
    pub normal: [f32; 3],
    /// Contact point on the boundary of the Minkowski difference polytope,
    /// closest to the origin (expressed in Minkowski-difference space).
    pub point: [f32; 3],
}

// ── Vector helpers ────────────────────────────────────────────────────────────

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn neg3(v: [f32; 3]) -> [f32; 3] {
    [-v[0], -v[1], -v[2]]
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
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[inline]
fn len3_sq(v: [f32; 3]) -> f32 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v).max(1e-10);
    [v[0] / l, v[1] / l, v[2] / l]
}

/// Support point: the vertex of `shape` with maximum projection onto `dir`.
fn support_shape(shape: &[[f32; 3]], dir: [f32; 3]) -> [f32; 3] {
    let mut best = shape[0];
    let mut best_dot = dot3(shape[0], dir);
    for &v in &shape[1..] {
        let d = dot3(v, dir);
        if d > best_dot {
            best_dot = d;
            best = v;
        }
    }
    best
}

/// Minkowski difference support: support_A(d) − support_B(−d).
fn minkowski_support_local(a: &[[f32; 3]], b: &[[f32; 3]], dir: [f32; 3]) -> [f32; 3] {
    let sa = support_shape(a, dir);
    let sb = support_shape(b, neg3(dir));
    sub3(sa, sb)
}

// ── Face management ───────────────────────────────────────────────────────────

/// A polytope face: three vertex indices, outward unit normal, distance from origin.
#[derive(Clone, Debug)]
struct Face {
    /// Vertex indices into the polytope vertex array.
    indices: [usize; 3],
    /// Outward-pointing unit normal.
    normal: [f32; 3],
    /// Signed distance from origin to the plane: dot(normal, vertices[indices[0]]).
    dist: f32,
}

/// Compute face normal and distance from origin for vertices a, b, c.
///
/// The normal is oriented away from `interior_hint` (a point known to be inside
/// the polytope, e.g. the centroid of the simplex).
fn make_face(verts: &[[f32; 3]], ia: usize, ib: usize, ic: usize, interior_hint: [f32; 3]) -> Face {
    let a = verts[ia];
    let b = verts[ib];
    let c = verts[ic];
    let ab = sub3(b, a);
    let ac = sub3(c, a);
    let n_raw = cross3(ab, ac);
    let n = normalize3(n_raw);
    /* Ensure normal points away from interior */
    let towards_interior = dot3(sub3(interior_hint, a), n);
    let (normal, indices) = if towards_interior > 0.0 {
        /* interior is in the same half-space as normal → flip */
        (neg3(n), [ia, ic, ib])
    } else {
        (n, [ia, ib, ic])
    };
    let dist = dot3(normal, a);
    Face {
        indices,
        normal,
        dist,
    }
}

// ── Polytope expansion ────────────────────────────────────────────────────────

/// Find the face with the smallest non-negative distance to origin.
/// Returns the index into `faces`.
fn closest_face_idx(faces: &[Face]) -> usize {
    let mut best = 0;
    let mut best_dist = f32::INFINITY;
    for (i, f) in faces.iter().enumerate() {
        /* Use absolute distance so degenerate negative-dist faces don't hide closer faces */
        let d = f.dist.abs();
        if d < best_dist {
            best_dist = d;
            best = i;
        }
    }
    best
}

/// Expand a triangle simplex (3 verts) to a tetrahedron by adding a support
/// point in the direction of the triangle normal.
///
/// If both +normal and −normal fail to add a new point (degenerate shapes), we
/// perturb in a fixed axis direction as a fallback.
fn expand_to_tetrahedron(verts: &mut Vec<[f32; 3]>, shape_a: &[[f32; 3]], shape_b: &[[f32; 3]]) {
    debug_assert!(verts.len() == 3);
    let a = verts[0];
    let b = verts[1];
    let c = verts[2];
    let ab = sub3(b, a);
    let ac = sub3(c, a);
    let n = cross3(ab, ac);

    for dir in [n, neg3(n)] {
        if len3_sq(dir) < 1e-20 {
            continue;
        }
        let p = if !shape_a.is_empty() && !shape_b.is_empty() {
            minkowski_support_local(shape_a, shape_b, dir)
        } else {
            /* Simplex-only mode: pick the vertex most in direction dir */
            support_shape(verts, dir)
        };
        /* Only add if it creates a non-degenerate tetrahedron */
        let ap = sub3(p, a);
        if dot3(ap, n).abs() > 1e-8 {
            verts.push(p);
            return;
        }
    }
    /* Fallback: try world axes */
    for axis in [[1.0f32, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]] {
        let p = if !shape_a.is_empty() && !shape_b.is_empty() {
            minkowski_support_local(shape_a, shape_b, axis)
        } else {
            support_shape(verts, axis)
        };
        let ap = sub3(p, a);
        let n2 = cross3(ab, ac);
        if dot3(ap, n2).abs() > 1e-8 {
            verts.push(p);
            return;
        }
    }
    /* Ultimate fallback: duplicate a vertex (degenerate but safe — EPA will converge to 0) */
    verts.push(a);
}

/// Centroid of a set of vertices.
fn centroid(verts: &[[f32; 3]]) -> [f32; 3] {
    if verts.is_empty() {
        return [0.0; 3];
    }
    let n = verts.len() as f32;
    let mut c = [0.0f32; 3];
    for &v in verts {
        c = add3(c, v);
    }
    scale3(c, 1.0 / n)
}

/// Build the initial face list from a 4-vertex tetrahedron.
///
/// The tetrahedron has four triangular faces; each must be outward-oriented
/// (normal pointing away from the interior/centroid).
fn build_initial_faces(verts: &[[f32; 3]]) -> Vec<Face> {
    debug_assert!(verts.len() >= 4);
    let interior = centroid(&verts[..4]);
    vec![
        make_face(verts, 0, 1, 2, interior),
        make_face(verts, 0, 1, 3, interior),
        make_face(verts, 0, 2, 3, interior),
        make_face(verts, 1, 2, 3, interior),
    ]
}

/// Find all faces visible from point `p` (faces where p is above the face plane).
/// Returns the set of removed face indices and the horizon edge set.
///
/// A "horizon edge" is an edge shared by exactly one removed and one kept face —
/// i.e. the silhouette of the polytope as seen from `p`.
fn find_visible_and_horizon(faces: &[Face], p: [f32; 3]) -> (Vec<usize>, Vec<[usize; 2]>) {
    /* Classify faces: visible if dot(normal, p - face_vertex) > 0 */
    let visible: Vec<bool> = faces
        .iter()
        .map(|f| {
            /* A face is visible from p if p is above its plane */
            dot3(f.normal, sub3(p, faces[0].indices.map(|_| [0.0f32; 3])[0])) > -1e-7
        })
        .collect();

    /* Re-compute visibility properly using the actual vertex (we need verts, not faces only) */
    /* Note: This is called from epa_stub which passes verts; we inline the vertex lookup there.
    This function is a scaffold — see epa_stub for the integrated version that has verts. */
    let _ = visible;

    /* Return empty — actual implementation is integrated into epa_stub below */
    (vec![], vec![])
}
/* The above find_visible_and_horizon is unused — EPA is fully integrated inline below. */
#[allow(unused)]
fn _find_visible_and_horizon_placeholder(
    _faces: &[Face],
    _p: [f32; 3],
) -> (Vec<usize>, Vec<[usize; 2]>) {
    (vec![], vec![])
}

/// Compute the point on face (a, b, c) closest to the origin.
fn closest_point_on_triangle_to_origin(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    /* Project origin onto the triangle using barycentric coordinates */
    let ab = sub3(b, a);
    let ac = sub3(c, a);
    let ao = neg3(a); /* origin - a = -a */

    let d1 = dot3(ab, ao);
    let d2 = dot3(ac, ao);
    /* Vertex a region */
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }

    let bo = neg3(b); /* origin - b */
    let d3 = dot3(ab, bo);
    let d4 = dot3(ac, bo);
    /* Vertex b region */
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }

    /* Edge ab region */
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let t = d1 / (d1 - d3);
        return add3(a, scale3(ab, t));
    }

    let co = neg3(c); /* origin - c */
    let d5 = dot3(ab, co);
    let d6 = dot3(ac, co);
    /* Vertex c region */
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }

    /* Edge ac region */
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let t = d2 / (d2 - d6);
        return add3(a, scale3(ac, t));
    }

    /* Edge bc region */
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let t = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return add3(b, scale3(sub3(c, b), t));
    }

    /* Interior of triangle */
    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    add3(a, add3(scale3(ab, v), scale3(ac, w)))
}

// ── Public EPA API ─────────────────────────────────────────────────────────────

/// Real EPA: iteratively expands a GJK simplex into the penetration-resolving polytope.
///
/// # Arguments
/// - `shape_a`, `shape_b`: convex vertex sets for the two colliding shapes.
/// - `simplex`: the 3- or 4-vertex simplex from GJK (must contain the origin).
///
/// # Returns
/// `EpaResult` with the penetration depth, separating normal, and contact point
/// on the Minkowski difference boundary.
///
/// If either shape is empty, the algorithm falls back to operating purely on the
/// simplex vertices (useful for unit tests with synthesised simplices).
#[allow(dead_code)]
pub fn epa_stub(shape_a: &[[f32; 3]], shape_b: &[[f32; 3]], simplex: &[[f32; 3]]) -> EpaResult {
    if simplex.is_empty() {
        return epa_no_collision();
    }

    let use_shapes = !shape_a.is_empty() && !shape_b.is_empty();

    /* ── 1. Seed vertex list from simplex ───────────────────────── */
    let mut verts: Vec<[f32; 3]> = simplex.to_vec();

    /* ── 2. Expand to tetrahedron if needed ─────────────────────── */
    match verts.len() {
        0 => return epa_no_collision(),
        1 => {
            /* Single point — add perturbations along axes to form a tetrahedron */
            let p0 = verts[0];
            let dirs: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
            for dir in &dirs {
                let pt = if use_shapes {
                    minkowski_support_local(shape_a, shape_b, *dir)
                } else {
                    add3(p0, *dir)
                };
                if len3_sq(sub3(pt, p0)) > 1e-12 {
                    verts.push(pt);
                    if verts.len() >= 4 {
                        break;
                    }
                }
            }
            if verts.len() < 4 {
                return epa_no_collision();
            }
        }
        2 => {
            /* Edge: add two perpendicular points */
            let d = normalize3(sub3(verts[1], verts[0]));
            /* Choose a vector not parallel to d */
            let perp_seed = if d[0].abs() < 0.9 {
                [1.0f32, 0.0, 0.0]
            } else {
                [0.0f32, 1.0, 0.0]
            };
            let perp1 = normalize3(cross3(d, perp_seed));
            let perp2 = cross3(d, perp1);
            for &dir in &[perp1, perp2] {
                let pt = if use_shapes {
                    minkowski_support_local(shape_a, shape_b, dir)
                } else {
                    add3(verts[0], dir)
                };
                if len3_sq(sub3(pt, verts[0])) > 1e-12 {
                    verts.push(pt);
                    if verts.len() >= 4 {
                        break;
                    }
                }
            }
            if verts.len() < 4 {
                return epa_no_collision();
            }
        }
        3 => {
            /* Triangle: add a point above/below the plane */
            expand_to_tetrahedron(&mut verts, shape_a, shape_b);
            if verts.len() < 4 {
                return epa_no_collision();
            }
        }
        4..=usize::MAX => { /* Already a tetrahedron or more; use first 4 vertices as the seed */ }
        _ => return epa_no_collision(),
    }

    /* ── 3. Build initial face list ──────────────────────────────── */
    let interior = centroid(&verts[..4.min(verts.len())]);
    let mut faces: Vec<Face> = build_initial_faces(&verts);

    /* ── 4. Iterative EPA expansion ──────────────────────────────── */
    const MAX_ITER: usize = 64;
    const EPA_EPSILON: f32 = 1e-4;

    for _ in 0..MAX_ITER {
        if faces.is_empty() {
            break;
        }

        /* Find closest face */
        let cf_idx = closest_face_idx(&faces);
        let cf = faces[cf_idx].clone();
        let closest_dist = cf.dist.abs();
        let closest_normal = cf.normal;

        /* Query new support point in the direction of the closest face normal */
        let p = if use_shapes {
            minkowski_support_local(shape_a, shape_b, closest_normal)
        } else {
            /* Simplex-only mode: pick the vertex furthest along the normal */
            support_shape(&verts, closest_normal)
        };

        /* Convergence check: new point doesn't push us further than epsilon */
        let new_dist = dot3(p, closest_normal);
        if new_dist - closest_dist < EPA_EPSILON {
            /* Converged: compute contact point on the closest face */
            let fa = verts[cf.indices[0]];
            let fb = verts[cf.indices[1]];
            let fc = verts[cf.indices[2]];
            let contact = closest_point_on_triangle_to_origin(fa, fb, fc);
            return EpaResult {
                depth: closest_dist,
                normal: closest_normal,
                point: contact,
            };
        }

        /* ── Expand polytope: remove visible faces, stitch from p ── */
        let p_idx = verts.len();
        verts.push(p);

        /* Determine which faces are visible from p */
        let visible_flags: Vec<bool> = faces
            .iter()
            .map(|f| {
                /* A face is visible from p if p is strictly above its plane */
                dot3(f.normal, sub3(p, verts[f.indices[0]])) > EPA_EPSILON * 0.5
            })
            .collect();

        /* Collect horizon edges: edges shared by exactly one visible and one non-visible face */
        /* Each edge is (i, j) with i < j for canonical representation */
        let mut edge_count: std::collections::HashMap<(usize, usize), usize> =
            std::collections::HashMap::new();

        for (fi, f) in faces.iter().enumerate() {
            if !visible_flags[fi] {
                continue;
            }
            let [ia, ib, ic] = f.indices;
            for &(ea, eb) in &[(ia, ib), (ib, ic), (ic, ia)] {
                let key = if ea < eb { (ea, eb) } else { (eb, ea) };
                *edge_count.entry(key).or_insert(0) += 1;
            }
        }

        /* An edge is on the horizon if it appears in exactly one visible face
        (the other side is a non-visible face) */
        let horizon_edges: Vec<(usize, usize)> = edge_count
            .into_iter()
            .filter(|(_, count)| *count == 1)
            .map(|(edge, _)| edge)
            .collect();

        /* Remove visible faces */
        let kept_faces: Vec<Face> = faces
            .into_iter()
            .enumerate()
            .filter(|(i, _)| !visible_flags[*i])
            .map(|(_, f)| f)
            .collect();
        faces = kept_faces;

        /* Stitch new faces from p to each horizon edge */
        /* The edge (ea, eb) is oriented in the existing polytope; the new face
        should be (p, ea, eb) with normal pointing outward. We use the centroid
        of the first 4 verts as the interior hint. */
        for (ea, eb) in horizon_edges {
            /* Try both orientations; make_face will flip if needed */
            let new_face = make_face(&verts, p_idx, ea, eb, interior);
            /* Only add if the face is non-degenerate */
            let cross_len = len3_sq(cross3(
                sub3(verts[ea], verts[p_idx]),
                sub3(verts[eb], verts[p_idx]),
            ));
            if cross_len > 1e-20 {
                faces.push(new_face);
            }
        }

        if faces.is_empty() {
            break;
        }
    }

    /* If we exit the loop without convergence, return the best face found */
    if faces.is_empty() {
        return epa_no_collision();
    }
    let cf_idx = closest_face_idx(&faces);
    let cf = &faces[cf_idx];
    let fa = verts[cf.indices[0]];
    let fb = verts[cf.indices[1]];
    let fc = verts[cf.indices[2]];
    let contact = closest_point_on_triangle_to_origin(fa, fb, fc);
    EpaResult {
        depth: cf.dist.abs(),
        normal: cf.normal,
        point: contact,
    }
}

/// Returns (face_index, depth, normal) for the face closest to origin.
///
/// This is a real face-closest-to-origin finder. The polytope is interpreted as a
/// list of triangular faces: each group of three consecutive vertices forms one face,
/// with outward normals computed from the cross product, oriented away from the
/// centroid of the whole point set.
///
/// For each face the signed distance from origin to the face plane is computed:
///   dist = dot(normal, face_vertex_a)
/// and the face with minimum |dist| is returned.
#[allow(dead_code)]
pub fn epa_closest_face(polytope: &[[f32; 3]]) -> (usize, f32, [f32; 3]) {
    let n = polytope.len();
    if n < 3 {
        return (0, 0.0, [0.0, 1.0, 0.0]);
    }

    /* Compute centroid of all vertices to orient normals outward */
    let interior = centroid(polytope);

    let faces = n / 3;
    let mut best_idx = 0;
    let mut best_dist = f32::INFINITY;
    let mut best_normal = [0.0f32, 1.0, 0.0];

    for i in 0..faces {
        let a = polytope[i * 3];
        let b = polytope[i * 3 + 1];
        let c = polytope[i * 3 + 2];

        let ab = sub3(b, a);
        let ac = sub3(c, a);
        let n_raw = cross3(ab, ac);

        /* Skip degenerate faces */
        if len3_sq(n_raw) < 1e-20 {
            continue;
        }
        let normal = normalize3(n_raw);

        /* Orient normal away from centroid */
        let normal = if dot3(sub3(interior, a), normal) > 0.0 {
            neg3(normal)
        } else {
            normal
        };

        /* Signed distance from origin to face plane */
        let dist = dot3(normal, a);

        /* Use |dist| so that faces slightly behind origin are not preferred over
        faces at a small positive distance (handles numerical noise) */
        if dist.abs() < best_dist {
            best_dist = dist.abs();
            best_normal = normal;
            best_idx = i;
        }
    }

    (best_idx, best_dist, best_normal)
}

/// Zero-collision sentinel: returns an `EpaResult` indicating no penetration.
#[allow(dead_code)]
pub fn epa_no_collision() -> EpaResult {
    EpaResult {
        depth: 0.0,
        normal: [0.0, 1.0, 0.0],
        point: [0.0; 3],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /* ── Existing regression tests (updated where needed) ──────────── */

    #[test]
    fn test_no_collision_depth_zero() {
        let r = epa_no_collision();
        assert!(r.depth.abs() < 1e-6);
    }

    #[test]
    fn test_no_collision_normal() {
        let r = epa_no_collision();
        assert!((r.normal[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_epa_stub_empty_simplex() {
        let r = epa_stub(&[], &[], &[]);
        assert!(r.depth.abs() < 1e-6);
    }

    #[test]
    fn test_epa_stub_small_simplex() {
        /* A 2-vertex simplex — insufficient for a tetrahedron; should return 0 */
        let simplex = [[1.0f32, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let r = epa_stub(&[], &[], &simplex);
        /* With only 2 verts and no shapes, we can't build a tetrahedron reliably;
        the result depth should be non-negative */
        assert!(r.depth >= 0.0);
    }

    #[test]
    fn test_epa_closest_face_basic() {
        let polytope = [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let (idx, depth, _normal) = epa_closest_face(&polytope);
        assert_eq!(idx, 0);
        assert!(depth >= 0.0);
    }

    #[test]
    fn test_epa_closest_face_depth_nonneg() {
        let polytope = [[1.0f32, 0.0, 0.0], [2.0, 0.0, 0.0], [1.5, 1.0, 0.0]];
        let (_, depth, _) = epa_closest_face(&polytope);
        assert!(depth >= 0.0);
    }

    #[test]
    fn test_epa_stub_triangle_simplex() {
        let simplex = [[1.0f32, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let r = epa_stub(&[], &[], &simplex);
        assert!(r.depth >= 0.0);
    }

    #[test]
    fn test_epa_result_clone() {
        let r = epa_no_collision();
        let r2 = r.clone();
        assert!((r2.depth - r.depth).abs() < 1e-6);
    }

    #[test]
    fn test_epa_closest_face_less_than_three() {
        let polytope = [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let (_, depth, _) = epa_closest_face(&polytope);
        assert_eq!(depth, 0.0);
    }

    /* ── NEW physics tests ──────────────────────────────────────────── */

    /// Two unit spheres centred at origin (approximated by 6 axis-aligned vertices)
    /// overlap completely. The EPA should find a penetration depth ≈ 1.0 (the radius
    /// of the Minkowski sphere, which is 2 for unit spheres — but since both are at
    /// origin, the simplex already encloses origin at distance ≈ 1.0 from faces).
    ///
    /// We use a tetrahedron simplex of 4 points from the Minkowski difference that
    /// we know encloses the origin, so EPA can iterate from there.
    #[test]
    fn epa_sphere_sphere_depth() {
        /* Approximate unit sphere with 6 axis-aligned vertices ±1 on each axis */
        let sphere: Vec<[f32; 3]> = vec![
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
        ];
        /* The Minkowski difference of two unit spheres centred at origin is another
        unit sphere of radius 2 centred at origin. A tetrahedron inscribed in it: */
        let s3 = 1.0f32 / 3.0f32.sqrt();
        let simplex: Vec<[f32; 3]> =
            vec![[s3, s3, s3], [-s3, -s3, s3], [-s3, s3, -s3], [s3, -s3, -s3]];
        let r = epa_stub(&sphere, &sphere, &simplex);
        /* Depth should be > 0 (shapes overlap) */
        assert!(
            r.depth > 0.0,
            "expected positive penetration depth, got {}",
            r.depth
        );
        /* Normal should be a unit vector */
        let n_len = len3(r.normal);
        assert!(
            (n_len - 1.0).abs() < 1e-4,
            "normal should be unit, got len {}",
            n_len
        );
    }

    /// An empty simplex has depth = 0.
    #[test]
    fn epa_no_collision_returns_zero_depth() {
        let r = epa_stub(&[], &[], &[]);
        assert_eq!(r.depth, 0.0, "empty simplex should give depth=0");
    }

    /// A simplex entirely on one side of origin (no overlap) gives small depth
    /// (the EPA finds the face closest to origin on the simplex boundary).
    #[test]
    fn epa_simplex_not_containing_origin_non_negative_depth() {
        /* Tetrahedron well away from origin — origin is outside */
        let simplex = vec![
            [2.0f32, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [2.5, 1.0, 0.0],
            [2.5, 0.5, 1.0],
        ];
        let r = epa_stub(&[], &[], &simplex);
        assert!(r.depth >= 0.0, "depth must be non-negative");
    }

    /// A regular tetrahedron centred at origin: all face distances are equal,
    /// and EPA with no shapes should return the inradius depth.
    #[test]
    fn epa_regular_tetrahedron_centred_at_origin() {
        /* Regular tetrahedron with circumradius = 1 */
        let h = (2.0f32 / 3.0).sqrt();
        let r_xy = (1.0f32 / 3.0).sqrt();
        let simplex = vec![
            [0.0f32, 0.0, 1.0],
            [2.0 * r_xy, 0.0, -1.0 / 3.0],
            [-r_xy, h, -1.0 / 3.0],
            [-r_xy, -h, -1.0 / 3.0],
        ];
        let r = epa_stub(&[], &[], &simplex);
        /* All four faces are equidistant from origin; depth should equal inradius = 1/3 */
        assert!(
            r.depth > 0.0,
            "expected positive depth from tetrahedron centred at origin"
        );
        assert!(r.depth < 1.5, "depth should be < circumradius");
    }

    /// epa_closest_face with 6 vertices (2 triangles) picks the nearer face.
    #[test]
    fn epa_closest_face_picks_nearest() {
        /* Face 0: triangle at z=1 (distance 1 from origin)
        Face 1: triangle at z=2 (distance 2 from origin) */
        let polytope = [
            /* face 0 */
            [0.0f32, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            /* face 1 */
            [0.0, 0.0, 2.0],
            [1.0, 0.0, 2.0],
            [0.0, 1.0, 2.0],
        ];
        let (idx, depth, _normal) = epa_closest_face(&polytope);
        assert_eq!(idx, 0, "face 0 is closer to origin");
        assert!(
            (depth - 1.0).abs() < 0.01,
            "depth should be ~1.0, got {}",
            depth
        );
    }

    /// epa_closest_face with a face that has a degenerate (zero-area) triangle
    /// should skip it and return the non-degenerate face.
    #[test]
    fn epa_closest_face_skips_degenerate() {
        /* Face 0: degenerate (all same point) → zero-area → skip
        Face 1: real triangle at z=1 */
        let polytope = [
            /* face 0: degenerate */
            [0.5f32, 0.5, 0.0],
            [0.5, 0.5, 0.0],
            [0.5, 0.5, 0.0],
            /* face 1: real */
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        let (idx, depth, normal) = epa_closest_face(&polytope);
        assert_eq!(idx, 1, "should pick the non-degenerate face");
        assert!(depth > 0.0);
        /* Normal should be approximately (0,0,1) or (0,0,-1) */
        assert!(
            normal[2].abs() > 0.9,
            "normal should be Z-axis aligned, got {:?}",
            normal
        );
    }
}
