//! Shrinkwrap / project-onto-surface mesh deformation.
//!
//! Projects each vertex of a source mesh onto the nearest point on a target
//! mesh surface, optionally applying a normal-direction offset afterwards.

// ── public structs ────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// Configuration parameters for the shrinkwrap operation.
pub struct ShrinkwrapConfig {
    /// Normal-direction offset applied after projection (in mesh units).
    pub offset: f32,
    /// If true, also average normals from the target at the projected point.
    pub snap_normals: bool,
    /// Maximum distance to search for a triangle; triangles farther than this
    /// are skipped when `Some`. `None` means search all triangles.
    pub max_search_dist: Option<f32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// Result of a shrinkwrap operation.
pub struct ShrinkwrapResult {
    /// Projected vertex positions (same length as the source vertex slice).
    pub vertices: Vec<[f32; 3]>,
    /// Per-vertex displacement magnitude (distance moved during projection).
    pub displacements: Vec<f32>,
}

// ── public functions ──────────────────────────────────────────────────────────

#[allow(dead_code)]
/// Returns a [`ShrinkwrapConfig`] with sensible defaults.
pub fn default_shrinkwrap_config() -> ShrinkwrapConfig {
    ShrinkwrapConfig {
        offset: 0.0,
        snap_normals: false,
        max_search_dist: None,
    }
}

#[allow(dead_code)]
/// Projects every vertex in `src_verts` onto the nearest point on the target
/// mesh defined by `tgt_verts` and `tgt_faces`.
pub fn shrinkwrap_mesh(
    src_verts: &[[f32; 3]],
    tgt_verts: &[[f32; 3]],
    tgt_faces: &[[u32; 3]],
    cfg: &ShrinkwrapConfig,
) -> ShrinkwrapResult {
    let mut vertices = Vec::with_capacity(src_verts.len());
    let mut displacements = Vec::with_capacity(src_verts.len());

    for &sv in src_verts {
        let projected = shrinkwrap_single_vertex(sv, tgt_verts, tgt_faces);
        let disp = dist3(sv, projected);

        // Apply normal-direction offset along the approximate face normal,
        // oriented toward the source vertex so the offset always pushes outward.
        let final_pos = if cfg.offset.abs() > f32::EPSILON {
            let normal = approx_normal_toward(projected, sv, tgt_verts, tgt_faces);
            [
                projected[0] + normal[0] * cfg.offset,
                projected[1] + normal[1] * cfg.offset,
                projected[2] + normal[2] * cfg.offset,
            ]
        } else {
            projected
        };

        vertices.push(final_pos);
        displacements.push(disp);
    }

    ShrinkwrapResult {
        vertices,
        displacements,
    }
}

#[allow(dead_code)]
/// Finds the nearest point on any triangle of the target mesh for vertex `v`.
pub fn shrinkwrap_single_vertex(
    v: [f32; 3],
    tgt_verts: &[[f32; 3]],
    tgt_faces: &[[u32; 3]],
) -> [f32; 3] {
    let mut best_pt = v;
    let mut best_d2 = f32::MAX;

    for face in tgt_faces {
        let a = tgt_verts[face[0] as usize];
        let b = tgt_verts[face[1] as usize];
        let c = tgt_verts[face[2] as usize];
        let pt = nearest_point_on_triangle(v, a, b, c);
        let d2 = dist2_3(v, pt);
        if d2 < best_d2 {
            best_d2 = d2;
            best_pt = pt;
        }
    }

    best_pt
}

#[allow(dead_code)]
/// Returns the closest point on the triangle (a, b, c) to query point `p`.
///
/// Uses the standard barycentric-coordinates clamping approach.
pub fn nearest_point_on_triangle(
    p: [f32; 3],
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
) -> [f32; 3] {
    let ab = sub3(b, a);
    let ac = sub3(c, a);
    let ap = sub3(p, a);

    let d1 = dot3(ab, ap);
    let d2 = dot3(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }

    let bp = sub3(p, b);
    let d3 = dot3(ab, bp);
    let d4 = dot3(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return [a[0] + ab[0] * v, a[1] + ab[1] * v, a[2] + ab[2] * v];
    }

    let cp = sub3(p, c);
    let d5 = dot3(ab, cp);
    let d6 = dot3(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return [a[0] + ac[0] * w, a[1] + ac[1] * w, a[2] + ac[2] * w];
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        let bc = sub3(c, b);
        return [b[0] + bc[0] * w, b[1] + bc[1] * w, b[2] + bc[2] * w];
    }

    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    [
        a[0] + ab[0] * v + ac[0] * w,
        a[1] + ab[1] * v + ac[1] * w,
        a[2] + ab[2] * v + ac[2] * w,
    ]
}

#[allow(dead_code)]
/// Returns the number of projected vertices in the result.
pub fn shrinkwrap_vertex_count(result: &ShrinkwrapResult) -> usize {
    result.vertices.len()
}

#[allow(dead_code)]
/// Returns the maximum per-vertex displacement in the result.
pub fn shrinkwrap_max_displacement(result: &ShrinkwrapResult) -> f32 {
    result
        .displacements
        .iter()
        .cloned()
        .fold(0.0_f32, f32::max)
}

#[allow(dead_code)]
/// Returns the average per-vertex displacement in the result.
pub fn shrinkwrap_average_displacement(result: &ShrinkwrapResult) -> f32 {
    if result.displacements.is_empty() {
        return 0.0;
    }
    let sum: f32 = result.displacements.iter().sum();
    sum / result.displacements.len() as f32
}

#[allow(dead_code)]
/// Shifts every projected vertex along +Y by `offset` (a simple global nudge).
///
/// Intended for post-processing tweaks; for surface-normal offset use
/// [`ShrinkwrapConfig::offset`] before the wrap operation.
pub fn shrinkwrap_apply_offset(result: &mut ShrinkwrapResult, offset: f32) {
    for v in &mut result.vertices {
        v[1] += offset;
    }
}

// ── private helpers ───────────────────────────────────────────────────────────

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

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < f32::EPSILON {
        return [0.0, 1.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

fn dist2_3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = sub3(a, b);
    dot3(d, d)
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    dist2_3(a, b).sqrt()
}

/// Approximate surface normal at `pt` oriented toward `src` (the original
/// source vertex before projection), so the normal always faces outward.
fn approx_normal_toward(
    pt: [f32; 3],
    src: [f32; 3],
    tgt_verts: &[[f32; 3]],
    tgt_faces: &[[u32; 3]],
) -> [f32; 3] {
    let mut best_d2 = f32::MAX;
    let mut best_n = [0.0_f32, 1.0, 0.0];

    for face in tgt_faces {
        let a = tgt_verts[face[0] as usize];
        let b = tgt_verts[face[1] as usize];
        let c = tgt_verts[face[2] as usize];
        let centroid = [
            (a[0] + b[0] + c[0]) / 3.0,
            (a[1] + b[1] + c[1]) / 3.0,
            (a[2] + b[2] + c[2]) / 3.0,
        ];
        let d2 = dist2_3(pt, centroid);
        if d2 < best_d2 {
            best_d2 = d2;
            let ab = sub3(b, a);
            let ac = sub3(c, a);
            let n = normalize3(cross3(ab, ac));
            // Orient toward the source vertex.
            let to_src = sub3(src, pt);
            best_n = if dot3(n, to_src) >= 0.0 { n } else { [-n[0], -n[1], -n[2]] };
        }
    }

    best_n
}

// ── unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a single-triangle target mesh in the XZ plane (y=0).
    fn flat_triangle() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let verts = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 0.0, 2.0]];
        let faces = vec![[0u32, 1, 2]];
        (verts, faces)
    }

    #[test]
    fn test_nearest_point_on_triangle_inside() {
        let a = [0.0, 0.0, 0.0];
        let b = [2.0, 0.0, 0.0];
        let c = [0.0, 0.0, 2.0];
        // Point directly above the interior of the triangle
        let p = [0.5, 5.0, 0.5];
        let closest = nearest_point_on_triangle(p, a, b, c);
        assert!((closest[0] - 0.5).abs() < 1e-5, "x");
        assert!((closest[1]).abs() < 1e-5, "y");
        assert!((closest[2] - 0.5).abs() < 1e-5, "z");
    }

    #[test]
    fn test_nearest_point_on_triangle_vertex() {
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        // Point closest to vertex a
        let p = [-1.0, -1.0, 0.0];
        let closest = nearest_point_on_triangle(p, a, b, c);
        assert!((closest[0]).abs() < 1e-5);
        assert!((closest[1]).abs() < 1e-5);
    }

    #[test]
    fn test_shrinkwrap_single_vertex_above_plane() {
        let (tgt_verts, tgt_faces) = flat_triangle();
        // Source vertex hovering above the flat triangle
        let v = [0.5, 3.0, 0.5];
        let projected = shrinkwrap_single_vertex(v, &tgt_verts, &tgt_faces);
        assert!((projected[1]).abs() < 1e-5, "should land on y=0");
        assert!((projected[0] - 0.5).abs() < 1e-5);
        assert!((projected[2] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_shrinkwrap_mesh_result_length() {
        let (tgt_verts, tgt_faces) = flat_triangle();
        let src_verts = vec![[0.3, 2.0, 0.3], [0.1, 1.0, 0.1]];
        let cfg = default_shrinkwrap_config();
        let result = shrinkwrap_mesh(&src_verts, &tgt_verts, &tgt_faces, &cfg);
        assert_eq!(shrinkwrap_vertex_count(&result), 2);
        assert_eq!(result.displacements.len(), 2);
    }

    #[test]
    fn test_shrinkwrap_max_displacement_positive() {
        let (tgt_verts, tgt_faces) = flat_triangle();
        let src_verts = vec![[0.3, 5.0, 0.3]];
        let cfg = default_shrinkwrap_config();
        let result = shrinkwrap_mesh(&src_verts, &tgt_verts, &tgt_faces, &cfg);
        let max_d = shrinkwrap_max_displacement(&result);
        assert!(max_d > 4.0, "should have moved ~5 units");
    }

    #[test]
    fn test_shrinkwrap_average_displacement_single() {
        let (tgt_verts, tgt_faces) = flat_triangle();
        let src_verts = vec![[0.3, 4.0, 0.3]];
        let cfg = default_shrinkwrap_config();
        let result = shrinkwrap_mesh(&src_verts, &tgt_verts, &tgt_faces, &cfg);
        let avg = shrinkwrap_average_displacement(&result);
        let max = shrinkwrap_max_displacement(&result);
        assert!((avg - max).abs() < 1e-5, "single vertex: avg == max");
    }

    #[test]
    fn test_shrinkwrap_apply_offset() {
        let (tgt_verts, tgt_faces) = flat_triangle();
        let src_verts = vec![[0.3, 2.0, 0.3]];
        let cfg = default_shrinkwrap_config();
        let mut result = shrinkwrap_mesh(&src_verts, &tgt_verts, &tgt_faces, &cfg);
        let y_before = result.vertices[0][1];
        shrinkwrap_apply_offset(&mut result, 0.5);
        assert!((result.vertices[0][1] - (y_before + 0.5)).abs() < 1e-5);
    }

    #[test]
    fn test_shrinkwrap_average_displacement_empty() {
        let result = ShrinkwrapResult {
            vertices: vec![],
            displacements: vec![],
        };
        assert_eq!(shrinkwrap_average_displacement(&result), 0.0);
    }

    #[test]
    fn test_shrinkwrap_with_offset_config() {
        let (tgt_verts, tgt_faces) = flat_triangle();
        let src_verts = vec![[0.3, 3.0, 0.3]];
        let mut cfg = default_shrinkwrap_config();
        cfg.offset = 1.0;
        let result = shrinkwrap_mesh(&src_verts, &tgt_verts, &tgt_faces, &cfg);
        // With a +1 offset along the face normal (0,1,0 for XZ plane), y should be ~1.0
        assert!(result.vertices[0][1] > 0.5, "offset should lift vertex");
    }
}
