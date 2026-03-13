// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Push/pull vertex displacement along normals.

/// Push vertices along their normals by the given distance.
/// Positive distance = push outward, negative = pull inward.
pub fn push_pull(
    positions: &mut [[f32; 3]],
    normals: &[[f32; 3]],
    vertex_mask: &[bool],
    distance: f32,
) {
    let n = positions.len().min(normals.len()).min(vertex_mask.len());
    for i in 0..n {
        if !vertex_mask[i] {
            continue;
        }
        let nx = normals[i][0];
        let ny = normals[i][1];
        let nz = normals[i][2];
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        if len < 1e-8 {
            continue;
        }
        positions[i][0] += nx / len * distance;
        positions[i][1] += ny / len * distance;
        positions[i][2] += nz / len * distance;
    }
}

/// Compute the average displacement magnitude after push/pull (estimation).
pub fn push_pull_magnitude(normals: &[[f32; 3]], vertex_mask: &[bool], distance: f32) -> f32 {
    let n = normals.len().min(vertex_mask.len());
    let selected: usize = (0..n).filter(|&i| vertex_mask[i]).count();
    if selected == 0 {
        return 0.0;
    }
    distance.abs() * selected as f32
}

/// Return count of selected vertices.
pub fn selected_vertex_count(vertex_mask: &[bool]) -> usize {
    vertex_mask.iter().filter(|&&v| v).count()
}

/// Push all vertices outward by distance (no mask).
pub fn push_all(positions: &mut [[f32; 3]], normals: &[[f32; 3]], distance: f32) {
    let mask: Vec<bool> = vec![true; positions.len()];
    push_pull(positions, normals, &mask, distance);
}

/// Clamp positions to a bounding box after push/pull.
pub fn clamp_positions(positions: &mut [[f32; 3]], min: [f32; 3], max: [f32; 3]) {
    for p in positions.iter_mut() {
        p[0] = p[0].clamp(min[0], max[0]);
        p[1] = p[1].clamp(min[1], max[1]);
        p[2] = p[2].clamp(min[2], max[2]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_data() -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
        let positions = vec![[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let normals = vec![[0.0_f32, 1.0, 0.0]; 3];
        (positions, normals)
    }

    #[test]
    fn test_push_along_normal() {
        /* push moves vertex along normal */
        let (mut pos, normals) = flat_data();
        push_pull(&mut pos, &normals, &[true, false, false], 1.0);
        assert!((pos[0][1] - 1.0).abs() < 1e-5);
        assert!((pos[1][1] - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_pull_along_normal() {
        /* negative distance pulls inward */
        let (mut pos, normals) = flat_data();
        push_pull(&mut pos, &normals, &[true, false, false], -1.0);
        assert!((pos[0][1] - (-1.0)).abs() < 1e-5);
    }

    #[test]
    fn test_unmasked_vertex_unchanged() {
        /* unmasked vertices are not moved */
        let (mut pos, normals) = flat_data();
        push_pull(&mut pos, &normals, &[false, false, false], 5.0);
        assert!((pos[0][1] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_selected_vertex_count() {
        /* count of selected vertices is correct */
        assert_eq!(selected_vertex_count(&[true, false, true]), 2);
    }

    #[test]
    fn test_push_all() {
        /* push_all moves all vertices */
        let (mut pos, normals) = flat_data();
        push_all(&mut pos, &normals, 2.0);
        for p in &pos {
            assert!((p[1] - 2.0).abs() < 1e-5);
        }
    }

    #[test]
    fn test_clamp_positions() {
        /* positions are clamped to given bounds */
        let mut pos = vec![[10.0_f32, -5.0, 0.0]];
        clamp_positions(&mut pos, [-1.0, -1.0, -1.0], [5.0, 5.0, 5.0]);
        assert!((pos[0][0] - 5.0).abs() < 1e-6);
        assert!((pos[0][1] - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_push_pull_magnitude() {
        /* magnitude estimation is correct */
        let normals = vec![[0.0_f32, 1.0, 0.0]; 3];
        let mag = push_pull_magnitude(&normals, &[true, true, false], 2.0);
        assert!((mag - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_zero_normal_skipped() {
        /* vertices with zero normals are skipped */
        let mut pos = vec![[0.0_f32, 0.0, 0.0]];
        let normals = vec![[0.0_f32, 0.0, 0.0]];
        push_pull(&mut pos, &normals, &[true], 5.0);
        assert!((pos[0][0] - 0.0).abs() < 1e-6);
    }
}
