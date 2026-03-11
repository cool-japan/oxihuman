// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Compute and use the volume of a cage mesh.

/// Cage volume result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CageVolumeResult {
    pub signed_volume: f32,
    pub abs_volume: f32,
    pub face_count: usize,
}

/// Compute signed volume contribution of one triangle (divergence theorem).
#[allow(dead_code)]
pub fn signed_tet_volume(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> f32 {
    let a = v0;
    let b = v1;
    let c = v2;
    (a[0] * (b[1] * c[2] - b[2] * c[1])
        + a[1] * (b[2] * c[0] - b[0] * c[2])
        + a[2] * (b[0] * c[1] - b[1] * c[0]))
        / 6.0
}

/// Compute the signed volume of a closed triangle mesh.
#[allow(dead_code)]
pub fn compute_cage_volume(positions: &[[f32; 3]], indices: &[u32]) -> CageVolumeResult {
    let tri_count = indices.len() / 3;
    let mut vol = 0.0f32;
    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        if i0 < positions.len() && i1 < positions.len() && i2 < positions.len() {
            vol += signed_tet_volume(positions[i0], positions[i1], positions[i2]);
        }
    }
    CageVolumeResult {
        signed_volume: vol,
        abs_volume: vol.abs(),
        face_count: tri_count,
    }
}

/// Check if the computed volume indicates an outward-facing mesh (positive volume).
#[allow(dead_code)]
pub fn is_outward_cage(result: &CageVolumeResult) -> bool {
    result.signed_volume > 0.0
}

/// Scale the cage volume by a uniform scale factor.
#[allow(dead_code)]
pub fn scaled_cage_volume(abs_vol: f32, scale: f32) -> f32 {
    abs_vol * scale * scale * scale
}

/// Estimate the equivalent sphere radius from a volume.
#[allow(dead_code)]
pub fn equivalent_sphere_radius(vol: f32) -> f32 {
    use std::f32::consts::PI;
    (3.0 * vol.abs() / (4.0 * PI)).powf(1.0 / 3.0)
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn cage_volume_to_json(result: &CageVolumeResult) -> String {
    format!(
        "{{\"signed_volume\":{:.6},\"abs_volume\":{:.6},\"face_count\":{}}}",
        result.signed_volume, result.abs_volume, result.face_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn unit_tetrahedron() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let idx = vec![0u32, 2, 1, 0, 1, 3, 0, 3, 2, 1, 2, 3];
        (pos, idx)
    }

    #[test]
    fn test_signed_tet_volume_positive() {
        let v = signed_tet_volume([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(v.is_finite());
    }

    #[test]
    fn test_compute_cage_volume_nonempty() {
        let (pos, idx) = unit_tetrahedron();
        let r = compute_cage_volume(&pos, &idx);
        assert!(r.abs_volume > 0.0);
    }

    #[test]
    fn test_face_count() {
        let (pos, idx) = unit_tetrahedron();
        let r = compute_cage_volume(&pos, &idx);
        assert_eq!(r.face_count, 4);
    }

    #[test]
    fn test_is_outward_cage() {
        let r = CageVolumeResult {
            signed_volume: 0.1,
            abs_volume: 0.1,
            face_count: 4,
        };
        assert!(is_outward_cage(&r));
    }

    #[test]
    fn test_is_inward_cage() {
        let r = CageVolumeResult {
            signed_volume: -0.1,
            abs_volume: 0.1,
            face_count: 4,
        };
        assert!(!is_outward_cage(&r));
    }

    #[test]
    fn test_scaled_cage_volume() {
        let v = scaled_cage_volume(1.0, 2.0);
        assert!((v - 8.0).abs() < 1e-5);
    }

    #[test]
    fn test_equivalent_sphere_radius_positive() {
        let r = equivalent_sphere_radius(PI * 4.0 / 3.0);
        assert!((r - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_cage_volume_to_json() {
        let r = CageVolumeResult {
            signed_volume: 1.0,
            abs_volume: 1.0,
            face_count: 4,
        };
        let j = cage_volume_to_json(&r);
        assert!(j.contains("abs_volume"));
    }

    #[test]
    fn test_empty_mesh_zero_volume() {
        let r = compute_cage_volume(&[], &[]);
        assert!(r.abs_volume.abs() < 1e-9);
    }
}
