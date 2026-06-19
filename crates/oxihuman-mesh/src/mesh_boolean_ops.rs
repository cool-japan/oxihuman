//! Mesh boolean operations — volumetric SDF-based dispatcher.
//!
//! Routes Union / Intersect / Subtract to the per-operation modules that use
//! a shared SDF grid for correct volumetric boolean evaluation.

/// The type of boolean operation to perform.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOpType {
    /// Union: combine both meshes.
    Union,
    /// Intersect: keep only the overlapping region.
    Intersect,
    /// Subtract: remove mesh B from mesh A.
    Subtract,
}

/// Configuration for boolean operations.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BooleanConfig {
    /// Tolerance for boundary classification.
    pub tolerance: f32,
    /// Whether to attempt manifold repair after the operation.
    pub repair_manifold: bool,
}

/// Result of a boolean mesh operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BooleanResult {
    /// Output vertices.
    pub vertices: Vec<[f32; 3]>,
    /// Output triangular faces (indices into `vertices`).
    pub faces: Vec<[u32; 3]>,
    /// Whether the result is considered manifold.
    pub is_manifold: bool,
}

/// Returns the default boolean operation configuration.
#[allow(dead_code)]
pub fn default_boolean_config() -> BooleanConfig {
    BooleanConfig {
        tolerance: 1e-5,
        repair_manifold: true,
    }
}

/// Computes the axis-aligned bounding box of a vertex set.
///
/// Returns `(min, max)`. If `verts` is empty returns `([0;3], [0;3])`.
#[allow(dead_code)]
pub fn aabb_from_verts(verts: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    if verts.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = verts[0];
    let mut mx = verts[0];
    for v in verts.iter().skip(1) {
        for k in 0..3 {
            if v[k] < mn[k] {
                mn[k] = v[k];
            }
            if v[k] > mx[k] {
                mx[k] = v[k];
            }
        }
    }
    (mn, mx)
}

/// Returns `true` if point `p` lies strictly inside the AABB.
#[allow(dead_code)]
pub fn point_in_aabb(p: [f32; 3], aabb_min: [f32; 3], aabb_max: [f32; 3]) -> bool {
    p[0] > aabb_min[0]
        && p[0] < aabb_max[0]
        && p[1] > aabb_min[1]
        && p[1] < aabb_max[1]
        && p[2] > aabb_min[2]
        && p[2] < aabb_max[2]
}

/// Returns the centroid of a triangle defined by three vertices.
fn tri_centroid(
    verts: &[[f32; 3]],
    face: [u32; 3],
) -> [f32; 3] {
    let a = verts[face[0] as usize];
    let b = verts[face[1] as usize];
    let c = verts[face[2] as usize];
    [
        (a[0] + b[0] + c[0]) / 3.0,
        (a[1] + b[1] + c[1]) / 3.0,
        (a[2] + b[2] + c[2]) / 3.0,
    ]
}

/// Performs the boolean operation and returns the assembled result mesh.
///
/// Delegates to the per-operation volumetric SDF implementations:
/// - `BooleanOpType::Union`     → `mesh_boolean_union`
/// - `BooleanOpType::Intersect` → `mesh_boolean_intersection`
/// - `BooleanOpType::Subtract`  → `mesh_boolean_difference`
#[allow(dead_code)]
pub fn boolean_op(
    verts_a: &[[f32; 3]],
    faces_a: &[[u32; 3]],
    verts_b: &[[f32; 3]],
    faces_b: &[[u32; 3]],
    op: BooleanOpType,
    _cfg: &BooleanConfig,
) -> BooleanResult {
    use super::mesh_boolean_difference::{
        mesh_boolean_difference, BooleanDifferenceConfig,
    };
    use super::mesh_boolean_intersection::{
        mesh_boolean_intersection, BooleanIntersectionConfig,
    };
    use super::mesh_boolean_union::{mesh_boolean_union, BooleanUnionConfig};

    let (out_verts, out_faces) = match op {
        BooleanOpType::Union => {
            let r = mesh_boolean_union(
                verts_a,
                faces_a,
                verts_b,
                faces_b,
                &BooleanUnionConfig::default(),
            );
            (r.vertices, r.triangles)
        }
        BooleanOpType::Intersect => {
            let r = mesh_boolean_intersection(
                verts_a,
                faces_a,
                verts_b,
                faces_b,
                &BooleanIntersectionConfig::default(),
            );
            (r.vertices, r.triangles)
        }
        BooleanOpType::Subtract => {
            let r = mesh_boolean_difference(
                verts_a,
                faces_a,
                verts_b,
                faces_b,
                &BooleanDifferenceConfig::default(),
            );
            (r.vertices, r.triangles)
        }
    };

    let is_manifold = check_manifold(&out_faces, out_verts.len());
    BooleanResult {
        vertices: out_verts,
        faces: out_faces,
        is_manifold,
    }
}

/// Naively checks manifold-ness: every edge (pair of vertex indices) should
/// appear at most twice across all faces.
fn check_manifold(faces: &[[u32; 3]], _vert_count: usize) -> bool {
    use std::collections::HashMap;
    let mut edge_count: HashMap<(u32, u32), u32> = HashMap::new();
    for face in faces {
        let edges = [
            (face[0].min(face[1]), face[0].max(face[1])),
            (face[1].min(face[2]), face[1].max(face[2])),
            (face[0].min(face[2]), face[0].max(face[2])),
        ];
        for e in &edges {
            *edge_count.entry(*e).or_insert(0) += 1;
        }
    }
    edge_count.values().all(|&c| c <= 2)
}

/// Returns the vertex count of a boolean result.
#[allow(dead_code)]
pub fn boolean_result_vertex_count(result: &BooleanResult) -> usize {
    result.vertices.len()
}

/// Returns the face count of a boolean result.
#[allow(dead_code)]
pub fn boolean_result_face_count(result: &BooleanResult) -> usize {
    result.faces.len()
}

/// Returns a human-readable name for a boolean operation type.
#[allow(dead_code)]
pub fn boolean_op_name(op: BooleanOpType) -> &'static str {
    match op {
        BooleanOpType::Union => "union",
        BooleanOpType::Intersect => "intersect",
        BooleanOpType::Subtract => "subtract",
    }
}

/// Returns whether the boolean result is manifold.
#[allow(dead_code)]
pub fn boolean_result_is_manifold(result: &BooleanResult) -> bool {
    result.is_manifold
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two-triangle quad mesh occupying [0,1]^3.
    fn unit_box_mesh() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let verts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let faces = vec![[0, 1, 2], [0, 2, 3]];
        (verts, faces)
    }

    /// Small mesh occupying [0.5, 1.5]^3.
    fn offset_box_mesh() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let verts = vec![
            [0.5, 0.5, 0.0],
            [1.5, 0.5, 0.0],
            [1.5, 1.5, 0.0],
            [0.5, 1.5, 0.0],
        ];
        let faces = vec![[0, 1, 2], [0, 2, 3]];
        (verts, faces)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_boolean_config();
        assert!(cfg.tolerance > 0.0);
        assert!(cfg.repair_manifold);
    }

    #[test]
    fn test_boolean_op_name() {
        assert_eq!(boolean_op_name(BooleanOpType::Union), "union");
        assert_eq!(boolean_op_name(BooleanOpType::Intersect), "intersect");
        assert_eq!(boolean_op_name(BooleanOpType::Subtract), "subtract");
    }

    #[test]
    fn test_aabb_from_verts() {
        let (verts, _) = unit_box_mesh();
        let (mn, mx) = aabb_from_verts(&verts);
        assert_eq!(mn, [0.0, 0.0, 0.0]);
        assert_eq!(mx, [1.0, 1.0, 0.0]);
    }

    #[test]
    fn test_aabb_from_verts_empty() {
        let (mn, mx) = aabb_from_verts(&[]);
        assert_eq!(mn, [0.0, 0.0, 0.0]);
        assert_eq!(mx, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_point_in_aabb() {
        assert!(point_in_aabb(
            [0.5, 0.5, 0.5],
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0]
        ));
        assert!(!point_in_aabb(
            [1.5, 0.5, 0.5],
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0]
        ));
    }

    #[test]
    fn test_subtract_removes_overlap() {
        let (va, fa) = unit_box_mesh();
        let (vb, fb) = offset_box_mesh();
        let cfg = default_boolean_config();
        let result = boolean_op(&va, &fa, &vb, &fb, BooleanOpType::Subtract, &cfg);
        // After subtracting the overlapping region, some faces should remain.
        // (Exact count depends on AABB classification.)
        let _vc = boolean_result_vertex_count(&result);
        let fc = boolean_result_face_count(&result);
        assert!(fc <= fa.len(), "subtract should not add faces");
    }

    #[test]
    fn test_union_has_all_b_faces() {
        let (va, fa) = unit_box_mesh();
        let (vb, fb) = offset_box_mesh();
        let cfg = default_boolean_config();
        let result = boolean_op(&va, &fa, &vb, &fb, BooleanOpType::Union, &cfg);
        // Union must include at least all B faces.
        assert!(
            boolean_result_face_count(&result) >= fb.len(),
            "union face count={}",
            boolean_result_face_count(&result)
        );
    }

    #[test]
    fn test_intersect_empty_when_no_overlap() {
        let (va, fa) = unit_box_mesh();
        // Cutter far away.
        let vb = vec![
            [10.0, 10.0, 0.0],
            [11.0, 10.0, 0.0],
            [11.0, 11.0, 0.0],
            [10.0, 11.0, 0.0],
        ];
        let fb = vec![[0u32, 1, 2], [0, 2, 3]];
        let cfg = default_boolean_config();
        let result = boolean_op(&va, &fa, &vb, &fb, BooleanOpType::Intersect, &cfg);
        assert_eq!(
            boolean_result_face_count(&result),
            0,
            "no intersection expected"
        );
    }

    #[test]
    fn test_manifold_check() {
        let (va, fa) = unit_box_mesh();
        let (vb, fb) = offset_box_mesh();
        let cfg = default_boolean_config();
        let result = boolean_op(&va, &fa, &vb, &fb, BooleanOpType::Subtract, &cfg);
        // Just verify the accessor works.
        let _m = boolean_result_is_manifold(&result);
    }
}
