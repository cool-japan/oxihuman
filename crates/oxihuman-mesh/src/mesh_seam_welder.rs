// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! UV seam welding — finds boundary edges that share the same 3-D position
//! but differ in UV, then merges them for correct normal/lighting continuity.

#[allow(dead_code)]
pub struct SeamEdge {
    pub vid_a: u32,
    pub vid_b: u32,
    pub uv_a: [f32; 2],
    pub uv_b: [f32; 2],
    pub twin_vid_a: u32,
    pub twin_vid_b: u32,
}

#[allow(dead_code)]
pub struct WeldSeamResult {
    pub welded_pairs: Vec<(u32, u32)>,
    pub new_positions: Vec<[f32; 3]>,
    pub new_normals: Vec<[f32; 3]>,
    pub new_uvs: Vec<[f32; 2]>,
    pub new_triangles: Vec<[u32; 3]>,
    pub seams_welded: usize,
}

/// Alias for the verbose return type of [`merge_vertex_groups`].
pub type MergeGroupResult = (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>);

// ── helpers ──────────────────────────────────────────────────────────────────

#[inline]
fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[inline]
fn norm3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

// ── union-find helpers (use slices, not Vec) ─────────────────────────────────

fn uf_find(parent: &mut [usize], mut x: usize) -> usize {
    while parent[x] != x {
        parent[x] = parent[parent[x]];
        x = parent[x];
    }
    x
}

fn uf_union(parent: &mut [usize], a: usize, b: usize) {
    let ra = uf_find(parent, a);
    let rb = uf_find(parent, b);
    if ra != rb {
        parent[rb] = ra;
    }
}

// ── public API ────────────────────────────────────────────────────────────────

/// Find edges that share the same 3-D positions but have different UVs
/// (i.e. UV seam boundaries).
#[allow(dead_code)]
pub fn find_seam_edges(
    positions: &[[f32; 3]],
    uvs: &[[f32; 2]],
    triangles: &[[u32; 3]],
    pos_threshold: f32,
) -> Vec<SeamEdge> {
    if positions.is_empty() || uvs.is_empty() || triangles.is_empty() {
        return vec![];
    }

    // Collect all half-edges: (vid_a, vid_b) → list of triangles that own it
    use std::collections::HashMap;
    let mut edge_map: HashMap<(u32, u32), Vec<(u32, u32)>> = HashMap::new();

    for tri in triangles {
        for k in 0..3usize {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_map.entry(key).or_default().push((a, b));
        }
    }

    let mut seams = Vec::new();

    // Boundary edges appear only once. Among two boundary edges whose 3-D
    // endpoints coincide but differ in UV → seam edge pair.
    let boundary: Vec<(u32, u32)> = edge_map
        .values()
        .filter(|v| v.len() == 1)
        .map(|v| v[0])
        .collect();

    for (i, &(a0, b0)) in boundary.iter().enumerate() {
        let pa0 = positions.get(a0 as usize).copied().unwrap_or([0.0; 3]);
        let pb0 = positions.get(b0 as usize).copied().unwrap_or([0.0; 3]);

        for &(a1, b1) in boundary.iter().skip(i + 1) {
            let pa1 = positions.get(a1 as usize).copied().unwrap_or([0.0; 3]);
            let pb1 = positions.get(b1 as usize).copied().unwrap_or([0.0; 3]);

            let forward = dist3(pa0, pa1) < pos_threshold && dist3(pb0, pb1) < pos_threshold;
            let cross = dist3(pa0, pb1) < pos_threshold && dist3(pb0, pa1) < pos_threshold;

            if forward || cross {
                let uv_a = uvs.get(a0 as usize).copied().unwrap_or([0.0; 2]);
                let uv_b = uvs.get(b0 as usize).copied().unwrap_or([0.0; 2]);
                let twin_a = if forward { a1 } else { b1 };
                let twin_b = if forward { b1 } else { a1 };
                seams.push(SeamEdge {
                    vid_a: a0,
                    vid_b: b0,
                    uv_a,
                    uv_b,
                    twin_vid_a: twin_a,
                    twin_vid_b: twin_b,
                });
            }
        }
    }

    seams
}

/// Weld UV seams: merge vertices that share the same 3-D position, average
/// their normals, and keep one UV (from the representative vertex).
#[allow(dead_code)]
pub fn weld_seams(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    triangles: &[[u32; 3]],
    threshold: f32,
) -> WeldSeamResult {
    let n = positions.len();
    if n == 0 {
        return WeldSeamResult {
            welded_pairs: vec![],
            new_positions: vec![],
            new_normals: vec![],
            new_uvs: vec![],
            new_triangles: vec![],
            seams_welded: 0,
        };
    }

    let mut parent: Vec<usize> = (0..n).collect();

    // Find duplicate positions
    let dups = find_duplicate_positions(positions, threshold);
    let mut welded_pairs = Vec::new();
    for (a, b) in &dups {
        uf_union(&mut parent, *a, *b);
        welded_pairs.push((*a as u32, *b as u32));
    }

    // Collect group members per representative
    let mut groups: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    for i in 0..n {
        let r = uf_find(&mut parent, i);
        groups.entry(r).or_default().push(i);
    }

    let mut new_positions: Vec<[f32; 3]> = Vec::new();
    let mut new_normals: Vec<[f32; 3]> = Vec::new();
    let mut new_uvs: Vec<[f32; 2]> = Vec::new();
    let mut old_to_new: Vec<usize> = vec![0; n];
    let mut rep_to_new: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    for (rep, members) in &groups {
        let new_id = new_positions.len();
        rep_to_new.insert(*rep, new_id);
        let inv = 1.0 / members.len() as f32;
        let mut s = [0f32; 3];
        for &m in members {
            let p = positions[m];
            s[0] += p[0];
            s[1] += p[1];
            s[2] += p[2];
        }
        let avg_pos = [s[0] * inv, s[1] * inv, s[2] * inv];
        let avg_norm = compute_average_normal(normals, members);
        let uv = uvs.get(*rep).copied().unwrap_or([0.0; 2]);
        new_positions.push(avg_pos);
        new_normals.push(avg_norm);
        new_uvs.push(uv);
    }

    for (i, entry) in old_to_new.iter_mut().enumerate() {
        let r = uf_find(&mut parent, i);
        *entry = rep_to_new[&r];
    }

    let new_triangles: Vec<[u32; 3]> = triangles
        .iter()
        .map(|tri| {
            [
                old_to_new[tri[0] as usize] as u32,
                old_to_new[tri[1] as usize] as u32,
                old_to_new[tri[2] as usize] as u32,
            ]
        })
        .collect();

    WeldSeamResult {
        seams_welded: welded_pairs.len(),
        welded_pairs,
        new_positions,
        new_normals,
        new_uvs,
        new_triangles,
    }
}

/// Count vertices that lie on the boundary (appear in only one triangle edge).
#[allow(dead_code)]
pub fn count_boundary_verts(positions: &[[f32; 3]], triangles: &[[u32; 3]]) -> usize {
    if positions.is_empty() {
        return 0;
    }
    use std::collections::HashMap;
    let mut edge_count: HashMap<(u32, u32), u32> = HashMap::new();
    for tri in triangles {
        for k in 0..3usize {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    let mut on_boundary = vec![false; positions.len()];
    for ((a, b), cnt) in &edge_count {
        if *cnt == 1 {
            on_boundary[*a as usize] = true;
            on_boundary[*b as usize] = true;
        }
    }
    on_boundary.iter().filter(|&&b| b).count()
}

/// Find pairs of vertices at the same 3-D position (within threshold).
#[allow(dead_code)]
pub fn find_duplicate_positions(positions: &[[f32; 3]], threshold: f32) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    for i in 0..positions.len() {
        for j in (i + 1)..positions.len() {
            if dist3(positions[i], positions[j]) < threshold {
                result.push((i, j));
            }
        }
    }
    result
}

/// Merge vertex groups: average position/normal, keep first UV, build remap table.
/// Returns `(new_positions, new_normals, new_uvs, remap)` where `remap[old] = new`.
#[allow(dead_code)]
pub fn merge_vertex_groups(
    groups: &[Vec<usize>],
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
) -> MergeGroupResult {
    let n_orig = positions.len();
    let mut remap = vec![0u32; n_orig];

    let mut new_positions: Vec<[f32; 3]> = Vec::new();
    let mut new_normals: Vec<[f32; 3]> = Vec::new();
    let mut new_uvs: Vec<[f32; 2]> = Vec::new();

    for group in groups {
        if group.is_empty() {
            continue;
        }
        let new_id = new_positions.len() as u32;
        let inv = 1.0 / group.len() as f32;
        let mut s = [0f32; 3];
        for &idx in group {
            if let Some(p) = positions.get(idx) {
                s[0] += p[0];
                s[1] += p[1];
                s[2] += p[2];
            }
        }
        let avg_pos = [s[0] * inv, s[1] * inv, s[2] * inv];
        let avg_norm = compute_average_normal(normals, group);
        let uv = uvs.get(group[0]).copied().unwrap_or([0.0; 2]);
        new_positions.push(avg_pos);
        new_normals.push(avg_norm);
        new_uvs.push(uv);
        for &idx in group {
            if idx < n_orig {
                remap[idx] = new_id;
            }
        }
    }

    (new_positions, new_normals, new_uvs, remap)
}

/// Compute the normalised average of the given normals.
#[allow(dead_code)]
pub fn compute_average_normal(normals: &[[f32; 3]], indices: &[usize]) -> [f32; 3] {
    if indices.is_empty() || normals.is_empty() {
        return [0.0, 1.0, 0.0];
    }
    let mut sum = [0f32; 3];
    let mut count = 0usize;
    for &i in indices {
        if let Some(n) = normals.get(i) {
            sum[0] += n[0];
            sum[1] += n[1];
            sum[2] += n[2];
            count += 1;
        }
    }
    if count == 0 {
        return [0.0, 1.0, 0.0];
    }
    let len = norm3(sum);
    if len < 1e-9 {
        [0.0, 1.0, 0.0]
    } else {
        [sum[0] / len, sum[1] / len, sum[2] / len]
    }
}

/// JSON summary of a weld result.
#[allow(dead_code)]
pub fn seam_weld_report(result: &WeldSeamResult) -> String {
    format!(
        "{{\"seams_welded\":{},\"welded_pairs\":{},\"new_vertex_count\":{}}}",
        result.seams_welded,
        result.welded_pairs.len(),
        result.new_positions.len()
    )
}

/// Linearly interpolate between two UV coordinates at parameter `t`.
#[allow(dead_code)]
pub fn project_uv_along_edge(uv_a: [f32; 2], uv_b: [f32; 2], t: f32) -> [f32; 2] {
    [
        uv_a[0] + (uv_b[0] - uv_a[0]) * t,
        uv_a[1] + (uv_b[1] - uv_a[1]) * t,
    ]
}

/// Sum of 3-D edge lengths for all seam edges.
#[allow(dead_code)]
pub fn seam_boundary_length(seams: &[SeamEdge], positions: &[[f32; 3]]) -> f32 {
    seams
        .iter()
        .map(|s| {
            let pa = positions.get(s.vid_a as usize).copied().unwrap_or([0.0; 3]);
            let pb = positions.get(s.vid_b as usize).copied().unwrap_or([0.0; 3]);
            dist3(pa, pb)
        })
        .sum()
}

/// Connected-component detection in UV space.
/// Two triangles are in the same island if they share a UV-space edge.
#[allow(dead_code)]
pub fn detect_uv_islands(uvs: &[[f32; 2]], triangles: &[[u32; 3]]) -> Vec<Vec<usize>> {
    let n_tris = triangles.len();
    if n_tris == 0 {
        return vec![];
    }

    // Build adjacency: two triangles are adjacent if they share a UV edge
    use std::collections::HashMap;
    let mut uv_edge_map: HashMap<[u32; 2], Vec<usize>> = HashMap::new();

    for (ti, tri) in triangles.iter().enumerate() {
        for k in 0..3usize {
            let a = tri[k] as usize;
            let b = tri[(k + 1) % 3] as usize;
            if a >= uvs.len() || b >= uvs.len() {
                continue;
            }
            let qa = quantise_uv(uvs[a]);
            let qb = quantise_uv(uvs[b]);
            let key = if qa <= qb { [qa, qb] } else { [qb, qa] };
            uv_edge_map.entry(key).or_default().push(ti);
        }
    }

    let mut comp = vec![usize::MAX; n_tris];
    let mut islands: Vec<Vec<usize>> = Vec::new();

    for start in 0..n_tris {
        if comp[start] != usize::MAX {
            continue;
        }
        let island_id = islands.len();
        let mut island = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(start);
        comp[start] = island_id;
        while let Some(ti) = queue.pop_front() {
            island.push(ti);
            let tri = &triangles[ti];
            for k in 0..3usize {
                let a = tri[k] as usize;
                let b = tri[(k + 1) % 3] as usize;
                if a >= uvs.len() || b >= uvs.len() {
                    continue;
                }
                let qa = quantise_uv(uvs[a]);
                let qb = quantise_uv(uvs[b]);
                let key = if qa <= qb { [qa, qb] } else { [qb, qa] };
                if let Some(nbrs) = uv_edge_map.get(&key) {
                    for &nti in nbrs {
                        if comp[nti] == usize::MAX {
                            comp[nti] = island_id;
                            queue.push_back(nti);
                        }
                    }
                }
            }
        }
        islands.push(island);
    }

    islands
}

fn quantise_uv(uv: [f32; 2]) -> u32 {
    let x = (uv[0] * 65536.0).round() as i32;
    let y = (uv[1] * 65536.0).round() as i32;
    ((x as u32).wrapping_mul(73856093)) ^ ((y as u32).wrapping_mul(19349663))
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_tri_positions() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]]
    }

    fn simple_tri_uvs() -> Vec<[f32; 2]> {
        vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]]
    }

    fn simple_triangles() -> Vec<[u32; 3]> {
        vec![[0, 1, 2]]
    }

    // 1
    #[test]
    fn find_duplicate_positions_exact() {
        let pos = vec![[0.0f32; 3], [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let dups = find_duplicate_positions(&pos, 1e-4);
        assert_eq!(dups.len(), 1);
        assert_eq!(dups[0], (0, 2));
    }

    // 2
    #[test]
    fn find_duplicate_positions_no_dups() {
        let pos = vec![[0.0f32; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let dups = find_duplicate_positions(&pos, 1e-4);
        assert!(dups.is_empty());
    }

    // 3
    #[test]
    fn count_boundary_verts_single_tri() {
        let pos = simple_tri_positions();
        let tris = simple_triangles();
        let count = count_boundary_verts(&pos, &tris);
        assert_eq!(count, 3);
    }

    // 4
    #[test]
    fn count_boundary_verts_empty() {
        assert_eq!(count_boundary_verts(&[], &[]), 0);
    }

    // 5
    #[test]
    fn count_boundary_verts_two_tris_shared_edge() {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, -1.0, 0.0],
        ];
        let tris = vec![[0, 1, 2], [0, 3, 1]];
        let count = count_boundary_verts(&pos, &tris);
        assert_eq!(count, 4);
    }

    // 6
    #[test]
    fn compute_average_normal_single() {
        let normals = vec![[0.0f32, 0.0, 1.0]];
        let n = compute_average_normal(&normals, &[0]);
        assert!((n[2] - 1.0).abs() < 1e-5);
    }

    // 7
    #[test]
    fn compute_average_normal_two_cancel() {
        let normals = vec![[0.0f32, 0.0, 1.0], [0.0, 0.0, -1.0]];
        let n = compute_average_normal(&normals, &[0, 1]);
        assert!((n[1] - 1.0).abs() < 1e-5);
    }

    // 8
    #[test]
    fn compute_average_normal_empty() {
        let n = compute_average_normal(&[], &[]);
        assert!((n[1] - 1.0).abs() < 1e-5);
    }

    // 9
    #[test]
    fn project_uv_along_edge_midpoint() {
        let uv = project_uv_along_edge([0.0, 0.0], [1.0, 1.0], 0.5);
        assert!((uv[0] - 0.5).abs() < 1e-6);
        assert!((uv[1] - 0.5).abs() < 1e-6);
    }

    // 10
    #[test]
    fn project_uv_along_edge_endpoints() {
        let a = [0.2f32, 0.3];
        let b = [0.8f32, 0.9];
        let uv0 = project_uv_along_edge(a, b, 0.0);
        let uv1 = project_uv_along_edge(a, b, 1.0);
        assert!((uv0[0] - a[0]).abs() < 1e-6);
        assert!((uv1[0] - b[0]).abs() < 1e-6);
    }

    // 11
    #[test]
    fn seam_boundary_length_single() {
        let pos = vec![[0.0f32, 0.0, 0.0], [3.0, 4.0, 0.0]];
        let seam = SeamEdge {
            vid_a: 0,
            vid_b: 1,
            uv_a: [0.0, 0.0],
            uv_b: [1.0, 0.0],
            twin_vid_a: 0,
            twin_vid_b: 1,
        };
        let len = seam_boundary_length(&[seam], &pos);
        assert!((len - 5.0).abs() < 1e-4);
    }

    // 12
    #[test]
    fn detect_uv_islands_single_triangle() {
        let uvs = simple_tri_uvs();
        let tris = simple_triangles();
        let islands = detect_uv_islands(&uvs, &tris);
        assert_eq!(islands.len(), 1);
        assert_eq!(islands[0].len(), 1);
    }

    // 13
    #[test]
    fn weld_seams_no_duplicates() {
        let pos = simple_tri_positions();
        let norms = vec![[0.0f32, 0.0, 1.0]; 3];
        let uvs = simple_tri_uvs();
        let tris = simple_triangles();
        let result = weld_seams(&pos, &norms, &uvs, &tris, 1e-4);
        assert_eq!(result.new_triangles.len(), tris.len());
    }

    // 14
    #[test]
    fn weld_seams_with_duplicate_position() {
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.0, 0.0, 0.0], // duplicate of 0
            [1.0, 0.0, 0.0], // duplicate of 1
            [0.5, -1.0, 0.0],
        ];
        let norms = vec![[0.0f32, 0.0, 1.0]; 6];
        let uvs: Vec<[f32; 2]> = (0..6).map(|i| [i as f32 * 0.1, 0.0]).collect();
        let tris = vec![[0u32, 1, 2], [3, 4, 5]];
        let result = weld_seams(&pos, &norms, &uvs, &tris, 1e-3);
        assert!(result.new_positions.len() < 6);
    }

    // 15
    #[test]
    fn seam_weld_report_contains_keys() {
        let result = WeldSeamResult {
            welded_pairs: vec![(0, 1)],
            new_positions: vec![[0.0; 3]],
            new_normals: vec![[0.0, 1.0, 0.0]],
            new_uvs: vec![[0.0; 2]],
            new_triangles: vec![],
            seams_welded: 1,
        };
        let report = seam_weld_report(&result);
        assert!(report.contains("seams_welded"));
        assert!(report.contains("welded_pairs"));
        assert!(report.contains("new_vertex_count"));
    }

    // 16
    #[test]
    fn merge_vertex_groups_single_group() {
        let groups = vec![vec![0usize, 1, 2]];
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 0.0, 0.0]];
        let norms = vec![[0.0f32, 0.0, 1.0]; 3];
        let uvs = vec![[0.0f32, 0.0]; 3];
        let (new_pos, _, _, remap) = merge_vertex_groups(&groups, &pos, &norms, &uvs);
        assert_eq!(new_pos.len(), 1);
        assert_eq!(remap[0], 0);
        assert_eq!(remap[1], 0);
        assert_eq!(remap[2], 0);
    }

    // 17
    #[test]
    fn find_seam_edges_empty() {
        let se = find_seam_edges(&[], &[], &[], 1e-4);
        assert!(se.is_empty());
    }
}
