// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Signed volume measurement for closed meshes.

#[allow(dead_code)]
pub fn vm_tetrahedron_signed_volume(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let cross = [
        b[1] * c[2] - b[2] * c[1],
        b[2] * c[0] - b[0] * c[2],
        b[0] * c[1] - b[1] * c[0],
    ];
    (a[0] * cross[0] + a[1] * cross[1] + a[2] * cross[2]) / 6.0
}

#[allow(dead_code)]
pub fn vm_mesh_volume(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> f32 {
    let mut vol = 0.0f32;
    for tri in indices {
        let a = tri[0] as usize;
        let b = tri[1] as usize;
        let c = tri[2] as usize;
        if a < positions.len() && b < positions.len() && c < positions.len() {
            vol += vm_tetrahedron_signed_volume(positions[a], positions[b], positions[c]);
        }
    }
    vol.abs()
}

#[allow(dead_code)]
pub fn vm_is_closed_estimate(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> bool {
    vm_mesh_volume(positions, indices) > 0.0
}

#[allow(dead_code)]
pub fn vm_centroid(positions: &[[f32; 3]], _indices: &[[u32; 3]]) -> [f32; 3] {
    let n = positions.len();
    if n == 0 { return [0.0; 3]; }
    let mut sum = [0f32; 3];
    for p in positions {
        sum[0] += p[0]; sum[1] += p[1]; sum[2] += p[2];
    }
    [sum[0] / n as f32, sum[1] / n as f32, sum[2] / n as f32]
}

#[allow(dead_code)]
pub fn vm_bounding_box_volume(positions: &[[f32; 3]]) -> f32 {
    if positions.is_empty() { return 0.0; }
    let mut mn = positions[0];
    let mut mx = positions[0];
    for p in positions {
        for k in 0..3 {
            if p[k] < mn[k] { mn[k] = p[k]; }
            if p[k] > mx[k] { mx[k] = p[k]; }
        }
    }
    (mx[0] - mn[0]) * (mx[1] - mn[1]) * (mx[2] - mn[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tet_volume_unit() {
        let vol = vm_tetrahedron_signed_volume(
            [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0],
        );
        assert!((vol.abs() - 1.0 / 6.0).abs() < 1e-4);
    }

    #[test]
    fn test_tet_volume_zero_for_flat() {
        let vol = vm_tetrahedron_signed_volume(
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0],
        );
        assert!(vol.abs() < 1e-10);
    }

    #[test]
    fn test_mesh_volume_empty() {
        assert_eq!(vm_mesh_volume(&[], &[]), 0.0);
    }

    #[test]
    fn test_is_closed_estimate_flat_false() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let indices = vec![[0u32, 1, 2]];
        assert!(!vm_is_closed_estimate(&positions, &indices));
    }

    #[test]
    fn test_centroid_simple() {
        let positions = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]];
        let c = vm_centroid(&positions, &[]);
        assert!((c[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_centroid_empty() {
        assert_eq!(vm_centroid(&[], &[]), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_bounding_box_volume_unit_cube() {
        let positions = vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0],
            [0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [0.0, 1.0, 1.0], [1.0, 1.0, 1.0],
        ];
        assert!((vm_bounding_box_volume(&positions) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_bounding_box_volume_empty() {
        assert_eq!(vm_bounding_box_volume(&[]), 0.0);
    }
}
