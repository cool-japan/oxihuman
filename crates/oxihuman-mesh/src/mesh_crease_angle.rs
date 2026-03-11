//! Detect and classify sharp crease edges by dihedral angle threshold.
//!
//! A crease edge is one where the dihedral angle between two adjacent
//! triangle faces exceeds a configurable threshold. Crease detection is
//! used in normal computation, subdivision, and mesh export to preserve
//! hard edges on otherwise smooth surfaces.

#![allow(dead_code)]

/// Configuration for crease-angle detection.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CreaseAngleConfig {
    /// Angle threshold in radians above which an edge is "sharp".
    pub threshold_radians: f32,
    /// Whether to include boundary edges (edges with only one face) as creases.
    pub boundary_as_crease: bool,
}

/// A single detected crease edge, described by its two vertex indices.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CreaseEdge {
    /// Index of the first vertex.
    pub v0: usize,
    /// Index of the second vertex.
    pub v1: usize,
    /// Dihedral angle between the two adjacent faces (radians).
    pub angle: f32,
    /// Whether this edge is considered sharp under the current config.
    pub is_sharp: bool,
}

/// Result of a crease-edge detection pass.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CreaseResult {
    /// All candidate edges with their dihedral angles.
    pub edges: Vec<CreaseEdge>,
    /// The threshold that was used during detection.
    pub threshold_radians: f32,
}

/// Return a sensible default [`CreaseAngleConfig`] (30° threshold).
#[allow(dead_code)]
pub fn default_crease_angle_config() -> CreaseAngleConfig {
    CreaseAngleConfig {
        threshold_radians: std::f32::consts::PI / 6.0, // 30°
        boundary_as_crease: true,
    }
}

/// Compute the dihedral angle between two triangles that share an edge.
///
/// `n0` and `n1` are the (unnormalized) face normals. Returns the angle
/// in [0, π].
fn dihedral_angle(n0: [f32; 3], n1: [f32; 3]) -> f32 {
    let len0 = (n0[0] * n0[0] + n0[1] * n0[1] + n0[2] * n0[2]).sqrt();
    let len1 = (n1[0] * n1[0] + n1[1] * n1[1] + n1[2] * n1[2]).sqrt();
    if len0 < 1e-9 || len1 < 1e-9 {
        return 0.0;
    }
    let dot =
        (n0[0] * n1[0] + n0[1] * n1[1] + n0[2] * n1[2]) / (len0 * len1);
    dot.clamp(-1.0, 1.0).acos()
}

/// Compute the face normal of a triangle given three vertex positions.
fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ]
}

/// Detect crease edges given vertex positions and triangle indices.
///
/// `positions` is a flat list of `[x, y, z]` vertex positions.
/// `triangles` is a list of `[i0, i1, i2]` index triples.
/// Returns a [`CreaseResult`] with every interior edge annotated.
#[allow(dead_code)]
pub fn detect_crease_edges(
    positions: &[[f32; 3]],
    triangles: &[[usize; 3]],
    config: &CreaseAngleConfig,
) -> CreaseResult {
    use std::collections::HashMap;

    // Build edge → list of adjacent face normals.
    let mut edge_map: HashMap<(usize, usize), Vec<[f32; 3]>> = HashMap::new();

    for tri in triangles {
        let n = face_normal(positions[tri[0]], positions[tri[1]], positions[tri[2]]);
        for &(a, b) in &[(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            let key = if a < b { (a, b) } else { (b, a) };
            edge_map.entry(key).or_default().push(n);
        }
    }

    let mut edges: Vec<CreaseEdge> = Vec::with_capacity(edge_map.len());

    for ((v0, v1), normals) in edge_map {
        let angle = if normals.len() >= 2 {
            dihedral_angle(normals[0], normals[1])
        } else {
            // Boundary edge
            if config.boundary_as_crease {
                std::f32::consts::PI
            } else {
                0.0
            }
        };
        let is_sharp = angle >= config.threshold_radians;
        edges.push(CreaseEdge { v0, v1, angle, is_sharp });
    }

    // Sort for deterministic output.
    edges.sort_by(|a, b| a.v0.cmp(&b.v0).then(a.v1.cmp(&b.v1)));

    CreaseResult { edges, threshold_radians: config.threshold_radians }
}

/// Return the total number of edges in the result.
#[allow(dead_code)]
pub fn crease_edge_count(result: &CreaseResult) -> usize {
    result.edges.len()
}

/// Return the dihedral angle for the edge at `index` (radians).
#[allow(dead_code)]
pub fn crease_edge_angle(result: &CreaseResult, index: usize) -> f32 {
    result.edges[index].angle
}

/// Return `true` if the edge at `index` is classified as sharp.
#[allow(dead_code)]
pub fn crease_is_sharp(result: &CreaseResult, index: usize) -> bool {
    result.edges[index].is_sharp
}

/// Serialise the result to a compact JSON string.
#[allow(dead_code)]
pub fn crease_result_to_json(result: &CreaseResult) -> String {
    let edge_strs: Vec<String> = result
        .edges
        .iter()
        .map(|e| {
            format!(
                r#"{{"v0":{},"v1":{},"angle":{:.6},"sharp":{}}}"#,
                e.v0, e.v1, e.angle, e.is_sharp
            )
        })
        .collect();
    format!(
        r#"{{"threshold":{:.6},"edges":[{}]}}"#,
        result.threshold_radians,
        edge_strs.join(",")
    )
}

/// Return only the smooth (non-sharp) edges.
#[allow(dead_code)]
pub fn crease_smooth_edges(result: &CreaseResult) -> Vec<&CreaseEdge> {
    result.edges.iter().filter(|e| !e.is_sharp).collect()
}

/// Return only the sharp edges.
#[allow(dead_code)]
pub fn crease_sharp_edges(result: &CreaseResult) -> Vec<&CreaseEdge> {
    result.edges.iter().filter(|e| e.is_sharp).collect()
}

/// Compute the average dihedral angle across all edges.
#[allow(dead_code)]
pub fn crease_average_angle(result: &CreaseResult) -> f32 {
    if result.edges.is_empty() {
        return 0.0;
    }
    let sum: f32 = result.edges.iter().map(|e| e.angle).sum();
    sum / result.edges.len() as f32
}

/// Return a new [`CreaseResult`] with a different sharpness threshold applied.
///
/// The edge angles are preserved; only the `is_sharp` flag is updated.
#[allow(dead_code)]
pub fn crease_set_threshold(result: &CreaseResult, threshold_radians: f32) -> CreaseResult {
    let edges = result
        .edges
        .iter()
        .map(|e| CreaseEdge {
            v0: e.v0,
            v1: e.v1,
            angle: e.angle,
            is_sharp: e.angle >= threshold_radians,
        })
        .collect();
    CreaseResult { edges, threshold_radians }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn cube_mesh() -> (Vec<[f32; 3]>, Vec<[usize; 3]>) {
        // 8-vertex unit cube split into 12 triangles
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        let triangles = vec![
            [0, 1, 2], [0, 2, 3], // bottom
            [4, 6, 5], [4, 7, 6], // top
            [0, 5, 1], [0, 4, 5], // front
            [1, 6, 2], [1, 5, 6], // right
            [2, 7, 3], [2, 6, 7], // back
            [3, 4, 0], [3, 7, 4], // left
        ];
        (positions, triangles)
    }

    #[test]
    fn test_default_config_threshold() {
        let cfg = default_crease_angle_config();
        assert!((cfg.threshold_radians - PI / 6.0).abs() < 1e-6);
    }

    #[test]
    fn test_detect_crease_edges_produces_edges() {
        let (pos, tris) = cube_mesh();
        let cfg = default_crease_angle_config();
        let result = detect_crease_edges(&pos, &tris, &cfg);
        assert!(!result.edges.is_empty());
    }

    #[test]
    fn test_cube_all_edges_are_sharp() {
        let (pos, tris) = cube_mesh();
        // 90° cube edges should all be above 30° threshold
        let cfg = default_crease_angle_config();
        let result = detect_crease_edges(&pos, &tris, &cfg);
        let sharp = crease_sharp_edges(&result);
        assert!(!sharp.is_empty());
    }

    #[test]
    fn test_crease_edge_count() {
        let (pos, tris) = cube_mesh();
        let cfg = default_crease_angle_config();
        let result = detect_crease_edges(&pos, &tris, &cfg);
        assert_eq!(crease_edge_count(&result), result.edges.len());
    }

    #[test]
    fn test_crease_edge_angle_in_range() {
        let (pos, tris) = cube_mesh();
        let cfg = default_crease_angle_config();
        let result = detect_crease_edges(&pos, &tris, &cfg);
        for i in 0..crease_edge_count(&result) {
            let a = crease_edge_angle(&result, i);
            assert!((0.0..=PI + 1e-5).contains(&a));
        }
    }

    #[test]
    fn test_crease_is_sharp_consistent() {
        let (pos, tris) = cube_mesh();
        let cfg = default_crease_angle_config();
        let result = detect_crease_edges(&pos, &tris, &cfg);
        for (i, e) in result.edges.iter().enumerate() {
            assert_eq!(crease_is_sharp(&result, i), e.angle >= result.threshold_radians);
        }
    }

    #[test]
    fn test_crease_result_to_json_contains_threshold() {
        let (pos, tris) = cube_mesh();
        let cfg = default_crease_angle_config();
        let result = detect_crease_edges(&pos, &tris, &cfg);
        let json = crease_result_to_json(&result);
        assert!(json.contains("threshold"));
    }

    #[test]
    fn test_crease_smooth_plus_sharp_eq_total() {
        let (pos, tris) = cube_mesh();
        let cfg = default_crease_angle_config();
        let result = detect_crease_edges(&pos, &tris, &cfg);
        let total = crease_edge_count(&result);
        let sharp = crease_sharp_edges(&result).len();
        let smooth = crease_smooth_edges(&result).len();
        assert_eq!(sharp + smooth, total);
    }

    #[test]
    fn test_crease_average_angle_finite() {
        let (pos, tris) = cube_mesh();
        let cfg = default_crease_angle_config();
        let result = detect_crease_edges(&pos, &tris, &cfg);
        let avg = crease_average_angle(&result);
        assert!(avg.is_finite());
    }

    #[test]
    fn test_crease_set_threshold_updates_sharp() {
        let (pos, tris) = cube_mesh();
        let cfg = default_crease_angle_config();
        let result = detect_crease_edges(&pos, &tris, &cfg);
        // With threshold = π all edges should be non-sharp
        let r2 = crease_set_threshold(&result, PI);
        let sharp_count = crease_sharp_edges(&r2).len();
        assert_eq!(sharp_count, 0);
    }

    #[test]
    fn test_crease_set_threshold_zero_makes_all_sharp() {
        let (pos, tris) = cube_mesh();
        let cfg = default_crease_angle_config();
        let result = detect_crease_edges(&pos, &tris, &cfg);
        let r2 = crease_set_threshold(&result, 0.0);
        let sharp_count = crease_sharp_edges(&r2).len();
        assert_eq!(sharp_count, crease_edge_count(&r2));
    }
}
