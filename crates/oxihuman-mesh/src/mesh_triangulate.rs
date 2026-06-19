// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Polygon triangulation (fan and ear-clipping).
//!
//! The ear-clipping algorithm correctly handles both convex and concave
//! (simple) polygons, unlike the fan method which only works for convex inputs.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum TriangulateMethod {
    Fan,
    EarClip,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TriangulateConfig {
    pub method: TriangulateMethod,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TriangulateResult {
    pub indices: Vec<u32>,
    pub triangle_count: usize,
}

#[allow(dead_code)]
pub fn default_triangulate_config() -> TriangulateConfig {
    TriangulateConfig { method: TriangulateMethod::Fan }
}

/// Triangulate a polygon using only vertex indices (no positional data).
/// For `EarClip`, this falls back to fan triangulation because positions are
/// required for the ear-clip winding / containment tests.
/// Use [`triangulate_polygon_3d`] or [`triangulate_earclip_2d`] when you have
/// vertex positions available.
#[allow(dead_code)]
pub fn triangulate_polygon(polygon: &[u32], config: &TriangulateConfig) -> TriangulateResult {
    match config.method {
        TriangulateMethod::Fan => triangulate_fan(polygon),
        // Without positions we cannot run ear-clip properly; fan is the best we
        // can do for a topology-only call.
        TriangulateMethod::EarClip => triangulate_fan(polygon),
    }
}

/// Triangulate a 3D polygon by projecting to 2D (x,y), then dispatching to
/// the configured method.
///
/// - `Fan` → [`triangulate_fan`]
/// - `EarClip` → [`triangulate_earclip_2d`] with the x,y projection
#[allow(dead_code)]
pub fn triangulate_polygon_3d(
    polygon: &[u32],
    positions: &[[f32; 3]],
    config: &TriangulateConfig,
) -> TriangulateResult {
    match config.method {
        TriangulateMethod::Fan => triangulate_fan(polygon),
        TriangulateMethod::EarClip => {
            let positions_2d: Vec<[f32; 2]> = positions.iter().map(|p| [p[0], p[1]]).collect();
            triangulate_earclip_2d(polygon, &positions_2d)
        }
    }
}

#[allow(dead_code)]
pub fn triangulate_fan(polygon: &[u32]) -> TriangulateResult {
    if polygon.len() < 3 {
        return TriangulateResult { indices: vec![], triangle_count: 0 };
    }
    let mut indices = Vec::new();
    let pivot = polygon[0];
    for i in 1..(polygon.len() - 1) {
        indices.push(pivot);
        indices.push(polygon[i]);
        indices.push(polygon[i + 1]);
    }
    let triangle_count = indices.len() / 3;
    TriangulateResult { indices, triangle_count }
}

#[allow(dead_code)]
pub fn triangulate_triangle_count(result: &TriangulateResult) -> usize {
    result.triangle_count
}

#[allow(dead_code)]
pub fn triangulate_validate(polygon: &[u32]) -> bool {
    polygon.len() >= 3
}

#[allow(dead_code)]
pub fn triangulate_to_json(result: &TriangulateResult) -> String {
    format!(
        r#"{{"triangle_count":{},"index_count":{}}}"#,
        result.triangle_count,
        result.indices.len()
    )
}

// ---------------------------------------------------------------------------
// Ear-clip implementation
// ---------------------------------------------------------------------------

/// Signed area of the triangle formed by three 2-D points (cross product).
/// Positive → CCW turn, negative → CW turn, zero → collinear.
#[inline]
fn cross2d(o: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
}

/// Returns `true` when point `p` lies strictly inside (or on the boundary of)
/// the triangle `(a, b, c)`.  Uses the sign-consistency test: all three
/// cross products must have the same sign.
#[inline]
fn point_in_triangle(p: [f32; 2], a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> bool {
    let d1 = cross2d(p, a, b);
    let d2 = cross2d(p, b, c);
    let d3 = cross2d(p, c, a);
    let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(has_neg && has_pos)
}

/// Compute the signed area of a 2-D polygon to determine winding order.
/// Positive → CCW, negative → CW.
fn polygon_signed_area(polygon: &[u32], positions: &[[f32; 2]]) -> f32 {
    let n = polygon.len();
    let mut area = 0.0f32;
    for i in 0..n {
        let j = (i + 1) % n;
        let pi = positions[polygon[i] as usize];
        let pj = positions[polygon[j] as usize];
        area += pi[0] * pj[1];
        area -= pj[0] * pi[1];
    }
    area * 0.5
}

/// Triangulate a simple polygon using the ear-clipping algorithm.
///
/// `polygon` — vertex indices into `positions` (in winding order; the function
///             normalises to CCW internally).
/// `positions` — 2-D vertex positions (z = 0 plane, or pre-projected).
///
/// The algorithm correctly handles both convex and concave simple polygons.
/// It does **not** handle polygons with self-intersections or holes.
#[allow(dead_code)]
pub fn triangulate_earclip_2d(polygon: &[u32], positions: &[[f32; 2]]) -> TriangulateResult {
    let n = polygon.len();
    if n < 3 {
        return TriangulateResult { indices: vec![], triangle_count: 0 };
    }
    if n == 3 {
        return TriangulateResult {
            indices: vec![polygon[0], polygon[1], polygon[2]],
            triangle_count: 1,
        };
    }

    // Normalise to CCW winding so the convexity test (positive cross product)
    // is consistent regardless of the input winding.
    let mut remaining: Vec<u32> = polygon.to_vec();
    if polygon_signed_area(&remaining, positions) < 0.0 {
        remaining.reverse();
    }

    let mut result_indices: Vec<u32> = Vec::with_capacity((n - 2) * 3);

    // Safety valve: if an entire pass finds no ear we are stuck (degenerate or
    // self-intersecting input).  We allow at most n² iterations before giving
    // up and draining any remaining vertices.
    let max_iters = n * n + n;
    let mut iters = 0usize;

    while remaining.len() > 3 {
        let m = remaining.len();
        let mut ear_found = false;

        for i in 0..m {
            iters += 1;
            if iters > max_iters {
                break;
            }
            let prev_idx = remaining[(i + m - 1) % m];
            let curr_idx = remaining[i];
            let next_idx = remaining[(i + 1) % m];

            let prev_pos = positions[prev_idx as usize];
            let curr_pos = positions[curr_idx as usize];
            let next_pos = positions[next_idx as usize];

            // ── Convexity test ────────────────────────────────────────────
            // For a CCW polygon the cross product at curr must be ≥ 0.
            let cp = cross2d(prev_pos, curr_pos, next_pos);
            if cp <= 0.0 {
                // Reflex vertex or degenerate — skip
                continue;
            }

            // ── No-other-vertex-inside test ───────────────────────────────
            let mut has_interior_point = false;
            for j in 0..m {
                if j == (i + m - 1) % m || j == i || j == (i + 1) % m {
                    continue;
                }
                let p = positions[remaining[j] as usize];
                // Use a strict containment test: exclude the triangle's own
                // boundary vertices to avoid false positives on shared edges.
                if point_in_triangle(p, prev_pos, curr_pos, next_pos) {
                    // Only a true interior point (not a boundary shared vertex)
                    // disqualifies the ear.  Check that p is not one of the
                    // three triangle corners (floating-point equality is fine
                    // here because the coordinates come directly from the same
                    // positions array).
                    if p != prev_pos && p != curr_pos && p != next_pos {
                        has_interior_point = true;
                        break;
                    }
                }
            }

            if has_interior_point {
                continue;
            }

            // ── Ear found ─────────────────────────────────────────────────
            result_indices.push(prev_idx);
            result_indices.push(curr_idx);
            result_indices.push(next_idx);
            remaining.remove(i);
            ear_found = true;
            break;
        }

        if iters > max_iters || !ear_found {
            // Degenerate polygon: emit remaining as a fan to avoid hanging.
            let pivot = remaining[0];
            for k in 1..(remaining.len() - 1) {
                result_indices.push(pivot);
                result_indices.push(remaining[k]);
                result_indices.push(remaining[k + 1]);
            }
            let triangle_count = result_indices.len() / 3;
            return TriangulateResult { indices: result_indices, triangle_count };
        }
    }

    // Emit the last triangle
    result_indices.push(remaining[0]);
    result_indices.push(remaining[1]);
    result_indices.push(remaining[2]);

    let triangle_count = result_indices.len() / 3;
    TriangulateResult { indices: result_indices, triangle_count }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fan_triangle_from_quad() {
        let quad = vec![0u32, 1, 2, 3];
        let res = triangulate_fan(&quad);
        assert_eq!(res.triangle_count, 2);
        assert_eq!(res.indices.len(), 6);
    }

    #[test]
    fn fan_triangle_from_pentagon() {
        let pent = vec![0u32, 1, 2, 3, 4];
        let res = triangulate_fan(&pent);
        assert_eq!(res.triangle_count, 3);
    }

    #[test]
    fn triangle_unchanged() {
        let tri = vec![0u32, 1, 2];
        let res = triangulate_fan(&tri);
        assert_eq!(res.triangle_count, 1);
    }

    #[test]
    fn less_than_three_returns_empty() {
        let poly = vec![0u32, 1];
        let res = triangulate_fan(&poly);
        assert_eq!(res.triangle_count, 0);
        assert!(res.indices.is_empty());
    }

    #[test]
    fn validate_rejects_small_polygon() {
        assert!(!triangulate_validate(&[0u32, 1]));
        assert!(triangulate_validate(&[0u32, 1, 2]));
    }

    #[test]
    fn default_method_is_fan() {
        let cfg = default_triangulate_config();
        assert_eq!(cfg.method, TriangulateMethod::Fan);
    }

    #[test]
    fn earclip_method_produces_triangles() {
        let cfg = TriangulateConfig { method: TriangulateMethod::EarClip };
        let quad = vec![0u32, 1, 2, 3];
        let res = triangulate_polygon(&quad, &cfg);
        assert_eq!(res.triangle_count, 2);
    }

    #[test]
    fn to_json_correct_format() {
        let quad = vec![0u32, 1, 2, 3];
        let res = triangulate_fan(&quad);
        let json = triangulate_to_json(&res);
        assert!(json.contains("triangle_count"));
        assert!(json.contains("index_count"));
    }

    // ── Ear-clip tests ──────────────────────────────────────────────────────

    #[test]
    fn earclip_concave_l_shape() {
        // L-shaped polygon (concave) — fan would produce wrong triangulation.
        // Vertices in CCW order:
        // (0,0) → (2,0) → (2,1) → (1,1) → (1,2) → (0,2)
        let positions = [
            [0.0f32, 0.0],
            [2.0, 0.0],
            [2.0, 1.0],
            [1.0, 1.0],
            [1.0, 2.0],
            [0.0, 2.0],
        ];
        let polygon = [0u32, 1, 2, 3, 4, 5];
        let res = triangulate_earclip_2d(&polygon, &positions);
        assert_eq!(res.triangle_count, 4, "hexagon → 4 triangles, got {}", res.triangle_count);
        assert_eq!(res.indices.len(), 12);
    }

    #[test]
    fn earclip_quad_matches_fan_for_convex() {
        // For a convex quad, ear-clip and fan should produce the same triangle count.
        let positions = [[0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let polygon = [0u32, 1, 2, 3];
        let res = triangulate_earclip_2d(&polygon, &positions);
        assert_eq!(res.triangle_count, 2);
    }

    #[test]
    fn earclip_triangle_direct() {
        let positions = [[0.0f32, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let polygon = [0u32, 1, 2];
        let res = triangulate_earclip_2d(&polygon, &positions);
        assert_eq!(res.triangle_count, 1);
        assert_eq!(res.indices.len(), 3);
    }

    #[test]
    fn earclip_empty_input() {
        let positions: [[f32; 2]; 0] = [];
        let polygon: [u32; 0] = [];
        let res = triangulate_earclip_2d(&polygon, &positions);
        assert_eq!(res.triangle_count, 0);
        assert!(res.indices.is_empty());
    }

    #[test]
    fn earclip_cw_polygon_normalised() {
        // Same quad as earclip_quad_matches_fan_for_convex but in CW order.
        // The function should auto-reverse and still produce 2 triangles.
        let positions = [[0.0f32, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];
        let polygon = [0u32, 1, 2, 3];
        let res = triangulate_earclip_2d(&polygon, &positions);
        assert_eq!(res.triangle_count, 2);
    }

    #[test]
    fn triangulate_polygon_3d_earclip_square() {
        let positions_3d = [
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let polygon = [0u32, 1, 2, 3];
        let cfg = TriangulateConfig { method: TriangulateMethod::EarClip };
        let res = triangulate_polygon_3d(&polygon, &positions_3d, &cfg);
        assert_eq!(res.triangle_count, 2);
    }

    #[test]
    fn triangulate_polygon_3d_fan_square() {
        let positions_3d = [
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let polygon = [0u32, 1, 2, 3];
        let cfg = TriangulateConfig { method: TriangulateMethod::Fan };
        let res = triangulate_polygon_3d(&polygon, &positions_3d, &cfg);
        assert_eq!(res.triangle_count, 2);
    }
}
