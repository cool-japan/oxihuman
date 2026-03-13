// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Rip/disconnect vertices: duplicate a vertex to create a gap.

/// Result of ripping a vertex.
#[derive(Debug, Clone)]
pub struct RipResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    /// Number of new (duplicate) vertices created.
    pub new_vertex_count: usize,
}

/// Rip a vertex by duplicating it for each face that uses it except the first.
/// Returns the updated mesh.
pub fn rip_vertex(positions: &[[f32; 3]], indices: &[u32], vertex: u32) -> RipResult {
    let mut out_pos = positions.to_vec();
    let mut out_idx = indices.to_vec();
    let mut new_count = 0;
    let mut first = true;
    for tri in out_idx.chunks_mut(3) {
        if tri.contains(&vertex) {
            if first {
                first = false;
                /* keep original vertex for first face */
            } else {
                /* duplicate vertex for subsequent faces */
                let new_idx = out_pos.len() as u32;
                out_pos.push(positions[vertex as usize]);
                for v in tri.iter_mut() {
                    if *v == vertex {
                        *v = new_idx;
                    }
                }
                new_count += 1;
            }
        }
    }
    RipResult {
        positions: out_pos,
        indices: out_idx,
        new_vertex_count: new_count,
    }
}

/// Number of new vertices created by ripping.
pub fn rip_vertex_count_new(result: &RipResult) -> usize {
    result.new_vertex_count
}

/// Number of face triples in the result.
pub fn rip_face_count_new(result: &RipResult) -> usize {
    result.indices.len() / 3
}

/// Length of the seam (number of ripped vertices × 1.0 unit stub).
pub fn rip_seam_length(result: &RipResult) -> f32 {
    result.new_vertex_count as f32
}

/// Returns true if at least one vertex was ripped (gap created).
pub fn rip_is_valid_gap(result: &RipResult) -> bool {
    result.new_vertex_count > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_tri_fan() -> (Vec<[f32; 3]>, Vec<u32>) {
        /* two triangles sharing vertex 0 */
        let p = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let i = vec![0u32, 1, 2, 0, 2, 3];
        (p, i)
    }

    #[test]
    fn test_rip_creates_new_vertex() {
        let (p, i) = two_tri_fan();
        let r = rip_vertex(&p, &i, 0);
        assert_eq!(r.new_vertex_count, 1);
    }

    #[test]
    fn test_rip_vertex_count_new() {
        let (p, i) = two_tri_fan();
        let r = rip_vertex(&p, &i, 0);
        assert_eq!(rip_vertex_count_new(&r), 1);
    }

    #[test]
    fn test_rip_face_count_unchanged() {
        let (p, i) = two_tri_fan();
        let r = rip_vertex(&p, &i, 0);
        assert_eq!(rip_face_count_new(&r), 2);
    }

    #[test]
    fn test_rip_is_valid_gap() {
        let (p, i) = two_tri_fan();
        let r = rip_vertex(&p, &i, 0);
        assert!(rip_is_valid_gap(&r));
    }

    #[test]
    fn test_rip_no_gap_isolated_vertex() {
        /* vertex 3 only used once → no duplicate needed */
        let (p, i) = two_tri_fan();
        let r = rip_vertex(&p, &i, 3);
        assert!(!rip_is_valid_gap(&r));
    }
}
