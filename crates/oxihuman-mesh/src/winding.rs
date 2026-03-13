// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use crate::mesh::MeshBuffers;

// ──────────────────────────────────────────────────────────────────────────────
// Constants
// ──────────────────────────────────────────────────────────────────────────────

/// Winding number threshold: |W(q)| >= this value means the point is inside.
pub const WINDING_THRESHOLD: f32 = 0.5;

// ──────────────────────────────────────────────────────────────────────────────
// Internal math helpers
// ──────────────────────────────────────────────────────────────────────────────

#[inline]
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn norm(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}

// ──────────────────────────────────────────────────────────────────────────────
// Core primitive
// ──────────────────────────────────────────────────────────────────────────────

/// Solid angle subtended by a single triangle at a query point (steradians).
///
/// Uses the Van Oosterom & Strackee formula:
/// ```text
/// a = A - P,  b = B - P,  c = C - P
/// numerator   = a · (b × c)
/// denominator = |a||b||c| + (a·b)|c| + (b·c)|a| + (a·c)|b|
/// solid_angle = 2 * atan2(numerator, denominator)
/// ```
/// Returns 0.0 for degenerate triangles (any vertex coincides with the query
/// point, or the denominator is zero).
pub fn triangle_solid_angle(a: [f32; 3], b: [f32; 3], c: [f32; 3], query: [f32; 3]) -> f32 {
    let va = sub(a, query);
    let vb = sub(b, query);
    let vc = sub(c, query);

    let na = norm(va);
    let nb = norm(vb);
    let nc = norm(vc);

    // Degenerate: any vertex coincides with the query point.
    if na < f32::EPSILON || nb < f32::EPSILON || nc < f32::EPSILON {
        return 0.0;
    }

    let numerator = dot(va, cross(vb, vc));
    let denominator = na * nb * nc + dot(va, vb) * nc + dot(vb, vc) * na + dot(va, vc) * nb;

    if denominator.abs() < f32::EPSILON && numerator.abs() < f32::EPSILON {
        return 0.0;
    }

    2.0 * numerator.atan2(denominator)
}

// ──────────────────────────────────────────────────────────────────────────────
// Winding number
// ──────────────────────────────────────────────────────────────────────────────

/// Winding number W(q) at a query point.
///
/// For a closed, consistently-wound mesh:
/// - W ≈  1.0  →  point is inside (winding once)
/// - W ≈  0.0  →  point is outside
/// - W ≈ -1.0  →  point is inside, opposite orientation
pub fn winding_number(mesh: &MeshBuffers, query: [f32; 3]) -> f32 {
    let indices = &mesh.indices;
    let positions = &mesh.positions;

    let n_faces = indices.len() / 3;
    let mut total_solid_angle = 0.0f32;

    for i in 0..n_faces {
        let ia = indices[3 * i] as usize;
        let ib = indices[3 * i + 1] as usize;
        let ic = indices[3 * i + 2] as usize;

        let a = positions[ia];
        let b = positions[ib];
        let c = positions[ic];

        total_solid_angle += triangle_solid_angle(a, b, c, query);
    }

    total_solid_angle / (4.0 * std::f32::consts::PI)
}

// ──────────────────────────────────────────────────────────────────────────────
// Classification helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Returns `true` if the query point is inside the mesh (|W(q)| >= 0.5).
pub fn is_inside(mesh: &MeshBuffers, query: [f32; 3]) -> bool {
    winding_number(mesh, query).abs() >= WINDING_THRESHOLD
}

/// Sign based on inside/outside: returns `-1.0` if inside, `1.0` if outside.
pub fn winding_sign(mesh: &MeshBuffers, query: [f32; 3]) -> f32 {
    if is_inside(mesh, query) {
        -1.0
    } else {
        1.0
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Batch queries
// ──────────────────────────────────────────────────────────────────────────────

/// Compute the winding number for many query points at once.
pub fn winding_numbers_batch(mesh: &MeshBuffers, queries: &[[f32; 3]]) -> Vec<f32> {
    queries.iter().map(|&q| winding_number(mesh, q)).collect()
}

/// Classify many points: `true` = inside, `false` = outside.
pub fn classify_points(mesh: &MeshBuffers, queries: &[[f32; 3]]) -> Vec<bool> {
    queries.iter().map(|&q| is_inside(mesh, q)).collect()
}

// ──────────────────────────────────────────────────────────────────────────────
// Mesh geometry helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Total surface area of the mesh (sum of all triangle areas).
///
/// Area of each triangle = 0.5 * |cross(B-A, C-A)|
pub fn mesh_surface_area(mesh: &MeshBuffers) -> f32 {
    let indices = &mesh.indices;
    let positions = &mesh.positions;

    let n_faces = indices.len() / 3;
    let mut total = 0.0f32;

    for i in 0..n_faces {
        let ia = indices[3 * i] as usize;
        let ib = indices[3 * i + 1] as usize;
        let ic = indices[3 * i + 2] as usize;

        let a = positions[ia];
        let b = positions[ib];
        let c = positions[ic];

        let ab = sub(b, a);
        let ac = sub(c, a);
        total += 0.5 * norm(cross(ab, ac));
    }

    total
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    // ── Tetrahedron helper ────────────────────────────────────────────────────
    //
    // Vertices:
    //   A = (0,0,0), B = (1,0,0), C = (0,1,0), D = (0,0,1)
    //
    // Faces with outward normals (counter-clockwise when viewed from outside):
    //   (A,C,B), (A,B,D), (A,D,C), (B,C,D)
    //
    fn tetrahedron() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![
                [0.0f32, 0.0, 0.0], // A = 0
                [1.0, 0.0, 0.0],    // B = 1
                [0.0, 1.0, 0.0],    // C = 2
                [0.0, 0.0, 1.0],    // D = 3
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0]; 4],
            indices: vec![
                0, 2, 1, // face A,C,B
                0, 1, 3, // face A,B,D
                0, 3, 2, // face A,D,C
                1, 2, 3, // face B,C,D
            ],
            has_suit: false,
        })
    }

    // ── triangle_solid_angle tests ────────────────────────────────────────────

    #[test]
    fn test_triangle_solid_angle_basic() {
        // A large triangle directly "wrapping" the query point should give a
        // non-zero solid angle.
        let a = [1.0f32, 0.0, 0.0];
        let b = [0.0, 1.0, 0.0];
        let c = [0.0, 0.0, 1.0];
        let q = [0.1, 0.1, 0.1];
        let sa = triangle_solid_angle(a, b, c, q);
        assert!(sa.abs() > 1e-4, "expected non-zero solid angle, got {}", sa);
    }

    #[test]
    fn test_triangle_solid_angle_degenerate() {
        // Query point coincides with vertex A → should return 0.
        let a = [1.0f32, 0.0, 0.0];
        let b = [0.0, 1.0, 0.0];
        let c = [0.0, 0.0, 1.0];
        let q = a; // same as vertex A
        let sa = triangle_solid_angle(a, b, c, q);
        assert_eq!(sa, 0.0, "degenerate case must return 0.0");
    }

    // ── winding_number tests ──────────────────────────────────────────────────

    #[test]
    fn test_winding_number_outside() {
        let mesh = tetrahedron();
        // Far outside the tetrahedron
        let w = winding_number(&mesh, [10.0, 10.0, 10.0]);
        assert!(
            w.abs() < WINDING_THRESHOLD,
            "point far outside should have |W| < 0.5, got {}",
            w
        );
    }

    #[test]
    fn test_winding_number_inside_single_triangle() {
        // Single-triangle "mesh" — winding number of a single open triangle
        // at a point in front of it should have a very small absolute value
        // (not a closed surface).
        let mesh = MeshBuffers::from_morph(MB {
            positions: vec![[1.0f32, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit: false,
        });
        let w = winding_number(&mesh, [0.1, 0.1, 0.1]);
        // |W| must be in [0, 1) for a single triangle
        assert!(
            w.abs() < 1.0,
            "single open triangle: |W| < 1.0 expected, got {}",
            w
        );
    }

    // ── is_inside tests ───────────────────────────────────────────────────────

    #[test]
    fn test_is_inside_point() {
        let mesh = tetrahedron();
        // Centroid of tetrahedron: (A+B+C+D)/4 = (0.25, 0.25, 0.25)
        let centroid = [0.25f32, 0.25, 0.25];
        assert!(
            is_inside(&mesh, centroid),
            "centroid of tetrahedron must be classified as inside"
        );

        // Far outside point
        assert!(
            !is_inside(&mesh, [10.0, 10.0, 10.0]),
            "far-outside point must not be inside"
        );
    }

    // ── winding_sign tests ────────────────────────────────────────────────────

    #[test]
    fn test_winding_sign() {
        let mesh = tetrahedron();
        let centroid = [0.25f32, 0.25, 0.25];
        assert_eq!(
            winding_sign(&mesh, centroid),
            -1.0,
            "inside point must return -1.0"
        );
        assert_eq!(
            winding_sign(&mesh, [10.0, 10.0, 10.0]),
            1.0,
            "outside point must return 1.0"
        );
    }

    // ── batch query tests ─────────────────────────────────────────────────────

    #[test]
    fn test_batch_winding_numbers() {
        let mesh = tetrahedron();
        let queries = vec![
            [0.25f32, 0.25, 0.25], // inside
            [10.0, 10.0, 10.0],    // outside
        ];
        let results = winding_numbers_batch(&mesh, &queries);
        assert_eq!(results.len(), 2);
        assert!(
            results[0].abs() >= WINDING_THRESHOLD,
            "first point should be inside"
        );
        assert!(
            results[1].abs() < WINDING_THRESHOLD,
            "second point should be outside"
        );
    }

    #[test]
    fn test_classify_points() {
        let mesh = tetrahedron();
        let queries = vec![
            [0.25f32, 0.25, 0.25], // inside
            [10.0, 10.0, 10.0],    // outside
            [0.1, 0.1, 0.1],       // inside (near but inside tetrahedron)
        ];
        let classification = classify_points(&mesh, &queries);
        assert_eq!(classification.len(), 3);
        assert!(classification[0], "centroid must be inside");
        assert!(!classification[1], "far point must be outside");
    }

    // ── mesh_surface_area tests ───────────────────────────────────────────────

    #[test]
    fn test_mesh_surface_area_triangle() {
        // A right-angle triangle with legs of length 1 → area = 0.5
        let mesh = MeshBuffers::from_morph(MB {
            positions: vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit: false,
        });
        let area = mesh_surface_area(&mesh);
        assert!(
            (area - 0.5).abs() < 1e-5,
            "right-angle unit triangle area must be 0.5, got {}",
            area
        );
    }

    #[test]
    fn test_mesh_surface_area_quad() {
        // A unit square split into two triangles → area = 1.0
        let mesh = MeshBuffers::from_morph(MB {
            positions: vec![
                [0.0f32, 0.0, 0.0], // 0
                [1.0, 0.0, 0.0],    // 1
                [1.0, 1.0, 0.0],    // 2
                [0.0, 1.0, 0.0],    // 3
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0]; 4],
            indices: vec![0, 1, 2, 0, 2, 3],
            has_suit: false,
        });
        let area = mesh_surface_area(&mesh);
        assert!(
            (area - 1.0).abs() < 1e-5,
            "unit-square area must be 1.0, got {}",
            area
        );
    }

    // ── constant & edge-case tests ────────────────────────────────────────────

    #[test]
    fn test_winding_threshold_constant() {
        assert_eq!(WINDING_THRESHOLD, 0.5f32);
    }

    #[test]
    fn test_batch_empty() {
        let mesh = tetrahedron();
        let results = winding_numbers_batch(&mesh, &[]);
        assert!(
            results.is_empty(),
            "empty query slice must yield empty result"
        );

        let classification = classify_points(&mesh, &[]);
        assert!(
            classification.is_empty(),
            "empty classify must yield empty result"
        );
    }
}
