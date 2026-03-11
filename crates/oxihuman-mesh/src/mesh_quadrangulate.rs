//! Convert triangle meshes to quad-dominant meshes.
//!
//! Pairs adjacent triangles that share an edge into quads when the resulting
//! quad meets a planarity threshold.  Unpaired triangles remain as triangles.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Config / structs
// ---------------------------------------------------------------------------

/// Configuration for the quadrangulation algorithm.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct QuadrangulateConfig {
    /// Maximum planarity error (distance of 4th vertex from the plane of the
    /// first three) allowed for a quad to be accepted.
    pub max_planarity_error: f32,
    /// If true, prefer pairing triangles whose shared edge is the longest.
    pub prefer_longest_edge: bool,
}

/// A single quad face: four vertex indices in order.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuadFace {
    /// Vertex indices of the quad (counter-clockwise winding).
    pub indices: [u32; 4],
}

/// Result of [`quadrangulate_mesh`].
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct QuadrangulateResult {
    /// Quads formed by pairing adjacent triangles.
    pub quads: Vec<QuadFace>,
    /// Triangle faces that could not be paired into a quad.
    pub remaining_tris: Vec<[u32; 3]>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Returns a [`QuadrangulateConfig`] with sensible defaults.
#[allow(dead_code)]
pub fn default_quadrangulate_config() -> QuadrangulateConfig {
    QuadrangulateConfig {
        max_planarity_error: 0.01,
        prefer_longest_edge: true,
    }
}

/// Pair adjacent triangles into quads where planarity allows.
#[allow(dead_code)]
pub fn quadrangulate_mesh(
    verts: &[[f32; 3]],
    faces: &[[u32; 3]],
    cfg: &QuadrangulateConfig,
) -> QuadrangulateResult {
    // Build edge -> face adjacency
    // For each canonical edge (a,b) store the face indices that use it
    let mut edge_faces: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for (fi, face) in faces.iter().enumerate() {
        let [a, b, c] = *face;
        for (p, q) in [(a, b), (b, c), (c, a)] {
            edge_faces.entry(edge_key(p, q)).or_default().push(fi);
        }
    }

    let mut used = vec![false; faces.len()];
    let mut quads: Vec<QuadFace> = Vec::new();
    let mut remaining_tris: Vec<[u32; 3]> = Vec::new();

    // Collect candidate pairings: (score, fi, fj, quad_indices)
    // Score = edge_length (descending) when prefer_longest_edge is set, else 0.
    let mut candidates: Vec<(f32, usize, usize, [u32; 4])> = Vec::new();

    for (&(ea, eb), adj_faces) in &edge_faces {
        if adj_faces.len() != 2 {
            continue;
        }
        let fi = adj_faces[0];
        let fj = adj_faces[1];

        // Find the quad vertices: the two non-shared vertices + shared edge
        let opp_i = opposite_vertex(&faces[fi], ea, eb);
        let opp_j = opposite_vertex(&faces[fj], ea, eb);

        // Build quad: opp_i, ea, opp_j, eb  (wind correctly)
        let quad_vs = [opp_i, ea, opp_j, eb];

        // Check planarity
        let v0 = verts[quad_vs[0] as usize];
        let v1 = verts[quad_vs[1] as usize];
        let v2 = verts[quad_vs[2] as usize];
        let v3 = verts[quad_vs[3] as usize];
        let arr = [v0, v1, v2, v3];
        let err = quad_planarity_error(&arr);
        if err > cfg.max_planarity_error {
            continue;
        }

        let score = if cfg.prefer_longest_edge {
            let va = verts[ea as usize];
            let vb = verts[eb as usize];
            edge_length(va, vb)
        } else {
            0.0
        };

        candidates.push((score, fi, fj, quad_vs));
    }

    // Sort descending by score so we prefer best pairs first
    candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    for (_, fi, fj, quad_vs) in candidates {
        if used[fi] || used[fj] {
            continue;
        }
        used[fi] = true;
        used[fj] = true;
        quads.push(QuadFace { indices: quad_vs });
    }

    for (fi, face) in faces.iter().enumerate() {
        if !used[fi] {
            remaining_tris.push(*face);
        }
    }

    QuadrangulateResult { quads, remaining_tris }
}

/// Returns the number of quads in the result.
#[allow(dead_code)]
pub fn quad_count(result: &QuadrangulateResult) -> usize {
    result.quads.len()
}

/// Returns the number of remaining (unpaired) triangles in the result.
#[allow(dead_code)]
pub fn remaining_tri_count(result: &QuadrangulateResult) -> usize {
    result.remaining_tris.len()
}

/// Split a quad back into two triangles.
#[allow(dead_code)]
pub fn quad_to_tris(quad: &QuadFace) -> [[u32; 3]; 2] {
    let [a, b, c, d] = quad.indices;
    [[a, b, c], [a, c, d]]
}

/// Returns true if the four vertices form a planar quad (within tolerance).
#[allow(dead_code)]
pub fn is_planar_quad(v: &[[f32; 3]; 4]) -> bool {
    quad_planarity_error(v) < 1e-4
}

/// Planarity error: distance of vertex 3 from the plane defined by 0,1,2.
#[allow(dead_code)]
pub fn quad_planarity_error(v: &[[f32; 3]; 4]) -> f32 {
    let v0 = v[0];
    let v1 = v[1];
    let v2 = v[2];
    let v3 = v[3];

    let ab = vsub(v1, v0);
    let ac = vsub(v2, v0);
    let n = cross(ab, ac);
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len < 1e-10 {
        return 0.0;
    }
    let n_unit = [n[0] / len, n[1] / len, n[2] / len];
    let d3 = vsub(v3, v0);
    (d3[0] * n_unit[0] + d3[1] * n_unit[1] + d3[2] * n_unit[2]).abs()
}

/// Convert a slice of quads into a flat triangle list (2 triangles per quad).
#[allow(dead_code)]
pub fn quads_to_triangle_list(quads: &[QuadFace]) -> Vec<[u32; 3]> {
    let mut tris = Vec::with_capacity(quads.len() * 2);
    for q in quads {
        let [a, b, c, d] = q.indices;
        tris.push([a, b, c]);
        tris.push([a, c, d]);
    }
    tris
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn edge_key(a: u32, b: u32) -> (u32, u32) {
    if a < b { (a, b) } else { (b, a) }
}

fn opposite_vertex(face: &[u32; 3], ea: u32, eb: u32) -> u32 {
    for &v in face {
        if v != ea && v != eb {
            return v;
        }
    }
    face[0]
}

fn vsub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn edge_length(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = vsub(b, a);
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn two_tris_square() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let verts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let faces = vec![[0, 1, 2], [0, 2, 3]];
        (verts, faces)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_quadrangulate_config();
        assert!(cfg.max_planarity_error > 0.0);
    }

    #[test]
    fn test_quadrangulate_planar_square() {
        let (v, f) = two_tris_square();
        let cfg = default_quadrangulate_config();
        let result = quadrangulate_mesh(&v, &f, &cfg);
        assert_eq!(quad_count(&result), 1);
        assert_eq!(remaining_tri_count(&result), 0);
    }

    #[test]
    fn test_quad_to_tris() {
        let q = QuadFace { indices: [0, 1, 2, 3] };
        let [t0, t1] = quad_to_tris(&q);
        assert_eq!(t0, [0, 1, 2]);
        assert_eq!(t1, [0, 2, 3]);
    }

    #[test]
    fn test_quads_to_triangle_list() {
        let q = QuadFace { indices: [0, 1, 2, 3] };
        let tris = quads_to_triangle_list(&[q]);
        assert_eq!(tris.len(), 2);
    }

    #[test]
    fn test_is_planar_quad_flat() {
        let v = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
        assert!(is_planar_quad(&v));
    }

    #[test]
    fn test_is_planar_quad_non_planar() {
        let v = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 1.0]];
        assert!(!is_planar_quad(&v));
    }

    #[test]
    fn test_planarity_error_zero_for_flat() {
        let v = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [2.0, 2.0, 0.0], [0.0, 2.0, 0.0]];
        assert!(quad_planarity_error(&v) < 1e-6);
    }

    #[test]
    fn test_isolated_triangle_not_paired() {
        // Three triangles sharing no edges with each other
        let verts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [2.5, 1.0, 0.0],
        ];
        let faces = vec![[0, 1, 2], [3, 4, 5]];
        let cfg = default_quadrangulate_config();
        let result = quadrangulate_mesh(&verts, &faces, &cfg);
        // These two tris share no edge, so no quads formed
        assert_eq!(quad_count(&result), 0);
        assert_eq!(remaining_tri_count(&result), 2);
    }

    #[test]
    fn test_non_planar_rejected() {
        // Force strict planarity threshold
        let verts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 5.0], // way off-plane
        ];
        let faces = vec![[0, 1, 2], [0, 2, 3]];
        let cfg = QuadrangulateConfig { max_planarity_error: 0.001, prefer_longest_edge: false };
        let result = quadrangulate_mesh(&verts, &faces, &cfg);
        assert_eq!(quad_count(&result), 0);
        assert_eq!(remaining_tri_count(&result), 2);
    }
}
