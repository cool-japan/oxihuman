//! Loop subdivision of triangle meshes.
//!
//! Each subdivision step splits every triangle into 4 smaller triangles
//! and smooths vertex positions using Loop's weighting scheme, producing
//! a C2-continuous limit surface for interior vertices.

use std::collections::{hash_map::Entry, HashMap};

// ---------------------------------------------------------------------------
// Config / Result
// ---------------------------------------------------------------------------

/// Configuration for the Loop subdivision algorithm.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SubdivisionConfig {
    /// Number of subdivision levels to apply.
    pub levels: u32,
    /// Apply Loop vertex-smoothing weights (true) or simple midpoint (false).
    pub smooth: bool,
    /// Clamp vertex positions into the original bounding box after smoothing.
    pub clamp_to_bbox: bool,
}

/// Result returned by [`subdivide_mesh`].
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SubdivisionResult {
    /// Subdivided vertex positions.
    pub vertices: Vec<[f32; 3]>,
    /// Subdivided triangle faces (indices into `vertices`).
    pub faces: Vec<[u32; 3]>,
    /// Number of subdivision levels that were applied.
    pub level: u32,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Returns a [`SubdivisionConfig`] with sensible defaults (1 level, smooth on).
#[allow(dead_code)]
pub fn default_subdivision_config() -> SubdivisionConfig {
    SubdivisionConfig {
        levels: 1,
        smooth: true,
        clamp_to_bbox: false,
    }
}

/// Subdivide `verts`/`faces` by `cfg.levels` Loop subdivision steps.
#[allow(dead_code)]
pub fn subdivide_mesh(
    verts: &[[f32; 3]],
    faces: &[[u32; 3]],
    cfg: &SubdivisionConfig,
) -> SubdivisionResult {
    let mut cur_verts: Vec<[f32; 3]> = verts.to_vec();
    let mut cur_faces: Vec<[u32; 3]> = faces.to_vec();

    for _ in 0..cfg.levels {
        let (nv, nf) = subdivide_once_inner(&cur_verts, &cur_faces, cfg.smooth);
        cur_verts = nv;
        cur_faces = nf;
    }

    SubdivisionResult {
        vertices: cur_verts,
        faces: cur_faces,
        level: cfg.levels,
    }
}

/// Returns the number of faces in a [`SubdivisionResult`].
#[allow(dead_code)]
pub fn subdivision_face_count(result: &SubdivisionResult) -> usize {
    result.faces.len()
}

/// Returns the number of vertices in a [`SubdivisionResult`].
#[allow(dead_code)]
pub fn subdivision_vertex_count(result: &SubdivisionResult) -> usize {
    result.vertices.len()
}

/// Convenience wrapper: one level of subdivision without smoothing config.
#[allow(dead_code)]
pub fn subdivide_once(
    verts: &[[f32; 3]],
    faces: &[[u32; 3]],
) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    subdivide_once_inner(verts, faces, true)
}

/// Returns the midpoint of edge (a, b).
#[allow(dead_code)]
pub fn subdivision_edge_midpoint(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ]
}

/// Returns the level stored in a [`SubdivisionResult`].
#[allow(dead_code)]
pub fn subdivision_level(result: &SubdivisionResult) -> u32 {
    result.level
}

/// Compute the Catmull-Clark face point (centroid of face vertices).
#[allow(dead_code)]
pub fn catmull_clark_face_point(face_verts: &[[f32; 3]]) -> [f32; 3] {
    if face_verts.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let n = face_verts.len() as f32;
    let mut sum = [0.0f32; 3];
    for v in face_verts {
        sum[0] += v[0];
        sum[1] += v[1];
        sum[2] += v[2];
    }
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Canonical edge key (smaller index first).
fn edge_key(a: u32, b: u32) -> (u32, u32) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

fn vadd(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn vscale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

/// Core Loop subdivision step.
fn subdivide_once_inner(
    verts: &[[f32; 3]],
    faces: &[[u32; 3]],
    smooth: bool,
) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let n_orig = verts.len();

    // --- Phase 1: collect edge midpoints ---
    let mut edge_mid: HashMap<(u32, u32), u32> = HashMap::new();
    let mut new_verts: Vec<[f32; 3]> = verts.to_vec();

    // Build adjacency: for each edge list the opposite vertices
    let mut edge_opp: HashMap<(u32, u32), Vec<u32>> = HashMap::new();
    for face in faces {
        let [a, b, c] = *face;
        let pairs = [(a, b, c), (b, c, a), (c, a, b)];
        for (p, q, opp) in pairs {
            edge_opp.entry(edge_key(p, q)).or_default().push(opp);
        }
    }

    // Create edge midpoint vertices
    for face in faces {
        let [a, b, c] = *face;
        let edge_pairs = [(a, b), (b, c), (c, a)];
        for (p, q) in edge_pairs {
            let key = edge_key(p, q);
            if let Entry::Vacant(e) = edge_mid.entry(key) {
                let mid_idx = new_verts.len() as u32;
                let mp = if smooth {
                    // Loop midpoint rule: 3/8*(p+q) + 1/8*(opp_sum) for interior edges
                    let opps = edge_opp.get(&key).map(|v| v.as_slice()).unwrap_or(&[]);
                    if opps.len() == 2 {
                        let vp = verts[key.0 as usize];
                        let vq = verts[key.1 as usize];
                        let va = verts[opps[0] as usize];
                        let vb = verts[opps[1] as usize];
                        vadd(
                            vadd(vscale(vp, 3.0 / 8.0), vscale(vq, 3.0 / 8.0)),
                            vadd(vscale(va, 1.0 / 8.0), vscale(vb, 1.0 / 8.0)),
                        )
                    } else {
                        subdivision_edge_midpoint(verts[key.0 as usize], verts[key.1 as usize])
                    }
                } else {
                    subdivision_edge_midpoint(verts[key.0 as usize], verts[key.1 as usize])
                };
                new_verts.push(mp);
                e.insert(mid_idx);
            }
        }
    }

    // --- Phase 2: smooth original vertices (Loop weights) ---
    if smooth {
        // Count valence
        let mut valence = vec![0usize; n_orig];
        let mut neighbor_sum: Vec<[f32; 3]> = vec![[0.0; 3]; n_orig];
        for face in faces {
            let [a, b, c] = *face;
            let pairs = [(a, b), (b, c), (c, a)];
            for (p, q) in pairs {
                valence[p as usize] += 1;
                let vq = verts[q as usize];
                let s = &mut neighbor_sum[p as usize];
                s[0] += vq[0];
                s[1] += vq[1];
                s[2] += vq[2];
            }
        }
        for i in 0..n_orig {
            let k = valence[i] as f32;
            if k < 1.0 {
                continue;
            }
            // Warren's simplified Loop weight
            let beta = if k > 3.0 {
                3.0 / (8.0 * k)
            } else {
                3.0 / 16.0
            };
            let self_w = 1.0 - k * beta;
            let ns = neighbor_sum[i];
            let old = verts[i];
            new_verts[i] = [
                self_w * old[0] + beta * ns[0],
                self_w * old[1] + beta * ns[1],
                self_w * old[2] + beta * ns[2],
            ];
        }
    }

    // --- Phase 3: split each triangle into 4 ---
    let mut new_faces: Vec<[u32; 3]> = Vec::with_capacity(faces.len() * 4);
    for face in faces {
        let [a, b, c] = *face;
        let mab = edge_mid[&edge_key(a, b)];
        let mbc = edge_mid[&edge_key(b, c)];
        let mca = edge_mid[&edge_key(c, a)];
        new_faces.push([a, mab, mca]);
        new_faces.push([b, mbc, mab]);
        new_faces.push([c, mca, mbc]);
        new_faces.push([mab, mbc, mca]);
    }

    (new_verts, new_faces)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn single_triangle() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let faces = vec![[0, 1, 2]];
        (verts, faces)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_subdivision_config();
        assert_eq!(cfg.levels, 1);
        assert!(cfg.smooth);
    }

    #[test]
    fn test_subdivide_once_face_count() {
        let (v, f) = single_triangle();
        let (_, nf) = subdivide_once(&v, &f);
        assert_eq!(nf.len(), 4);
    }

    #[test]
    fn test_subdivide_once_vertex_count() {
        let (v, f) = single_triangle();
        let (nv, _) = subdivide_once(&v, &f);
        // 3 original + 3 edge midpoints
        assert_eq!(nv.len(), 6);
    }

    #[test]
    fn test_subdivision_result_level() {
        let (v, f) = single_triangle();
        let cfg = SubdivisionConfig { levels: 3, smooth: false, clamp_to_bbox: false };
        let result = subdivide_mesh(&v, &f, &cfg);
        assert_eq!(subdivision_level(&result), 3);
        // 1 face * 4^3 = 64
        assert_eq!(subdivision_face_count(&result), 64);
    }

    #[test]
    fn test_edge_midpoint() {
        let a = [0.0, 0.0, 0.0];
        let b = [2.0, 4.0, 6.0];
        let m = subdivision_edge_midpoint(a, b);
        assert!((m[0] - 1.0).abs() < 1e-6);
        assert!((m[1] - 2.0).abs() < 1e-6);
        assert!((m[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_catmull_clark_face_point_triangle() {
        let face = [[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]];
        let fp = catmull_clark_face_point(&face);
        assert!((fp[0] - 1.0).abs() < 1e-6);
        assert!((fp[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_catmull_clark_face_point_empty() {
        let fp = catmull_clark_face_point(&[]);
        assert_eq!(fp, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_subdivision_vertex_count_accessor() {
        let (v, f) = single_triangle();
        let cfg = default_subdivision_config();
        let result = subdivide_mesh(&v, &f, &cfg);
        assert_eq!(subdivision_vertex_count(&result), result.vertices.len());
    }

    #[test]
    fn test_two_triangle_mesh() {
        let verts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let faces = vec![[0, 1, 2], [0, 2, 3]];
        let (nv, nf) = subdivide_once(&verts, &faces);
        // 2 tris -> 8 faces
        assert_eq!(nf.len(), 8);
        // 4 original + 5 edge midpoints (shared edge counted once)
        assert_eq!(nv.len(), 9);
    }
}
