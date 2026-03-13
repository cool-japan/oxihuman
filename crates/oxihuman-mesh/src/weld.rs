// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Mesh vertex welding and seam repair utilities.

use crate::mesh::MeshBuffers;
use std::collections::{HashMap, HashSet};

/// Result of a weld operation.
pub struct WeldResult {
    /// Number of vertices removed by welding.
    pub verts_removed: usize,
    /// Mapping from old vertex index → new vertex index.
    pub remap: Vec<u32>,
}

/// Weld vertices that share the same position (within `tolerance`).
/// Averages normals, UVs, colors, and tangents for merged vertices.
/// Updates index buffer to use new vertex indices.
/// Returns a new [`MeshBuffers`] with merged vertices.
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn weld_by_position(mesh: &MeshBuffers, tolerance: f32) -> (MeshBuffers, WeldResult) {
    let n = mesh.positions.len();
    let inv_tol = if tolerance > 0.0 {
        1.0 / tolerance
    } else {
        1.0
    };

    // Map: quantized (x,y,z) → canonical vertex index in the new mesh.
    let mut pos_map: HashMap<(i64, i64, i64), u32> = HashMap::new();
    // old_to_new[i] = new index for old vertex i.
    let mut old_to_new: Vec<u32> = vec![0; n];
    // Track which old vertices map to each canonical new index.
    let mut canonical_members: Vec<Vec<usize>> = Vec::new();

    for (i, p) in mesh.positions.iter().enumerate() {
        let key = (
            (p[0] * inv_tol).round() as i64,
            (p[1] * inv_tol).round() as i64,
            (p[2] * inv_tol).round() as i64,
        );
        let next_new = canonical_members.len() as u32;
        let new_idx = *pos_map.entry(key).or_insert(next_new);
        if new_idx == next_new {
            canonical_members.push(Vec::new());
        }
        canonical_members[new_idx as usize].push(i);
        old_to_new[i] = new_idx;
    }

    let new_n = canonical_members.len();
    let mut positions = vec![[0.0f32; 3]; new_n];
    let mut normals = vec![[0.0f32; 3]; new_n];
    let mut uvs = vec![[0.0f32; 2]; new_n];
    let mut tangents: Vec<[f32; 4]> = if mesh.tangents.is_empty() {
        vec![]
    } else {
        vec![[0.0f32; 4]; new_n]
    };
    let mut colors: Option<Vec<[f32; 4]>> = mesh.colors.as_ref().map(|_| vec![[0.0f32; 4]; new_n]);

    for (new_idx, members) in canonical_members.iter().enumerate() {
        let count = members.len() as f32;
        let mut pos_acc = [0.0f32; 3];
        let mut nrm_acc = [0.0f32; 3];
        let mut uv_acc = [0.0f32; 2];
        let mut tan_acc = [0.0f32; 4];
        let mut col_acc = [0.0f32; 4];

        for &old in members {
            let p = mesh.positions[old];
            pos_acc[0] += p[0];
            pos_acc[1] += p[1];
            pos_acc[2] += p[2];

            if old < mesh.normals.len() {
                let nr = mesh.normals[old];
                nrm_acc[0] += nr[0];
                nrm_acc[1] += nr[1];
                nrm_acc[2] += nr[2];
            }
            if old < mesh.uvs.len() {
                let uv = mesh.uvs[old];
                uv_acc[0] += uv[0];
                uv_acc[1] += uv[1];
            }
            if !mesh.tangents.is_empty() && old < mesh.tangents.len() {
                let t = mesh.tangents[old];
                tan_acc[0] += t[0];
                tan_acc[1] += t[1];
                tan_acc[2] += t[2];
                tan_acc[3] += t[3];
            }
            if let Some(ref cols) = mesh.colors {
                if old < cols.len() {
                    let c = cols[old];
                    col_acc[0] += c[0];
                    col_acc[1] += c[1];
                    col_acc[2] += c[2];
                    col_acc[3] += c[3];
                }
            }
        }

        positions[new_idx] = [pos_acc[0] / count, pos_acc[1] / count, pos_acc[2] / count];
        uvs[new_idx] = [uv_acc[0] / count, uv_acc[1] / count];

        // Average and renormalize normal.
        let nx = nrm_acc[0] / count;
        let ny = nrm_acc[1] / count;
        let nz = nrm_acc[2] / count;
        let nlen = (nx * nx + ny * ny + nz * nz).sqrt();
        normals[new_idx] = if nlen > 1e-8 {
            [nx / nlen, ny / nlen, nz / nlen]
        } else {
            [0.0, 0.0, 1.0]
        };

        if !mesh.tangents.is_empty() {
            let tx = tan_acc[0] / count;
            let ty = tan_acc[1] / count;
            let tz = tan_acc[2] / count;
            let tw = tan_acc[3] / count;
            let tlen = (tx * tx + ty * ty + tz * tz).sqrt();
            let sign = if tw >= 0.0 { 1.0 } else { -1.0 };
            tangents[new_idx] = if tlen > 1e-8 {
                [tx / tlen, ty / tlen, tz / tlen, sign]
            } else {
                [1.0, 0.0, 0.0, sign]
            };
        }

        if let Some(ref mut out_cols) = colors {
            out_cols[new_idx] = [
                col_acc[0] / count,
                col_acc[1] / count,
                col_acc[2] / count,
                col_acc[3] / count,
            ];
        }
    }

    // Remap index buffer.
    let new_indices: Vec<u32> = mesh
        .indices
        .iter()
        .map(|&i| old_to_new[i as usize])
        .collect();

    let verts_removed = n - new_n;
    let result = MeshBuffers {
        positions,
        normals,
        tangents,
        uvs,
        indices: new_indices,
        colors,
        has_suit: mesh.has_suit,
    };
    let weld_result = WeldResult {
        verts_removed,
        remap: old_to_new,
    };
    (result, weld_result)
}

/// Weld vertices that share both the same position AND UV (within tolerance).
/// More conservative — only merges vertices that are truly identical in attribute space.
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn weld_by_position_and_uv(mesh: &MeshBuffers, tolerance: f32) -> (MeshBuffers, WeldResult) {
    let n = mesh.positions.len();
    let inv_tol = if tolerance > 0.0 {
        1.0 / tolerance
    } else {
        1.0
    };

    let mut key_map: HashMap<(i64, i64, i64, i64, i64), u32> = HashMap::new();
    let mut old_to_new: Vec<u32> = vec![0; n];
    let mut canonical_members: Vec<Vec<usize>> = Vec::new();

    for (i, p) in mesh.positions.iter().enumerate() {
        let uv = if i < mesh.uvs.len() {
            mesh.uvs[i]
        } else {
            [0.0, 0.0]
        };
        let key = (
            (p[0] * inv_tol).round() as i64,
            (p[1] * inv_tol).round() as i64,
            (p[2] * inv_tol).round() as i64,
            (uv[0] * inv_tol).round() as i64,
            (uv[1] * inv_tol).round() as i64,
        );
        let next_new = canonical_members.len() as u32;
        let new_idx = *key_map.entry(key).or_insert(next_new);
        if new_idx == next_new {
            canonical_members.push(Vec::new());
        }
        canonical_members[new_idx as usize].push(i);
        old_to_new[i] = new_idx;
    }

    let new_n = canonical_members.len();
    let mut positions = vec![[0.0f32; 3]; new_n];
    let mut normals = vec![[0.0f32; 3]; new_n];
    let mut uvs = vec![[0.0f32; 2]; new_n];
    let mut tangents: Vec<[f32; 4]> = if mesh.tangents.is_empty() {
        vec![]
    } else {
        vec![[0.0f32; 4]; new_n]
    };
    let mut colors: Option<Vec<[f32; 4]>> = mesh.colors.as_ref().map(|_| vec![[0.0f32; 4]; new_n]);

    for (new_idx, members) in canonical_members.iter().enumerate() {
        let count = members.len() as f32;
        let mut pos_acc = [0.0f32; 3];
        let mut nrm_acc = [0.0f32; 3];
        let mut uv_acc = [0.0f32; 2];
        let mut tan_acc = [0.0f32; 4];
        let mut col_acc = [0.0f32; 4];

        for &old in members {
            let p = mesh.positions[old];
            pos_acc[0] += p[0];
            pos_acc[1] += p[1];
            pos_acc[2] += p[2];

            if old < mesh.normals.len() {
                let nr = mesh.normals[old];
                nrm_acc[0] += nr[0];
                nrm_acc[1] += nr[1];
                nrm_acc[2] += nr[2];
            }
            if old < mesh.uvs.len() {
                let uv = mesh.uvs[old];
                uv_acc[0] += uv[0];
                uv_acc[1] += uv[1];
            }
            if !mesh.tangents.is_empty() && old < mesh.tangents.len() {
                let t = mesh.tangents[old];
                tan_acc[0] += t[0];
                tan_acc[1] += t[1];
                tan_acc[2] += t[2];
                tan_acc[3] += t[3];
            }
            if let Some(ref cols) = mesh.colors {
                if old < cols.len() {
                    let c = cols[old];
                    col_acc[0] += c[0];
                    col_acc[1] += c[1];
                    col_acc[2] += c[2];
                    col_acc[3] += c[3];
                }
            }
        }

        positions[new_idx] = [pos_acc[0] / count, pos_acc[1] / count, pos_acc[2] / count];
        uvs[new_idx] = [uv_acc[0] / count, uv_acc[1] / count];

        let nx = nrm_acc[0] / count;
        let ny = nrm_acc[1] / count;
        let nz = nrm_acc[2] / count;
        let nlen = (nx * nx + ny * ny + nz * nz).sqrt();
        normals[new_idx] = if nlen > 1e-8 {
            [nx / nlen, ny / nlen, nz / nlen]
        } else {
            [0.0, 0.0, 1.0]
        };

        if !mesh.tangents.is_empty() {
            let tx = tan_acc[0] / count;
            let ty = tan_acc[1] / count;
            let tz = tan_acc[2] / count;
            let tw = tan_acc[3] / count;
            let tlen = (tx * tx + ty * ty + tz * tz).sqrt();
            let sign = if tw >= 0.0 { 1.0 } else { -1.0 };
            tangents[new_idx] = if tlen > 1e-8 {
                [tx / tlen, ty / tlen, tz / tlen, sign]
            } else {
                [1.0, 0.0, 0.0, sign]
            };
        }

        if let Some(ref mut out_cols) = colors {
            out_cols[new_idx] = [
                col_acc[0] / count,
                col_acc[1] / count,
                col_acc[2] / count,
                col_acc[3] / count,
            ];
        }
    }

    let new_indices: Vec<u32> = mesh
        .indices
        .iter()
        .map(|&i| old_to_new[i as usize])
        .collect();

    let verts_removed = n - new_n;
    let result = MeshBuffers {
        positions,
        normals,
        tangents,
        uvs,
        indices: new_indices,
        colors,
        has_suit: mesh.has_suit,
    };
    let weld_result = WeldResult {
        verts_removed,
        remap: old_to_new,
    };
    (result, weld_result)
}

/// Remove unused vertices (vertices not referenced by any index).
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn remove_unused_vertices(mesh: &MeshBuffers) -> (MeshBuffers, WeldResult) {
    let n = mesh.positions.len();
    let mut referenced = vec![false; n];
    for &i in &mesh.indices {
        if (i as usize) < n {
            referenced[i as usize] = true;
        }
    }

    // old_to_new[i] = new compact index (or u32::MAX if unused).
    let mut old_to_new = vec![u32::MAX; n];
    let mut new_positions = Vec::new();
    let mut new_normals = Vec::new();
    let mut new_uvs = Vec::new();
    let mut new_tangents: Vec<[f32; 4]> = Vec::new();
    let mut new_colors: Option<Vec<[f32; 4]>> = mesh.colors.as_ref().map(|_| Vec::new());

    let mut next_new: u32 = 0;
    for i in 0..n {
        if referenced[i] {
            old_to_new[i] = next_new;
            next_new += 1;
            new_positions.push(mesh.positions[i]);
            if i < mesh.normals.len() {
                new_normals.push(mesh.normals[i]);
            }
            if i < mesh.uvs.len() {
                new_uvs.push(mesh.uvs[i]);
            }
            if !mesh.tangents.is_empty() && i < mesh.tangents.len() {
                new_tangents.push(mesh.tangents[i]);
            }
            if let Some(ref cols) = mesh.colors {
                if let Some(ref mut out) = new_colors {
                    if i < cols.len() {
                        out.push(cols[i]);
                    }
                }
            }
        }
    }

    let new_indices: Vec<u32> = mesh
        .indices
        .iter()
        .map(|&i| old_to_new[i as usize])
        .collect();

    let verts_removed = n - new_positions.len();
    let result = MeshBuffers {
        positions: new_positions,
        normals: new_normals,
        tangents: new_tangents,
        uvs: new_uvs,
        indices: new_indices,
        colors: new_colors,
        has_suit: mesh.has_suit,
    };
    let weld_result = WeldResult {
        verts_removed,
        remap: old_to_new,
    };
    (result, weld_result)
}

/// Deduplicate triangles: remove faces where all 3 indices are identical to an existing face.
#[allow(dead_code)]
pub fn deduplicate_faces(mesh: &MeshBuffers) -> MeshBuffers {
    let mut seen: HashSet<(u32, u32, u32)> = HashSet::new();
    let mut new_indices = Vec::with_capacity(mesh.indices.len());

    for tri in mesh.indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0], tri[1], tri[2]);
        // Canonical key: sort indices so that (a,b,c) == (b,c,a) etc.
        let mut sorted = [i0, i1, i2];
        sorted.sort_unstable();
        let key = (sorted[0], sorted[1], sorted[2]);
        if seen.insert(key) {
            new_indices.push(i0);
            new_indices.push(i1);
            new_indices.push(i2);
        }
    }

    MeshBuffers {
        positions: mesh.positions.clone(),
        normals: mesh.normals.clone(),
        tangents: mesh.tangents.clone(),
        uvs: mesh.uvs.clone(),
        indices: new_indices,
        colors: mesh.colors.clone(),
        has_suit: mesh.has_suit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrity::check_index_bounds;

    /// Two verts at the same position (indices 0 and 1), plus two unique verts.
    fn dup_verts_mesh() -> MeshBuffers {
        MeshBuffers {
            positions: vec![
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0]; 4],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 4],
            colors: None,
            indices: vec![0, 2, 3, 1, 2, 3],
            has_suit: true,
        }
    }

    #[test]
    fn weld_duplicate_verts_reduces_count() {
        let mesh = dup_verts_mesh();
        let (welded, result) = weld_by_position(&mesh, 1e-4);
        // Verts 0 and 1 are identical → merged into one → 3 unique verts total.
        assert_eq!(welded.positions.len(), 3);
        assert_eq!(result.verts_removed, 1);
    }

    #[test]
    fn weld_preserves_triangle_integrity() {
        let mesh = dup_verts_mesh();
        let (welded, _) = weld_by_position(&mesh, 1e-4);
        assert!(
            check_index_bounds(&welded),
            "index out of bounds after weld"
        );
    }

    #[test]
    fn remove_unused_removes_orphan() {
        // 3 verts, but only verts 0 and 1 are referenced (vert 2 is orphaned).
        let mesh = MeshBuffers {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [99.0, 99.0, 99.0], // unreferenced orphan
            ],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            tangents: vec![],
            colors: None,
            // Only a degenerate edge; doesn't reference vert 2.
            indices: vec![0, 1, 0],
            has_suit: false,
        };
        let (compacted, result) = remove_unused_vertices(&mesh);
        assert_eq!(compacted.positions.len(), 2);
        assert_eq!(result.verts_removed, 1);
    }

    #[test]
    fn deduplicate_removes_repeated_face() {
        let mesh = MeshBuffers {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            tangents: vec![],
            colors: None,
            // Same face listed twice.
            indices: vec![0, 1, 2, 0, 1, 2],
            has_suit: false,
        };
        let deduped = deduplicate_faces(&mesh);
        assert_eq!(deduped.indices.len(), 3, "duplicate face should be removed");
    }

    #[test]
    fn weld_result_remap_covers_all_old_verts() {
        let mesh = dup_verts_mesh();
        let (_, result) = weld_by_position(&mesh, 1e-4);
        assert_eq!(result.remap.len(), mesh.positions.len());
    }

    #[test]
    fn weld_by_position_and_uv_different_uv_not_merged() {
        // Two verts at identical positions but different UVs → must NOT be merged.
        let mesh = MeshBuffers {
            positions: vec![
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0], // same position as vert 0
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![
                [0.0, 0.0],
                [1.0, 0.0], // different UV from vert 0
                [0.5, 0.0],
                [0.0, 0.5],
            ],
            tangents: vec![],
            colors: None,
            indices: vec![0, 2, 3, 1, 2, 3],
            has_suit: false,
        };
        let (welded, result) = weld_by_position_and_uv(&mesh, 1e-4);
        // Verts 0 and 1 differ in UV → should NOT be merged → still 4 verts.
        assert_eq!(welded.positions.len(), 4);
        assert_eq!(result.verts_removed, 0);
    }
}
