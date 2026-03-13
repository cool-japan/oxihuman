// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Convert quad mesh to triangle mesh.

/// A quad face with four vertex indices.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct QuadFaceV2 {
    pub indices: [u32; 4],
}

/// Result of quad-to-triangle conversion.
#[allow(dead_code)]
pub struct QuadToTriResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub original_quad_count: usize,
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Check if diagonal (i0→i2) is shorter than (i1→i3) for better split.
fn choose_diagonal_02(p: [f32; 3], q: [f32; 3], r: [f32; 3], s: [f32; 3]) -> bool {
    let d02 = sub3(r, p);
    let d13 = sub3(s, q);
    let len02 = d02[0] * d02[0] + d02[1] * d02[1] + d02[2] * d02[2];
    let len13 = d13[0] * d13[0] + d13[1] * d13[1] + d13[2] * d13[2];
    len02 <= len13
}

/// Convert a list of quads to triangles.
/// Each quad (i0, i1, i2, i3) is split into two triangles using the shorter diagonal.
#[allow(dead_code)]
pub fn quads_to_triangles(positions: &[[f32; 3]], quads: &[QuadFaceV2]) -> QuadToTriResult {
    let mut indices = Vec::with_capacity(quads.len() * 6);
    for q in quads {
        let [i0, i1, i2, i3] = q.indices;
        let p0 = positions.get(i0 as usize).copied().unwrap_or([0.0; 3]);
        let p1 = positions.get(i1 as usize).copied().unwrap_or([0.0; 3]);
        let p2 = positions.get(i2 as usize).copied().unwrap_or([0.0; 3]);
        let p3 = positions.get(i3 as usize).copied().unwrap_or([0.0; 3]);
        if choose_diagonal_02(p0, p1, p2, p3) {
            indices.extend_from_slice(&[i0, i1, i2]);
            indices.extend_from_slice(&[i0, i2, i3]);
        } else {
            indices.extend_from_slice(&[i0, i1, i3]);
            indices.extend_from_slice(&[i1, i2, i3]);
        }
    }
    QuadToTriResult {
        positions: positions.to_vec(),
        indices,
        original_quad_count: quads.len(),
    }
}

/// Convert a flat index buffer of quads (each quad = 4 consecutive indices) to triangles.
#[allow(dead_code)]
pub fn quad_buffer_to_triangles(positions: &[[f32; 3]], quad_indices: &[u32]) -> QuadToTriResult {
    let n_quads = quad_indices.len() / 4;
    let quads: Vec<QuadFaceV2> = (0..n_quads)
        .map(|i| QuadFaceV2 {
            indices: [
                quad_indices[i * 4],
                quad_indices[i * 4 + 1],
                quad_indices[i * 4 + 2],
                quad_indices[i * 4 + 3],
            ],
        })
        .collect();
    quads_to_triangles(positions, &quads)
}

/// Count triangles in result.
#[allow(dead_code)]
pub fn result_triangle_count(result: &QuadToTriResult) -> usize {
    result.indices.len() / 3
}

/// Check that indices are in bounds.
#[allow(dead_code)]
pub fn result_indices_valid(result: &QuadToTriResult) -> bool {
    let n = result.positions.len() as u32;
    result.indices.iter().all(|&i| i < n)
}

/// Compute area of a triangle (helper).
#[allow(dead_code)]
pub fn triangle_area_q2t(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> f32 {
    let e1 = sub3(p1, p0);
    let e2 = sub3(p2, p0);
    let c = cross3(e1, e2);
    let len = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
    len * 0.5
}

/// Total surface area of result.
#[allow(dead_code)]
pub fn total_surface_area_q2t(result: &QuadToTriResult) -> f32 {
    let n_tri = result.indices.len() / 3;
    let mut area = 0.0_f32;
    for t in 0..n_tri {
        let p0 = result.positions[result.indices[t * 3] as usize];
        let p1 = result.positions[result.indices[t * 3 + 1] as usize];
        let p2 = result.positions[result.indices[t * 3 + 2] as usize];
        area += triangle_area_q2t(p0, p1, p2);
    }
    area
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_quad_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    fn single_quad() -> Vec<QuadFaceV2> {
        vec![QuadFaceV2 {
            indices: [0, 1, 2, 3],
        }]
    }

    #[test]
    fn single_quad_to_two_tris() {
        let pos = unit_quad_positions();
        let quads = single_quad();
        let result = quads_to_triangles(&pos, &quads);
        assert_eq!(result_triangle_count(&result), 2);
    }

    #[test]
    fn result_indices_valid_check() {
        let pos = unit_quad_positions();
        let quads = single_quad();
        let result = quads_to_triangles(&pos, &quads);
        assert!(result_indices_valid(&result));
    }

    #[test]
    fn quad_buffer_to_triangles_two_tris() {
        let pos = unit_quad_positions();
        let quad_idx: Vec<u32> = vec![0, 1, 2, 3];
        let result = quad_buffer_to_triangles(&pos, &quad_idx);
        assert_eq!(result_triangle_count(&result), 2);
    }

    #[test]
    fn total_area_unit_quad() {
        let pos = unit_quad_positions();
        let quads = single_quad();
        let result = quads_to_triangles(&pos, &quads);
        let area = total_surface_area_q2t(&result);
        assert!((area - 1.0).abs() < 1e-4, "area: {area}");
    }

    #[test]
    fn multiple_quads() {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let quads = vec![
            QuadFaceV2 {
                indices: [0, 1, 2, 3],
            },
            QuadFaceV2 {
                indices: [4, 5, 6, 7],
            },
        ];
        let result = quads_to_triangles(&pos, &quads);
        assert_eq!(result_triangle_count(&result), 4);
    }

    #[test]
    fn original_quad_count_correct() {
        let pos = unit_quad_positions();
        let quads = single_quad();
        let result = quads_to_triangles(&pos, &quads);
        assert_eq!(result.original_quad_count, 1);
    }

    #[test]
    fn triangle_area_unit_right_triangle() {
        let area = triangle_area_q2t([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((area - 0.5).abs() < 1e-5);
    }

    #[test]
    fn empty_quads_empty_result() {
        let pos = unit_quad_positions();
        let result = quads_to_triangles(&pos, &[]);
        assert_eq!(result_triangle_count(&result), 0);
    }

    #[test]
    fn positions_preserved() {
        let pos = unit_quad_positions();
        let quads = single_quad();
        let result = quads_to_triangles(&pos, &quads);
        assert_eq!(result.positions.len(), pos.len());
    }

    #[test]
    fn choose_diagonal_02_shorter() {
        let p0 = [0.0, 0.0, 0.0];
        let p1 = [1.0, 0.0, 0.0];
        let p2 = [1.0, 1.0, 0.0];
        let p3 = [0.0, 10.0, 0.0];
        assert!(choose_diagonal_02(p0, p1, p2, p3));
    }
}
