// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Mirror and stitch a half-mesh across a symmetry plane.
#[allow(dead_code)]
pub struct MirrorStitchResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub seam_vertex_count: usize,
}

#[allow(dead_code)]
pub enum MirrorAxis {
    X,
    Y,
    Z,
}

fn mirror_pos(p: [f32; 3], axis: &MirrorAxis) -> [f32; 3] {
    match axis {
        MirrorAxis::X => [-p[0], p[1], p[2]],
        MirrorAxis::Y => [p[0], -p[1], p[2]],
        MirrorAxis::Z => [p[0], p[1], -p[2]],
    }
}

fn axis_value(p: [f32; 3], axis: &MirrorAxis) -> f32 {
    match axis {
        MirrorAxis::X => p[0],
        MirrorAxis::Y => p[1],
        MirrorAxis::Z => p[2],
    }
}

/// Mirror positions across axis plane and stitch seam vertices.
#[allow(dead_code)]
pub fn mirror_stitch(
    positions: &[[f32; 3]],
    indices: &[u32],
    axis: &MirrorAxis,
    seam_threshold: f32,
) -> MirrorStitchResult {
    if positions.is_empty() {
        return MirrorStitchResult {
            positions: vec![],
            indices: vec![],
            seam_vertex_count: 0,
        };
    }

    let n = positions.len();
    let mut new_positions: Vec<[f32; 3]> = positions.to_vec();
    let mut remap = vec![0u32; n];

    // Find seam vertices (on the mirror plane)
    let mut seam_count = 0;
    for (i, &p) in positions.iter().enumerate() {
        if axis_value(p, axis).abs() <= seam_threshold {
            remap[i] = i as u32; // stays on seam
            seam_count += 1;
        } else {
            let mirrored = mirror_pos(p, axis);
            remap[i] = new_positions.len() as u32;
            new_positions.push(mirrored);
        }
    }

    let mut new_indices = indices.to_vec();
    // Add mirrored faces with reversed winding
    for chunk in indices.chunks(3) {
        if chunk.len() == 3 {
            let ma = remap[chunk[0] as usize];
            let mb = remap[chunk[1] as usize];
            let mc = remap[chunk[2] as usize];
            new_indices.extend_from_slice(&[mc, mb, ma]);
        }
    }

    MirrorStitchResult {
        positions: new_positions,
        indices: new_indices,
        seam_vertex_count: seam_count,
    }
}

#[allow(dead_code)]
pub fn stitch_result_vertex_count(r: &MirrorStitchResult) -> usize {
    r.positions.len()
}

#[allow(dead_code)]
pub fn stitch_result_face_count(r: &MirrorStitchResult) -> usize {
    r.indices.len() / 3
}

#[allow(dead_code)]
pub fn validate_stitch_result(r: &MirrorStitchResult) -> bool {
    for &idx in &r.indices {
        if idx as usize >= r.positions.len() {
            return false;
        }
    }
    true
}

#[allow(dead_code)]
pub fn stitch_bounds(r: &MirrorStitchResult) -> ([f32; 3], [f32; 3]) {
    if r.positions.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = r.positions[0];
    let mut mx = r.positions[0];
    for p in &r.positions {
        for k in 0..3 {
            if p[k] < mn[k] {
                mn[k] = p[k];
            }
            if p[k] > mx[k] {
                mx[k] = p[k];
            }
        }
    }
    (mn, mx)
}

#[allow(dead_code)]
pub fn stitch_result_to_json(r: &MirrorStitchResult) -> String {
    format!(
        "{{\"vertex_count\":{},\"face_count\":{},\"seam_vertex_count\":{}}}",
        r.positions.len(),
        r.indices.len() / 3,
        r.seam_vertex_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn half_quad() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 0, 2, 3];
        (pos, idx)
    }

    #[test]
    fn test_mirrored_face_count_doubled() {
        let (pos, idx) = half_quad();
        let r = mirror_stitch(&pos, &idx, &MirrorAxis::X, 0.001);
        assert_eq!(stitch_result_face_count(&r), 4);
    }

    #[test]
    fn test_seam_vertices_detected() {
        let (pos, idx) = half_quad();
        let r = mirror_stitch(&pos, &idx, &MirrorAxis::X, 0.001);
        assert_eq!(r.seam_vertex_count, 2);
    }

    #[test]
    fn test_validate_stitch() {
        let (pos, idx) = half_quad();
        let r = mirror_stitch(&pos, &idx, &MirrorAxis::X, 0.001);
        assert!(validate_stitch_result(&r));
    }

    #[test]
    fn test_empty_mesh() {
        let r = mirror_stitch(&[], &[], &MirrorAxis::X, 0.001);
        assert_eq!(r.positions.len(), 0);
    }

    #[test]
    fn test_mirror_y_axis() {
        let (pos, idx) = half_quad();
        let r = mirror_stitch(&pos, &idx, &MirrorAxis::Y, 0.001);
        assert!(stitch_result_vertex_count(&r) > 0);
    }

    #[test]
    fn test_mirror_z_axis() {
        let (pos, idx) = half_quad();
        let r = mirror_stitch(&pos, &idx, &MirrorAxis::Z, 0.001);
        assert!(stitch_result_vertex_count(&r) > 0);
    }

    #[test]
    fn test_bounds_symmetric_x() {
        let (pos, idx) = half_quad();
        let r = mirror_stitch(&pos, &idx, &MirrorAxis::X, 0.001);
        let (mn, mx) = stitch_bounds(&r);
        assert!((mn[0] + mx[0]).abs() < 1e-4);
    }

    #[test]
    fn test_to_json() {
        let (pos, idx) = half_quad();
        let r = mirror_stitch(&pos, &idx, &MirrorAxis::X, 0.001);
        let j = stitch_result_to_json(&r);
        assert!(j.contains("vertex_count"));
        assert!(j.contains("seam_vertex_count"));
    }

    #[test]
    fn test_original_faces_preserved() {
        let (pos, idx) = half_quad();
        let r = mirror_stitch(&pos, &idx, &MirrorAxis::X, 0.001);
        // first 6 indices should be from original
        assert_eq!(&r.indices[..6], &idx[..]);
    }

    #[test]
    fn test_all_seam_vertices() {
        // All vertices on seam plane
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let r = mirror_stitch(&pos, &idx, &MirrorAxis::X, 2.0);
        assert_eq!(r.seam_vertex_count, 3);
    }
}
