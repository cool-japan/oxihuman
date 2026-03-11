// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use crate::mesh::MeshBuffers;

// ---------------------------------------------------------------------------
// Internal math helpers
// ---------------------------------------------------------------------------

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        [0.0, 0.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn neg3(v: [f32; 3]) -> [f32; 3] {
    [-v[0], -v[1], -v[2]]
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Axis-aligned 2D bounding box for a decal in world space.
pub struct DecalBounds {
    pub center: [f32; 3],
    pub normal: [f32; 3], // projection direction (normalized)
    pub up: [f32; 3],     // up direction for UV orientation (normalized)
    pub width: f32,       // decal width in world units
    pub height: f32,      // decal height in world units
    pub depth: f32,       // projection depth (how far the box extends behind center)
}

/// A vertex affected by the decal with its computed UV.
pub struct DecalVertex {
    pub vertex_index: usize,
    pub decal_uv: [f32; 2], // UV within the decal [0,1]^2
    pub blend_weight: f32,  // 1.0 at center, fades at edges
}

/// Result of projecting a decal onto a mesh.
pub struct DecalResult {
    pub affected_vertices: Vec<DecalVertex>,
    pub affected_faces: Vec<usize>, // face indices (triangle index)
}

/// Falloff mode for decal weight computation.
pub enum DecalFalloff {
    None,   // weight = 1.0 everywhere
    Linear, // linear fade from center to edge
    Smooth, // smoothstep fade
    Cosine, // cosine fade
}

/// Configuration for decal projection.
pub struct DecalConfig {
    pub falloff_mode: DecalFalloff,
    /// minimum dot(face_normal, decal_normal) to include face (default 0.0)
    pub min_dot: f32,
}

/// Statistics about a projected decal.
pub struct DecalStats {
    pub vertex_count: usize,
    pub face_count: usize,
    pub avg_weight: f32,
    pub coverage_fraction: f32,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Compute the blend weight at normalized UV coordinates (u, v in `[0,1]`).
/// Center is (0.5, 0.5) where weight = 1.0; edges fade according to mode.
pub fn decal_falloff_weight(u: f32, v: f32, mode: &DecalFalloff) -> f32 {
    // Distance from center in normalized [0,1] space; max = 1.0 at corners
    let du = (u - 0.5).abs() * 2.0; // 0 at center, 1 at edge
    let dv = (v - 0.5).abs() * 2.0;
    let d = du.max(dv).clamp(0.0, 1.0); // Chebyshev distance to edge

    match mode {
        DecalFalloff::None => 1.0,
        DecalFalloff::Linear => 1.0 - d,
        DecalFalloff::Smooth => {
            // smoothstep: 3t^2 - 2t^3
            let t = 1.0 - d;
            t * t * (3.0 - 2.0 * t)
        }
        DecalFalloff::Cosine => (0.5 * (1.0 + (std::f32::consts::PI * d).cos())).clamp(0.0, 1.0),
    }
}

/// Project a decal onto a mesh, returning affected vertices with UVs and weights.
pub fn project_decal(
    mesh: &MeshBuffers,
    bounds: &DecalBounds,
    config: &DecalConfig,
) -> DecalResult {
    // Build local frame
    let normal = normalize3(bounds.normal);
    let up_hint = normalize3(bounds.up);
    let right = normalize3(cross3(normal, up_hint));
    let up_actual = normalize3(cross3(right, normal));
    let neg_normal = neg3(normal);

    let half_w = bounds.width * 0.5;
    let half_h = bounds.height * 0.5;

    // Build set of affected vertex indices for fast face lookup
    let mut affected_set = std::collections::HashSet::new();

    let mut affected_vertices: Vec<DecalVertex> = Vec::new();

    for (i, &pos) in mesh.positions.iter().enumerate() {
        let diff = sub3(pos, bounds.center);
        let u_local = dot3(diff, right);
        let v_local = dot3(diff, up_actual);
        let depth_local = dot3(diff, neg_normal);

        if u_local.abs() <= half_w
            && v_local.abs() <= half_h
            && depth_local >= 0.0
            && depth_local <= bounds.depth
        {
            let u = (u_local + half_w) / bounds.width;
            let v = (v_local + half_h) / bounds.height;
            let weight = decal_falloff_weight(u, v, &config.falloff_mode);
            affected_set.insert(i);
            affected_vertices.push(DecalVertex {
                vertex_index: i,
                decal_uv: [u, v],
                blend_weight: weight,
            });
        }
    }

    // Find faces where all 3 verts are in the affected set AND face normal passes min_dot test
    let mut affected_faces: Vec<usize> = Vec::new();
    let n_faces = mesh.indices.len() / 3;
    for fi in 0..n_faces {
        let i0 = mesh.indices[fi * 3] as usize;
        let i1 = mesh.indices[fi * 3 + 1] as usize;
        let i2 = mesh.indices[fi * 3 + 2] as usize;
        if affected_set.contains(&i0) && affected_set.contains(&i1) && affected_set.contains(&i2) {
            // Check face normal dot product
            let p0 = mesh.positions[i0];
            let p1 = mesh.positions[i1];
            let p2 = mesh.positions[i2];
            let e1 = sub3(p1, p0);
            let e2 = sub3(p2, p0);
            let face_n = normalize3(cross3(e1, e2));
            let d = dot3(face_n, normal);
            if d >= config.min_dot {
                affected_faces.push(fi);
            }
        }
    }

    DecalResult {
        affected_vertices,
        affected_faces,
    }
}

/// Return face indices where all 3 vertices are in the affected set.
pub fn affected_faces(mesh: &MeshBuffers, result: &DecalResult) -> Vec<usize> {
    let affected_set: std::collections::HashSet<usize> = result
        .affected_vertices
        .iter()
        .map(|dv| dv.vertex_index)
        .collect();

    let n_faces = mesh.indices.len() / 3;
    let mut faces = Vec::new();
    for fi in 0..n_faces {
        let i0 = mesh.indices[fi * 3] as usize;
        let i1 = mesh.indices[fi * 3 + 1] as usize;
        let i2 = mesh.indices[fi * 3 + 2] as usize;
        if affected_set.contains(&i0) && affected_set.contains(&i1) && affected_set.contains(&i2) {
            faces.push(fi);
        }
    }
    faces
}

/// Compute statistics for a decal projection result.
pub fn decal_stats(result: &DecalResult) -> DecalStats {
    let vertex_count = result.affected_vertices.len();
    let face_count = result.affected_faces.len();
    let avg_weight = if vertex_count == 0 {
        0.0
    } else {
        result
            .affected_vertices
            .iter()
            .map(|dv| dv.blend_weight)
            .sum::<f32>()
            / vertex_count as f32
    };
    // coverage_fraction: fraction of faces affected (unknown total without mesh, use face_count as numerator)
    // Store face_count / face_count = 1.0 if any; caller can compute against total mesh faces separately.
    // Per spec: coverage_fraction is just a ratio field in the struct; we populate based on available info.
    let coverage_fraction = if vertex_count == 0 { 0.0 } else { avg_weight };
    DecalStats {
        vertex_count,
        face_count,
        avg_weight,
        coverage_fraction,
    }
}

/// Blend `decal_rgba` into mesh vertex colors for all affected vertices.
/// Blending: `out = mesh_color * (1 - weight) + decal_rgba * weight`.
/// If mesh has no colors, they are initialized to white `[1,1,1,1]` first.
pub fn apply_decal_colors(
    mesh: &MeshBuffers,
    result: &DecalResult,
    decal_rgba: [f32; 4],
) -> MeshBuffers {
    let n = mesh.positions.len();
    let base_colors: Vec<[f32; 4]> = match &mesh.colors {
        Some(c) => c.clone(),
        None => vec![[1.0, 1.0, 1.0, 1.0]; n],
    };

    let mut out_colors = base_colors;

    for dv in &result.affected_vertices {
        let i = dv.vertex_index;
        if i >= n {
            continue;
        }
        let w = dv.blend_weight;
        let base = out_colors[i];
        out_colors[i] = [
            base[0] * (1.0 - w) + decal_rgba[0] * w,
            base[1] * (1.0 - w) + decal_rgba[1] * w,
            base[2] * (1.0 - w) + decal_rgba[2] * w,
            base[3] * (1.0 - w) + decal_rgba[3] * w,
        ];
    }

    MeshBuffers {
        positions: mesh.positions.clone(),
        normals: mesh.normals.clone(),
        tangents: mesh.tangents.clone(),
        uvs: mesh.uvs.clone(),
        indices: mesh.indices.clone(),
        colors: Some(out_colors),
        has_suit: mesh.has_suit,
    }
}

/// Helper: create a DecalBounds with up=`[0,1,0]` and depth=0.1.
pub fn standard_decal(center: [f32; 3], normal: [f32; 3], size: f32) -> DecalBounds {
    DecalBounds {
        center,
        normal,
        up: [0.0, 1.0, 0.0],
        width: size,
        height: size,
        depth: 0.1,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn make_mesh(positions: Vec<[f32; 3]>, indices: Vec<u32>) -> MeshBuffers {
        let n = positions.len();
        MeshBuffers::from_morph(MB {
            positions,
            normals: vec![[0.0, 0.0, 1.0]; n],
            uvs: vec![[0.0, 0.0]; n],
            indices,
            has_suit: false,
        })
    }

    /// Flat quad in XY plane (z=0), facing +Z.
    fn flat_quad() -> MeshBuffers {
        // 4 verts at corners of a 2x2 quad, 2 triangles
        make_mesh(
            vec![
                [-1.0, -1.0, 0.0],
                [1.0, -1.0, 0.0],
                [1.0, 1.0, 0.0],
                [-1.0, 1.0, 0.0],
            ],
            vec![0, 1, 2, 0, 2, 3],
        )
    }

    fn default_config() -> DecalConfig {
        DecalConfig {
            falloff_mode: DecalFalloff::None,
            min_dot: 0.0,
        }
    }

    // ---- project_decal: vertices inside box ----

    #[test]
    fn project_decal_all_verts_inside() {
        let mesh = flat_quad();
        // Decal covers the whole quad: 2x2 world units, projecting along +Z
        let bounds = DecalBounds {
            center: [0.0, 0.0, 0.1],
            normal: [0.0, 0.0, 1.0],
            up: [0.0, 1.0, 0.0],
            width: 3.0,
            height: 3.0,
            depth: 0.5,
        };
        let result = project_decal(&mesh, &bounds, &default_config());
        assert_eq!(
            result.affected_vertices.len(),
            4,
            "all 4 verts should be inside decal"
        );
    }

    #[test]
    fn project_decal_no_verts_outside_box() {
        let mesh = flat_quad();
        // Tiny decal at center, should not capture corner verts at ±1
        let bounds = DecalBounds {
            center: [0.0, 0.0, 0.1],
            normal: [0.0, 0.0, 1.0],
            up: [0.0, 1.0, 0.0],
            width: 0.5,
            height: 0.5,
            depth: 0.5,
        };
        let result = project_decal(&mesh, &bounds, &default_config());
        assert_eq!(
            result.affected_vertices.len(),
            0,
            "corners at ±1 should be outside 0.5-wide decal"
        );
    }

    #[test]
    fn project_decal_uv_range_valid() {
        let mesh = flat_quad();
        let bounds = DecalBounds {
            center: [0.0, 0.0, 0.1],
            normal: [0.0, 0.0, 1.0],
            up: [0.0, 1.0, 0.0],
            width: 3.0,
            height: 3.0,
            depth: 0.5,
        };
        let result = project_decal(&mesh, &bounds, &default_config());
        for dv in &result.affected_vertices {
            assert!(
                dv.decal_uv[0] >= 0.0 && dv.decal_uv[0] <= 1.0,
                "U out of [0,1]"
            );
            assert!(
                dv.decal_uv[1] >= 0.0 && dv.decal_uv[1] <= 1.0,
                "V out of [0,1]"
            );
        }
    }

    #[test]
    fn project_decal_depth_filter() {
        let mesh = flat_quad();
        // Decal center is at z=10, depth=0.5 → depth_local = dot(pos - center, -normal)
        // For pos.z=0: diff.z = 0 - 10 = -10, depth_local = dot([*, *, -10], [0,0,-1]) = 10 → outside
        let bounds = DecalBounds {
            center: [0.0, 0.0, 10.0],
            normal: [0.0, 0.0, 1.0],
            up: [0.0, 1.0, 0.0],
            width: 3.0,
            height: 3.0,
            depth: 0.5,
        };
        let result = project_decal(&mesh, &bounds, &default_config());
        assert_eq!(
            result.affected_vertices.len(),
            0,
            "vertices should be behind the decal projection depth"
        );
    }

    #[test]
    fn project_decal_faces_populated() {
        let mesh = flat_quad();
        let bounds = DecalBounds {
            center: [0.0, 0.0, 0.1],
            normal: [0.0, 0.0, 1.0],
            up: [0.0, 1.0, 0.0],
            width: 3.0,
            height: 3.0,
            depth: 0.5,
        };
        let result = project_decal(&mesh, &bounds, &default_config());
        assert_eq!(
            result.affected_faces.len(),
            2,
            "both triangles of quad should be affected"
        );
    }

    // ---- decal_falloff_weight ----

    #[test]
    fn falloff_none_always_one() {
        for (u, v) in [(0.0, 0.0), (0.5, 0.5), (1.0, 1.0), (0.3, 0.7)] {
            let w = decal_falloff_weight(u, v, &DecalFalloff::None);
            assert!((w - 1.0).abs() < 1e-6, "None falloff should be 1.0");
        }
    }

    #[test]
    fn falloff_linear_center_is_one() {
        let w = decal_falloff_weight(0.5, 0.5, &DecalFalloff::Linear);
        assert!(
            (w - 1.0).abs() < 1e-6,
            "linear falloff at center should be 1.0"
        );
    }

    #[test]
    fn falloff_linear_edge_is_zero() {
        let w = decal_falloff_weight(0.0, 0.5, &DecalFalloff::Linear);
        assert!(w.abs() < 1e-6, "linear falloff at edge u=0 should be 0.0");
    }

    #[test]
    fn falloff_smooth_center_is_one() {
        let w = decal_falloff_weight(0.5, 0.5, &DecalFalloff::Smooth);
        assert!(
            (w - 1.0).abs() < 1e-6,
            "smooth falloff at center should be 1.0"
        );
    }

    #[test]
    fn falloff_cosine_center_is_one() {
        let w = decal_falloff_weight(0.5, 0.5, &DecalFalloff::Cosine);
        assert!(
            (w - 1.0).abs() < 1e-4,
            "cosine falloff at center should be ~1.0"
        );
    }

    #[test]
    fn falloff_all_modes_monotone_from_center() {
        // At (0.5, 0.5) weight >= weight at (0.5, 0.0) for all modes
        for mode in [
            &DecalFalloff::Linear,
            &DecalFalloff::Smooth,
            &DecalFalloff::Cosine,
        ] {
            let wc = decal_falloff_weight(0.5, 0.5, mode);
            let we = decal_falloff_weight(0.5, 0.0, mode);
            assert!(wc >= we, "center weight should be >= edge weight");
        }
    }

    // ---- affected_faces ----

    #[test]
    fn affected_faces_returns_full_triangles_only() {
        let mesh = flat_quad();
        let bounds = DecalBounds {
            center: [0.0, 0.0, 0.1],
            normal: [0.0, 0.0, 1.0],
            up: [0.0, 1.0, 0.0],
            width: 3.0,
            height: 3.0,
            depth: 0.5,
        };
        let result = project_decal(&mesh, &bounds, &default_config());
        let faces = affected_faces(&mesh, &result);
        assert_eq!(faces.len(), 2);
    }

    // ---- decal_stats ----

    #[test]
    fn decal_stats_empty_result() {
        let result = DecalResult {
            affected_vertices: vec![],
            affected_faces: vec![],
        };
        let stats = decal_stats(&result);
        assert_eq!(stats.vertex_count, 0);
        assert_eq!(stats.face_count, 0);
        assert!((stats.avg_weight).abs() < 1e-6);
    }

    #[test]
    fn decal_stats_correct_counts() {
        let mesh = flat_quad();
        let bounds = DecalBounds {
            center: [0.0, 0.0, 0.1],
            normal: [0.0, 0.0, 1.0],
            up: [0.0, 1.0, 0.0],
            width: 3.0,
            height: 3.0,
            depth: 0.5,
        };
        let result = project_decal(&mesh, &bounds, &default_config());
        let stats = decal_stats(&result);
        assert_eq!(stats.vertex_count, 4);
        assert_eq!(stats.face_count, 2);
        assert!(
            (stats.avg_weight - 1.0).abs() < 1e-6,
            "None falloff → avg_weight=1.0"
        );
    }

    // ---- apply_decal_colors ----

    #[test]
    fn apply_decal_colors_creates_colors_when_none() {
        let mesh = flat_quad();
        assert!(mesh.colors.is_none());
        let bounds = DecalBounds {
            center: [0.0, 0.0, 0.1],
            normal: [0.0, 0.0, 1.0],
            up: [0.0, 1.0, 0.0],
            width: 3.0,
            height: 3.0,
            depth: 0.5,
        };
        let result = project_decal(&mesh, &bounds, &default_config());
        let out = apply_decal_colors(&mesh, &result, [1.0, 0.0, 0.0, 1.0]);
        assert!(out.colors.is_some());
        assert_eq!(out.colors.as_ref().unwrap().len(), 4);
    }

    #[test]
    fn apply_decal_colors_full_weight_replaces_base() {
        // With None falloff, weight=1.0 everywhere → out_color = decal_rgba
        let mesh = flat_quad();
        let bounds = DecalBounds {
            center: [0.0, 0.0, 0.1],
            normal: [0.0, 0.0, 1.0],
            up: [0.0, 1.0, 0.0],
            width: 3.0,
            height: 3.0,
            depth: 0.5,
        };
        let result = project_decal(&mesh, &bounds, &default_config());
        let decal_rgba = [1.0f32, 0.0, 0.5, 0.8];
        let out = apply_decal_colors(&mesh, &result, decal_rgba);
        let colors = out.colors.as_ref().unwrap();
        for c in colors {
            for ch in 0..4 {
                assert!(
                    (c[ch] - decal_rgba[ch]).abs() < 1e-5,
                    "channel {} mismatch",
                    ch
                );
            }
        }
    }

    // ---- standard_decal helper ----

    #[test]
    fn standard_decal_up_is_world_up() {
        let d = standard_decal([0.0, 1.0, 0.0], [0.0, 0.0, 1.0], 0.5);
        assert!((d.up[1] - 1.0).abs() < 1e-6);
        assert!((d.up[0]).abs() < 1e-6);
        assert!((d.up[2]).abs() < 1e-6);
    }

    #[test]
    fn standard_decal_depth_is_point_one() {
        let d = standard_decal([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 1.0);
        assert!((d.depth - 0.1).abs() < 1e-6);
    }

    #[test]
    fn standard_decal_size_maps_to_width_height() {
        let d = standard_decal([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 2.5);
        assert!((d.width - 2.5).abs() < 1e-6);
        assert!((d.height - 2.5).abs() < 1e-6);
    }

    // ---- edge cases ----

    #[test]
    fn empty_mesh_returns_empty_result() {
        let mesh = make_mesh(vec![], vec![]);
        let bounds = standard_decal([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0);
        let result = project_decal(&mesh, &bounds, &default_config());
        assert_eq!(result.affected_vertices.len(), 0);
        assert_eq!(result.affected_faces.len(), 0);
    }

    #[test]
    fn zero_size_decal_no_verts() {
        let mesh = flat_quad();
        let bounds = DecalBounds {
            center: [0.0, 0.0, 0.1],
            normal: [0.0, 0.0, 1.0],
            up: [0.0, 1.0, 0.0],
            width: 0.0,
            height: 0.0,
            depth: 0.5,
        };
        let result = project_decal(&mesh, &bounds, &default_config());
        // width=0 means half_w=0 → |u_local| <= 0 only if u_local==0
        // center verts at x=±1 will not pass
        assert_eq!(result.affected_vertices.len(), 0);
    }
}
