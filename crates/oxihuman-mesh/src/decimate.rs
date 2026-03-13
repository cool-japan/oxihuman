// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;

/// Result metadata from a decimation operation.
pub struct DecimateResult {
    pub original_faces: usize,
    pub result_faces: usize,
    pub original_verts: usize,
    pub result_verts: usize,
    pub ratio_achieved: f32,
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn edge_len(positions: &[[f32; 3]], v0: usize, v1: usize) -> f32 {
    let p0 = positions[v0];
    let p1 = positions[v1];
    let dx = p1[0] - p0[0];
    let dy = p1[1] - p0[1];
    let dz = p1[2] - p0[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Walk the representative chain to find the root, with path compression.
fn find(rep: &mut [u32], mut v: u32) -> u32 {
    while rep[v as usize] != v {
        let gp = rep[rep[v as usize] as usize];
        rep[v as usize] = gp;
        v = gp;
    }
    v
}

// ── core decimation ───────────────────────────────────────────────────────────

/// Decimate a mesh to approximately `target_faces` triangles using edge-collapse.
/// Returns a new `MeshBuffers` with fewer vertices and faces.
/// If `target_faces >= current_faces`, returns a clone of the input.
pub fn decimate(mesh: &MeshBuffers, target_faces: usize) -> MeshBuffers {
    let orig_face_count = mesh.indices.len() / 3;
    if target_faces >= orig_face_count {
        return mesh.clone();
    }

    let nv = mesh.positions.len();

    // Working index buffer stored as a flat Vec; each triple is one face.
    let mut indices: Vec<u32> = mesh.indices.clone();

    // Union-find: representative[v] == v means v is active.
    let mut representative: Vec<u32> = (0..nv as u32).collect();

    // Build unique edge list with canonical ordering (min, max).
    // Min-heap via Reverse: stores (Reverse<length_bits>, v0, v1).
    let mut heap: BinaryHeap<(Reverse<u32>, u32, u32)> = {
        use std::collections::HashSet;
        let mut seen: HashSet<(u32, u32)> = HashSet::new();
        let mut h = BinaryHeap::new();
        for tri in indices.chunks_exact(3) {
            let (a, b, c) = (tri[0], tri[1], tri[2]);
            for &(ea, eb) in &[(a, b), (b, c), (a, c)] {
                let key = (ea.min(eb), ea.max(eb));
                if seen.insert(key) {
                    let len = edge_len(&mesh.positions, key.0 as usize, key.1 as usize);
                    h.push((Reverse(len.to_bits()), key.0, key.1));
                }
            }
        }
        h
    };

    // Track which faces have been removed.
    let mut face_removed: Vec<bool> = vec![false; orig_face_count];
    let mut active_faces = orig_face_count;

    // Greedy collapse loop.
    while active_faces > target_faces {
        let (_, raw_v0, raw_v1) = match heap.pop() {
            Some(e) => e,
            None => break,
        };

        let v0 = find(&mut representative, raw_v0);
        let v1 = find(&mut representative, raw_v1);

        if v0 == v1 {
            continue; // already merged
        }

        // Merge v1 → v0.
        representative[v1 as usize] = v0;

        // Update index buffer and detect newly degenerate faces.
        for (fi, removed) in face_removed.iter_mut().enumerate() {
            if *removed {
                continue;
            }
            let base = fi * 3;
            let mut changed = false;
            for k in 0..3 {
                let rv = find(&mut representative, indices[base + k]);
                if rv != indices[base + k] {
                    indices[base + k] = rv;
                    changed = true;
                }
            }
            if changed {
                let (a, b, c) = (indices[base], indices[base + 1], indices[base + 2]);
                if a == b || b == c || a == c {
                    *removed = true;
                    active_faces -= 1;
                }
            }
        }
    }

    // ── Compact vertex buffer ────────────────────────────────────────────────
    let mut used = vec![false; nv];
    for (fi, &removed) in face_removed.iter().enumerate() {
        if removed {
            continue;
        }
        let base = fi * 3;
        for k in 0..3 {
            used[indices[base + k] as usize] = true;
        }
    }

    // Build remap: old index → new index.
    let mut remap: Vec<u32> = vec![0; nv];
    let mut new_positions: Vec<[f32; 3]> = Vec::new();
    let mut new_uvs: Vec<[f32; 2]> = Vec::new();
    let new_colors: Option<Vec<[f32; 4]>>;

    {
        let mut color_buf: Option<Vec<[f32; 4]>> = mesh.colors.as_ref().map(|_| Vec::new());
        let mut next = 0u32;
        for (old, &is_used) in used.iter().enumerate() {
            if is_used {
                remap[old] = next;
                next += 1;
                new_positions.push(mesh.positions[old]);
                new_uvs.push(if old < mesh.uvs.len() {
                    mesh.uvs[old]
                } else {
                    [0.0, 0.0]
                });
                if let (Some(ref mc), Some(ref mut nc)) = (&mesh.colors, &mut color_buf) {
                    nc.push(if old < mc.len() {
                        mc[old]
                    } else {
                        [1.0, 1.0, 1.0, 1.0]
                    });
                }
            }
        }
        new_colors = color_buf;
    }

    // Build new index buffer.
    let mut new_indices: Vec<u32> = Vec::new();
    for (fi, &removed) in face_removed.iter().enumerate() {
        if removed {
            continue;
        }
        let base = fi * 3;
        new_indices.push(remap[indices[base] as usize]);
        new_indices.push(remap[indices[base + 1] as usize]);
        new_indices.push(remap[indices[base + 2] as usize]);
    }

    let new_nv = new_positions.len();
    let mut result = MeshBuffers {
        positions: new_positions,
        normals: vec![[0.0, 1.0, 0.0]; new_nv],
        tangents: vec![[1.0, 0.0, 0.0, 1.0]; new_nv],
        uvs: new_uvs,
        indices: new_indices,
        colors: new_colors,
        has_suit: mesh.has_suit,
    };

    compute_normals(&mut result);
    result
}

/// Decimate to a fraction of the original face count.
/// `ratio` is in (0..1]: 0.5 means halve the face count.
pub fn decimate_ratio(mesh: &MeshBuffers, ratio: f32) -> MeshBuffers {
    let target = ((mesh.indices.len() / 3) as f32 * ratio.clamp(0.01, 1.0)) as usize;
    decimate(mesh, target.max(1))
}

/// Decimate with result metadata.
pub fn decimate_with_info(
    mesh: &MeshBuffers,
    target_faces: usize,
) -> (MeshBuffers, DecimateResult) {
    let original_faces = mesh.indices.len() / 3;
    let original_verts = mesh.positions.len();
    let result = decimate(mesh, target_faces);
    let result_faces = result.indices.len() / 3;
    let result_verts = result.positions.len();
    let ratio_achieved = if original_faces == 0 {
        1.0
    } else {
        result_faces as f32 / original_faces as f32
    };
    (
        result,
        DecimateResult {
            original_faces,
            result_faces,
            original_verts,
            result_verts,
            ratio_achieved,
        },
    )
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn grid_mesh(n: usize) -> MeshBuffers {
        let mut positions = Vec::new();
        for i in 0..n {
            for j in 0..n {
                positions.push([i as f32, 0.0, j as f32]);
            }
        }
        let mut indices = Vec::new();
        for i in 0..n - 1 {
            for j in 0..n - 1 {
                let b = (i * n + j) as u32;
                indices.extend_from_slice(&[b, b + 1, b + n as u32]);
                indices.extend_from_slice(&[b + 1, b + n as u32 + 1, b + n as u32]);
            }
        }
        MeshBuffers {
            positions,
            normals: vec![[0.0, 1.0, 0.0]; n * n],
            uvs: vec![[0.0, 0.0]; n * n],
            tangents: vec![],
            colors: None,
            indices,
            has_suit: true,
        }
    }

    #[test]
    fn decimate_reduces_faces() {
        let mesh = grid_mesh(5); // 4*4*2 = 32 faces
        assert_eq!(mesh.face_count(), 32);
        let result = decimate(&mesh, 10);
        assert!(
            result.face_count() <= 20,
            "expected <= 20 faces, got {}",
            result.face_count()
        );
    }

    #[test]
    fn decimate_above_target_no_change() {
        let mesh = grid_mesh(5);
        let original_faces = mesh.face_count();
        let result = decimate(&mesh, original_faces + 100);
        assert_eq!(result.face_count(), original_faces);
    }

    #[test]
    fn decimate_index_bounds_valid() {
        let mesh = grid_mesh(5);
        let result = decimate(&mesh, 10);
        let nv = result.positions.len();
        for &idx in &result.indices {
            assert!(
                (idx as usize) < nv,
                "index {} out of bounds (nv={})",
                idx,
                nv
            );
        }
    }

    #[test]
    fn decimate_ratio_half() {
        let mesh = grid_mesh(5);
        let original = mesh.face_count();
        let result = decimate_ratio(&mesh, 0.5);
        assert!(
            result.face_count() <= original,
            "ratio=0.5 should not increase face count"
        );
    }

    #[test]
    fn decimate_with_info_metadata() {
        let mesh = grid_mesh(5);
        let original_faces = mesh.indices.len() / 3;
        let (_, info) = decimate_with_info(&mesh, 10);
        assert_eq!(info.original_faces, original_faces);
    }

    #[test]
    fn decimate_result_has_vertices() {
        let mesh = grid_mesh(5);
        let result = decimate(&mesh, 1);
        assert!(
            result.positions.len() >= 3,
            "result must have at least 3 vertices"
        );
    }
}
