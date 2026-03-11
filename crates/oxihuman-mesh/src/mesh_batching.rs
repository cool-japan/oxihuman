// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Batch multiple meshes into a single draw call (static batching).

/// A source mesh to batch.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BatchSource {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    /// Optional uniform color (RGBA).
    pub color: Option<[f32; 4]>,
}

/// Result of batching multiple meshes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BatchResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub colors: Vec<[f32; 4]>,
    /// Starting index in `indices` for each source mesh.
    pub submesh_starts: Vec<usize>,
}

/// Combine multiple meshes into one batched mesh.
#[allow(dead_code)]
pub fn batch_meshes(sources: &[BatchSource]) -> BatchResult {
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut colors = Vec::new();
    let mut submesh_starts = Vec::new();

    for src in sources {
        submesh_starts.push(indices.len());
        let offset = positions.len() as u32;
        positions.extend_from_slice(&src.positions);
        for &idx in &src.indices {
            indices.push(idx + offset);
        }
        let col = src.color.unwrap_or([1.0, 1.0, 1.0, 1.0]);
        for _ in 0..src.positions.len() {
            colors.push(col);
        }
    }
    BatchResult {
        positions,
        indices,
        colors,
        submesh_starts,
    }
}

/// Count total vertices in the batch.
#[allow(dead_code)]
pub fn batch_vertex_count(result: &BatchResult) -> usize {
    result.positions.len()
}

/// Count total triangles in the batch.
#[allow(dead_code)]
pub fn batch_triangle_count(result: &BatchResult) -> usize {
    result.indices.len() / 3
}

/// Check that all indices are in bounds.
#[allow(dead_code)]
pub fn batch_indices_valid(result: &BatchResult) -> bool {
    let n = result.positions.len() as u32;
    result.indices.iter().all(|&i| i < n)
}

/// Number of submeshes.
#[allow(dead_code)]
pub fn submesh_count(result: &BatchResult) -> usize {
    result.submesh_starts.len()
}

/// Serialize batch stats to JSON.
#[allow(dead_code)]
pub fn batch_to_json(result: &BatchResult) -> String {
    format!(
        "{{\"vertices\":{},\"triangles\":{},\"submeshes\":{}}}",
        batch_vertex_count(result),
        batch_triangle_count(result),
        submesh_count(result)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src_tri(offset: f32) -> BatchSource {
        BatchSource {
            positions: vec![
                [offset, 0.0, 0.0],
                [offset + 1.0, 0.0, 0.0],
                [offset, 1.0, 0.0],
            ],
            indices: vec![0, 1, 2],
            color: None,
        }
    }

    #[test]
    fn test_batch_two_tris_vertex_count() {
        let r = batch_meshes(&[src_tri(0.0), src_tri(2.0)]);
        assert_eq!(batch_vertex_count(&r), 6);
    }

    #[test]
    fn test_batch_two_tris_triangle_count() {
        let r = batch_meshes(&[src_tri(0.0), src_tri(2.0)]);
        assert_eq!(batch_triangle_count(&r), 2);
    }

    #[test]
    fn test_batch_indices_valid() {
        let r = batch_meshes(&[src_tri(0.0), src_tri(2.0)]);
        assert!(batch_indices_valid(&r));
    }

    #[test]
    fn test_submesh_count() {
        let r = batch_meshes(&[src_tri(0.0), src_tri(2.0)]);
        assert_eq!(submesh_count(&r), 2);
    }

    #[test]
    fn test_batch_empty() {
        let r = batch_meshes(&[]);
        assert_eq!(batch_vertex_count(&r), 0);
        assert_eq!(submesh_count(&r), 0);
    }

    #[test]
    fn test_batch_color_applied() {
        let mut src = src_tri(0.0);
        src.color = Some([1.0, 0.0, 0.0, 1.0]);
        let r = batch_meshes(&[src]);
        assert_eq!(r.colors[0], [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_batch_submesh_starts() {
        let r = batch_meshes(&[src_tri(0.0), src_tri(2.0)]);
        assert_eq!(r.submesh_starts[0], 0);
        assert_eq!(r.submesh_starts[1], 3);
    }

    #[test]
    fn test_batch_to_json() {
        let r = batch_meshes(&[src_tri(0.0)]);
        let j = batch_to_json(&r);
        assert!(j.contains("vertices"));
    }

    #[test]
    fn test_batch_default_color_white() {
        let r = batch_meshes(&[src_tri(0.0)]);
        assert_eq!(r.colors[0], [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_batch_index_offset() {
        let r = batch_meshes(&[src_tri(0.0), src_tri(2.0)]);
        // Second triangle's indices should start at 3
        assert_eq!(r.indices[3], 3);
    }
}
