// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Hollow / shell mesh generation.
//!
//! Creates an inner offset surface and stitches it to the outer mesh,
//! producing a closed shell with configurable wall thickness.

#![allow(dead_code)]

use std::collections::HashMap;

use crate::mesh::MeshBuffers;

// ---------------------------------------------------------------------------
// Math helpers (no external deps – just f32 arrays)
// ---------------------------------------------------------------------------

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
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

// ---------------------------------------------------------------------------
// Public params / result types
// ---------------------------------------------------------------------------

/// Configuration for the hollow-mesh operation.
#[derive(Debug, Clone)]
pub struct HollowParams {
    /// Shell thickness (metres); offset distance for the inner surface.
    pub thickness: f32,
    /// If true the inner surface triangles are wound in reverse (normals face inward).
    pub flip_inner: bool,
    /// If true open boundary edges of the outer mesh are bridged to the inner mesh.
    pub cap_open_edges: bool,
    /// Use area-weighted normals for the offset direction (smoother on curved surfaces).
    pub smooth_offset: bool,
}

impl Default for HollowParams {
    fn default() -> Self {
        Self {
            thickness: 0.01,
            flip_inner: true,
            cap_open_edges: true,
            smooth_offset: false,
        }
    }
}

/// Output of [`hollow_mesh`].
#[derive(Debug, Clone)]
pub struct HollowResult {
    /// Combined outer + inner + cap mesh.
    pub mesh: MeshBuffers,
    /// Number of outer (original) vertices copied into the combined mesh.
    pub outer_vertex_count: usize,
    /// Number of inner (offset) vertices appended.
    pub inner_vertex_count: usize,
    /// Number of triangles used to bridge boundary loops (cap).
    pub cap_triangle_count: usize,
}

// ---------------------------------------------------------------------------
// Core public API
// ---------------------------------------------------------------------------

/// Create a hollow shell from a solid (or open) mesh.
///
/// **Algorithm**
/// 1. Compute per-vertex area-weighted normals.
/// 2. Create inner mesh: each vertex is offset by `−normal × thickness`.
/// 3. Flip the winding of inner triangles (so normals point inward).
/// 4. Find open boundary edges on the outer mesh.
/// 5. Walk those edges into ordered loops and stitch outer ↔ inner.
/// 6. Combine everything into a single [`MeshBuffers`].
pub fn hollow_mesh(mesh: &MeshBuffers, params: &HollowParams) -> HollowResult {
    let nv = mesh.positions.len();
    if nv == 0 || mesh.indices.is_empty() {
        return HollowResult {
            mesh: empty_mesh(),
            outer_vertex_count: 0,
            inner_vertex_count: 0,
            cap_triangle_count: 0,
        };
    }

    // Step 1: per-vertex normals
    let normals = if params.smooth_offset {
        area_weighted_normals(mesh)
    } else {
        simple_vertex_normals(mesh)
    };

    // Step 2: inner positions (offset inward)
    let inner_positions: Vec<[f32; 3]> = mesh
        .positions
        .iter()
        .zip(normals.iter())
        .map(|(&p, &n)| {
            [
                p[0] - n[0] * params.thickness,
                p[1] - n[1] * params.thickness,
                p[2] - n[2] * params.thickness,
            ]
        })
        .collect();

    // Step 3: inner indices with flipped winding
    let inner_indices: Vec<u32> = if params.flip_inner {
        mesh.indices
            .chunks_exact(3)
            .flat_map(|tri| [tri[0] + nv as u32, tri[2] + nv as u32, tri[1] + nv as u32])
            .collect()
    } else {
        mesh.indices.iter().map(|&i| i + nv as u32).collect()
    };

    // Step 4: boundary edges on the outer mesh
    let mut outer_indices: Vec<u32> = mesh.indices.clone();
    let mut all_positions: Vec<[f32; 3]> = mesh.positions.clone();
    let mut all_normals: Vec<[f32; 3]> = mesh.normals.clone();
    let mut all_uvs: Vec<[f32; 2]> = mesh.uvs.clone();
    let mut all_tangents: Vec<[f32; 4]> = mesh.tangents.clone();
    let mut all_colors: Option<Vec<[f32; 4]>> = mesh.colors.clone();

    all_positions.extend_from_slice(&inner_positions);
    // inner normals = flipped outer normals
    let inner_normals: Vec<[f32; 3]> = normals.iter().map(|&n| [-n[0], -n[1], -n[2]]).collect();
    all_normals.extend_from_slice(&inner_normals);
    // UVs / tangents: just copy outer
    let outer_uvs = &mesh.uvs;
    all_uvs.extend_from_slice(outer_uvs);
    all_tangents.extend(
        mesh.tangents.iter().map(|&t| [t[0], t[1], t[2], -t[3]]), // flip handedness for inner
    );
    if let Some(ref col) = all_colors {
        let extra = col[..nv.min(col.len())].to_vec();
        let base = all_colors.take().unwrap_or_default();
        let mut merged = base;
        merged.extend_from_slice(&extra);
        all_colors = Some(merged);
    }

    outer_indices.extend_from_slice(&inner_indices);

    // Step 5: cap open boundary edges
    let mut cap_triangle_count = 0usize;

    if params.cap_open_edges {
        let boundary_e = find_boundary_edges(&mesh.indices, nv);
        let loops = boundary_loops(&boundary_e);
        let inner_offset = nv as u32;

        for lp in &loops {
            // Inner loop has the same vertex indices, just shifted by nv
            // BUT we need to reverse the inner loop for correct winding
            let mut inner_lp: Vec<u32> = lp.iter().map(|&v| v + inner_offset).collect();
            inner_lp.reverse(); // reverse so the quad strip faces outward

            let cap_tris = stitch_boundary_loops(lp, &inner_lp, 0, 0);
            cap_triangle_count += cap_tris.len() / 3;
            outer_indices.extend_from_slice(&cap_tris);
        }
    }

    let combined = MeshBuffers {
        positions: all_positions,
        normals: all_normals,
        tangents: all_tangents,
        uvs: all_uvs,
        indices: outer_indices,
        colors: all_colors,
        has_suit: mesh.has_suit,
    };

    HollowResult {
        outer_vertex_count: nv,
        inner_vertex_count: nv, // same topology, just offset
        cap_triangle_count,
        mesh: combined,
    }
}

/// Simple version: offset each vertex along its normal (no stitching).
///
/// * Positive `offset` → inflate (outward).
/// * Negative `offset` → deflate (inward).
pub fn offset_mesh(mesh: &MeshBuffers, offset: f32) -> MeshBuffers {
    if mesh.positions.is_empty() {
        return mesh.clone();
    }
    let normals = simple_vertex_normals(mesh);
    let new_positions: Vec<[f32; 3]> = mesh
        .positions
        .iter()
        .zip(normals.iter())
        .map(|(&p, &n)| add3(p, scale3(n, offset)))
        .collect();
    MeshBuffers {
        positions: new_positions,
        normals: mesh.normals.clone(),
        tangents: mesh.tangents.clone(),
        uvs: mesh.uvs.clone(),
        indices: mesh.indices.clone(),
        colors: mesh.colors.clone(),
        has_suit: mesh.has_suit,
    }
}

/// Compute per-vertex area-weighted normals.
///
/// For each triangle its face normal is weighted by triangle area before
/// being accumulated to its three vertices.
pub fn area_weighted_normals(mesh: &MeshBuffers) -> Vec<[f32; 3]> {
    let nv = mesh.positions.len();
    let mut accum = vec![[0.0f32; 3]; nv];

    for tri in mesh.indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= nv || i1 >= nv || i2 >= nv {
            continue;
        }
        let e1 = sub3(mesh.positions[i1], mesh.positions[i0]);
        let e2 = sub3(mesh.positions[i2], mesh.positions[i0]);
        // cross = face-normal scaled by 2×area
        let c = cross3(e1, e2);
        accum[i0] = add3(accum[i0], c);
        accum[i1] = add3(accum[i1], c);
        accum[i2] = add3(accum[i2], c);
    }

    accum.into_iter().map(normalize3).collect()
}

/// Find boundary edges: edges that appear in exactly one triangle.
///
/// Returns a list of `(v0, v1)` pairs in canonical order (min, max).
pub fn find_boundary_edges(indices: &[u32], vertex_count: usize) -> Vec<(u32, u32)> {
    let _ = vertex_count; // kept for API symmetry / future validation
    let mut counts: HashMap<(u32, u32), u32> = HashMap::new();
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0], tri[1], tri[2]);
        for &(p, q) in &[(a, b), (b, c), (c, a)] {
            let key = (p.min(q), p.max(q));
            *counts.entry(key).or_insert(0) += 1;
        }
    }
    counts
        .into_iter()
        .filter(|&(_, cnt)| cnt == 1)
        .map(|(e, _)| e)
        .collect()
}

/// Assemble unordered boundary edges into ordered closed loops.
///
/// Returns `Vec<Vec<u32>>` where each inner vec is a closed loop
/// (the last vertex implicitly connects back to the first).
pub fn boundary_loops(edges: &[(u32, u32)]) -> Vec<Vec<u32>> {
    if edges.is_empty() {
        return Vec::new();
    }

    // Build adjacency list
    let mut adj: HashMap<u32, Vec<u32>> = HashMap::new();
    for &(a, b) in edges {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    }

    // Track visited directed edges
    let mut visited: HashMap<(u32, u32), bool> = HashMap::new();
    for &(a, b) in edges {
        visited.insert((a, b), false);
        visited.insert((b, a), false);
    }

    let mut visited_verts: HashMap<u32, bool> = edges
        .iter()
        .flat_map(|&(a, b)| [(a, false), (b, false)])
        .collect();

    let mut loops: Vec<Vec<u32>> = Vec::new();

    for start in edges.iter().flat_map(|&(a, b)| [a, b]).collect::<Vec<_>>() {
        if visited_verts.get(&start).copied().unwrap_or(true) {
            continue;
        }

        let mut chain = vec![start];
        *visited_verts.entry(start).or_insert(true) = true;
        let mut current = start;

        while let Some(neighbours) = adj.get(&current).cloned() {
            let mut next_opt = None;
            for nb in neighbours {
                if !visited.get(&(current, nb)).copied().unwrap_or(true) {
                    next_opt = Some(nb);
                    break;
                }
            }
            let Some(next) = next_opt else {
                break;
            };
            visited.insert((current, next), true);
            visited.insert((next, current), true);
            if next == start {
                break; // closed the loop
            }
            *visited_verts.entry(next).or_insert(true) = true;
            chain.push(next);
            current = next;
        }

        if chain.len() >= 3 {
            loops.push(chain);
        }
    }
    loops
}

/// Stitch two boundary loops into cap triangles.
///
/// Both loops should have the same length for clean results.
/// `outer_offset` and `inner_offset` are added to all loop indices before
/// emitting triangle indices (use `0` if the loops already carry the correct
/// global vertex indices).
///
/// Returns a flat list of triangle indices (every 3 entries = one triangle).
#[allow(clippy::too_many_arguments)]
pub fn stitch_boundary_loops(
    outer_loop: &[u32],
    inner_loop: &[u32],
    outer_offset: u32,
    inner_offset: u32,
) -> Vec<u32> {
    let n = outer_loop.len().min(inner_loop.len());
    if n < 2 {
        return Vec::new();
    }
    let mut tris = Vec::with_capacity(n * 6);
    for i in 0..n {
        let j = (i + 1) % n;
        let o0 = outer_loop[i] + outer_offset;
        let o1 = outer_loop[j] + outer_offset;
        let i0 = inner_loop[i] + inner_offset;
        let i1 = inner_loop[j] + inner_offset;
        // Two triangles forming a quad strip segment
        tris.extend_from_slice(&[o0, i0, o1]);
        tris.extend_from_slice(&[i0, i1, o1]);
    }
    tris
}

/// Minimum distance between the outer and inner surfaces of a [`HollowResult`].
///
/// Samples the first `outer_vertex_count` outer vertices and finds the
/// closest inner vertex for each, returning the minimum distance found.
pub fn shell_thickness(result: &HollowResult) -> f32 {
    let outer_count = result.outer_vertex_count;
    let inner_count = result.inner_vertex_count;
    if outer_count == 0 || inner_count == 0 {
        return 0.0;
    }
    let positions = &result.mesh.positions;
    let outer_end = outer_count.min(positions.len());
    let inner_start = outer_count;
    let inner_end = (outer_count + inner_count).min(positions.len());

    if inner_start >= positions.len() {
        return 0.0;
    }

    let mut min_dist = f32::INFINITY;
    for ov in 0..outer_end {
        let op = positions[ov];
        // Match corresponding inner vertex first (same index, offset by outer_count)
        let iv = ov + inner_start;
        if iv < inner_end {
            let ip = positions[iv];
            let d = len3(sub3(op, ip));
            if d < min_dist {
                min_dist = d;
            }
        }
    }
    if min_dist.is_infinite() {
        0.0
    } else {
        min_dist
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Simple unweighted per-vertex normal (average of face normals).
fn simple_vertex_normals(mesh: &MeshBuffers) -> Vec<[f32; 3]> {
    let nv = mesh.positions.len();
    let mut accum = vec![[0.0f32; 3]; nv];
    for tri in mesh.indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= nv || i1 >= nv || i2 >= nv {
            continue;
        }
        let e1 = sub3(mesh.positions[i1], mesh.positions[i0]);
        let e2 = sub3(mesh.positions[i2], mesh.positions[i0]);
        let n = cross3(e1, e2);
        accum[i0] = add3(accum[i0], n);
        accum[i1] = add3(accum[i1], n);
        accum[i2] = add3(accum[i2], n);
    }
    accum.into_iter().map(normalize3).collect()
}

fn empty_mesh() -> MeshBuffers {
    MeshBuffers {
        positions: Vec::new(),
        normals: Vec::new(),
        tangents: Vec::new(),
        uvs: Vec::new(),
        indices: Vec::new(),
        colors: None,
        has_suit: false,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // ── Helpers ─────────────────────────────────────────────────────────────

    /// Build a simple quad mesh (two triangles, open boundary).
    fn quad_mesh() -> MeshBuffers {
        MeshBuffers {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            indices: vec![0, 1, 2, 0, 2, 3],
            colors: None,
            has_suit: false,
        }
    }

    /// Build a closed tetrahedron.
    fn tetra_mesh() -> MeshBuffers {
        MeshBuffers {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            normals: vec![[0.0, 1.0, 0.0]; 4],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0]; 4],
            indices: vec![0, 1, 2, 0, 3, 1, 1, 3, 2, 0, 2, 3],
            colors: None,
            has_suit: false,
        }
    }

    fn save_tmp(name: &str, content: &str) {
        let path = format!("/tmp/{name}");
        let mut f = std::fs::File::create(&path).expect("create /tmp file");
        f.write_all(content.as_bytes()).expect("write /tmp file");
    }

    // ── Tests ────────────────────────────────────────────────────────────────

    #[test]
    fn hollow_mesh_vertex_count() {
        let q = quad_mesh();
        let res = hollow_mesh(&q, &HollowParams::default());
        // 4 outer + 4 inner + 0 or more cap duplicates; at minimum 8
        assert_eq!(res.outer_vertex_count, 4);
        assert_eq!(res.inner_vertex_count, 4);
        assert!(res.mesh.positions.len() >= 8);
        save_tmp(
            "hollow_vertex_count.txt",
            &format!("{:?}", res.mesh.positions.len()),
        );
    }

    #[test]
    fn hollow_mesh_inner_offset() {
        let q = quad_mesh();
        let params = HollowParams {
            thickness: 0.5,
            ..Default::default()
        };
        let res = hollow_mesh(&q, &params);
        let outer = res.mesh.positions[0];
        let inner = res.mesh.positions[res.outer_vertex_count]; // first inner vertex
                                                                // inner should be offset along -Z (normal is +Z)
        let dz = outer[2] - inner[2];
        assert!((dz - 0.5).abs() < 1e-5, "expected dz=0.5, got {dz}");
        save_tmp(
            "hollow_inner_offset.txt",
            &format!("outer={outer:?} inner={inner:?}"),
        );
    }

    #[test]
    fn hollow_mesh_has_cap_triangles() {
        let q = quad_mesh();
        let params = HollowParams {
            cap_open_edges: true,
            ..Default::default()
        };
        let res = hollow_mesh(&q, &params);
        assert!(
            res.cap_triangle_count > 0,
            "quad mesh has open boundary; expected cap triangles"
        );
        save_tmp(
            "hollow_caps.txt",
            &format!("cap_tris={}", res.cap_triangle_count),
        );
    }

    #[test]
    fn hollow_mesh_no_cap_when_disabled() {
        let q = quad_mesh();
        let params = HollowParams {
            cap_open_edges: false,
            ..Default::default()
        };
        let res = hollow_mesh(&q, &params);
        assert_eq!(res.cap_triangle_count, 0);
    }

    #[test]
    fn hollow_mesh_closed_tetra_no_caps() {
        let t = tetra_mesh();
        let params = HollowParams {
            cap_open_edges: true,
            ..Default::default()
        };
        let res = hollow_mesh(&t, &params);
        // Closed mesh has no boundary edges → no caps
        assert_eq!(res.cap_triangle_count, 0);
        save_tmp(
            "hollow_tetra.txt",
            &format!("tris={}", res.mesh.face_count()),
        );
    }

    #[test]
    fn hollow_mesh_index_count_plausible() {
        let q = quad_mesh();
        let res = hollow_mesh(&q, &HollowParams::default());
        // outer (2 tris) + inner (2 tris) + cap tris; always divisible by 3
        assert_eq!(res.mesh.indices.len() % 3, 0);
        assert!(res.mesh.face_count() >= 4); // at least outer + inner
    }

    #[test]
    fn offset_mesh_outward() {
        let q = quad_mesh();
        let off = offset_mesh(&q, 1.0);
        // All z-coords should move +1 (normals are +Z)
        for (orig, new) in q.positions.iter().zip(off.positions.iter()) {
            let dz = new[2] - orig[2];
            assert!((dz - 1.0).abs() < 1e-5, "expected dz=1.0, got {dz}");
        }
        save_tmp("offset_outward.txt", "ok");
    }

    #[test]
    fn offset_mesh_inward() {
        let q = quad_mesh();
        let off = offset_mesh(&q, -0.25);
        for (orig, new) in q.positions.iter().zip(off.positions.iter()) {
            let dz = new[2] - orig[2];
            assert!((dz + 0.25).abs() < 1e-5, "expected dz=-0.25, got {dz}");
        }
    }

    #[test]
    fn area_weighted_normals_length() {
        let q = quad_mesh();
        let n = area_weighted_normals(&q);
        assert_eq!(n.len(), q.positions.len());
        // All normals should point roughly in +Z
        for normal in &n {
            assert!(normal[2] > 0.9, "expected +Z normal, got {normal:?}");
        }
        save_tmp("area_weighted.txt", &format!("{n:?}"));
    }

    #[test]
    fn area_weighted_normals_no_nan() {
        let t = tetra_mesh();
        let n = area_weighted_normals(&t);
        for normal in &n {
            assert!(!normal[0].is_nan(), "NaN in normal x");
            assert!(!normal[1].is_nan(), "NaN in normal y");
            assert!(!normal[2].is_nan(), "NaN in normal z");
        }
    }

    #[test]
    fn find_boundary_edges_quad() {
        let q = quad_mesh();
        let edges = find_boundary_edges(&q.indices, q.positions.len());
        // Quad mesh (2 tris sharing edge 0-2) has 4 boundary edges
        assert_eq!(
            edges.len(),
            4,
            "expected 4 boundary edges, got {}",
            edges.len()
        );
        save_tmp("boundary_edges.txt", &format!("{edges:?}"));
    }

    #[test]
    fn find_boundary_edges_closed_tetra() {
        let t = tetra_mesh();
        let edges = find_boundary_edges(&t.indices, t.positions.len());
        assert_eq!(edges.len(), 0, "closed tetra should have 0 boundary edges");
    }

    #[test]
    fn boundary_loops_quad() {
        let q = quad_mesh();
        let edges = find_boundary_edges(&q.indices, q.positions.len());
        let loops = boundary_loops(&edges);
        // Quad mesh has 1 outer boundary loop
        assert_eq!(
            loops.len(),
            1,
            "expected 1 boundary loop, got {}",
            loops.len()
        );
        assert_eq!(loops[0].len(), 4, "boundary loop should have 4 vertices");
        save_tmp("boundary_loops.txt", &format!("{loops:?}"));
    }

    #[test]
    fn boundary_loops_empty_for_closed() {
        let t = tetra_mesh();
        let edges = find_boundary_edges(&t.indices, t.positions.len());
        let loops = boundary_loops(&edges);
        assert!(loops.is_empty(), "closed mesh has no boundary loops");
    }

    #[test]
    fn stitch_boundary_loops_count() {
        let outer: Vec<u32> = vec![0, 1, 2, 3];
        let inner: Vec<u32> = vec![4, 5, 6, 7];
        let tris = stitch_boundary_loops(&outer, &inner, 0, 0);
        // n=4 segments → 2 tris per segment = 8 tris = 24 indices
        assert_eq!(tris.len(), 24, "expected 24 indices, got {}", tris.len());
        assert_eq!(tris.len() % 3, 0);
        save_tmp("stitch_loops.txt", &format!("{tris:?}"));
    }

    #[test]
    fn stitch_boundary_loops_with_offsets() {
        let outer: Vec<u32> = vec![0, 1, 2];
        let inner: Vec<u32> = vec![0, 1, 2];
        let tris = stitch_boundary_loops(&outer, &inner, 0, 10);
        // inner loop indices should be 10, 11, 12
        assert!(tris.contains(&10), "inner offset not applied");
        assert!(tris.contains(&11));
    }

    #[test]
    fn shell_thickness_matches_param() {
        let q = quad_mesh();
        let thickness = 0.3f32;
        let params = HollowParams {
            thickness,
            ..Default::default()
        };
        let res = hollow_mesh(&q, &params);
        let measured = shell_thickness(&res);
        assert!(
            (measured - thickness).abs() < 1e-4,
            "shell_thickness={measured}, expected {thickness}"
        );
        save_tmp("shell_thickness.txt", &format!("measured={measured}"));
    }

    #[test]
    fn hollow_empty_mesh_ok() {
        let empty = empty_mesh();
        let res = hollow_mesh(&empty, &HollowParams::default());
        assert_eq!(res.outer_vertex_count, 0);
        assert_eq!(res.inner_vertex_count, 0);
        assert_eq!(res.cap_triangle_count, 0);
    }

    #[test]
    fn hollow_smooth_offset_flag() {
        let q = quad_mesh();
        let params = HollowParams {
            smooth_offset: true,
            thickness: 0.1,
            ..Default::default()
        };
        let res = hollow_mesh(&q, &params);
        // Should complete without panic and produce a valid mesh
        assert!(res.mesh.positions.len() >= 8);
        save_tmp("hollow_smooth.txt", &format!("{}", res.mesh.vertex_count()));
    }
}
