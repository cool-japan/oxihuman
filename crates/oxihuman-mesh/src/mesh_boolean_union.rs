// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Mesh boolean union (CSG) stub.

/// Configuration for mesh boolean union operations.
#[derive(Debug, Clone)]
pub struct BooleanUnionConfig {
    pub tolerance: f32,
    pub max_iterations: usize,
    pub weld_threshold: f32,
}

impl Default for BooleanUnionConfig {
    fn default() -> Self {
        Self {
            tolerance: 1e-6,
            max_iterations: 128,
            weld_threshold: 1e-5,
        }
    }
}

/// Result of a mesh boolean union operation.
#[derive(Debug, Clone, Default)]
pub struct BooleanUnionResult {
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
    pub vertex_count: usize,
    pub triangle_count: usize,
}

impl BooleanUnionResult {
    pub fn new(vertices: Vec<[f32; 3]>, triangles: Vec<[u32; 3]>) -> Self {
        let vertex_count = vertices.len();
        let triangle_count = triangles.len();
        Self {
            vertices,
            triangles,
            vertex_count,
            triangle_count,
        }
    }
}

/// Compute the boolean union of two triangle meshes.
pub fn mesh_boolean_union(
    verts_a: &[[f32; 3]],
    tris_a: &[[u32; 3]],
    verts_b: &[[f32; 3]],
    tris_b: &[[u32; 3]],
    _cfg: &BooleanUnionConfig,
) -> BooleanUnionResult {
    /* Stub: concatenate both meshes with offset indices as a placeholder */
    let offset = verts_a.len() as u32;
    let mut vertices = verts_a.to_vec();
    vertices.extend_from_slice(verts_b);
    let mut triangles = tris_a.to_vec();
    for tri in tris_b {
        triangles.push([tri[0] + offset, tri[1] + offset, tri[2] + offset]);
    }
    BooleanUnionResult::new(vertices, triangles)
}

/// Validate that a mesh is suitable for boolean operations.
pub fn validate_mesh_for_boolean(verts: &[[f32; 3]], tris: &[[u32; 3]]) -> bool {
    /* Check basic integrity: all indices in range */
    let n = verts.len() as u32;
    tris.iter().all(|t| t[0] < n && t[1] < n && t[2] < n)
}

/// Compute vertex count after union (stub estimate).
pub fn estimate_union_vertex_count(a_count: usize, b_count: usize) -> usize {
    a_count + b_count
}

/// Check if two meshes have overlapping bounding boxes.
pub fn meshes_aabb_overlap(verts_a: &[[f32; 3]], verts_b: &[[f32; 3]]) -> bool {
    /* Compute axis-aligned bounding boxes and test overlap */
    if verts_a.is_empty() || verts_b.is_empty() {
        return false;
    }
    let (mut amin, mut amax) = ([f32::MAX; 3], [f32::MIN; 3]);
    for v in verts_a {
        for k in 0..3 {
            if v[k] < amin[k] {
                amin[k] = v[k];
            }
            if v[k] > amax[k] {
                amax[k] = v[k];
            }
        }
    }
    let (mut bmin, mut bmax) = ([f32::MAX; 3], [f32::MIN; 3]);
    for v in verts_b {
        for k in 0..3 {
            if v[k] < bmin[k] {
                bmin[k] = v[k];
            }
            if v[k] > bmax[k] {
                bmax[k] = v[k];
            }
        }
    }
    (0..3).all(|k| amin[k] <= bmax[k] && bmin[k] <= amax[k])
}

/// Merge duplicate vertices within threshold.
pub fn weld_union_vertices(vertices: &[[f32; 3]], threshold: f32) -> Vec<[f32; 3]> {
    /* Stub: return unique vertices within threshold distance */
    let mut result: Vec<[f32; 3]> = Vec::new();
    'outer: for &v in vertices {
        for existing in &result {
            let dx = v[0] - existing[0];
            let dy = v[1] - existing[1];
            let dz = v[2] - existing[2];
            if (dx * dx + dy * dy + dz * dz).sqrt() < threshold {
                continue 'outer;
            }
        }
        result.push(v);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = BooleanUnionConfig::default();
        assert!(cfg.tolerance > 0.0 /* tolerance must be positive */);
        assert!(cfg.max_iterations > 0 /* iterations must be positive */);
    }

    #[test]
    fn test_union_empty_meshes() {
        let cfg = BooleanUnionConfig::default();
        let result = mesh_boolean_union(&[], &[], &[], &[], &cfg);
        assert_eq!(
            result.vertex_count,
            0 /* empty union has no vertices */
        );
        assert_eq!(
            result.triangle_count,
            0 /* empty union has no triangles */
        );
    }

    #[test]
    fn test_union_concatenates_vertices() {
        let cfg = BooleanUnionConfig::default();
        let va = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let ta = vec![[0u32, 1, 2]];
        let vb = vec![[2.0f32, 0.0, 0.0], [3.0, 0.0, 0.0], [2.0, 1.0, 0.0]];
        let tb = vec![[0u32, 1, 2]];
        let result = mesh_boolean_union(&va, &ta, &vb, &tb, &cfg);
        assert_eq!(result.vertex_count, 6 /* 3 + 3 vertices */);
        assert_eq!(result.triangle_count, 2 /* 1 + 1 triangles */);
    }

    #[test]
    fn test_union_index_offset() {
        let cfg = BooleanUnionConfig::default();
        let va = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let ta = vec![[0u32, 1, 2]];
        let vb = vec![[5.0f32, 0.0, 0.0], [6.0, 0.0, 0.0], [5.0, 1.0, 0.0]];
        let tb = vec![[0u32, 1, 2]];
        let result = mesh_boolean_union(&va, &ta, &vb, &tb, &cfg);
        /* Second triangle indices must be offset by 3 */
        assert_eq!(result.triangles[1], [3, 4, 5]);
    }

    #[test]
    fn test_validate_mesh_valid() {
        let verts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0u32, 1, 2]];
        assert!(validate_mesh_for_boolean(&verts, &tris) /* valid mesh */);
    }

    #[test]
    fn test_validate_mesh_invalid_index() {
        let verts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let tris = vec![[0u32, 1, 5]]; /* index 5 out of range */
        assert!(!validate_mesh_for_boolean(&verts, &tris));
    }

    #[test]
    fn test_estimate_vertex_count() {
        assert_eq!(
            estimate_union_vertex_count(10, 20),
            30 /* simple addition */
        );
    }

    #[test]
    fn test_aabb_overlap_true() {
        let va = vec![[0.0f32, 0.0, 0.0], [1.0, 1.0, 1.0]];
        let vb = vec![[0.5f32, 0.5, 0.5], [1.5, 1.5, 1.5]];
        assert!(meshes_aabb_overlap(&va, &vb) /* overlapping boxes */);
    }

    #[test]
    fn test_aabb_no_overlap() {
        let va = vec![[0.0f32, 0.0, 0.0], [1.0, 1.0, 1.0]];
        let vb = vec![[2.0f32, 2.0, 2.0], [3.0, 3.0, 3.0]];
        assert!(!meshes_aabb_overlap(&va, &vb) /* non-overlapping boxes */);
    }

    #[test]
    fn test_weld_removes_duplicates() {
        let verts = vec![[0.0f32, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let welded = weld_union_vertices(&verts, 1e-4);
        assert_eq!(welded.len(), 2 /* duplicate removed */);
    }
}
