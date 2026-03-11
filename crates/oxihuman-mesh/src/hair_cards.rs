// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

//! Hair card strip mesh generation.
//!
//! Hair cards are rectangular/tapered strip meshes that follow guide curves,
//! used as an efficient real-time hair representation.

use crate::mesh::MeshBuffers;
use crate::mesh_merge::merge_many;
use crate::normals::compute_normals;
use crate::sampling::Lcg;

// ---------------------------------------------------------------------------
// Parameters
// ---------------------------------------------------------------------------

/// Normal computation mode for hair card geometry.
#[derive(Debug, Clone, PartialEq)]
pub enum CardNormalMode {
    /// Normals always face the camera (billboarding — not applicable for static, use mesh normals).
    FacingNormal,
    /// Normals computed from card geometry.
    Geometric,
    /// Normals blended toward scalp surface normal.
    BlendedWithSurface,
}

/// Parameters controlling hair card mesh generation.
#[derive(Debug, Clone)]
pub struct HairCardParams {
    /// Card width at root.
    pub width: f32,
    /// Card width at tip.
    pub tip_width: f32,
    /// Longitudinal segments per card.
    pub segments: usize,
    /// UV tiling along width.
    pub uv_tile_u: f32,
    /// UV tiling along length.
    pub uv_tile_v: f32,
    /// Normal computation mode.
    pub normal_mode: CardNormalMode,
}

impl Default for HairCardParams {
    fn default() -> Self {
        Self {
            width: 0.02,
            tip_width: 0.005,
            segments: 8,
            uv_tile_u: 1.0,
            uv_tile_v: 1.0,
            normal_mode: CardNormalMode::Geometric,
        }
    }
}

// ---------------------------------------------------------------------------
// HairGuide
// ---------------------------------------------------------------------------

/// A single hair guide curve (polyline of 3D control points).
pub struct HairGuide {
    /// Control points along the hair strand.
    pub points: Vec<[f32; 3]>,
    /// Surface normal at root (used for BlendedWithSurface normal mode).
    pub root_normal: [f32; 3],
}

impl HairGuide {
    /// Create a new guide with the given points and default root normal [0, 1, 0].
    pub fn new(points: Vec<[f32; 3]>) -> Self {
        Self {
            points,
            root_normal: [0.0, 1.0, 0.0],
        }
    }

    /// Set the root surface normal.
    pub fn with_root_normal(mut self, normal: [f32; 3]) -> Self {
        self.root_normal = normal;
        self
    }

    /// Compute the total arc length of the guide polyline.
    pub fn length(&self) -> f32 {
        if self.points.len() < 2 {
            return 0.0;
        }
        self.points.windows(2).map(|w| dist3(w[0], w[1])).sum()
    }

    /// Normalized tangent direction at parameter `t` in [0, 1].
    pub fn direction_at(&self, t: f32) -> [f32; 3] {
        let n = self.points.len();
        if n < 2 {
            return [0.0, 1.0, 0.0];
        }
        let t = t.clamp(0.0, 1.0);
        // Number of segments
        let segs = (n - 1) as f32;
        let scaled = t * segs;
        let seg = (scaled as usize).min(n - 2);
        let a = self.points[seg];
        let b = self.points[seg + 1];
        let d = sub3(b, a);
        normalize3(d)
    }

    /// Interpolated position at parameter `t` in [0, 1].
    pub fn point_at(&self, t: f32) -> [f32; 3] {
        let n = self.points.len();
        if n == 0 {
            return [0.0, 0.0, 0.0];
        }
        if n == 1 {
            return self.points[0];
        }
        let t = t.clamp(0.0, 1.0);
        if t >= 1.0 {
            return self.points[self.points.len() - 1];
        }
        let segs = (n - 1) as f32;
        let scaled = t * segs;
        let seg = scaled as usize;
        let seg = seg.min(n - 2);
        let frac = scaled - seg as f32;
        let a = self.points[seg];
        let b = self.points[seg + 1];
        lerp3(a, b, frac)
    }

    /// Number of polyline segments (points - 1).
    pub fn segment_count(&self) -> usize {
        self.points.len().saturating_sub(1)
    }
}

// ---------------------------------------------------------------------------
// Public generation functions
// ---------------------------------------------------------------------------

/// Generate a single hair card mesh from a guide curve.
pub fn hair_card_from_guide(guide: &HairGuide, params: &HairCardParams) -> MeshBuffers {
    let segs = params.segments.max(1);
    let num_rings = segs + 1; // number of vertex rings
    let num_verts = num_rings * 2; // 2 verts per ring (left, right)

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(num_verts);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(num_verts);
    let mut indices: Vec<u32> = Vec::new();

    for i in 0..=segs {
        let t = i as f32 / segs as f32;
        let pos = guide.point_at(t);
        let tangent = guide.direction_at(t);
        let width_at_t = lerp_f32(params.width, params.tip_width, t);

        // Compute right vector: cross(tangent, up), fallback to [1,0,0]
        let up = [0.0f32, 1.0, 0.0];
        let right_raw = cross3(tangent, up);
        let right = if len3(right_raw) < 1e-6 {
            // tangent is nearly parallel to up, fall back to world X
            cross3(tangent, [1.0, 0.0, 0.0])
        } else {
            right_raw
        };
        let right = normalize3(right);

        let half_w = width_at_t * 0.5;

        // Left vertex
        let left = sub3(pos, scale3(right, half_w));
        // Right vertex
        let right_v = add3(pos, scale3(right, half_w));

        let v_coord = t * params.uv_tile_v;

        positions.push(left);
        uvs.push([0.0 * params.uv_tile_u, v_coord]);

        positions.push(right_v);
        uvs.push([1.0 * params.uv_tile_u, v_coord]);
    }

    // Build quad strip triangles
    // Ring i has vertices: (2*i, 2*i+1)
    // Ring i+1 has vertices: (2*(i+1), 2*(i+1)+1)
    // Quad: left0, right0, right1, left1
    // Triangle 1: left0, right1, right0 (flipped for correct winding)
    // Triangle 2: left0, left1, right1
    for i in 0..segs {
        let l0 = (2 * i) as u32;
        let r0 = (2 * i + 1) as u32;
        let l1 = (2 * (i + 1)) as u32;
        let r1 = (2 * (i + 1) + 1) as u32;

        // Triangle 1
        indices.push(l0);
        indices.push(r0);
        indices.push(r1);
        // Triangle 2
        indices.push(l0);
        indices.push(r1);
        indices.push(l1);
    }

    let normals = vec![[0.0f32, 0.0, 1.0]; num_verts];
    let tangents = vec![[1.0f32, 0.0, 0.0, 1.0]; num_verts];

    let mut mesh = MeshBuffers {
        positions,
        normals,
        tangents,
        uvs,
        indices,
        colors: None,
        has_suit: false,
    };

    // Blend normals toward root surface normal if requested
    if matches!(params.normal_mode, CardNormalMode::BlendedWithSurface) {
        let root_n = normalize3(guide.root_normal);
        let n = mesh.normals.len();
        for (idx, norm) in mesh.normals.iter_mut().enumerate() {
            let t = idx as f32 / n.max(1) as f32;
            // At root (t=0) fully surface normal, blend toward geometric at tip
            let blended = lerp3(root_n, *norm, t);
            *norm = normalize3(blended);
        }
    }

    compute_normals(&mut mesh);
    mesh
}

/// Generate multiple hair cards from a collection of guides.
pub fn hair_cards_from_guides(guides: &[HairGuide], params: &HairCardParams) -> MeshBuffers {
    if guides.is_empty() {
        return empty_mesh();
    }
    let cards: Vec<MeshBuffers> = guides
        .iter()
        .map(|g| hair_card_from_guide(g, params))
        .collect();
    merge_many(&cards)
}

/// Generate a simple straight hair card (for testing).
pub fn straight_hair_card(root: [f32; 3], tip: [f32; 3], params: &HairCardParams) -> MeshBuffers {
    let guide = HairGuide::new(vec![root, tip]);
    hair_card_from_guide(&guide, params)
}

/// Generate a curled hair card following a helical path.
pub fn curled_hair_card(
    root: [f32; 3],
    height: f32,
    curl_radius: f32,
    turns: f32,
    params: &HairCardParams,
) -> MeshBuffers {
    let steps = (params.segments * 4).max(16);
    let mut points = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let angle = t * turns * std::f32::consts::TAU;
        let x = root[0] + curl_radius * angle.cos();
        let y = root[1] + height * t;
        let z = root[2] + curl_radius * angle.sin();
        points.push([x, y, z]);
    }
    let guide = HairGuide::new(points);
    hair_card_from_guide(&guide, params)
}

/// Generate guide curves distributed over a mesh surface.
///
/// Uses an LCG seeded with `seed` to pick random surface points.
/// Each guide follows the vertex normal direction for `length`, with 5 points
/// in a slight random curve.
pub fn guides_from_mesh(
    mesh: &MeshBuffers,
    count: usize,
    length: f32,
    seed: u32,
) -> Vec<HairGuide> {
    if mesh.positions.is_empty() || mesh.indices.len() < 3 {
        return Vec::new();
    }

    let mut lcg = Lcg::new(seed as u64);
    let face_count = mesh.indices.len() / 3;
    let mut guides = Vec::with_capacity(count);

    for _ in 0..count {
        // Pick a random face
        let face_idx = (lcg.next_u64() as usize) % face_count;
        let base = face_idx * 3;
        let i0 = mesh.indices[base] as usize;
        let i1 = mesh.indices[base + 1] as usize;
        let i2 = mesh.indices[base + 2] as usize;

        // Random barycentric coords
        let u = (lcg.next_u64() as f32) / u64::MAX as f32;
        let v_max = 1.0 - u;
        let v = (lcg.next_u64() as f32) / u64::MAX as f32 * v_max;
        let w = 1.0 - u - v;

        let p0 = mesh.positions[i0];
        let p1 = mesh.positions[i1];
        let p2 = mesh.positions[i2];
        let root_pos = add3(add3(scale3(p0, u), scale3(p1, v)), scale3(p2, w));

        let n0 = mesh.normals[i0];
        let n1 = mesh.normals[i1];
        let n2 = mesh.normals[i2];
        let root_norm_raw = add3(add3(scale3(n0, u), scale3(n1, v)), scale3(n2, w));
        let root_norm = normalize3(root_norm_raw);

        // Build 5 points along the normal direction with slight random curl
        let num_pts = 5usize;
        let mut points = Vec::with_capacity(num_pts);
        for j in 0..num_pts {
            let t = j as f32 / (num_pts - 1) as f32;
            // Slight random offset perpendicular to normal
            let rand_x = ((lcg.next_u64() as f32) / u64::MAX as f32 - 0.5) * 0.01 * length;
            let rand_z = ((lcg.next_u64() as f32) / u64::MAX as f32 - 0.5) * 0.01 * length;
            let pt = [
                root_pos[0] + root_norm[0] * t * length + rand_x,
                root_pos[1] + root_norm[1] * t * length,
                root_pos[2] + root_norm[2] * t * length + rand_z,
            ];
            points.push(pt);
        }

        guides.push(HairGuide::new(points).with_root_normal(root_norm));
    }

    guides
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn empty_mesh() -> MeshBuffers {
    MeshBuffers {
        positions: Vec::new(),
        normals: Vec::new(),
        tangents: Vec::new(),
        uvs: Vec::new(),
        indices: Vec::new(),
        colors: None,
        has_suit: false,
    }
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
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

fn len3(a: [f32; 3]) -> f32 {
    dot3(a, a).sqrt()
}

fn normalize3(a: [f32; 3]) -> [f32; 3] {
    let l = len3(a);
    if l < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        scale3(a, 1.0 / l)
    }
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    len3(sub3(b, a))
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_mesh() -> MeshBuffers {
        // A simple triangle mesh (floor plane)
        MeshBuffers {
            positions: vec![
                [-1.0, 0.0, -1.0],
                [1.0, 0.0, -1.0],
                [1.0, 0.0, 1.0],
                [-1.0, 0.0, 1.0],
            ],
            normals: vec![
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            indices: vec![0, 1, 2, 0, 2, 3],
            colors: None,
            has_suit: false,
        }
    }

    #[test]
    fn test_hair_guide_new() {
        let pts = vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let guide = HairGuide::new(pts.clone());
        assert_eq!(guide.points.len(), 2);
        assert_eq!(guide.points[0], pts[0]);
        assert_eq!(guide.root_normal, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_hair_guide_length() {
        let guide = HairGuide::new(vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 2.0, 0.0]]);
        let len = guide.length();
        assert!((len - 2.0).abs() < 1e-5, "expected length 2.0, got {}", len);
    }

    #[test]
    fn test_hair_guide_point_at() {
        let guide = HairGuide::new(vec![[0.0, 0.0, 0.0], [0.0, 2.0, 0.0]]);
        let mid = guide.point_at(0.5);
        assert!(
            (mid[1] - 1.0).abs() < 1e-5,
            "midpoint y should be 1.0, got {}",
            mid[1]
        );
        let start = guide.point_at(0.0);
        assert!((start[1] - 0.0).abs() < 1e-5);
        let end = guide.point_at(1.0);
        assert!((end[1] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_hair_guide_direction_at() {
        let guide = HairGuide::new(vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        let dir = guide.direction_at(0.5);
        // Should point in +Y direction
        assert!(
            (dir[1] - 1.0).abs() < 1e-5,
            "direction should be [0,1,0], got {:?}",
            dir
        );
        assert!(dir[0].abs() < 1e-5);
        assert!(dir[2].abs() < 1e-5);
    }

    #[test]
    fn test_straight_hair_card_vertices() {
        let params = HairCardParams::default();
        let mesh = straight_hair_card([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], &params);
        // (segments+1)*2 vertices
        let expected_verts = (params.segments + 1) * 2;
        assert_eq!(
            mesh.positions.len(),
            expected_verts,
            "expected {} vertices, got {}",
            expected_verts,
            mesh.positions.len()
        );
    }

    #[test]
    fn test_straight_hair_card_segments() {
        let params = HairCardParams {
            segments: 4,
            ..Default::default()
        };
        let mesh = straight_hair_card([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], &params);
        // segments quads * 2 triangles each = segments*2 triangles, each 3 indices
        let expected_indices = params.segments * 2 * 3;
        assert_eq!(
            mesh.indices.len(),
            expected_indices,
            "expected {} indices, got {}",
            expected_indices,
            mesh.indices.len()
        );
    }

    #[test]
    fn test_hair_card_uv_coords() {
        let params = HairCardParams::default();
        let mesh = straight_hair_card([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], &params);
        // First ring: u=0 (left), u=1 (right)
        let uv_left_root = mesh.uvs[0];
        let uv_right_root = mesh.uvs[1];
        assert!((uv_left_root[0] - 0.0).abs() < 1e-5, "left u should be 0.0");
        assert!(
            (uv_right_root[0] - 1.0).abs() < 1e-5,
            "right u should be 1.0"
        );
        // Both at root: v=0
        assert!((uv_left_root[1] - 0.0).abs() < 1e-5, "root v should be 0.0");
        // Last ring: v=1
        let last_left = mesh.uvs[mesh.uvs.len() - 2];
        assert!(
            (last_left[1] - 1.0).abs() < 1e-5,
            "tip v should be 1.0, got {}",
            last_left[1]
        );
    }

    #[test]
    fn test_curled_hair_card() {
        let params = HairCardParams::default();
        let mesh = curled_hair_card([0.0, 0.0, 0.0], 1.0, 0.1, 2.0, &params);
        assert!(
            !mesh.positions.is_empty(),
            "curled card should have vertices"
        );
        assert!(!mesh.indices.is_empty(), "curled card should have indices");
        // All indices should be valid
        for &idx in &mesh.indices {
            assert!(
                (idx as usize) < mesh.positions.len(),
                "index {} out of bounds ({})",
                idx,
                mesh.positions.len()
            );
        }
    }

    #[test]
    fn test_hair_cards_from_guides_empty() {
        let params = HairCardParams::default();
        let mesh = hair_cards_from_guides(&[], &params);
        assert_eq!(
            mesh.positions.len(),
            0,
            "empty input should produce empty mesh"
        );
        assert_eq!(mesh.indices.len(), 0);
    }

    #[test]
    fn test_hair_cards_from_guides_multiple() {
        let params = HairCardParams {
            segments: 4,
            ..Default::default()
        };
        let guides = vec![
            HairGuide::new(vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0]]),
            HairGuide::new(vec![[1.0, 0.0, 0.0], [1.0, 1.0, 0.0]]),
            HairGuide::new(vec![[2.0, 0.0, 0.0], [2.0, 1.0, 0.0]]),
        ];
        let mesh = hair_cards_from_guides(&guides, &params);
        let single_verts = (params.segments + 1) * 2;
        assert_eq!(
            mesh.positions.len(),
            3 * single_verts,
            "merged mesh should have 3x single card vertices"
        );
    }

    #[test]
    fn test_guides_from_mesh() {
        let floor = simple_mesh();
        let guides = guides_from_mesh(&floor, 10, 0.1, 42);
        assert_eq!(guides.len(), 10, "should generate exactly 10 guides");
        for g in &guides {
            assert_eq!(g.points.len(), 5, "each guide should have 5 points");
        }
    }

    #[test]
    fn test_hair_card_params_default() {
        let p = HairCardParams::default();
        assert!((p.width - 0.02).abs() < 1e-6);
        assert!((p.tip_width - 0.005).abs() < 1e-6);
        assert_eq!(p.segments, 8);
        assert!((p.uv_tile_u - 1.0).abs() < 1e-6);
        assert!((p.uv_tile_v - 1.0).abs() < 1e-6);
        assert_eq!(p.normal_mode, CardNormalMode::Geometric);
    }

    #[test]
    fn test_hair_guide_segment_count() {
        let g = HairGuide::new(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        assert_eq!(g.segment_count(), 2);
        let single = HairGuide::new(vec![[0.0, 0.0, 0.0]]);
        assert_eq!(single.segment_count(), 0);
    }

    #[test]
    fn test_hair_guide_with_root_normal() {
        let guide = HairGuide::new(vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0]])
            .with_root_normal([0.0, 0.0, 1.0]);
        assert_eq!(guide.root_normal, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_card_normals_valid() {
        let params = HairCardParams::default();
        let mesh = straight_hair_card([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], &params);
        for n in &mesh.normals {
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!(l > 0.5, "normal should be non-zero, got {:?}", n);
        }
    }

    #[test]
    fn test_blended_surface_normal_mode() {
        let params = HairCardParams {
            normal_mode: CardNormalMode::BlendedWithSurface,
            ..Default::default()
        };
        let guide = HairGuide::new(vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0]])
            .with_root_normal([1.0, 0.0, 0.0]);
        let mesh = hair_card_from_guide(&guide, &params);
        assert!(!mesh.normals.is_empty(), "should produce normals");
    }
}
