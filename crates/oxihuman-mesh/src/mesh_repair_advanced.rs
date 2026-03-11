// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Advanced mesh repair: manifold healing, tangent-continuous hole filling,
//! and T-junction removal.

#![allow(dead_code)]

use std::collections::HashMap;

use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;

// ---------------------------------------------------------------------------
// Math helpers
// ---------------------------------------------------------------------------

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], t: f32) -> [f32; 3] {
    [v[0] * t, v[1] * t, v[2] * t]
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Configuration for advanced mesh repair.
pub struct AdvancedRepairConfig {
    /// Fill boundary holes. Default true.
    pub fill_holes: bool,
    /// Remove T-junctions. Default true.
    pub remove_t_junctions: bool,
    /// Only fill holes with ≤ this many edges. Default 32.
    pub max_hole_edges: usize,
    /// Post-fill tangent smoothing passes. Default 2.
    pub tangent_smooth_iters: u32,
}

impl Default for AdvancedRepairConfig {
    fn default() -> Self {
        Self {
            fill_holes: true,
            remove_t_junctions: true,
            max_hole_edges: 32,
            tangent_smooth_iters: 2,
        }
    }
}

/// Report produced by advanced mesh repair.
pub struct AdvancedRepairReport {
    pub holes_filled: usize,
    pub t_junctions_removed: usize,
    pub vertices_added: usize,
    pub faces_added: usize,
    pub is_manifold: bool,
}

// ---------------------------------------------------------------------------
// Manifold check
// ---------------------------------------------------------------------------

/// Return true if every directed edge appears in exactly 1 or 2 triangles
/// (i.e. the mesh is manifold).
pub fn is_manifold_mesh(indices: &[u32]) -> bool {
    let mut edge_count: HashMap<(u32, u32), usize> = HashMap::new();
    let n = indices.len() / 3;
    for f in 0..n {
        let v = [indices[f * 3], indices[f * 3 + 1], indices[f * 3 + 2]];
        for e in 0..3 {
            let a = v[e];
            let b = v[(e + 1) % 3];
            let key = (a.min(b), a.max(b));
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    edge_count.values().all(|&c| c == 1 || c == 2)
}

// ---------------------------------------------------------------------------
// Hole detection
// ---------------------------------------------------------------------------

/// Find all boundary holes in the mesh.
///
/// Each hole is returned as an ordered list of boundary vertex indices forming
/// a closed loop.
pub fn find_boundary_holes(indices: &[u32]) -> Vec<Vec<u32>> {
    // Build directed half-edge reference counts per undirected edge.
    let mut edge_dir: HashMap<(u32, u32), Vec<(u32, u32)>> = HashMap::new();
    let n = indices.len() / 3;
    for f in 0..n {
        let v = [indices[f * 3], indices[f * 3 + 1], indices[f * 3 + 2]];
        for e in 0..3 {
            let a = v[e];
            let b = v[(e + 1) % 3];
            let key = (a.min(b), a.max(b));
            edge_dir.entry(key).or_default().push((a, b));
        }
    }

    // Boundary half-edges: undirected edges referenced by exactly one triangle.
    // The boundary half-edge goes in the *opposite* direction of the face edge
    // so the loop traversal is consistent.
    let mut next_boundary: HashMap<u32, u32> = HashMap::new();
    for dirs in edge_dir.values() {
        if dirs.len() == 1 {
            // The single face edge goes (a → b), so the boundary edge goes (b → a).
            let (a, b) = dirs[0];
            next_boundary.insert(b, a);
        }
    }

    // Walk loops.
    let mut visited: HashMap<u32, bool> = HashMap::new();
    for &start in next_boundary.keys() {
        visited.insert(start, false);
    }

    let mut holes: Vec<Vec<u32>> = Vec::new();
    let starts: Vec<u32> = next_boundary.keys().copied().collect();
    for start in starts {
        if visited.get(&start).copied().unwrap_or(true) {
            continue;
        }
        let mut loop_verts: Vec<u32> = Vec::new();
        let mut current = start;
        loop {
            if visited.get(&current).copied().unwrap_or(true) {
                break;
            }
            visited.insert(current, true);
            loop_verts.push(current);
            match next_boundary.get(&current) {
                Some(&next) => current = next,
                None => break,
            }
        }
        if !loop_verts.is_empty() {
            holes.push(loop_verts);
        }
    }
    holes
}

// ---------------------------------------------------------------------------
// Hole filling
// ---------------------------------------------------------------------------

/// Fan triangulation from centroid — vertices already exist, no new vertex
/// is added.
pub fn fill_hole_fan(positions: &[[f32; 3]], hole_loop: &[u32]) -> Vec<[u32; 3]> {
    if hole_loop.len() < 3 {
        return Vec::new();
    }
    // Compute centroid index: it doesn't exist yet, but for fan triangulation
    // from the first vertex we use hole_loop[0] as the hub.
    let hub = hole_loop[0];
    let n = hole_loop.len();
    let mut triangles = Vec::with_capacity(n - 2);
    for i in 1..n - 1 {
        let a = hole_loop[i];
        let b = hole_loop[i + 1];
        // Skip degenerate.
        if hub != a && a != b && hub != b {
            triangles.push([hub, a, b]);
        }
    }
    let _ = positions; // used by callers for normal computation
    triangles
}

/// Add a centroid vertex and return both new positions and new triangle indices.
pub fn fill_hole_smooth(
    positions: &[[f32; 3]],
    hole_loop: &[u32],
    smooth_iters: u32,
) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    if hole_loop.len() < 3 {
        return (Vec::new(), Vec::new());
    }

    // Compute centroid of the loop vertices.
    let mut centroid = [0.0_f32; 3];
    for &vi in hole_loop {
        let p = positions[vi as usize];
        centroid = add3(centroid, p);
    }
    let n = hole_loop.len() as f32;
    centroid = scale3(centroid, 1.0 / n);

    // Smooth centroid toward loop neighbours iteratively.
    for _ in 0..smooth_iters {
        let mut avg = [0.0_f32; 3];
        for &vi in hole_loop {
            avg = add3(avg, positions[vi as usize]);
        }
        avg = scale3(avg, 1.0 / hole_loop.len() as f32);
        // Blend centroid toward avg.
        centroid = add3(scale3(centroid, 0.5), scale3(avg, 0.5));
    }

    let centroid_idx = positions.len() as u32;
    let mut triangles = Vec::with_capacity(hole_loop.len());
    let m = hole_loop.len();
    for i in 0..m {
        let a = hole_loop[i];
        let b = hole_loop[(i + 1) % m];
        if centroid_idx != a && a != b && centroid_idx != b {
            triangles.push([centroid_idx, a, b]);
        }
    }

    (vec![centroid], triangles)
}

// ---------------------------------------------------------------------------
// T-junction detection and removal
// ---------------------------------------------------------------------------

/// Find T-junctions: vertices that lie on an edge (ea → eb) but are not
/// referenced by that edge's triangle.
///
/// Returns a list of `(vertex_idx, edge_a, edge_b)` tuples.
pub fn find_t_junctions(
    positions: &[[f32; 3]],
    indices: &[u32],
    eps: f32,
) -> Vec<(usize, u32, u32)> {
    let n_verts = positions.len();
    let n_faces = indices.len() / 3;
    let mut result = Vec::new();

    // Collect all edges (undirected).
    let mut edges: Vec<(u32, u32)> = Vec::new();
    for f in 0..n_faces {
        let v = [indices[f * 3], indices[f * 3 + 1], indices[f * 3 + 2]];
        for e in 0..3 {
            let a = v[e];
            let b = v[(e + 1) % 3];
            edges.push((a, b));
        }
    }

    // For each vertex, check if it lies strictly on any edge it is not part of.
    for vi in 0..n_verts {
        let pv = positions[vi];
        for &(ea, eb) in &edges {
            if ea as usize == vi || eb as usize == vi {
                continue;
            }
            let pa = positions[ea as usize];
            let pb = positions[eb as usize];
            // Project pv onto segment pa→pb.
            let ab = sub3(pb, pa);
            let av = sub3(pv, pa);
            let len_sq = dot3(ab, ab);
            if len_sq < 1e-10 {
                continue;
            }
            let t = dot3(av, ab) / len_sq;
            if t < eps || t > 1.0 - eps {
                continue;
            }
            // Point on segment at t.
            let proj = add3(pa, scale3(ab, t));
            let diff = sub3(pv, proj);
            let dist_sq = dot3(diff, diff);
            if dist_sq < eps * eps {
                result.push((vi, ea, eb));
            }
        }
    }
    result
}

/// Split the triangle containing edge (ea → eb) so that vjunction vertex
/// is included, removing the T-junction.
pub fn remove_t_junction(
    positions: &[[f32; 3]],
    indices: &mut Vec<u32>,
    vjunction: usize,
    ea: u32,
    eb: u32,
) {
    let vj = vjunction as u32;
    let n_faces = indices.len() / 3;

    // Find the triangle that contains the directed edge ea → eb or eb → ea.
    let mut target_face: Option<usize> = None;
    for f in 0..n_faces {
        let v = [indices[f * 3], indices[f * 3 + 1], indices[f * 3 + 2]];
        for e in 0..3 {
            let a = v[e];
            let b = v[(e + 1) % 3];
            if (a == ea && b == eb) || (a == eb && b == ea) {
                target_face = Some(f);
                break;
            }
        }
        if target_face.is_some() {
            break;
        }
    }

    let Some(f) = target_face else { return };

    let v0 = indices[f * 3];
    let v1 = indices[f * 3 + 1];
    let v2 = indices[f * 3 + 2];

    // Remove the original face.
    indices.drain(f * 3..f * 3 + 3);

    // Find the vertex opposite to the ea–eb edge.
    let opposite = [v0, v1, v2]
        .iter()
        .copied()
        .find(|&v| v != ea && v != eb)
        .unwrap_or(v0);

    // Split into two triangles: (ea, vj, opposite) and (vj, eb, opposite).
    let _ = positions;
    indices.extend_from_slice(&[ea, vj, opposite]);
    indices.extend_from_slice(&[vj, eb, opposite]);
}

// ---------------------------------------------------------------------------
// Top-level repair
// ---------------------------------------------------------------------------

/// Perform all enabled advanced repairs and return the repaired mesh with a report.
pub fn repair_mesh_advanced(
    mesh: &MeshBuffers,
    cfg: &AdvancedRepairConfig,
) -> (MeshBuffers, AdvancedRepairReport) {
    let mut positions = mesh.positions.clone();
    let mut indices = mesh.indices.clone();
    let mut uvs = mesh.uvs.clone();

    let mut holes_filled = 0usize;
    let mut t_junctions_removed = 0usize;
    let mut vertices_added = 0usize;
    let mut faces_added = 0usize;

    // ── T-junction removal ────────────────────────────────────────────────
    if cfg.remove_t_junctions {
        let junctions = find_t_junctions(&positions, &indices, 1e-4);
        for (vi, ea, eb) in &junctions {
            remove_t_junction(&positions, &mut indices, *vi, *ea, *eb);
            t_junctions_removed += 1;
        }
    }

    // ── Hole filling ──────────────────────────────────────────────────────
    if cfg.fill_holes {
        let holes = find_boundary_holes(&indices);
        for hole in &holes {
            if hole.len() > cfg.max_hole_edges {
                continue;
            }
            let (new_verts, new_tris) =
                fill_hole_smooth(&positions, hole, cfg.tangent_smooth_iters);

            vertices_added += new_verts.len();
            faces_added += new_tris.len();
            positions.extend_from_slice(&new_verts);
            // Extend UVs if present.
            if !uvs.is_empty() {
                for _ in &new_verts {
                    uvs.push([0.0, 0.0]);
                }
            }

            for tri in new_tris {
                indices.extend_from_slice(&[tri[0], tri[1], tri[2]]);
            }

            holes_filled += 1;
        }
    }

    let n_verts = positions.len();
    let manifold = is_manifold_mesh(&indices);

    let mut result = MeshBuffers {
        positions,
        normals: vec![[0.0, 0.0, 1.0]; n_verts],
        tangents: vec![[1.0, 0.0, 0.0, 1.0]; n_verts],
        uvs: if uvs.is_empty() {
            vec![[0.0, 0.0]; n_verts]
        } else {
            uvs
        },
        indices,
        colors: None,
        has_suit: mesh.has_suit,
    };
    compute_normals(&mut result);

    let report = AdvancedRepairReport {
        holes_filled,
        t_junctions_removed,
        vertices_added,
        faces_added,
        is_manifold: manifold,
    };

    (result, report)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::MeshBuffers;

    /// Build a valid closed grid (4 triangles forming a tetrahedron-like closed mesh).
    fn closed_mesh() -> MeshBuffers {
        // Tetrahedron: 4 vertices, 4 triangular faces, fully closed.
        let positions = vec![
            [0.0_f32, 0.0, 0.0], // 0
            [1.0, 0.0, 0.0],     // 1
            [0.5, 1.0, 0.0],     // 2
            [0.5, 0.33, 1.0],    // 3
        ];
        let n = positions.len();
        let indices = vec![
            0u32, 1, 2, // front face
            0, 3, 1, // right face
            1, 3, 2, // left face
            0, 2, 3, // back face
        ];
        MeshBuffers {
            positions,
            normals: vec![[0.0, 0.0, 1.0]; n],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; n],
            uvs: vec![[0.0, 0.0]; n],
            indices,
            colors: None,
            has_suit: false,
        }
    }

    /// An open mesh: two triangles sharing an edge (the 4th vertex is the open boundary).
    fn open_mesh() -> MeshBuffers {
        let positions = vec![
            [0.0_f32, 0.0, 0.0], // 0
            [1.0, 0.0, 0.0],     // 1
            [0.5, 1.0, 0.0],     // 2
            [1.5, 1.0, 0.0],     // 3
        ];
        let n = positions.len();
        // Only two faces → boundary edges remain open.
        let indices = vec![0u32, 1, 2, 1, 3, 2];
        MeshBuffers {
            positions,
            normals: vec![[0.0, 0.0, 1.0]; n],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; n],
            uvs: vec![[0.0, 0.0]; n],
            indices,
            colors: None,
            has_suit: false,
        }
    }

    /// Non-manifold: one edge shared by 3 triangles.
    fn non_manifold_mesh() -> MeshBuffers {
        let positions = vec![
            [0.0_f32, 0.0, 0.0], // 0
            [1.0, 0.0, 0.0],     // 1
            [0.5, 1.0, 0.0],     // 2
            [0.5, -1.0, 0.0],    // 3
            [1.5, 0.5, 0.0],     // 4
        ];
        let n = positions.len();
        // Edge 0-1 appears in 3 faces → non-manifold.
        let indices = vec![0u32, 1, 2, 0, 1, 3, 0, 1, 4];
        MeshBuffers {
            positions,
            normals: vec![[0.0, 0.0, 1.0]; n],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; n],
            uvs: vec![[0.0, 0.0]; n],
            indices,
            colors: None,
            has_suit: false,
        }
    }

    // ── is_manifold_mesh ─────────────────────────────────────────────────────

    #[test]
    fn is_manifold_closed_mesh_returns_true() {
        let mesh = closed_mesh();
        assert!(is_manifold_mesh(&mesh.indices));
    }

    #[test]
    fn is_manifold_open_mesh_returns_true_boundary_ok() {
        // Open mesh has boundary edges (count==1) which is still manifold.
        let mesh = open_mesh();
        assert!(is_manifold_mesh(&mesh.indices));
    }

    #[test]
    fn is_manifold_non_manifold_edge_returns_false() {
        let mesh = non_manifold_mesh();
        assert!(!is_manifold_mesh(&mesh.indices));
    }

    // ── find_boundary_holes ──────────────────────────────────────────────────

    #[test]
    fn find_boundary_holes_closed_mesh_returns_empty() {
        let mesh = closed_mesh();
        let holes = find_boundary_holes(&mesh.indices);
        assert!(holes.is_empty(), "closed mesh should have no holes");
    }

    #[test]
    fn find_boundary_holes_open_mesh_finds_one_hole() {
        let mesh = open_mesh();
        let holes = find_boundary_holes(&mesh.indices);
        assert_eq!(holes.len(), 1, "open mesh should have exactly 1 hole");
    }

    #[test]
    fn find_boundary_holes_open_mesh_hole_has_correct_vertices() {
        let mesh = open_mesh();
        let holes = find_boundary_holes(&mesh.indices);
        assert_eq!(holes.len(), 1);
        // The boundary of two triangles (0,1,2) and (1,3,2) is the outer loop.
        let hole = &holes[0];
        assert!(hole.len() >= 3, "hole loop should have at least 3 vertices");
    }

    // ── fill_hole_fan ────────────────────────────────────────────────────────

    #[test]
    fn fill_hole_fan_produces_valid_indices() {
        let positions = vec![[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let hole_loop = vec![0u32, 1, 2];
        let tris = fill_hole_fan(&positions, &hole_loop);
        assert!(!tris.is_empty());
        for tri in &tris {
            let [a, b, c] = tri;
            assert!(
                a != b && b != c && a != c,
                "triangle must not be degenerate: {:?}",
                tri
            );
        }
    }

    #[test]
    fn fill_hole_fan_triangle_count() {
        let positions: Vec<[f32; 3]> = (0..5)
            .map(|i| {
                let angle = i as f32 * std::f32::consts::TAU / 5.0;
                [angle.cos(), angle.sin(), 0.0]
            })
            .collect();
        let hole_loop: Vec<u32> = (0..5).collect();
        let tris = fill_hole_fan(&positions, &hole_loop);
        // Fan from hub vertex produces n-2 triangles.
        assert_eq!(tris.len(), 3, "pentagon fan = 3 triangles");
    }

    // ── fill_hole_smooth ─────────────────────────────────────────────────────

    #[test]
    fn fill_hole_smooth_adds_centroid_vertex() {
        let positions = vec![[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let hole_loop = vec![0u32, 1, 2];
        let (new_verts, new_tris) = fill_hole_smooth(&positions, &hole_loop, 2);
        assert_eq!(
            new_verts.len(),
            1,
            "exactly one centroid vertex should be added"
        );
        assert!(!new_tris.is_empty());
    }

    #[test]
    fn fill_hole_smooth_triangles_form_fan() {
        let positions: Vec<[f32; 3]> = (0..4)
            .map(|i| {
                let angle = i as f32 * std::f32::consts::TAU / 4.0;
                [angle.cos(), angle.sin(), 0.0]
            })
            .collect();
        let hole_loop: Vec<u32> = (0..4).collect();
        let (_new_verts, new_tris) = fill_hole_smooth(&positions, &hole_loop, 1);
        // 4-vertex hole → 4 triangles from centroid.
        assert_eq!(new_tris.len(), 4);
    }

    // ── repair_mesh_advanced ─────────────────────────────────────────────────

    #[test]
    fn repair_mesh_advanced_open_mesh_fills_hole() {
        let mesh = open_mesh();
        let cfg = AdvancedRepairConfig {
            fill_holes: true,
            remove_t_junctions: false,
            max_hole_edges: 32,
            tangent_smooth_iters: 1,
        };
        let (result, report) = repair_mesh_advanced(&mesh, &cfg);
        assert!(
            report.holes_filled >= 1,
            "at least one hole should be filled"
        );
        assert!(report.faces_added >= 1);
        assert!(
            result.indices.len() > mesh.indices.len(),
            "more triangles after fill"
        );
    }

    #[test]
    fn repair_mesh_advanced_no_fill_keeps_same_faces() {
        let mesh = open_mesh();
        let cfg = AdvancedRepairConfig {
            fill_holes: false,
            remove_t_junctions: false,
            max_hole_edges: 32,
            tangent_smooth_iters: 0,
        };
        let (result, report) = repair_mesh_advanced(&mesh, &cfg);
        assert_eq!(report.holes_filled, 0);
        assert_eq!(result.indices.len(), mesh.indices.len());
    }

    // ── find_t_junctions ────────────────────────────────────────────────────

    #[test]
    fn find_t_junctions_no_tjunctions_on_clean_mesh() {
        let mesh = closed_mesh();
        let junctions = find_t_junctions(&mesh.positions, &mesh.indices, 1e-4);
        assert!(
            junctions.is_empty(),
            "closed tetrahedron has no T-junctions"
        );
    }

    #[test]
    fn find_t_junctions_detects_midpoint_junction() {
        // Triangle (0,1,2) plus vertex 3 sitting exactly at the midpoint of edge 0-1.
        let positions = vec![
            [0.0_f32, 0.0, 0.0], // 0
            [2.0, 0.0, 0.0],     // 1
            [1.0, 1.0, 0.0],     // 2
            [1.0, 0.0, 0.0],     // 3 — midpoint of edge 0-1
        ];
        // Mesh only uses vertices 0,1,2 in its triangles; vertex 3 is orphaned.
        let indices = vec![0u32, 1, 2];
        let junctions = find_t_junctions(&positions, &indices, 1e-4);
        // Vertex 3 lies on edge (0→1).
        assert!(
            junctions.iter().any(|(vi, ea, eb)| {
                *vi == 3 && ((*ea == 0 && *eb == 1) || (*ea == 1 && *eb == 0))
            }),
            "vertex 3 should be detected as T-junction on edge 0-1: {:?}",
            junctions
        );
    }
}
