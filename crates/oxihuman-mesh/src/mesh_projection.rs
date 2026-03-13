// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Surface projection — project points and meshes onto triangle meshes.

#[allow(dead_code)]
pub struct ProjectionResult {
    pub projected_point: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub barycentric: [f32; 3],
    pub face_index: usize,
    pub distance: f32,
}

#[allow(dead_code)]
pub enum ProjectionMode {
    ClosestPoint,
    AlongNormal { normal: [f32; 3] },
    AlongAxis { axis: u8 }, // 0=X, 1=Y, 2=Z
}

// ── math helpers ─────────────────────────────────────────────────────────────

#[inline]
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn length(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}

#[inline]
fn normalize(a: [f32; 3]) -> [f32; 3] {
    let len = length(a);
    if len < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        scale(a, 1.0 / len)
    }
}

#[inline]
fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    length(sub(a, b))
}

fn tri_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    normalize(cross(sub(b, a), sub(c, a)))
}

// ── public API ────────────────────────────────────────────────────────────────

/// Project a point to the closest point on the triangle (a,b,c).
/// Returns `(closest_point, barycentric_coords)`.
#[allow(dead_code)]
pub fn project_point_to_triangle(
    point: [f32; 3],
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
) -> ([f32; 3], [f32; 3]) {
    // Based on Ericson "Real-Time Collision Detection" §5.1
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(point, a);

    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);

    if d1 <= 0.0 && d2 <= 0.0 {
        return (a, [1.0, 0.0, 0.0]);
    }

    let bp = sub(point, b);
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return (b, [0.0, 1.0, 0.0]);
    }

    let cp = sub(point, c);
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return (c, [0.0, 0.0, 1.0]);
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        let p = add(a, scale(ab, v));
        return (p, [1.0 - v, v, 0.0]);
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        let p = add(a, scale(ac, w));
        return (p, [1.0 - w, 0.0, w]);
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        let p = add(b, scale(sub(c, b), w));
        return (p, [0.0, 1.0 - w, w]);
    }

    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    let u = 1.0 - v - w;
    let p = add(add(scale(a, u), scale(b, v)), scale(c, w));
    (p, [u, v, w])
}

/// Project a single point onto the nearest triangle of a mesh.
#[allow(dead_code)]
pub fn project_point_to_mesh(
    point: [f32; 3],
    positions: &[[f32; 3]],
    triangles: &[[u32; 3]],
    _mode: &ProjectionMode,
) -> Option<ProjectionResult> {
    if positions.is_empty() || triangles.is_empty() {
        return None;
    }

    let mut best_dist = f32::MAX;
    let mut best_result: Option<ProjectionResult> = None;

    for (fi, tri) in triangles.iter().enumerate() {
        let a = positions.get(tri[0] as usize).copied()?;
        let b = positions.get(tri[1] as usize).copied()?;
        let c = positions.get(tri[2] as usize).copied()?;

        let (closest, bary) = project_point_to_triangle(point, a, b, c);
        let d = dist3(point, closest);
        if d < best_dist {
            best_dist = d;
            let n = tri_normal(a, b, c);
            let uv = [bary[0], bary[1]]; // simple placeholder UV
            best_result = Some(ProjectionResult {
                projected_point: closest,
                normal: n,
                uv,
                barycentric: bary,
                face_index: fi,
                distance: d,
            });
        }
    }

    best_result
}

/// Project each vertex of `src_positions` onto the target mesh.
#[allow(dead_code)]
pub fn project_mesh_onto_mesh(
    src_positions: &[[f32; 3]],
    tgt_positions: &[[f32; 3]],
    tgt_triangles: &[[u32; 3]],
) -> Vec<ProjectionResult> {
    src_positions
        .iter()
        .filter_map(|&p| {
            project_point_to_mesh(
                p,
                tgt_positions,
                tgt_triangles,
                &ProjectionMode::ClosestPoint,
            )
        })
        .collect()
}

/// Compute barycentric coordinates of `point` relative to triangle (a, b, c).
#[allow(dead_code)]
pub fn compute_barycentric(point: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let (_, bary) = project_point_to_triangle(point, a, b, c);
    bary
}

/// Convert barycentric coordinates back to a 3-D point.
#[allow(dead_code)]
pub fn barycentric_to_point(bary: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    add(add(scale(a, bary[0]), scale(b, bary[1])), scale(c, bary[2]))
}

/// Interpolate a UV coordinate at given barycentric weights.
#[allow(dead_code)]
pub fn interpolate_uv_at_barycentric(
    bary: [f32; 3],
    uv_a: [f32; 2],
    uv_b: [f32; 2],
    uv_c: [f32; 2],
) -> [f32; 2] {
    [
        uv_a[0] * bary[0] + uv_b[0] * bary[1] + uv_c[0] * bary[2],
        uv_a[1] * bary[0] + uv_b[1] * bary[1] + uv_c[1] * bary[2],
    ]
}

/// Blend each source vertex between its original position and its projection
/// onto the target mesh, with `blend` = 0 → original, 1 → fully projected.
#[allow(dead_code)]
pub fn shrink_wrap_proj(
    src_positions: &[[f32; 3]],
    tgt_positions: &[[f32; 3]],
    tgt_triangles: &[[u32; 3]],
    blend: f32,
) -> Vec<[f32; 3]> {
    src_positions
        .iter()
        .map(|&p| {
            if let Some(res) = project_point_to_mesh(
                p,
                tgt_positions,
                tgt_triangles,
                &ProjectionMode::ClosestPoint,
            ) {
                let q = res.projected_point;
                [
                    p[0] + (q[0] - p[0]) * blend,
                    p[1] + (q[1] - p[1]) * blend,
                    p[2] + (q[2] - p[2]) * blend,
                ]
            } else {
                p
            }
        })
        .collect()
}

/// Project all vertices along a given axis by `offset`.
#[allow(dead_code)]
pub fn project_along_axis(
    positions: &[[f32; 3]],
    triangles: &[[u32; 3]],
    axis: u8,
    offset: f32,
) -> Vec<[f32; 3]> {
    let axis_dir: [f32; 3] = match axis {
        0 => [1.0, 0.0, 0.0],
        1 => [0.0, 1.0, 0.0],
        _ => [0.0, 0.0, 1.0],
    };
    positions
        .iter()
        .map(|&p| {
            let shifted = add(p, scale(axis_dir, offset));
            if let Some(res) = project_point_to_mesh(
                shifted,
                positions,
                triangles,
                &ProjectionMode::AlongAxis { axis },
            ) {
                res.projected_point
            } else {
                shifted
            }
        })
        .collect()
}

/// Snap vertices within `threshold` distance of the surface to the surface.
/// Returns the number of vertices snapped.
#[allow(dead_code)]
pub fn snap_to_surface(
    positions: &mut [[f32; 3]],
    surface_positions: &[[f32; 3]],
    surface_triangles: &[[u32; 3]],
    threshold: f32,
) -> usize {
    let mut count = 0;
    for p in positions.iter_mut() {
        if let Some(res) = project_point_to_mesh(
            *p,
            surface_positions,
            surface_triangles,
            &ProjectionMode::ClosestPoint,
        ) {
            if res.distance < threshold {
                *p = res.projected_point;
                count += 1;
            }
        }
    }
    count
}

/// Transfer UV coordinates from source mesh to target mesh vertices by
/// projecting each target vertex onto the source and interpolating UVs.
#[allow(dead_code)]
pub fn transfer_attributes(
    src_positions: &[[f32; 3]],
    src_uvs: &[[f32; 2]],
    tgt_positions: &[[f32; 3]],
    tgt_triangles: &[[u32; 3]],
) -> Vec<[f32; 2]> {
    tgt_positions
        .iter()
        .map(|&p| {
            if let Some(res) = project_point_to_mesh(
                p,
                src_positions,
                tgt_triangles,
                &ProjectionMode::ClosestPoint,
            ) {
                let fi = res.face_index;
                if fi < tgt_triangles.len() {
                    let tri = &tgt_triangles[fi];
                    let uv_a = src_uvs.get(tri[0] as usize).copied().unwrap_or([0.0; 2]);
                    let uv_b = src_uvs.get(tri[1] as usize).copied().unwrap_or([0.0; 2]);
                    let uv_c = src_uvs.get(tri[2] as usize).copied().unwrap_or([0.0; 2]);
                    interpolate_uv_at_barycentric(res.barycentric, uv_a, uv_b, uv_c)
                } else {
                    [0.0; 2]
                }
            } else {
                [0.0; 2]
            }
        })
        .collect()
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_tri() -> ([f32; 3], [f32; 3], [f32; 3]) {
        ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0])
    }

    // 1
    #[test]
    fn project_point_on_triangle_surface() {
        let (a, b, c) = unit_tri();
        let p = [0.25, 0.25, 0.0];
        let (proj, bary) = project_point_to_triangle(p, a, b, c);
        assert!((proj[0] - 0.25).abs() < 1e-4);
        assert!((proj[1] - 0.25).abs() < 1e-4);
        assert!((bary[0] + bary[1] + bary[2] - 1.0).abs() < 1e-4);
    }

    // 2
    #[test]
    fn project_point_above_triangle() {
        let (a, b, c) = unit_tri();
        let p = [0.25, 0.25, 10.0];
        let (proj, _) = project_point_to_triangle(p, a, b, c);
        assert!(proj[2].abs() < 1e-4); // projected to Z=0
    }

    // 3
    #[test]
    fn compute_barycentric_at_vertex() {
        let (a, b, c) = unit_tri();
        let bary = compute_barycentric(a, a, b, c);
        assert!((bary[0] - 1.0).abs() < 1e-4);
    }

    // 4
    #[test]
    fn barycentric_to_point_roundtrip() {
        let (a, b, c) = unit_tri();
        let p = [0.3, 0.3, 0.0];
        let bary = compute_barycentric(p, a, b, c);
        let q = barycentric_to_point(bary, a, b, c);
        assert!((q[0] - p[0]).abs() < 1e-4);
        assert!((q[1] - p[1]).abs() < 1e-4);
    }

    // 5
    #[test]
    fn interpolate_uv_center() {
        let bary = [1.0f32 / 3.0; 3];
        let uv = interpolate_uv_at_barycentric(bary, [0.0, 0.0], [1.0, 0.0], [0.0, 1.0]);
        assert!((uv[0] - 1.0 / 3.0).abs() < 1e-4);
        assert!((uv[1] - 1.0 / 3.0).abs() < 1e-4);
    }

    // 6
    #[test]
    fn project_point_to_mesh_finds_face() {
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0u32, 1, 2]];
        let result = project_point_to_mesh(
            [0.25, 0.25, 1.0],
            &pos,
            &tris,
            &ProjectionMode::ClosestPoint,
        );
        assert!(result.is_some());
        assert_eq!(result.expect("should succeed").face_index, 0);
    }

    // 7
    #[test]
    fn project_point_to_mesh_empty() {
        let result =
            project_point_to_mesh([0.0, 0.0, 0.0], &[], &[], &ProjectionMode::ClosestPoint);
        assert!(result.is_none());
    }

    // 8
    #[test]
    fn project_mesh_onto_mesh_count() {
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0u32, 1, 2]];
        let src = vec![[0.25f32, 0.25, 1.0]];
        let results = project_mesh_onto_mesh(&src, &pos, &tris);
        assert_eq!(results.len(), 1);
    }

    // 9
    #[test]
    fn shrink_wrap_blend_zero_is_identity() {
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0u32, 1, 2]];
        let src = vec![[0.25f32, 0.25, 1.0]];
        let result = shrink_wrap_proj(&src, &pos, &tris, 0.0);
        assert!((result[0][0] - 0.25).abs() < 1e-5);
        assert!((result[0][2] - 1.0).abs() < 1e-5);
    }

    // 10
    #[test]
    fn shrink_wrap_blend_one_projects_to_surface() {
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0u32, 1, 2]];
        let src = vec![[0.25f32, 0.25, 1.0]];
        let result = shrink_wrap_proj(&src, &pos, &tris, 1.0);
        assert!(result[0][2].abs() < 1e-4); // projected to Z≈0
    }

    // 11
    #[test]
    fn snap_to_surface_snaps_close_vertex() {
        let surf_pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let surf_tris = vec![[0u32, 1, 2]];
        let mut verts = vec![[0.25f32, 0.25, 0.01]];
        let count = snap_to_surface(&mut verts, &surf_pos, &surf_tris, 1.0);
        assert_eq!(count, 1);
        assert!(verts[0][2].abs() < 0.05);
    }

    // 12
    #[test]
    fn snap_to_surface_ignores_far_vertex() {
        let surf_pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let surf_tris = vec![[0u32, 1, 2]];
        let mut verts = vec![[0.25f32, 0.25, 100.0]];
        let count = snap_to_surface(&mut verts, &surf_pos, &surf_tris, 1.0);
        assert_eq!(count, 0);
    }

    // 13
    #[test]
    fn transfer_attributes_returns_one_per_target_vert() {
        let src_pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let src_uvs = vec![[0.0f32, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let tgt_pos = vec![[0.25f32, 0.25, 0.0]];
        let tgt_tris = vec![[0u32, 1, 2]];
        let uvs = transfer_attributes(&src_pos, &src_uvs, &tgt_pos, &tgt_tris);
        assert_eq!(uvs.len(), 1);
    }

    // 14
    #[test]
    fn project_along_axis_returns_same_count() {
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0u32, 1, 2]];
        let out = project_along_axis(&pos, &tris, 2, 0.0);
        assert_eq!(out.len(), pos.len());
    }

    // 15
    #[test]
    fn barycentric_coords_sum_to_one() {
        let (a, b, c) = ([0.0f32, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]);
        let p = [0.5, 0.5, 0.0];
        let bary = compute_barycentric(p, a, b, c);
        assert!((bary[0] + bary[1] + bary[2] - 1.0).abs() < 1e-4);
    }
}
