//! Axis-aligned plane cutting of triangle meshes.
//!
//! Classifies every vertex as "above" or "below" a given plane, identifies
//! faces that are fully above, fully below, or crossing the plane, and
//! records the cut edges generated at the intersection.

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// An axis-aligned cutting plane with a signed offset.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CutPlane {
    /// XY-plane: split by Z value.  `offset` shifts the plane along Z.
    XY(f32),
    /// YZ-plane: split by X value.  `offset` shifts the plane along X.
    YZ(f32),
    /// XZ-plane: split by Y value.  `offset` shifts the plane along Y.
    XZ(f32),
}

/// Configuration for the cut operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshCutConfig {
    /// Vertices whose signed distance to the plane is within this tolerance
    /// are treated as lying exactly on the plane (assigned to "above").
    pub epsilon: f32,
}

/// Result of a mesh cut operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CutResult {
    /// Indices of vertices classified as above (or on) the plane.
    pub above_verts: Vec<usize>,
    /// Indices of vertices classified as below the plane.
    pub below_verts: Vec<usize>,
    /// Pairs of vertex indices whose connecting edge crosses the plane.
    pub cut_edges: Vec<(usize, usize)>,
    /// Indices of faces that are entirely above the plane.
    pub above_faces: Vec<usize>,
    /// Indices of faces that are entirely below the plane.
    pub below_faces: Vec<usize>,
    /// Indices of faces that straddle the plane (at least one vertex on each side).
    pub crossing_faces: Vec<usize>,
}

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Returns the default cut configuration.
#[allow(dead_code)]
pub fn default_cut_config() -> MeshCutConfig {
    MeshCutConfig { epsilon: 1e-6 }
}

/// Cuts `verts`/`faces` along `plane`, classifying geometry above and below.
#[allow(dead_code)]
pub fn cut_mesh(
    verts: &[[f32; 3]],
    faces: &[[u32; 3]],
    plane: CutPlane,
    cfg: &MeshCutConfig,
) -> CutResult {
    // Classify each vertex.
    let sides: Vec<f32> = verts.iter().map(|&v| vertex_side(v, plane)).collect();

    let mut above_verts: Vec<usize> = Vec::new();
    let mut below_verts: Vec<usize> = Vec::new();
    for (i, &s) in sides.iter().enumerate() {
        if s >= -cfg.epsilon {
            above_verts.push(i);
        } else {
            below_verts.push(i);
        }
    }

    // Classify faces and detect cut edges.
    let mut above_faces: Vec<usize> = Vec::new();
    let mut below_faces: Vec<usize> = Vec::new();
    let mut crossing_faces: Vec<usize> = Vec::new();
    let mut cut_edge_set: std::collections::HashSet<(usize, usize)> =
        std::collections::HashSet::new();

    for (fi, face) in faces.iter().enumerate() {
        let s: Vec<f32> = face.iter().map(|&v| sides[v as usize]).collect();
        let any_above = s.iter().any(|&x| x >= -cfg.epsilon);
        let any_below = s.iter().any(|&x| x < -cfg.epsilon);

        match (any_above, any_below) {
            (true, false) => above_faces.push(fi),
            (false, true) => below_faces.push(fi),
            _ => {
                crossing_faces.push(fi);
                // Record edges that cross the plane.
                let vf = [face[0] as usize, face[1] as usize, face[2] as usize];
                for k in 0..3 {
                    let a = vf[k];
                    let b = vf[(k + 1) % 3];
                    let sa = sides[a];
                    let sb = sides[b];
                    if (sa >= -cfg.epsilon) != (sb >= -cfg.epsilon) {
                        let key = if a < b { (a, b) } else { (b, a) };
                        cut_edge_set.insert(key);
                    }
                }
            }
        }
    }

    let mut cut_edges: Vec<(usize, usize)> = cut_edge_set.into_iter().collect();
    cut_edges.sort_unstable();

    CutResult {
        above_verts,
        below_verts,
        cut_edges,
        above_faces,
        below_faces,
        crossing_faces,
    }
}

/// Returns the number of vertices classified as above (or on) the plane.
#[allow(dead_code)]
pub fn cut_result_above_count(result: &CutResult) -> usize {
    result.above_verts.len()
}

/// Returns the number of vertices classified as below the plane.
#[allow(dead_code)]
pub fn cut_result_below_count(result: &CutResult) -> usize {
    result.below_verts.len()
}

/// Returns the number of cut edges (edges that cross the plane).
#[allow(dead_code)]
pub fn cut_edge_count(result: &CutResult) -> usize {
    result.cut_edges.len()
}

/// Returns the signed distance of vertex `v` from `plane`.
///
/// Positive values are "above" the plane; negative values are "below".
#[allow(dead_code)]
pub fn vertex_side(v: [f32; 3], plane: CutPlane) -> f32 {
    match plane {
        CutPlane::XY(offset) => v[2] - offset,
        CutPlane::YZ(offset) => v[0] - offset,
        CutPlane::XZ(offset) => v[1] - offset,
    }
}

/// Returns the scalar offset stored inside `plane`.
#[allow(dead_code)]
pub fn cut_plane_offset(plane: CutPlane) -> f32 {
    match plane {
        CutPlane::XY(o) | CutPlane::YZ(o) | CutPlane::XZ(o) => o,
    }
}

/// Returns a human-readable name for the plane variant.
#[allow(dead_code)]
pub fn cut_plane_name(plane: CutPlane) -> &'static str {
    match plane {
        CutPlane::XY(_) => "XY",
        CutPlane::YZ(_) => "YZ",
        CutPlane::XZ(_) => "XZ",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> MeshCutConfig {
        default_cut_config()
    }

    /// All vertices above the XY plane (z > 0) → all above, no cut edges.
    #[test]
    fn test_all_above_xz_plane() {
        let verts: Vec<[f32; 3]> = vec![[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [0.5, 1.0, 1.0]];
        let faces = vec![[0u32, 1, 2]];
        let result = cut_mesh(&verts, &faces, CutPlane::XY(0.0), &cfg());
        assert_eq!(cut_result_above_count(&result), 3);
        assert_eq!(cut_result_below_count(&result), 0);
        assert_eq!(cut_edge_count(&result), 0);
        assert_eq!(result.above_faces.len(), 1);
        assert!(result.crossing_faces.is_empty());
    }

    /// All vertices below the XY plane (z < 0) → all below, no cut edges.
    #[test]
    fn test_all_below_xy_plane() {
        let verts: Vec<[f32; 3]> = vec![[0.0, 0.0, -1.0], [1.0, 0.0, -1.0], [0.5, 1.0, -1.0]];
        let faces = vec![[0u32, 1, 2]];
        let result = cut_mesh(&verts, &faces, CutPlane::XY(0.0), &cfg());
        assert_eq!(cut_result_above_count(&result), 0);
        assert_eq!(cut_result_below_count(&result), 3);
        assert_eq!(cut_edge_count(&result), 0);
        assert_eq!(result.below_faces.len(), 1);
    }

    /// Triangle straddling the XY plane → crossing face and cut edges.
    #[test]
    fn test_crossing_triangle() {
        // Two verts below, one above.
        let verts: Vec<[f32; 3]> = vec![[0.0, 0.0, -1.0], [1.0, 0.0, -1.0], [0.5, 1.0, 1.0]];
        let faces = vec![[0u32, 1, 2]];
        let result = cut_mesh(&verts, &faces, CutPlane::XY(0.0), &cfg());
        assert_eq!(result.crossing_faces.len(), 1);
        assert_eq!(cut_edge_count(&result), 2);
    }

    /// `vertex_side` returns positive for above, negative for below.
    #[test]
    fn test_vertex_side_sign() {
        assert!(vertex_side([0.0, 0.0, 1.0], CutPlane::XY(0.0)) > 0.0);
        assert!(vertex_side([0.0, 0.0, -1.0], CutPlane::XY(0.0)) < 0.0);
        assert_eq!(vertex_side([0.0, 0.0, 0.0], CutPlane::XY(0.0)), 0.0);
    }

    /// YZ plane splits by X value.
    #[test]
    fn test_yz_plane_split() {
        let verts: Vec<[f32; 3]> = vec![[2.0, 0.0, 0.0], [-2.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let faces = vec![[0u32, 1, 2]];
        let result = cut_mesh(&verts, &faces, CutPlane::YZ(0.0), &cfg());
        assert_eq!(result.crossing_faces.len(), 1);
    }

    /// XZ plane splits by Y value.
    #[test]
    fn test_xz_plane_split() {
        let verts: Vec<[f32; 3]> = vec![[0.0, 2.0, 0.0], [0.0, -2.0, 0.0], [1.0, 0.0, 0.0]];
        let faces = vec![[0u32, 1, 2]];
        let result = cut_mesh(&verts, &faces, CutPlane::XZ(0.0), &cfg());
        assert_eq!(result.crossing_faces.len(), 1);
    }

    /// `cut_plane_name` returns correct strings.
    #[test]
    fn test_cut_plane_name() {
        assert_eq!(cut_plane_name(CutPlane::XY(0.0)), "XY");
        assert_eq!(cut_plane_name(CutPlane::YZ(1.0)), "YZ");
        assert_eq!(cut_plane_name(CutPlane::XZ(-1.0)), "XZ");
    }

    /// `cut_plane_offset` returns the stored offset.
    #[test]
    fn test_cut_plane_offset() {
        assert!((cut_plane_offset(CutPlane::XY(3.5)) - 3.5).abs() < 1e-9);
        assert!((cut_plane_offset(CutPlane::YZ(-1.0)) - (-1.0)).abs() < 1e-9);
    }

    /// Offset plane: vertices above shifted plane are correctly classified.
    #[test]
    fn test_offset_plane() {
        // Plane at z = 2.0; verts at z = 1.5 (below) and z = 2.5 (above).
        let verts: Vec<[f32; 3]> =
            vec![[0.0, 0.0, 1.5], [1.0, 0.0, 2.5], [0.5, 1.0, 2.5]];
        let faces = vec![[0u32, 1, 2]];
        let result = cut_mesh(&verts, &faces, CutPlane::XY(2.0), &cfg());
        assert_eq!(cut_result_above_count(&result), 2);
        assert_eq!(cut_result_below_count(&result), 1);
    }
}
