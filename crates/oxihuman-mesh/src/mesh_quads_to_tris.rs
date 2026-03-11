// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Convert quads to triangles.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuadSplitMode {
    Diagonal01, /* split v0-v2 */
    Diagonal13, /* split v1-v3 */
    Shortest,   /* pick shorter diagonal */
    BestNormal, /* pick split that better preserves normals */
}

#[derive(Debug, Clone)]
pub struct QuadsToTrisResult {
    pub indices: Vec<u32>,
    pub triangle_count: usize,
}

fn dist2_3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

/// Split a single quad into two triangles.
pub fn split_quad(quad: [u32; 4], mode: QuadSplitMode, positions: &[[f32; 3]]) -> [u32; 6] {
    match mode {
        QuadSplitMode::Diagonal01 => [quad[0], quad[1], quad[2], quad[0], quad[2], quad[3]],
        QuadSplitMode::Diagonal13 => [quad[0], quad[1], quad[3], quad[1], quad[2], quad[3]],
        QuadSplitMode::Shortest => {
            let d02 = dist2_3(positions[quad[0] as usize], positions[quad[2] as usize]);
            let d13 = dist2_3(positions[quad[1] as usize], positions[quad[3] as usize]);
            if d02 <= d13 {
                [quad[0], quad[1], quad[2], quad[0], quad[2], quad[3]]
            } else {
                [quad[0], quad[1], quad[3], quad[1], quad[2], quad[3]]
            }
        }
        QuadSplitMode::BestNormal => {
            /* same as Shortest for this stub */
            let d02 = dist2_3(positions[quad[0] as usize], positions[quad[2] as usize]);
            let d13 = dist2_3(positions[quad[1] as usize], positions[quad[3] as usize]);
            if d02 <= d13 {
                [quad[0], quad[1], quad[2], quad[0], quad[2], quad[3]]
            } else {
                [quad[0], quad[1], quad[3], quad[1], quad[2], quad[3]]
            }
        }
    }
}

/// Convert a list of quads to triangles.
pub fn quads_to_tris(
    quads: &[[u32; 4]],
    positions: &[[f32; 3]],
    mode: QuadSplitMode,
) -> QuadsToTrisResult {
    let mut indices = Vec::with_capacity(quads.len() * 6);
    for &q in quads {
        let tris = split_quad(q, mode, positions);
        indices.extend_from_slice(&tris);
    }
    let triangle_count = quads.len() * 2;
    QuadsToTrisResult {
        indices,
        triangle_count,
    }
}

/// Convert a mixed index buffer (quads are encoded as groups of 4) to triangles.
pub fn mixed_to_tris(
    positions: &[[f32; 3]],
    faces: &[Vec<u32>],
    mode: QuadSplitMode,
) -> QuadsToTrisResult {
    let mut indices = Vec::new();
    for face in faces {
        match face.len() {
            3 => indices.extend_from_slice(face),
            4 => {
                let q = [face[0], face[1], face[2], face[3]];
                let tris = split_quad(q, mode, positions);
                indices.extend_from_slice(&tris);
            }
            n if n > 4 => {
                /* fan triangulate */
                for i in 1..n - 1 {
                    indices.extend_from_slice(&[face[0], face[i], face[i + 1]]);
                }
            }
            _ => {}
        }
    }
    let triangle_count = indices.len() / 3;
    QuadsToTrisResult {
        indices,
        triangle_count,
    }
}

pub fn triangle_count_from_quads(quad_count: usize) -> usize {
    quad_count * 2
}

pub fn validate_quad_input(quads: &[[u32; 4]], vertex_count: usize) -> bool {
    quads
        .iter()
        .all(|q| q.iter().all(|&v| (v as usize) < vertex_count))
}

pub fn default_split_mode() -> QuadSplitMode {
    QuadSplitMode::Shortest
}

pub fn tri_index_count(quad_count: usize) -> usize {
    quad_count * 6
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square_pos() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    #[test]
    fn test_split_quad_diagonal01() {
        let pos = square_pos();
        let tris = split_quad([0, 1, 2, 3], QuadSplitMode::Diagonal01, &pos);
        assert_eq!(tris.len(), 6);
        assert_eq!(tris[0], 0);
    }

    #[test]
    fn test_quads_to_tris_count() {
        let pos = square_pos();
        let quads = vec![[0u32, 1, 2, 3]];
        let res = quads_to_tris(&quads, &pos, QuadSplitMode::Shortest);
        assert_eq!(res.triangle_count, 2);
        assert_eq!(res.indices.len(), 6);
    }

    #[test]
    fn test_triangle_count_from_quads() {
        assert_eq!(triangle_count_from_quads(3), 6);
    }

    #[test]
    fn test_validate_quad_input() {
        let quads = vec![[0u32, 1, 2, 3]];
        assert!(validate_quad_input(&quads, 4));
        assert!(!validate_quad_input(&quads, 3));
    }

    #[test]
    fn test_default_split_mode() {
        assert_eq!(default_split_mode(), QuadSplitMode::Shortest);
    }

    #[test]
    fn test_tri_index_count() {
        assert_eq!(tri_index_count(4), 24);
    }

    #[test]
    fn test_mixed_to_tris_quad() {
        let pos = square_pos();
        let faces = vec![vec![0u32, 1, 2, 3]];
        let res = mixed_to_tris(&pos, &faces, QuadSplitMode::Diagonal01);
        assert_eq!(res.triangle_count, 2);
    }

    #[test]
    fn test_mixed_to_tris_tri() {
        let pos = square_pos();
        let faces = vec![vec![0u32, 1, 2]];
        let res = mixed_to_tris(&pos, &faces, QuadSplitMode::Diagonal01);
        assert_eq!(res.triangle_count, 1);
    }
}
