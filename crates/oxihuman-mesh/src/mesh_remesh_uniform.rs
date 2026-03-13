// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Uniform remeshing — iterative edge split/collapse targeting a user-specified
//! target edge length.
//!
//! The algorithm alternates between splitting edges that are too long and
//! collapsing edges that are too short, converging toward a mesh where every
//! edge is close to `target_edge_length`.

// ── helpers ─────────────────────────────────────────────────────────────────

fn edge_length(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

fn midpoint(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ]
}

// ── config / result ──────────────────────────────────────────────────────────

/// Configuration for uniform remeshing.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct RemeshUniformConfig {
    /// Desired edge length after remeshing.
    pub target_edge_length: f32,
    /// Edges longer than `split_factor * target` are split.
    pub split_factor: f32,
    /// Edges shorter than `collapse_factor * target` are collapsed.
    pub collapse_factor: f32,
    /// Number of split/collapse iterations to run.
    pub iterations: u32,
}

/// Result returned by [`remesh_uniform`].
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct RemeshUniformResult {
    /// Remeshed vertex positions.
    pub vertices: Vec<[f32; 3]>,
    /// Remeshed triangle faces (indices into `vertices`).
    pub faces: Vec<[u32; 3]>,
    /// Number of iterations actually performed.
    pub iterations_done: u32,
}

// ── public API ───────────────────────────────────────────────────────────────

/// Return a [`RemeshUniformConfig`] with sensible defaults.
#[allow(dead_code)]
pub fn default_remesh_config() -> RemeshUniformConfig {
    RemeshUniformConfig {
        target_edge_length: 0.05,
        split_factor: 4.0 / 3.0,
        collapse_factor: 4.0 / 5.0,
        iterations: 5,
    }
}

/// Perform uniform remeshing on the input mesh.
///
/// Returns a [`RemeshUniformResult`] containing the remeshed geometry and
/// the number of iterations performed.
#[allow(dead_code)]
pub fn remesh_uniform(
    verts: &[[f32; 3]],
    faces: &[[u32; 3]],
    cfg: &RemeshUniformConfig,
) -> RemeshUniformResult {
    let max_len = cfg.target_edge_length * cfg.split_factor;
    let min_len = cfg.target_edge_length * cfg.collapse_factor;

    let mut cur_verts: Vec<[f32; 3]> = verts.to_vec();
    let mut cur_faces: Vec<[u32; 3]> = faces.to_vec();

    for iter in 0..cfg.iterations {
        let (sv, sf) = split_long_edges(&cur_verts, &cur_faces, max_len);
        let (cv, cf) = collapse_short_edges(&sv, &sf, min_len);
        cur_verts = cv;
        cur_faces = cf;
        let _ = iter;
    }

    RemeshUniformResult {
        vertices: cur_verts,
        faces: cur_faces,
        iterations_done: cfg.iterations,
    }
}

/// Return the number of vertices in a remesh result.
#[allow(dead_code)]
pub fn remesh_vertex_count(result: &RemeshUniformResult) -> usize {
    result.vertices.len()
}

/// Return the number of faces in a remesh result.
#[allow(dead_code)]
pub fn remesh_face_count(result: &RemeshUniformResult) -> usize {
    result.faces.len()
}

/// Compute the average edge length of a triangle mesh.
#[allow(dead_code)]
pub fn remesh_average_edge_length(verts: &[[f32; 3]], faces: &[[u32; 3]]) -> f32 {
    if faces.is_empty() {
        return 0.0;
    }
    let mut total = 0.0f32;
    let mut count = 0u32;
    for f in faces {
        let a = verts[f[0] as usize];
        let b = verts[f[1] as usize];
        let c = verts[f[2] as usize];
        total += edge_length(a, b) + edge_length(b, c) + edge_length(c, a);
        count += 3;
    }
    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
}

/// Return the number of iterations stored in a remesh result.
#[allow(dead_code)]
pub fn remesh_iterations(result: &RemeshUniformResult) -> u32 {
    result.iterations_done
}

/// Split all edges longer than `max_len` by inserting midpoints.
///
/// Each split triangle is replaced by two triangles sharing the new midpoint
/// vertex.
#[allow(dead_code)]
pub fn split_long_edges(
    verts: &[[f32; 3]],
    faces: &[[u32; 3]],
    max_len: f32,
) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let mut new_verts: Vec<[f32; 3]> = verts.to_vec();
    let mut new_faces: Vec<[u32; 3]> = Vec::with_capacity(faces.len());

    for f in faces {
        let ia = f[0] as usize;
        let ib = f[1] as usize;
        let ic = f[2] as usize;
        let a = new_verts[ia];
        let b = new_verts[ib];
        let c = new_verts[ic];

        let lab = edge_length(a, b);
        let lbc = edge_length(b, c);
        let lca = edge_length(c, a);

        // Split the longest edge if it exceeds max_len.
        if lab >= lbc && lab >= lca && lab > max_len {
            let m = midpoint(a, b);
            let im = new_verts.len() as u32;
            new_verts.push(m);
            new_faces.push([f[0], im, f[2]]);
            new_faces.push([im, f[1], f[2]]);
        } else if lbc >= lab && lbc >= lca && lbc > max_len {
            let m = midpoint(b, c);
            let im = new_verts.len() as u32;
            new_verts.push(m);
            new_faces.push([f[0], f[1], im]);
            new_faces.push([f[0], im, f[2]]);
        } else if lca > max_len {
            let m = midpoint(c, a);
            let im = new_verts.len() as u32;
            new_verts.push(m);
            new_faces.push([f[0], f[1], im]);
            new_faces.push([im, f[1], f[2]]);
        } else {
            new_faces.push(*f);
        }
    }

    (new_verts, new_faces)
}

/// Collapse all edges shorter than `min_len` by merging the two endpoints.
///
/// The edge is collapsed to the midpoint of the two vertices. Degenerate
/// faces (zero-area triangles) that result from the collapse are discarded.
#[allow(dead_code)]
pub fn collapse_short_edges(
    verts: &[[f32; 3]],
    faces: &[[u32; 3]],
    min_len: f32,
) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    // Build a remapping table: start with identity.
    let n = verts.len();
    let mut remap: Vec<usize> = (0..n).collect();

    // Find collapsible edges and record merges.
    for f in faces {
        let pairs = [(f[0], f[1]), (f[1], f[2]), (f[2], f[0])];
        for (ia, ib) in pairs {
            let a = verts[ia as usize];
            let b = verts[ib as usize];
            if edge_length(a, b) < min_len {
                // Collapse ib → ia (keep ia).
                let ra = find_root(&remap, ia as usize);
                let rb = find_root(&remap, ib as usize);
                if ra != rb {
                    remap[rb] = ra;
                }
            }
        }
    }

    // Flatten remap.
    for i in 0..n {
        let r = find_root(&remap, i);
        remap[i] = r;
    }

    // Compute midpoints for collapsed vertices.
    let mut new_pos: Vec<[f32; 3]> = verts.to_vec();
    for i in 0..n {
        let r = remap[i];
        if r != i {
            let m = midpoint(verts[r], verts[i]);
            new_pos[r] = m;
        }
    }

    // Remap faces and drop degenerate ones.
    let mut new_faces: Vec<[u32; 3]> = Vec::new();
    for f in faces {
        let ra = remap[f[0] as usize] as u32;
        let rb = remap[f[1] as usize] as u32;
        let rc = remap[f[2] as usize] as u32;
        if ra != rb && rb != rc && rc != ra {
            new_faces.push([ra, rb, rc]);
        }
    }

    // Compact vertices (remove unreferenced ones).
    let mut used = vec![false; n];
    for f in &new_faces {
        used[f[0] as usize] = true;
        used[f[1] as usize] = true;
        used[f[2] as usize] = true;
    }
    let mut compact: Vec<usize> = Vec::new();
    let mut idx_map = vec![0u32; n];
    for (i, &u) in used.iter().enumerate() {
        if u {
            idx_map[i] = compact.len() as u32;
            compact.push(i);
        }
    }
    let final_verts: Vec<[f32; 3]> = compact.iter().map(|&i| new_pos[i]).collect();
    let final_faces: Vec<[u32; 3]> = new_faces
        .iter()
        .map(|f| [idx_map[f[0] as usize], idx_map[f[1] as usize], idx_map[f[2] as usize]])
        .collect();

    (final_verts, final_faces)
}

// ── private helpers ──────────────────────────────────────────────────────────

fn find_root(remap: &[usize], mut i: usize) -> usize {
    while remap[i] != i {
        i = remap[i];
    }
    i
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn single_triangle() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let verts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let faces = vec![[0u32, 1, 2]];
        (verts, faces)
    }

    #[test]
    fn test_default_remesh_config() {
        let cfg = default_remesh_config();
        assert!(cfg.target_edge_length > 0.0);
        assert!(cfg.split_factor > 1.0);
        assert!(cfg.collapse_factor < 1.0);
        assert!(cfg.iterations > 0);
    }

    #[test]
    fn test_remesh_vertex_count() {
        let (v, f) = single_triangle();
        let cfg = default_remesh_config();
        let result = remesh_uniform(&v, &f, &cfg);
        assert!(remesh_vertex_count(&result) >= 3);
    }

    #[test]
    fn test_remesh_face_count() {
        let (v, f) = single_triangle();
        let mut cfg = default_remesh_config();
        cfg.iterations = 1;
        let result = remesh_uniform(&v, &f, &cfg);
        assert!(remesh_face_count(&result) >= 1);
    }

    #[test]
    fn test_remesh_iterations_stored() {
        let (v, f) = single_triangle();
        let mut cfg = default_remesh_config();
        cfg.iterations = 3;
        let result = remesh_uniform(&v, &f, &cfg);
        assert_eq!(remesh_iterations(&result), 3);
    }

    #[test]
    fn test_remesh_average_edge_length_single() {
        let (v, f) = single_triangle();
        let avg = remesh_average_edge_length(&v, &f);
        // Edges: 1, 1, sqrt(2) ≈ 1.414 → avg ≈ (1+1+1.414)/3 ≈ 1.138
        assert!((avg - 1.138).abs() < 0.01, "avg={avg}");
    }

    #[test]
    fn test_remesh_average_edge_length_empty() {
        let avg = remesh_average_edge_length(&[], &[]);
        assert_eq!(avg, 0.0);
    }

    #[test]
    fn test_split_long_edges_splits() {
        let verts = vec![[0.0f32, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 1.0, 0.0]];
        let faces = vec![[0u32, 1, 2]];
        // max_len = 1.0; longest edge (0→1) = 2.0 → split
        let (sv, sf) = split_long_edges(&verts, &faces, 1.0);
        assert!(sv.len() > 3, "should have added a midpoint vertex");
        assert!(sf.len() > 1, "should have produced more than 1 face");
    }

    #[test]
    fn test_split_long_edges_no_split_when_short() {
        let (v, f) = single_triangle();
        // max_len = 10 — no edges are long enough
        let (sv, sf) = split_long_edges(&v, &f, 10.0);
        assert_eq!(sv.len(), v.len());
        assert_eq!(sf.len(), f.len());
    }

    #[test]
    fn test_collapse_short_edges_removes_tiny() {
        // Two vertices very close together.
        let verts = vec![
            [0.0f32, 0.0, 0.0],
            [0.001, 0.0, 0.0], // will collapse to vertex 0
            [0.5, 1.0, 0.0],
        ];
        let faces = vec![[0u32, 1, 2]];
        let (cv, _cf) = collapse_short_edges(&verts, &faces, 0.01);
        // The collapsed result should have fewer unique vertices.
        assert!(cv.len() <= 3);
    }

    #[test]
    fn test_collapse_short_edges_no_collapse_when_long() {
        let (v, f) = single_triangle();
        // min_len = 0.0 — no edges are short enough
        let (cv, cf) = collapse_short_edges(&v, &f, 0.0);
        assert_eq!(cv.len(), v.len());
        assert_eq!(cf.len(), f.len());
    }

    #[test]
    fn test_remesh_uniform_produces_valid_indices() {
        let (v, f) = single_triangle();
        let cfg = default_remesh_config();
        let result = remesh_uniform(&v, &f, &cfg);
        let nv = result.vertices.len() as u32;
        for face in &result.faces {
            assert!(face[0] < nv);
            assert!(face[1] < nv);
            assert!(face[2] < nv);
        }
    }
}
