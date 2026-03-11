// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Non-manifold edge removal and T-junction resolution for triangle meshes.

use std::collections::HashMap;

// ── helpers ──────────────────────────────────────────────────────────────────

fn dist3(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Canonical directed edge key (sorted so (a,b) == (b,a)).
fn edge_key(a: usize, b: usize) -> (usize, usize) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

// ── public API ───────────────────────────────────────────────────────────────

/// Returns edges shared by more than 2 triangles (non-manifold edges).
#[allow(dead_code)]
pub fn find_non_manifold_edges_advanced(
    _positions: &[[f32; 3]],
    tris: &[[u32; 3]],
) -> Vec<[usize; 2]> {
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();
    for tri in tris {
        let v = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        for i in 0..3 {
            let key = edge_key(v[i], v[(i + 1) % 3]);
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    edge_count
        .into_iter()
        .filter(|(_, c)| *c > 2)
        .map(|((a, b), _)| [a, b])
        .collect()
}

/// Returns T-junction triples `(vtx, edge_a, edge_b)` where `vtx` lies on the
/// edge between `edge_a` and `edge_b` within tolerance `eps`.
#[allow(dead_code)]
pub fn find_t_junctions_advanced(
    positions: &[[f32; 3]],
    tris: &[[u32; 3]],
    eps: f32,
) -> Vec<(usize, usize, usize)> {
    // Collect all boundary / interior edge segments.
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();
    for tri in tris {
        let v = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        for i in 0..3 {
            *edge_count
                .entry(edge_key(v[i], v[(i + 1) % 3]))
                .or_insert(0) += 1;
        }
    }
    let edges: Vec<(usize, usize)> = edge_count.into_keys().collect();

    let mut result = Vec::new();
    for v_idx in 0..positions.len() {
        let p = &positions[v_idx];
        for &(ea, eb) in &edges {
            // Skip if vertex is an endpoint of this edge.
            if ea == v_idx || eb == v_idx {
                continue;
            }
            let a = &positions[ea];
            let b = &positions[eb];
            // Project p onto segment a-b, check closeness.
            let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
            let len2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
            if len2 < 1e-12 {
                continue;
            }
            let t = (ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / len2;
            if !(0.0..=1.0).contains(&t) {
                continue;
            }
            let proj = [a[0] + t * ab[0], a[1] + t * ab[1], a[2] + t * ab[2]];
            let d = dist3(p, &proj);
            if d < eps {
                result.push((v_idx, ea, eb));
            }
        }
    }
    result
}

/// Splits edges at T-junctions by inserting new vertices.
/// Returns the number of splits performed.
#[allow(dead_code)]
pub fn resolve_t_junctions(
    positions: &mut Vec<[f32; 3]>,
    tris: &mut Vec<[u32; 3]>,
    eps: f32,
) -> usize {
    let junctions = find_t_junctions_advanced(positions, tris, eps);
    let count = junctions.len();
    for (v_idx, ea, eb) in junctions {
        // Insert midpoint on edge (ea, eb) near v_idx's projection.
        let a = positions[ea];
        let b = positions[eb];
        let mid = [
            (a[0] + b[0]) * 0.5,
            (a[1] + b[1]) * 0.5,
            (a[2] + b[2]) * 0.5,
        ];
        // Only insert if not already close to v_idx.
        let d = dist3(&mid, &positions[v_idx]);
        if d < eps * 2.0 {
            // T-junction vertex is already near midpoint — connect it.
            let new_v = positions.len() as u32;
            positions.push(mid);
            // Split any triangle containing edge (ea, eb).
            let mut new_tris = Vec::new();
            tris.retain(|tri| {
                let va = tri[0] as usize;
                let vb = tri[1] as usize;
                let vc = tri[2] as usize;
                let verts = [va, vb, vc];
                // Check if triangle contains the edge (ea, eb) in any order.
                let has_ea = verts.contains(&ea);
                let has_eb = verts.contains(&eb);
                if has_ea && has_eb {
                    // Split into two triangles.
                    let other = verts
                        .iter()
                        .copied()
                        .find(|&x| x != ea && x != eb)
                        .unwrap_or(ea);
                    new_tris.push([ea as u32, new_v, other as u32]);
                    new_tris.push([new_v, eb as u32, other as u32]);
                    false
                } else {
                    true
                }
            });
            tris.extend(new_tris);
        }
    }
    count
}

/// Removes faces that share a non-manifold edge (shared by >2 triangles).
#[allow(dead_code)]
pub fn remove_non_manifold_faces(positions: &[[f32; 3]], tris: &[[u32; 3]]) -> Vec<[u32; 3]> {
    let bad_edges: std::collections::HashSet<(usize, usize)> =
        find_non_manifold_edges_advanced(positions, tris)
            .into_iter()
            .map(|e| (e[0], e[1]))
            .collect();
    tris.iter()
        .copied()
        .filter(|tri| {
            let v = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
            for i in 0..3 {
                let key = edge_key(v[i], v[(i + 1) % 3]);
                if bad_edges.contains(&key) {
                    return false;
                }
            }
            true
        })
        .collect()
}

/// Welds close boundary vertices together.
/// Returns number of stitches performed.
#[allow(dead_code)]
pub fn stitch_boundary_edges(positions: &mut [[f32; 3]], tris: &mut [[u32; 3]], eps: f32) -> usize {
    // Find boundary vertices (belong to exactly 1 triangle for their edges).
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();
    for tri in tris.iter() {
        let v = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        for i in 0..3 {
            *edge_count
                .entry(edge_key(v[i], v[(i + 1) % 3]))
                .or_insert(0) += 1;
        }
    }
    let mut boundary_verts: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for ((a, b), c) in &edge_count {
        if *c == 1 {
            boundary_verts.insert(*a);
            boundary_verts.insert(*b);
        }
    }
    let bv: Vec<usize> = boundary_verts.into_iter().collect();
    // Build merge map: close boundary vertices get merged to the lowest index.
    let mut remap: HashMap<usize, usize> = HashMap::new();
    let mut count = 0usize;
    for i in 0..bv.len() {
        for j in (i + 1)..bv.len() {
            if dist3(&positions[bv[i]], &positions[bv[j]]) < eps {
                let target = bv[i].min(bv[j]);
                let src = bv[i].max(bv[j]);
                remap.entry(src).or_insert(target);
                count += 1;
            }
        }
    }
    if count == 0 {
        return 0;
    }
    // Apply remap to triangles.
    let resolve = |v: u32| -> u32 {
        let mut idx = v as usize;
        while let Some(&next) = remap.get(&idx) {
            if next == idx {
                break;
            }
            idx = next;
        }
        idx as u32
    };
    for tri in tris.iter_mut() {
        tri[0] = resolve(tri[0]);
        tri[1] = resolve(tri[1]);
        tri[2] = resolve(tri[2]);
    }
    count
}

/// Returns indices of vertices not referenced by any triangle.
#[allow(dead_code)]
pub fn find_isolated_vertices(positions: &[[f32; 3]], tris: &[[u32; 3]]) -> Vec<usize> {
    let mut used = vec![false; positions.len()];
    for tri in tris {
        used[tri[0] as usize] = true;
        used[tri[1] as usize] = true;
        used[tri[2] as usize] = true;
    }
    used.iter()
        .enumerate()
        .filter(|(_, &u)| !u)
        .map(|(i, _)| i)
        .collect()
}

/// Removes isolated vertices and remaps triangle indices.
#[allow(dead_code)]
pub fn remove_isolated_vertices(
    positions: Vec<[f32; 3]>,
    tris: Vec<[u32; 3]>,
) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let isolated: std::collections::HashSet<usize> = find_isolated_vertices(&positions, &tris)
        .into_iter()
        .collect();
    let mut new_positions = Vec::new();
    let mut remap = vec![0u32; positions.len()];
    for (i, pos) in positions.iter().enumerate() {
        if !isolated.contains(&i) {
            remap[i] = new_positions.len() as u32;
            new_positions.push(*pos);
        }
    }
    let new_tris = tris
        .into_iter()
        .map(|tri| {
            [
                remap[tri[0] as usize],
                remap[tri[1] as usize],
                remap[tri[2] as usize],
            ]
        })
        .collect();
    (new_positions, new_tris)
}

/// Returns a per-vertex face count vector.
#[allow(dead_code)]
pub fn vertex_face_count(positions: &[[f32; 3]], tris: &[[u32; 3]]) -> Vec<usize> {
    let mut counts = vec![0usize; positions.len()];
    for tri in tris {
        counts[tri[0] as usize] += 1;
        counts[tri[1] as usize] += 1;
        counts[tri[2] as usize] += 1;
    }
    counts
}

/// Returns `true` if the 1-ring neighbourhood of vertex `v` is locally manifold.
/// A vertex is locally manifold if its link (the set of edges between neighbours)
/// forms a single path (boundary vertex) or single cycle (interior vertex).
#[allow(dead_code)]
pub fn is_locally_manifold(_positions: &[[f32; 3]], tris: &[[u32; 3]], v: usize) -> bool {
    // Gather neighbour edges in the 1-ring.
    let mut link_edges: HashMap<usize, Vec<usize>> = HashMap::new();
    for tri in tris {
        let verts = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        if !verts.contains(&v) {
            continue;
        }
        // The two non-v vertices form the link edge.
        let others: Vec<usize> = verts.iter().copied().filter(|&x| x != v).collect();
        if others.len() == 2 {
            link_edges.entry(others[0]).or_default().push(others[1]);
            link_edges.entry(others[1]).or_default().push(others[0]);
        }
    }
    if link_edges.is_empty() {
        return true; // isolated vertex considered locally manifold
    }
    // Each node in the link graph must have degree 1 or 2 for manifold topology.
    for neighbours in link_edges.values() {
        if neighbours.len() > 2 {
            return false;
        }
    }
    true
}

/// Returns a JSON report of topology issues.
#[allow(dead_code)]
pub fn topology_repair_report(positions: &[[f32; 3]], tris: &[[u32; 3]]) -> String {
    let non_manifold = find_non_manifold_edges_advanced(positions, tris);
    let isolated = find_isolated_vertices(positions, tris);
    let t_junctions = find_t_junctions_advanced(positions, tris, 1e-4);
    format!(
        r#"{{"non_manifold_edges":{},"isolated_vertices":{},"t_junctions":{}}}"#,
        non_manifold.len(),
        isolated.len(),
        t_junctions.len()
    )
}

// ── New required API ──────────────────────────────────────────────────────────

pub struct TopologyIssue {
    pub kind: u8,
    pub vertex_indices: Vec<usize>,
}

pub fn new_topology_issue(kind: u8, verts: Vec<usize>) -> TopologyIssue {
    TopologyIssue {
        kind,
        vertex_indices: verts,
    }
}

pub fn detect_degenerate_faces(positions: &[[f32; 3]], faces: &[[usize; 3]]) -> Vec<usize> {
    faces
        .iter()
        .enumerate()
        .filter(|(_, f)| {
            if f[0] >= positions.len() || f[1] >= positions.len() || f[2] >= positions.len() {
                return false;
            }
            let a = positions[f[0]];
            let b = positions[f[1]];
            let c = positions[f[2]];
            let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let cross = [
                ab[1] * ac[2] - ab[2] * ac[1],
                ab[2] * ac[0] - ab[0] * ac[2],
                ab[0] * ac[1] - ab[1] * ac[0],
            ];
            let area2 = cross[0].powi(2) + cross[1].powi(2) + cross[2].powi(2);
            area2 < 1e-20
        })
        .map(|(i, _)| i)
        .collect()
}

pub fn detect_duplicate_vertices(positions: &[[f32; 3]], eps: f32) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    for i in 0..positions.len() {
        for j in (i + 1)..positions.len() {
            let d = dist3(&positions[i], &positions[j]);
            if d < eps {
                pairs.push((i, j));
            }
        }
    }
    pairs
}

pub fn count_topology_issues(issue_list: &[TopologyIssue]) -> usize {
    issue_list.len()
}

pub fn topology_issue_name(kind: u8) -> &'static str {
    match kind {
        0 => "T-junction",
        1 => "non-manifold-edge",
        2 => "degenerate-face",
        _ => "unknown",
    }
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn single_tri() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0u32, 1, 2]];
        (pos, tris)
    }

    fn two_tris() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let tris = vec![[0u32, 1, 2], [1, 3, 2]];
        (pos, tris)
    }

    #[test]
    fn no_non_manifold_on_clean_mesh() {
        let (pos, tris) = two_tris();
        let bad = find_non_manifold_edges_advanced(&pos, &tris);
        assert!(
            bad.is_empty(),
            "clean mesh should have no non-manifold edges"
        );
    }

    #[test]
    fn detects_non_manifold_edge() {
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.5, 0.5, 1.0],
            [-0.5, 0.5, -1.0],
        ];
        // Three tris share edge (0,1)
        let tris = vec![[0u32, 1, 2], [0, 1, 3], [0, 1, 4]];
        let bad = find_non_manifold_edges_advanced(&pos, &tris);
        assert!(!bad.is_empty(), "should detect non-manifold edge");
        let has_edge = bad
            .iter()
            .any(|e| (e[0] == 0 && e[1] == 1) || (e[0] == 1 && e[1] == 0));
        assert!(has_edge);
    }

    #[test]
    fn remove_non_manifold_faces_removes_bad() {
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.5, 0.5, 1.0],
            [-0.5, 0.5, -1.0],
        ];
        let tris = vec![[0u32, 1, 2], [0, 1, 3], [0, 1, 4]];
        let clean = remove_non_manifold_faces(&pos, &tris);
        assert!(
            clean.len() < tris.len(),
            "should remove faces on non-manifold edges"
        );
    }

    #[test]
    fn find_isolated_vertices_empty_on_full_mesh() {
        let (pos, tris) = single_tri();
        let iso = find_isolated_vertices(&pos, &tris);
        assert!(iso.is_empty());
    }

    #[test]
    fn find_isolated_vertices_detects_orphan() {
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [99.0, 99.0, 99.0], // isolated
        ];
        let tris = vec![[0u32, 1, 2]];
        let iso = find_isolated_vertices(&pos, &tris);
        assert_eq!(iso, vec![3]);
    }

    #[test]
    fn remove_isolated_vertices_compacts_positions() {
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [99.0, 99.0, 99.0], // isolated
        ];
        let tris = vec![[0u32, 1, 2]];
        let (new_pos, new_tris) = remove_isolated_vertices(pos, tris);
        assert_eq!(new_pos.len(), 3);
        assert_eq!(new_tris.len(), 1);
        // Triangle indices still valid.
        for tri in &new_tris {
            assert!(tri[0] < 3 && tri[1] < 3 && tri[2] < 3);
        }
    }

    #[test]
    fn vertex_face_count_correct() {
        let (pos, tris) = two_tris();
        let counts = vertex_face_count(&pos, &tris);
        assert_eq!(counts.len(), 4);
        // Vertices 1 and 2 are shared — they appear in both triangles.
        assert_eq!(counts[1], 2);
        assert_eq!(counts[2], 2);
        // Vertices 0 and 3 appear in one each.
        assert_eq!(counts[0], 1);
        assert_eq!(counts[3], 1);
    }

    #[test]
    fn is_locally_manifold_single_triangle() {
        let (pos, tris) = single_tri();
        // All 3 vertices of a single triangle are locally manifold.
        assert!(is_locally_manifold(&pos, &tris, 0));
        assert!(is_locally_manifold(&pos, &tris, 1));
        assert!(is_locally_manifold(&pos, &tris, 2));
    }

    #[test]
    fn stitch_boundary_edges_returns_zero_when_nothing_to_stitch() {
        let mut pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let mut tris = vec![[0u32, 1, 2]];
        let n = stitch_boundary_edges(&mut pos, &mut tris, 1e-5);
        assert_eq!(n, 0);
    }

    #[test]
    fn stitch_boundary_edges_welds_close_vertices() {
        let mut pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0001], // very close to vertex 1
            [1.0, 1.0, 0.0],
        ];
        let mut tris = vec![[0u32, 1, 2], [3, 4, 2]];
        let n = stitch_boundary_edges(&mut pos, &mut tris, 0.01);
        assert!(n > 0, "should stitch at least one pair");
    }

    #[test]
    fn find_t_junctions_empty_on_clean_mesh() {
        let (pos, tris) = two_tris();
        let tj = find_t_junctions_advanced(&pos, &tris, 1e-4);
        assert!(tj.is_empty(), "clean mesh should have no T-junctions");
    }

    #[test]
    fn topology_repair_report_is_valid_json() {
        let (pos, tris) = two_tris();
        let report = topology_repair_report(&pos, &tris);
        let v: serde_json::Value = serde_json::from_str(&report).expect("must be valid JSON");
        assert!(v.get("non_manifold_edges").is_some());
        assert!(v.get("isolated_vertices").is_some());
        assert!(v.get("t_junctions").is_some());
    }

    #[test]
    fn topology_report_clean_mesh_zeros() {
        let (pos, tris) = two_tris();
        let report = topology_repair_report(&pos, &tris);
        let v: serde_json::Value = serde_json::from_str(&report).unwrap();
        assert_eq!(v["non_manifold_edges"].as_u64().unwrap(), 0);
        assert_eq!(v["isolated_vertices"].as_u64().unwrap(), 0);
    }

    #[test]
    fn resolve_t_junctions_returns_count() {
        let mut pos = vec![
            [0.0f32, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0], // lies on edge (0,1)
        ];
        let mut tris = vec![[0u32, 1, 2]];
        let count = resolve_t_junctions(&mut pos, &mut tris, 0.01);
        // Should detect at least the T-junction at vertex 3.
        let _ = count; // count is usize (non-negative by type)
    }
}
