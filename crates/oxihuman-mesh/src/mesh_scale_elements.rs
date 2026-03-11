// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Scale individual faces/edges by a local scale factor.

/// Scale selected faces by moving their vertices toward/away from the face centroid.
#[allow(clippy::needless_range_loop)]
pub fn scale_faces(positions: &mut [[f32; 3]], indices: &[u32], face_mask: &[bool], scale: f32) {
    let face_count = indices.len() / 3;
    let flen = face_mask.len().min(face_count);
    for fi in 0..flen {
        if !face_mask[fi] {
            continue;
        }
        let i0 = indices[fi * 3] as usize;
        let i1 = indices[fi * 3 + 1] as usize;
        let i2 = indices[fi * 3 + 2] as usize;
        let cx = (positions[i0][0] + positions[i1][0] + positions[i2][0]) / 3.0;
        let cy = (positions[i0][1] + positions[i1][1] + positions[i2][1]) / 3.0;
        let cz = (positions[i0][2] + positions[i1][2] + positions[i2][2]) / 3.0;
        for vi in [i0, i1, i2] {
            positions[vi][0] = cx + (positions[vi][0] - cx) * scale;
            positions[vi][1] = cy + (positions[vi][1] - cy) * scale;
            positions[vi][2] = cz + (positions[vi][2] - cz) * scale;
        }
    }
}

/// Scale edges: move endpoints toward/away from their midpoint.
pub fn scale_edges(
    positions: &mut [[f32; 3]],
    edges: &[[usize; 2]],
    edge_mask: &[bool],
    scale: f32,
) {
    let len = edges.len().min(edge_mask.len());
    for i in 0..len {
        if !edge_mask[i] {
            continue;
        }
        let [a, b] = edges[i];
        let mx = (positions[a][0] + positions[b][0]) * 0.5;
        let my = (positions[a][1] + positions[b][1]) * 0.5;
        let mz = (positions[a][2] + positions[b][2]) * 0.5;
        positions[a][0] = mx + (positions[a][0] - mx) * scale;
        positions[a][1] = my + (positions[a][1] - my) * scale;
        positions[a][2] = mz + (positions[a][2] - mz) * scale;
        positions[b][0] = mx + (positions[b][0] - mx) * scale;
        positions[b][1] = my + (positions[b][1] - my) * scale;
        positions[b][2] = mz + (positions[b][2] - mz) * scale;
    }
}

/// Compute the centroid of selected faces.
pub fn face_centroid(positions: &[[f32; 3]], indices: &[u32], face_index: usize) -> [f32; 3] {
    let i0 = indices[face_index * 3] as usize;
    let i1 = indices[face_index * 3 + 1] as usize;
    let i2 = indices[face_index * 3 + 2] as usize;
    [
        (positions[i0][0] + positions[i1][0] + positions[i2][0]) / 3.0,
        (positions[i0][1] + positions[i1][1] + positions[i2][1]) / 3.0,
        (positions[i0][2] + positions[i1][2] + positions[i2][2]) / 3.0,
    ]
}

/// Count how many faces are selected.
pub fn selected_face_count(face_mask: &[bool]) -> usize {
    face_mask.iter().filter(|&&v| v).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0_f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let idx = vec![0u32, 1, 2, 0, 2, 3];
        (pos, idx)
    }

    #[test]
    fn test_scale_faces_identity() {
        /* scale 1.0 leaves positions unchanged */
        let (mut pos, idx) = square_mesh();
        let original = pos.clone();
        scale_faces(&mut pos, &idx, &[true, false], 1.0);
        for (p, o) in pos.iter().zip(original.iter()) {
            assert!((p[0] - o[0]).abs() < 1e-6);
        }
    }

    #[test]
    fn test_scale_faces_shrinks() {
        /* scale < 1 shrinks face */
        let (mut pos, idx) = square_mesh();
        scale_faces(&mut pos, &idx, &[true, false], 0.0);
        /* vertices of face 0 should collapse to centroid */
        let cx = (0.0 + 1.0 + 1.0) / 3.0;
        assert!((pos[0][0] - cx).abs() < 1e-5);
    }

    #[test]
    fn test_selected_face_count() {
        /* selected_face_count counts true entries */
        assert_eq!(selected_face_count(&[true, false, true]), 2);
    }

    #[test]
    fn test_face_centroid() {
        /* centroid of first face is correct */
        let (pos, idx) = square_mesh();
        let c = face_centroid(&pos, &idx, 0);
        assert!((c[0] - 2.0 / 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_scale_edges_shrinks() {
        /* scaling edges with 0 collapses to midpoint */
        let mut pos = vec![[0.0_f32, 0.0, 0.0], [2.0, 0.0, 0.0]];
        scale_edges(&mut pos, &[[0, 1]], &[true], 0.0);
        assert!((pos[0][0] - 1.0).abs() < 1e-5);
        assert!((pos[1][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_scale_edges_identity() {
        /* scale 1.0 leaves edges unchanged */
        let mut pos = vec![[0.0_f32, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let orig = pos.clone();
        scale_edges(&mut pos, &[[0, 1]], &[true], 1.0);
        assert!((pos[0][0] - orig[0][0]).abs() < 1e-6);
    }

    #[test]
    fn test_scale_unselected_face_unchanged() {
        /* unselected face is not modified */
        let (mut pos, idx) = square_mesh();
        let p3_before = pos[3];
        scale_faces(&mut pos, &idx, &[false, false], 0.0);
        assert!((pos[3][0] - p3_before[0]).abs() < 1e-6);
    }
}
