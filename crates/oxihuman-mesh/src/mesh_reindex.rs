#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Reindex mesh: compact vertex buffer, remove unused vertices.

#[allow(dead_code)]
pub struct ReindexResult {
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
    pub old_to_new: Vec<Option<u32>>,
}

#[allow(dead_code)]
pub fn reindex_mesh(verts: &[[f32; 3]], tris: &[[u32; 3]]) -> ReindexResult {
    let n = verts.len();
    let mut old_to_new: Vec<Option<u32>> = vec![None; n];
    let mut new_verts: Vec<[f32; 3]> = Vec::new();

    // Find used vertices
    for tri in tris {
        for &idx in tri.iter() {
            let i = idx as usize;
            if i < n && old_to_new[i].is_none() {
                old_to_new[i] = Some(new_verts.len() as u32);
                new_verts.push(verts[i]);
            }
        }
    }

    // Remap triangles
    let new_tris: Vec<[u32; 3]> = tris
        .iter()
        .filter_map(|tri| {
            let a = old_to_new.get(tri[0] as usize)?.as_ref().copied()?;
            let b = old_to_new.get(tri[1] as usize)?.as_ref().copied()?;
            let c = old_to_new.get(tri[2] as usize)?.as_ref().copied()?;
            Some([a, b, c])
        })
        .collect();

    ReindexResult {
        verts: new_verts,
        tris: new_tris,
        old_to_new,
    }
}

#[allow(dead_code)]
pub fn used_vertex_count(result: &ReindexResult) -> usize {
    result.verts.len()
}

#[allow(dead_code)]
pub fn removed_vertex_count(orig: usize, result: &ReindexResult) -> usize {
    orig.saturating_sub(result.verts.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reindex_removes_unused_verts() {
        let verts = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [5.0, 5.0, 5.0], // unused
        ];
        let tris = vec![[0u32, 1, 2]];
        let result = reindex_mesh(&verts, &tris);
        assert_eq!(result.verts.len(), 3);
    }

    #[test]
    fn reindex_remaps_triangle_indices() {
        let verts = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [99.0, 99.0, 99.0], // unused
            [0.0, 1.0, 0.0],
        ];
        let tris = vec![[0u32, 1, 3]];
        let result = reindex_mesh(&verts, &tris);
        assert_eq!(result.verts.len(), 3);
        assert_eq!(result.tris.len(), 1);
        // All indices must be < 3
        for &i in &result.tris[0] {
            assert!(i < 3);
        }
    }

    #[test]
    fn reindex_used_vertex_count() {
        let verts = vec![[0.0f32; 3]; 5];
        let tris = vec![[0u32, 1, 2]];
        let result = reindex_mesh(&verts, &tris);
        assert_eq!(used_vertex_count(&result), 3);
    }

    #[test]
    fn reindex_removed_vertex_count() {
        let verts = vec![[0.0f32; 3]; 5];
        let tris = vec![[0u32, 1, 2]];
        let result = reindex_mesh(&verts, &tris);
        assert_eq!(removed_vertex_count(5, &result), 2);
    }

    #[test]
    fn reindex_empty_mesh() {
        let result = reindex_mesh(&[], &[]);
        assert!(result.verts.is_empty());
        assert!(result.tris.is_empty());
    }

    #[test]
    fn reindex_all_used() {
        let verts = vec![[0.0f32; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0u32, 1, 2]];
        let result = reindex_mesh(&verts, &tris);
        assert_eq!(result.verts.len(), 3);
        assert_eq!(removed_vertex_count(3, &result), 0);
    }

    #[test]
    fn old_to_new_length_matches_original() {
        let verts = vec![[0.0f32; 3]; 4];
        let tris = vec![[0u32, 1, 2]];
        let result = reindex_mesh(&verts, &tris);
        assert_eq!(result.old_to_new.len(), 4);
    }

    #[test]
    fn old_to_new_none_for_unused() {
        let verts = vec![[0.0f32; 3]; 4];
        let tris = vec![[0u32, 1, 2]];
        let result = reindex_mesh(&verts, &tris);
        assert!(result.old_to_new[3].is_none());
    }

    #[test]
    fn old_to_new_some_for_used() {
        let verts = vec![[0.0f32; 3]; 3];
        let tris = vec![[0u32, 1, 2]];
        let result = reindex_mesh(&verts, &tris);
        assert!(result.old_to_new[0].is_some());
        assert!(result.old_to_new[1].is_some());
        assert!(result.old_to_new[2].is_some());
    }
}
