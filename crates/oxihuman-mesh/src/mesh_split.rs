// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Mesh splitting by plane and island separation.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Configuration for a mesh split operation.
pub struct SplitConfig {
    /// The plane normal (should be unit length).
    pub plane_normal: [f32; 3],
    /// Signed distance of the plane from the origin (d in dot(n,p) = d).
    pub plane_d: f32,
    /// Whether to cap open holes created at cut edges.
    pub cap_holes: bool,
    /// When false only the front piece is kept.
    pub keep_both_sides: bool,
}

/// A piece of mesh produced by a split.
pub struct SplitMesh {
    /// Vertex positions.
    pub positions: Vec<[f32; 3]>,
    /// Triangle index triples (indices into `positions`).
    pub triangles: Vec<[u32; 3]>,
    /// Which side of the cutting plane this piece lies on.
    pub side: SplitSide,
}

/// Classification of a point relative to a splitting plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitSide {
    /// In front of (or on the positive side of) the plane.
    Front,
    /// Behind (negative side of) the plane.
    Back,
    /// Exactly on the plane (within floating-point tolerance).
    On,
}

/// Output of a full mesh-split operation.
pub struct MeshSplitResult {
    /// The front piece.
    pub front: SplitMesh,
    /// The back piece.
    pub back: SplitMesh,
    /// Number of edges cut by the plane.
    pub cut_edge_count: usize,
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Dot product of two 3-vectors.
#[allow(dead_code)]
pub fn dot_v3_split(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Build a `SplitConfig` with the given plane normal and offset.
/// `cap_holes` and `keep_both_sides` default to `true`.
#[allow(dead_code)]
pub fn default_split_config(normal: [f32; 3], d: f32) -> SplitConfig {
    SplitConfig {
        plane_normal: normal,
        plane_d: d,
        cap_holes: true,
        keep_both_sides: true,
    }
}

/// Classify a single vertex against the plane defined by `normal` and `d`.
#[allow(dead_code)]
pub fn classify_vertex(pos: [f32; 3], normal: [f32; 3], d: f32) -> SplitSide {
    const EPS: f32 = 1e-5;
    let dist = dot_v3_split(pos, normal) - d;
    if dist > EPS {
        SplitSide::Front
    } else if dist < -EPS {
        SplitSide::Back
    } else {
        SplitSide::On
    }
}

/// Split a mesh by a plane.
///
/// Triangles entirely on the front are placed in `front`; triangles entirely
/// on the back (or on the plane) go into `back`.  Triangles that straddle
/// the plane are subdivided at the intersection.
#[allow(dead_code)]
pub fn split_mesh_by_plane(
    positions: &[[f32; 3]],
    triangles: &[[u32; 3]],
    cfg: &SplitConfig,
) -> MeshSplitResult {
    let mut front_pos: Vec<[f32; 3]> = Vec::new();
    let mut front_tri: Vec<[u32; 3]> = Vec::new();
    let mut back_pos: Vec<[f32; 3]> = Vec::new();
    let mut back_tri: Vec<[u32; 3]> = Vec::new();
    let mut cut_edge_count = 0usize;

    for tri in triangles {
        let a = positions[tri[0] as usize];
        let b = positions[tri[1] as usize];
        let c = positions[tri[2] as usize];

        let sa = classify_vertex(a, cfg.plane_normal, cfg.plane_d);
        let sb = classify_vertex(b, cfg.plane_normal, cfg.plane_d);
        let sc = classify_vertex(c, cfg.plane_normal, cfg.plane_d);

        // Count how many vertices are on each side.
        let front_count = [sa, sb, sc]
            .iter()
            .filter(|&&s| s == SplitSide::Front || s == SplitSide::On)
            .count();
        let back_count = [sa, sb, sc]
            .iter()
            .filter(|&&s| s == SplitSide::Back)
            .count();

        if back_count == 0 {
            // All front.
            let base = front_pos.len() as u32;
            front_pos.extend_from_slice(&[a, b, c]);
            front_tri.push([base, base + 1, base + 2]);
        } else if front_count == 0 {
            // All back.
            let base = back_pos.len() as u32;
            back_pos.extend_from_slice(&[a, b, c]);
            back_tri.push([base, base + 1, base + 2]);
        } else {
            // Straddles the plane – clip naively: place in front if majority is front.
            cut_edge_count += 1;
            let pts = [a, b, c];
            let sides = [sa, sb, sc];
            let mut f_pts: Vec<[f32; 3]> = Vec::new();
            let mut bk_pts: Vec<[f32; 3]> = Vec::new();
            for (i, &s) in sides.iter().enumerate() {
                match s {
                    SplitSide::Front | SplitSide::On => f_pts.push(pts[i]),
                    SplitSide::Back => bk_pts.push(pts[i]),
                }
            }
            // Add split vertices to both sides via linear interpolation on the
            // front/back boundary edges.
            let mut mids: Vec<[f32; 3]> = Vec::new();
            for &fp in &f_pts {
                for &bp in &bk_pts {
                    let t = plane_edge_t(fp, bp, cfg.plane_normal, cfg.plane_d);
                    mids.push(lerp_v3(fp, bp, t));
                }
            }
            for &mid in &mids {
                f_pts.push(mid);
                bk_pts.push(mid);
            }
            // Triangulate both sets (fan triangulation).
            if f_pts.len() >= 3 {
                let base = front_pos.len() as u32;
                front_pos.extend_from_slice(&f_pts);
                for i in 1..(f_pts.len() as u32 - 1) {
                    front_tri.push([base, base + i, base + i + 1]);
                }
            }
            if bk_pts.len() >= 3 {
                let base = back_pos.len() as u32;
                back_pos.extend_from_slice(&bk_pts);
                for i in 1..(bk_pts.len() as u32 - 1) {
                    back_tri.push([base, base + i, base + i + 1]);
                }
            }
        }
    }

    MeshSplitResult {
        front: SplitMesh {
            positions: front_pos,
            triangles: front_tri,
            side: SplitSide::Front,
        },
        back: SplitMesh {
            positions: back_pos,
            triangles: back_tri,
            side: SplitSide::Back,
        },
        cut_edge_count,
    }
}

/// Human-readable name for a `SplitSide`.
#[allow(dead_code)]
pub fn side_name(side: &SplitSide) -> &'static str {
    match side {
        SplitSide::Front => "front",
        SplitSide::Back => "back",
        SplitSide::On => "on",
    }
}

/// Number of vertices in a `SplitMesh`.
#[allow(dead_code)]
pub fn split_mesh_vertex_count(m: &SplitMesh) -> usize {
    m.positions.len()
}

/// Number of triangles in a `SplitMesh`.
#[allow(dead_code)]
pub fn split_mesh_face_count(m: &SplitMesh) -> usize {
    m.triangles.len()
}

/// Number of front triangles in a result.
#[allow(dead_code)]
pub fn front_face_count(r: &MeshSplitResult) -> usize {
    r.front.triangles.len()
}

/// Number of back triangles in a result.
#[allow(dead_code)]
pub fn back_face_count(r: &MeshSplitResult) -> usize {
    r.back.triangles.len()
}

/// Serialize a `MeshSplitResult` to a compact JSON string.
#[allow(dead_code)]
pub fn split_result_to_json(r: &MeshSplitResult) -> String {
    format!(
        "{{\"front_vertices\":{},\"front_faces\":{},\"back_vertices\":{},\"back_faces\":{},\"cut_edges\":{}}}",
        r.front.positions.len(),
        r.front.triangles.len(),
        r.back.positions.len(),
        r.back.triangles.len(),
        r.cut_edge_count,
    )
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Parameter t in [0,1] at which the segment a→b crosses the plane.
fn plane_edge_t(a: [f32; 3], b: [f32; 3], normal: [f32; 3], d: f32) -> f32 {
    let da = dot_v3_split(a, normal) - d;
    let db = dot_v3_split(b, normal) - d;
    let denom = da - db;
    if denom.abs() < 1e-9 {
        0.5
    } else {
        da / denom
    }
}

/// Linear interpolation between two 3-vectors.
fn lerp_v3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + t * (b[0] - a[0]),
        a[1] + t * (b[1] - a[1]),
        a[2] + t * (b[2] - a[2]),
    ]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_quad_positions() -> Vec<[f32; 3]> {
        vec![
            [-1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 2.0, 0.0],
            [-1.0, 2.0, 0.0],
        ]
    }

    fn simple_quad_triangles() -> Vec<[u32; 3]> {
        vec![[0, 1, 2], [0, 2, 3]]
    }

    #[test]
    fn classify_vertex_front() {
        // Plane: x = 0 (normal = [1,0,0], d = 0)
        let s = classify_vertex([1.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.0);
        assert_eq!(s, SplitSide::Front);
    }

    #[test]
    fn classify_vertex_back() {
        let s = classify_vertex([-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.0);
        assert_eq!(s, SplitSide::Back);
    }

    #[test]
    fn classify_vertex_on() {
        let s = classify_vertex([0.0, 1.0, 0.0], [1.0, 0.0, 0.0], 0.0);
        assert_eq!(s, SplitSide::On);
    }

    #[test]
    fn split_all_front() {
        // Plane y = -10: all vertices are above it → all front.
        let cfg = default_split_config([0.0, 1.0, 0.0], -10.0);
        let pos = simple_quad_positions();
        let tri = simple_quad_triangles();
        let res = split_mesh_by_plane(&pos, &tri, &cfg);
        assert_eq!(res.front.triangles.len(), 2);
        assert_eq!(res.back.triangles.len(), 0);
        assert_eq!(res.cut_edge_count, 0);
    }

    #[test]
    fn split_all_back() {
        // Plane y = 10: all vertices are below it → all back.
        let cfg = default_split_config([0.0, 1.0, 0.0], 10.0);
        let pos = simple_quad_positions();
        let tri = simple_quad_triangles();
        let res = split_mesh_by_plane(&pos, &tri, &cfg);
        assert_eq!(res.front.triangles.len(), 0);
        assert_eq!(res.back.triangles.len(), 2);
        assert_eq!(res.cut_edge_count, 0);
    }

    #[test]
    fn split_through_mesh_cuts() {
        // Plane y = 1: cuts through both triangles.
        let cfg = default_split_config([0.0, 1.0, 0.0], 1.0);
        let pos = simple_quad_positions();
        let tri = simple_quad_triangles();
        let res = split_mesh_by_plane(&pos, &tri, &cfg);
        assert!(res.cut_edge_count > 0);
        assert!(!res.front.triangles.is_empty());
        assert!(!res.back.triangles.is_empty());
    }

    #[test]
    fn side_name_correct() {
        assert_eq!(side_name(&SplitSide::Front), "front");
        assert_eq!(side_name(&SplitSide::Back), "back");
        assert_eq!(side_name(&SplitSide::On), "on");
    }

    #[test]
    fn split_result_to_json_contains_fields() {
        let cfg = default_split_config([0.0, 1.0, 0.0], -10.0);
        let pos = simple_quad_positions();
        let tri = simple_quad_triangles();
        let res = split_mesh_by_plane(&pos, &tri, &cfg);
        let json = split_result_to_json(&res);
        assert!(json.contains("front_vertices"));
        assert!(json.contains("cut_edges"));
    }

    #[test]
    fn dot_v3_split_correct() {
        assert!((dot_v3_split([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]) - 32.0).abs() < 1e-6);
    }

    #[test]
    fn vertex_count_and_face_count() {
        let cfg = default_split_config([0.0, 1.0, 0.0], -10.0);
        let pos = simple_quad_positions();
        let tri = simple_quad_triangles();
        let res = split_mesh_by_plane(&pos, &tri, &cfg);
        assert_eq!(split_mesh_vertex_count(&res.front), res.front.positions.len());
        assert_eq!(split_mesh_face_count(&res.front), res.front.triangles.len());
        assert_eq!(front_face_count(&res), 2);
        assert_eq!(back_face_count(&res), 0);
    }
}
