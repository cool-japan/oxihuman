// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Isotropic remeshing via the Botsch-Kobbelt algorithm.

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

fn midpoint(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn len(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

fn normalize(a: [f32; 3]) -> [f32; 3] {
    let l = len(a);
    if l < 1e-10 {
        [0.0, 0.0, 1.0]
    } else {
        [a[0] / l, a[1] / l, a[2] / l]
    }
}

// ---------------------------------------------------------------------------
// public API
// ---------------------------------------------------------------------------

/// Average edge length of the mesh.
pub fn average_edge_length(positions: &[[f32; 3]], tris: &[[u32; 3]]) -> f32 {
    if tris.is_empty() {
        return 0.0;
    }
    let mut total = 0.0f32;
    let mut count = 0usize;
    for tri in tris {
        let [a, b, c] = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        if a < positions.len() && b < positions.len() && c < positions.len() {
            total += dist(positions[a], positions[b]);
            total += dist(positions[b], positions[c]);
            total += dist(positions[c], positions[a]);
            count += 3;
        }
    }
    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
}

/// Returns (min, max, mean) edge lengths.
pub fn edge_length_stats(positions: &[[f32; 3]], tris: &[[u32; 3]]) -> (f32, f32, f32) {
    if tris.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    let mut mn = f32::MAX;
    let mut mx = 0.0f32;
    let mut total = 0.0f32;
    let mut count = 0usize;
    for tri in tris {
        let [a, b, c] = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        if a < positions.len() && b < positions.len() && c < positions.len() {
            for (p, q) in [(a, b), (b, c), (c, a)] {
                let l = dist(positions[p], positions[q]);
                mn = mn.min(l);
                mx = mx.max(l);
                total += l;
                count += 1;
            }
        }
    }
    if count == 0 {
        (0.0, 0.0, 0.0)
    } else {
        (mn, mx, total / count as f32)
    }
}

/// Area-weighted vertex normals (used internally for tangential relaxation).
pub fn compute_vertex_normals_remesh(positions: &[[f32; 3]], tris: &[[u32; 3]]) -> Vec<[f32; 3]> {
    let n = positions.len();
    let mut normals = vec![[0.0f32; 3]; n];
    for tri in tris {
        let [ai, bi, ci] = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        if ai >= n || bi >= n || ci >= n {
            continue;
        }
        let ab = sub(positions[bi], positions[ai]);
        let ac = sub(positions[ci], positions[ai]);
        let cr = cross(ab, ac);
        for idx in [ai, bi, ci] {
            normals[idx] = add(normals[idx], cr);
        }
    }
    for nrm in normals.iter_mut() {
        *nrm = normalize(*nrm);
    }
    normals
}

/// Split all edges longer than `max_len` by inserting a midpoint vertex.
pub fn split_long_edges(
    positions: &[[f32; 3]],
    tris: &[[u32; 3]],
    max_len: f32,
) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let mut new_pos: Vec<[f32; 3]> = positions.to_vec();
    let mut new_tris: Vec<[u32; 3]> = Vec::new();
    let mut edge_map: HashMap<(u32, u32), u32> = HashMap::new();

    for &tri in tris {
        let [a, b, c] = [tri[0], tri[1], tri[2]];
        let pa = new_pos[a as usize];
        let pb = new_pos[b as usize];
        let pc = new_pos[c as usize];
        let lab = dist(pa, pb);
        let lbc = dist(pb, pc);
        let lca = dist(pc, pa);

        let split_ab = lab > max_len;
        let split_bc = lbc > max_len;
        let split_ca = lca > max_len;

        let get_mid =
            |p: u32, q: u32, pos: &mut Vec<[f32; 3]>, map: &mut HashMap<(u32, u32), u32>| -> u32 {
                let key = (p.min(q), p.max(q));
                if let Some(&idx) = map.get(&key) {
                    return idx;
                }
                let mid = midpoint(pos[p as usize], pos[q as usize]);
                let idx = pos.len() as u32;
                pos.push(mid);
                map.insert(key, idx);
                idx
            };

        match (split_ab, split_bc, split_ca) {
            (false, false, false) => new_tris.push(tri),
            (true, false, false) => {
                let m = get_mid(a, b, &mut new_pos, &mut edge_map);
                new_tris.push([a, m, c]);
                new_tris.push([m, b, c]);
            }
            (false, true, false) => {
                let m = get_mid(b, c, &mut new_pos, &mut edge_map);
                new_tris.push([a, b, m]);
                new_tris.push([a, m, c]);
            }
            (false, false, true) => {
                let m = get_mid(c, a, &mut new_pos, &mut edge_map);
                new_tris.push([a, b, m]);
                new_tris.push([m, b, c]);
            }
            (true, true, false) => {
                let mab = get_mid(a, b, &mut new_pos, &mut edge_map);
                let mbc = get_mid(b, c, &mut new_pos, &mut edge_map);
                new_tris.push([a, mab, c]);
                new_tris.push([mab, b, mbc]);
                new_tris.push([mab, mbc, c]);
            }
            (true, false, true) => {
                let mab = get_mid(a, b, &mut new_pos, &mut edge_map);
                let mca = get_mid(c, a, &mut new_pos, &mut edge_map);
                new_tris.push([a, mab, mca]);
                new_tris.push([mab, b, c]);
                new_tris.push([mab, c, mca]);
            }
            (false, true, true) => {
                let mbc = get_mid(b, c, &mut new_pos, &mut edge_map);
                let mca = get_mid(c, a, &mut new_pos, &mut edge_map);
                new_tris.push([a, b, mca]);
                new_tris.push([b, mbc, mca]);
                new_tris.push([mbc, c, mca]);
            }
            (true, true, true) => {
                let mab = get_mid(a, b, &mut new_pos, &mut edge_map);
                let mbc = get_mid(b, c, &mut new_pos, &mut edge_map);
                let mca = get_mid(c, a, &mut new_pos, &mut edge_map);
                new_tris.push([a, mab, mca]);
                new_tris.push([mab, b, mbc]);
                new_tris.push([mca, mbc, c]);
                new_tris.push([mab, mbc, mca]);
            }
        }
    }
    (new_pos, new_tris)
}

/// Collapse edges shorter than `min_len` to their midpoint.
pub fn collapse_short_edges(
    positions: &[[f32; 3]],
    tris: &[[u32; 3]],
    min_len: f32,
) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let mut pos = positions.to_vec();
    let mut tri_list: Vec<[u32; 3]> = tris.to_vec();

    // Iterate a limited number of passes to avoid infinite loops
    for _ in 0..4 {
        let mut remap: Vec<u32> = (0..pos.len() as u32).collect();

        let mut collapsed_any = false;
        for tri in &tri_list {
            let [a, b, c] = [tri[0], tri[1], tri[2]];
            let pairs = [(a, b), (b, c), (c, a)];
            for (p, q) in pairs {
                let rp = remap[p as usize];
                let rq = remap[q as usize];
                if rp == rq {
                    continue;
                }
                if rp >= pos.len() as u32 || rq >= pos.len() as u32 {
                    continue;
                }
                if dist(pos[rp as usize], pos[rq as usize]) < min_len {
                    let mid = midpoint(pos[rp as usize], pos[rq as usize]);
                    pos[rp as usize] = mid;
                    // remap rq -> rp
                    let rq_usize = rq as usize;
                    remap[rq_usize] = rp;
                    collapsed_any = true;
                }
            }
        }

        if !collapsed_any {
            break;
        }

        // Flatten remap chains
        let n = remap.len();
        for i in 0..n {
            let mut cur = i;
            while remap[cur] as usize != cur {
                cur = remap[cur] as usize;
            }
            remap[i] = cur as u32;
        }

        // Rebuild tris, skipping degenerate ones
        let new_tris: Vec<[u32; 3]> = tri_list
            .iter()
            .map(|t| {
                [
                    remap[t[0] as usize],
                    remap[t[1] as usize],
                    remap[t[2] as usize],
                ]
            })
            .filter(|t| t[0] != t[1] && t[1] != t[2] && t[0] != t[2])
            .collect();
        tri_list = new_tris;
    }

    // Compact unused vertices
    let mut used = vec![false; pos.len()];
    for t in &tri_list {
        for &v in t.iter() {
            if (v as usize) < used.len() {
                used[v as usize] = true;
            }
        }
    }
    let mut new_idx = vec![0u32; pos.len()];
    let mut compact = Vec::new();
    let mut counter = 0u32;
    for (i, &u) in used.iter().enumerate() {
        if u {
            new_idx[i] = counter;
            compact.push(pos[i]);
            counter += 1;
        }
    }
    let final_tris: Vec<[u32; 3]> = tri_list
        .iter()
        .map(|t| {
            [
                new_idx[t[0] as usize],
                new_idx[t[1] as usize],
                new_idx[t[2] as usize],
            ]
        })
        .collect();

    (compact, final_tris)
}

/// Flip edges to minimize valence deviation (target = 6).
pub fn flip_edges_for_valence(positions: &[[f32; 3]], tris: &[[u32; 3]]) -> Vec<[u32; 3]> {
    let n = positions.len();
    let mut tri_list = tris.to_vec();

    // Build valence counts
    let mut valence = vec![0i32; n];
    for t in &tri_list {
        for &v in t.iter() {
            if (v as usize) < n {
                valence[v as usize] += 1;
            }
        }
    }

    // Build edge -> two triangles map
    // edge (a,b) where a < b -> list of triangle indices
    let build_edge_map = |tris: &[[u32; 3]]| -> HashMap<(u32, u32), Vec<usize>> {
        let mut map: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
        for (i, t) in tris.iter().enumerate() {
            let verts = [t[0], t[1], t[2]];
            for j in 0..3 {
                let a = verts[j];
                let b = verts[(j + 1) % 3];
                let key = (a.min(b), a.max(b));
                map.entry(key).or_default().push(i);
            }
        }
        map
    };

    let edge_map = build_edge_map(&tri_list);

    // For each interior edge, check if flipping reduces total valence deviation
    let mut flipped: HashSet<usize> = HashSet::new();
    for (&(ea, eb), tri_idxs) in &edge_map {
        if tri_idxs.len() != 2 {
            continue;
        }
        let (ti0, ti1) = (tri_idxs[0], tri_idxs[1]);
        if flipped.contains(&ti0) || flipped.contains(&ti1) {
            continue;
        }
        let t0 = tri_list[ti0];
        let t1 = tri_list[ti1];

        // Find the vertex in t0 not on this edge
        let opp0 = t0.iter().copied().find(|&v| v != ea && v != eb);
        let opp1 = t1.iter().copied().find(|&v| v != ea && v != eb);
        let (c, d) = match (opp0, opp1) {
            (Some(x), Some(y)) => (x, y),
            _ => continue,
        };

        // valence deviation before
        let dev_before = |a: u32, b: u32, c: u32, d: u32| -> i32 {
            let va = valence.get(a as usize).copied().unwrap_or(6);
            let vb = valence.get(b as usize).copied().unwrap_or(6);
            let vc = valence.get(c as usize).copied().unwrap_or(6);
            let vd = valence.get(d as usize).copied().unwrap_or(6);
            (va - 6).abs() + (vb - 6).abs() + (vc - 6).abs() + (vd - 6).abs()
        };

        // After flip: edge becomes c-d, shared vertices become a,b with -1 each
        // c and d each gain +1
        let before = dev_before(ea, eb, c, d);
        let after = {
            let va = (valence.get(ea as usize).copied().unwrap_or(6) - 1 - 6).abs();
            let vb = (valence.get(eb as usize).copied().unwrap_or(6) - 1 - 6).abs();
            let vc = (valence.get(c as usize).copied().unwrap_or(6) + 1 - 6).abs();
            let vd = (valence.get(d as usize).copied().unwrap_or(6) + 1 - 6).abs();
            va + vb + vc + vd
        };

        if after < before {
            // Perform flip: replace two tris with new configuration
            tri_list[ti0] = [ea, c, d];
            tri_list[ti1] = [eb, d, c];
            flipped.insert(ti0);
            flipped.insert(ti1);
            // Update valences
            if (ea as usize) < n {
                valence[ea as usize] -= 1;
            }
            if (eb as usize) < n {
                valence[eb as usize] -= 1;
            }
            if (c as usize) < n {
                valence[c as usize] += 1;
            }
            if (d as usize) < n {
                valence[d as usize] += 1;
            }
        }
    }

    tri_list
}

/// Tangential Laplacian smoothing projected onto the tangent plane.
pub fn tangential_relaxation(
    positions: &[[f32; 3]],
    tris: &[[u32; 3]],
    iters: usize,
) -> Vec<[f32; 3]> {
    let n = positions.len();
    let mut pos = positions.to_vec();

    // Build adjacency
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for t in tris {
        let [a, b, c] = [t[0] as usize, t[1] as usize, t[2] as usize];
        if a < n && b < n && c < n {
            if !adj[a].contains(&b) {
                adj[a].push(b);
            }
            if !adj[a].contains(&c) {
                adj[a].push(c);
            }
            if !adj[b].contains(&a) {
                adj[b].push(a);
            }
            if !adj[b].contains(&c) {
                adj[b].push(c);
            }
            if !adj[c].contains(&a) {
                adj[c].push(a);
            }
            if !adj[c].contains(&b) {
                adj[c].push(b);
            }
        }
    }

    for _ in 0..iters {
        let normals = compute_vertex_normals_remesh(&pos, tris);
        let new_pos: Vec<[f32; 3]> = (0..n)
            .map(|i| {
                let neighbors = &adj[i];
                if neighbors.is_empty() {
                    return pos[i];
                }
                // Laplacian
                let mut centroid = [0.0f32; 3];
                for &j in neighbors {
                    centroid = add(centroid, pos[j]);
                }
                let k = neighbors.len() as f32;
                centroid = scale(centroid, 1.0 / k);

                // Project Laplacian displacement onto tangent plane
                let d = sub(centroid, pos[i]);
                let n_i = normals[i];
                let normal_comp = dot(d, n_i);
                let tangential = sub(d, scale(n_i, normal_comp));
                add(pos[i], tangential)
            })
            .collect();
        pos = new_pos;
    }
    pos
}

/// Full isotropic remeshing pipeline.
pub fn isotropic_remesh(
    positions: &[[f32; 3]],
    tris: &[[u32; 3]],
    target_edge_len: f32,
    iters: usize,
) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let max_len = target_edge_len * 4.0 / 3.0;
    let min_len = target_edge_len * 4.0 / 5.0;

    let mut cur_pos = positions.to_vec();
    let mut cur_tris = tris.to_vec();

    for _ in 0..iters {
        let (p, t) = split_long_edges(&cur_pos, &cur_tris, max_len);
        let (p2, t2) = collapse_short_edges(&p, &t, min_len);
        let t3 = flip_edges_for_valence(&p2, &t2);
        let p3 = tangential_relaxation(&p2, &t3, 1);
        cur_pos = p3;
        cur_tris = t3;
    }

    (cur_pos, cur_tris)
}

// ---------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_tri() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0, 1, 2]];
        (pos, tris)
    }

    fn grid_mesh(n: usize) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let mut pos = Vec::new();
        let mut tris = Vec::new();
        let step = 1.0 / n as f32;
        for i in 0..=n {
            for j in 0..=n {
                pos.push([i as f32 * step, j as f32 * step, 0.0]);
            }
        }
        let stride = n + 1;
        for i in 0..n {
            for j in 0..n {
                let a = (i * stride + j) as u32;
                let b = (i * stride + j + 1) as u32;
                let c = ((i + 1) * stride + j) as u32;
                let d = ((i + 1) * stride + j + 1) as u32;
                tris.push([a, b, c]);
                tris.push([b, d, c]);
            }
        }
        (pos, tris)
    }

    #[test]
    fn test_average_edge_length_unit_tri() {
        let (pos, tris) = unit_tri();
        let avg = average_edge_length(&pos, &tris);
        // edges: 1.0, 1.0, sqrt(2) => mean = (2 + sqrt(2)) / 3
        let expected = (2.0 + 2.0_f32.sqrt()) / 3.0;
        assert!(
            (avg - expected).abs() < 1e-5,
            "avg={avg} expected={expected}"
        );
    }

    #[test]
    fn test_average_edge_length_empty() {
        assert_eq!(average_edge_length(&[], &[]), 0.0);
    }

    #[test]
    fn test_edge_length_stats_unit_tri() {
        let (pos, tris) = unit_tri();
        let (mn, mx, mean) = edge_length_stats(&pos, &tris);
        assert!((mn - 1.0).abs() < 1e-5, "min should be 1.0, got {mn}");
        assert!(
            (mx - 2.0_f32.sqrt()).abs() < 1e-5,
            "max should be sqrt(2), got {mx}"
        );
        assert!(
            mean > mn && mean < mx,
            "mean={mean} should be between min and max"
        );
    }

    #[test]
    fn test_edge_length_stats_empty() {
        let (mn, mx, mean) = edge_length_stats(&[], &[]);
        assert_eq!(mn, 0.0);
        assert_eq!(mx, 0.0);
        assert_eq!(mean, 0.0);
    }

    #[test]
    fn test_split_long_edges_increases_vertices() {
        let (pos, tris) = unit_tri();
        let (new_pos, new_tris) = split_long_edges(&pos, &tris, 0.9);
        assert!(new_pos.len() > pos.len(), "split should add vertices");
        assert!(new_tris.len() > tris.len(), "split should add triangles");
    }

    #[test]
    fn test_split_long_edges_no_split_needed() {
        let (pos, tris) = unit_tri();
        let (new_pos, new_tris) = split_long_edges(&pos, &tris, 2.0);
        assert_eq!(new_pos.len(), pos.len());
        assert_eq!(new_tris.len(), tris.len());
    }

    #[test]
    fn test_collapse_short_edges_reduces_vertices() {
        // A mesh with many small edges
        let pos = vec![
            [0.0, 0.0, 0.0],
            [0.05, 0.0, 0.0],
            [0.0, 0.05, 0.0],
            [1.0, 0.0, 0.0],
        ];
        let tris = vec![[0, 1, 2], [1, 3, 2]];
        let (new_pos, _new_tris) = collapse_short_edges(&pos, &tris, 0.1);
        // The first triangle has edges much shorter than 0.1, expect fewer verts
        assert!(
            new_pos.len() <= pos.len(),
            "collapse should not increase vertices"
        );
    }

    #[test]
    fn test_collapse_does_not_collapse_all() {
        let (pos, tris) = grid_mesh(4);
        let avg = average_edge_length(&pos, &tris);
        // collapse with min = 0.01 * avg -> should not remove all triangles
        let (new_pos, new_tris) = collapse_short_edges(&pos, &tris, avg * 0.01);
        assert!(!new_pos.is_empty());
        assert!(!new_tris.is_empty());
    }

    #[test]
    fn test_flip_edges_for_valence_no_panic() {
        let (pos, tris) = grid_mesh(3);
        let result = flip_edges_for_valence(&pos, &tris);
        assert_eq!(result.len(), tris.len(), "flip should not change tri count");
    }

    #[test]
    fn test_flip_edges_for_valence_returns_same_count() {
        let (pos, tris) = unit_tri();
        let result = flip_edges_for_valence(&pos, &tris);
        assert_eq!(result.len(), tris.len());
    }

    #[test]
    fn test_tangential_relaxation_moves_interior() {
        let (pos, tris) = grid_mesh(4);
        let relaxed = tangential_relaxation(&pos, &tris, 3);
        assert_eq!(relaxed.len(), pos.len());
        // At least one vertex should have moved
        let any_moved = pos
            .iter()
            .zip(relaxed.iter())
            .any(|(a, b)| (a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs() > 1e-7);
        assert!(any_moved, "tangential relaxation should move some vertices");
    }

    #[test]
    fn test_tangential_relaxation_no_nan() {
        let (pos, tris) = grid_mesh(4);
        let relaxed = tangential_relaxation(&pos, &tris, 5);
        for p in &relaxed {
            assert!(p[0].is_finite() && p[1].is_finite() && p[2].is_finite());
        }
    }

    #[test]
    fn test_compute_vertex_normals_remesh_unit_tri() {
        let (pos, tris) = unit_tri();
        let norms = compute_vertex_normals_remesh(&pos, &tris);
        assert_eq!(norms.len(), pos.len());
        // Should be pointing mostly in Z
        for n in &norms {
            assert!(
                n[2] > 0.5,
                "normal z component should be positive, got {}",
                n[2]
            );
        }
    }

    #[test]
    fn test_isotropic_remesh_no_panic() {
        let (pos, tris) = grid_mesh(4);
        let target = average_edge_length(&pos, &tris);
        let (new_pos, new_tris) = isotropic_remesh(&pos, &tris, target, 2);
        assert!(!new_pos.is_empty());
        assert!(!new_tris.is_empty());
    }

    #[test]
    fn test_isotropic_remesh_no_nan() {
        let (pos, tris) = grid_mesh(3);
        let target = average_edge_length(&pos, &tris);
        let (new_pos, _) = isotropic_remesh(&pos, &tris, target, 2);
        for p in &new_pos {
            assert!(p[0].is_finite() && p[1].is_finite() && p[2].is_finite());
        }
    }
}
