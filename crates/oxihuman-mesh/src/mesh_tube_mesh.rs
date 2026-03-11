// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Procedural tube/pipe mesh generation along a polyline path.

use std::f32::consts::TAU;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TubeMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub segments: usize,
    pub rings: usize,
}

#[allow(dead_code)]
pub fn generate_tube(path: &[[f32; 3]], radius: f32, segments: usize) -> TubeMesh {
    let rings = path.len();
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for (ri, &center) in path.iter().enumerate() {
        let v = ri as f32 / (rings.saturating_sub(1).max(1)) as f32;
        for si in 0..=segments {
            let angle = TAU * si as f32 / segments as f32;
            let (s, c) = angle.sin_cos();
            let nx = c;
            let nz = s;
            positions.push([center[0] + radius * nx, center[1], center[2] + radius * nz]);
            normals.push([nx, 0.0, nz]);
            uvs.push([si as f32 / segments as f32, v]);
        }
    }

    let cols = segments + 1;
    for ri in 0..rings.saturating_sub(1) {
        for si in 0..segments {
            let a = (ri * cols + si) as u32;
            let b = (ri * cols + si + 1) as u32;
            let c = ((ri + 1) * cols + si) as u32;
            let d = ((ri + 1) * cols + si + 1) as u32;
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }

    TubeMesh {
        positions,
        normals,
        uvs,
        indices,
        segments,
        rings,
    }
}

#[allow(dead_code)]
pub fn tube_vertex_count(tube: &TubeMesh) -> usize {
    tube.positions.len()
}

#[allow(dead_code)]
pub fn tube_face_count(tube: &TubeMesh) -> usize {
    tube.indices.len() / 3
}

#[allow(dead_code)]
pub fn tube_circumference(radius: f32) -> f32 {
    TAU * radius
}

#[allow(dead_code)]
pub fn tube_surface_area(path: &[[f32; 3]], radius: f32) -> f32 {
    let mut length = 0.0_f32;
    for pair in path.windows(2) {
        let d = [
            pair[1][0] - pair[0][0],
            pair[1][1] - pair[0][1],
            pair[1][2] - pair[0][2],
        ];
        length += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    }
    TAU * radius * length
}

#[allow(dead_code)]
pub fn tube_to_json(tube: &TubeMesh) -> String {
    format!(
        "{{\"vertices\":{},\"faces\":{},\"segments\":{},\"rings\":{}}}",
        tube_vertex_count(tube),
        tube_face_count(tube),
        tube.segments,
        tube.rings,
    )
}

#[allow(dead_code)]
pub fn tube_indices_valid(tube: &TubeMesh) -> bool {
    let n = tube.positions.len() as u32;
    tube.indices.iter().all(|&i| i < n)
}

#[allow(dead_code)]
pub fn tube_normals_unit(tube: &TubeMesh) -> bool {
    tube.normals.iter().all(|n| {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        (len - 1.0).abs() < 1e-4
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_path() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 2.0, 0.0]]
    }

    #[test]
    fn test_generate_tube_vertex_count() {
        let tube = generate_tube(&line_path(), 0.5, 8);
        assert_eq!(tube_vertex_count(&tube), 3 * 9);
    }

    #[test]
    fn test_tube_face_count() {
        let tube = generate_tube(&line_path(), 0.5, 8);
        assert!(tube_face_count(&tube) > 0);
    }

    #[test]
    fn test_indices_valid() {
        let tube = generate_tube(&line_path(), 0.5, 8);
        assert!(tube_indices_valid(&tube));
    }

    #[test]
    fn test_circumference() {
        let c = tube_circumference(1.0);
        assert!((c - std::f32::consts::TAU).abs() < 1e-5);
    }

    #[test]
    fn test_surface_area_positive() {
        let path = line_path();
        let area = tube_surface_area(&path, 0.5);
        assert!(area > 0.0);
    }

    #[test]
    fn test_normals_unit() {
        let tube = generate_tube(&line_path(), 0.5, 8);
        assert!(tube_normals_unit(&tube));
    }

    #[test]
    fn test_json_output() {
        let tube = generate_tube(&line_path(), 0.5, 8);
        let j = tube_to_json(&tube);
        assert!(j.contains("vertices"));
    }

    #[test]
    fn test_empty_path() {
        let tube = generate_tube(&[], 0.5, 8);
        assert_eq!(tube_vertex_count(&tube), 0);
    }

    #[test]
    fn test_single_ring() {
        let tube = generate_tube(&[[0.0, 0.0, 0.0]], 1.0, 4);
        assert_eq!(tube.rings, 1);
    }

    #[test]
    fn test_segments_stored() {
        let tube = generate_tube(&line_path(), 1.0, 12);
        assert_eq!(tube.segments, 12);
    }
}
