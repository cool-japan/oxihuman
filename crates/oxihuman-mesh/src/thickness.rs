// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

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
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
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
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l > 1e-10 {
        scale3(v, 1.0 / l)
    } else {
        [0.0, 1.0, 0.0]
    }
}

// ---------------------------------------------------------------------------
// LCG for cone sample generation
// ---------------------------------------------------------------------------

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        let state = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        Self {
            state: if state == 0 { 1 } else { state },
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    fn next_f32(&mut self) -> f32 {
        let bits = self.next_u64();
        (bits >> 40) as f32 / (1u64 << 24) as f32
    }
}

// ---------------------------------------------------------------------------
// ThicknessParams
// ---------------------------------------------------------------------------

/// Parameters for thickness map computation.
#[derive(Debug, Clone)]
pub struct ThicknessParams {
    /// Maximum thickness to sample (rays beyond this are clamped).
    pub max_distance: f32,
    /// Offset ray origin along normal to avoid self-intersection.
    pub ray_offset: f32,
    /// Normalize output to [0, 1].
    pub normalize: bool,
    /// Invert result: 1.0 - thickness.
    pub invert: bool,
    /// Number of rays per vertex for averaging (multi-sample).
    pub sample_count: usize,
}

impl Default for ThicknessParams {
    fn default() -> Self {
        Self {
            max_distance: 2.0,
            ray_offset: 0.001,
            normalize: true,
            invert: false,
            sample_count: 1,
        }
    }
}

// ---------------------------------------------------------------------------
// ThicknessMap
// ---------------------------------------------------------------------------

/// Per-vertex thickness map result.
pub struct ThicknessMap {
    /// Per-vertex thickness values (raw or normalized depending on params).
    pub values: Vec<f32>,
    /// Minimum raw thickness found.
    pub min_thickness: f32,
    /// Maximum raw thickness found.
    pub max_thickness: f32,
    /// Mean raw thickness.
    pub mean_thickness: f32,
    /// Number of vertices.
    pub vertex_count: usize,
}

impl ThicknessMap {
    /// Apply to mesh as vertex colors using a heat map (thin → red, thick → blue).
    pub fn to_vertex_colors(&self) -> Vec<[f32; 4]> {
        self.values
            .iter()
            .map(|&v| {
                let t = self.normalized_at_value(v);
                thickness_to_color(t)
            })
            .collect()
    }

    /// Get normalized value for vertex `i` in [0, 1].
    pub fn normalized_at(&self, i: usize) -> f32 {
        let v = self.values[i];
        self.normalized_at_value(v)
    }

    fn normalized_at_value(&self, v: f32) -> f32 {
        let range = self.max_thickness - self.min_thickness;
        if range < f32::EPSILON {
            return 0.5;
        }
        ((v - self.min_thickness) / range).clamp(0.0, 1.0)
    }
}

/// Map thickness t in [0, 1] to reversed rainbow color (thin=red, thick=blue).
///
/// Uses the reversed rainbow: t=0 → red [1,0,0], t=1 → blue [0,0,1].
fn thickness_to_color(t: f32) -> [f32; 4] {
    let t = t.clamp(0.0, 1.0);
    // Reversed rainbow: t=0 red, t=0.25 yellow, t=0.5 green, t=0.75 cyan, t=1 blue
    let hue = (1.0 - t) * 240.0_f32; // hue in degrees: 240=blue, 0=red
    hsv_to_rgba(hue, 1.0, 1.0)
}

fn hsv_to_rgba(h: f32, s: f32, v: f32) -> [f32; 4] {
    let h = h % 360.0;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r1, g1, b1) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    [r1 + m, g1 + m, b1 + m, 1.0]
}

// ---------------------------------------------------------------------------
// ray_triangle_hit  (Möller-Trumbore)
// ---------------------------------------------------------------------------

/// Ray-triangle intersection returning hit distance (Möller-Trumbore, all hits).
///
/// Returns `Some(t)` where `t > 1e-8`, or `None` if no intersection,
/// parallel, or intersection is behind the origin.
pub fn ray_triangle_hit(
    origin: [f32; 3],
    dir: [f32; 3],
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
) -> Option<f32> {
    let edge1 = sub3(v1, v0);
    let edge2 = sub3(v2, v0);
    let h = cross3(dir, edge2);
    let det = dot3(edge1, h);

    if det.abs() < 1e-8 {
        return None; // parallel
    }

    let f = 1.0 / det;
    let s = sub3(origin, v0);
    let u = f * dot3(s, h);

    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q = cross3(s, edge1);
    let v = f * dot3(dir, q);

    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = f * dot3(edge2, q);
    if t > 1e-8 {
        Some(t)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// ray_mesh_hits
// ---------------------------------------------------------------------------

/// Cast ray through mesh and return all hit distances, sorted ascending.
///
/// Brute-force tests every triangle. Only hits > 0 are returned.
pub fn ray_mesh_hits(
    mesh: &MeshBuffers,
    origin: [f32; 3],
    dir: [f32; 3],
    max_dist: f32,
) -> Vec<f32> {
    let mut hits = Vec::new();
    let tri_count = mesh.indices.len() / 3;

    for tri in 0..tri_count {
        let i0 = mesh.indices[tri * 3] as usize;
        let i1 = mesh.indices[tri * 3 + 1] as usize;
        let i2 = mesh.indices[tri * 3 + 2] as usize;

        if i0 >= mesh.positions.len() || i1 >= mesh.positions.len() || i2 >= mesh.positions.len() {
            continue;
        }

        let v0 = mesh.positions[i0];
        let v1 = mesh.positions[i1];
        let v2 = mesh.positions[i2];

        if let Some(t) = ray_triangle_hit(origin, dir, v0, v1, v2) {
            if t <= max_dist {
                hits.push(t);
            }
        }
    }

    hits.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    hits
}

// ---------------------------------------------------------------------------
// sample_thickness_at
// ---------------------------------------------------------------------------

/// Approximate local thickness at a point using opposite-normal ray casting.
///
/// Offsets origin by `ray_offset` along the normal, then casts in `-normal`
/// direction. Returns the first hit distance, or `params.max_distance` if
/// no hit is found.
pub fn sample_thickness_at(
    mesh: &MeshBuffers,
    pos: [f32; 3],
    normal: [f32; 3],
    params: &ThicknessParams,
) -> f32 {
    let n = normalize3(normal);
    // Offset origin slightly along normal to avoid self-intersection
    let origin = add3(pos, scale3(n, params.ray_offset));
    // Cast in opposite-normal direction
    let dir = scale3(n, -1.0);

    let hits = ray_mesh_hits(mesh, origin, dir, params.max_distance);
    if hits.is_empty() {
        params.max_distance
    } else {
        hits[0]
    }
}

// ---------------------------------------------------------------------------
// cone_samples
// ---------------------------------------------------------------------------

/// Generate `count` directions within `spread_angle` radians of `dir`.
///
/// Uses a simple LCG random perturbation seeded by `seed`. Returns unit vectors.
pub fn cone_samples(dir: [f32; 3], count: usize, spread_angle: f32, seed: u32) -> Vec<[f32; 3]> {
    if count == 0 {
        return Vec::new();
    }

    let mut rng = Lcg::new(seed as u64);
    let n = normalize3(dir);

    // Build orthonormal basis around dir
    let helper = if n[0].abs() < 0.9 {
        [1.0f32, 0.0, 0.0]
    } else {
        [0.0f32, 1.0, 0.0]
    };
    let tangent = normalize3(cross3(helper, n));
    let bitangent = cross3(n, tangent);

    let mut samples = Vec::with_capacity(count);
    for _ in 0..count {
        let r1 = rng.next_f32(); // azimuth: 0..1 → 0..2π
        let r2 = rng.next_f32(); // elevation: 0..1 → 0..spread_angle

        let phi = r1 * 2.0 * std::f32::consts::PI;
        // Uniform in cone cap: theta from 0 to spread_angle
        let cos_max = spread_angle.cos();
        let cos_theta = cos_max + (1.0 - cos_max) * r2;
        let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();

        let x = sin_theta * phi.cos();
        let z = sin_theta * phi.sin();
        let y = cos_theta;

        // Rotate from local frame (+Y = dir axis) to world frame
        let world = add3(add3(scale3(tangent, x), scale3(n, y)), scale3(bitangent, z));
        samples.push(normalize3(world));
    }

    samples
}

// ---------------------------------------------------------------------------
// compute_thickness
// ---------------------------------------------------------------------------

/// Compute thickness map for all mesh vertices.
///
/// For each vertex, casts a ray in the opposite-normal direction to find where
/// the ray exits the mesh. The distance is the local thickness at that vertex.
pub fn compute_thickness(mesh: &MeshBuffers, params: &ThicknessParams) -> ThicknessMap {
    let n_verts = mesh.positions.len();
    if n_verts == 0 {
        return ThicknessMap {
            values: Vec::new(),
            min_thickness: 0.0,
            max_thickness: 0.0,
            mean_thickness: 0.0,
            vertex_count: 0,
        };
    }

    let normals_available = mesh.normals.len() == n_verts;

    let raw_values: Vec<f32> = (0..n_verts)
        .map(|i| {
            let pos = mesh.positions[i];
            let normal = if normals_available {
                mesh.normals[i]
            } else {
                [0.0, 1.0, 0.0]
            };

            if params.sample_count <= 1 {
                sample_thickness_at(mesh, pos, normal, params)
            } else {
                // Multi-sample: average over cone
                let dirs = cone_samples(
                    scale3(normalize3(normal), -1.0),
                    params.sample_count,
                    0.3, // ~17 degree spread
                    i as u32,
                );
                if dirs.is_empty() {
                    sample_thickness_at(mesh, pos, normal, params)
                } else {
                    let sum: f32 = dirs
                        .iter()
                        .map(|&d| {
                            let n = normalize3(normal);
                            let origin = add3(pos, scale3(n, params.ray_offset));
                            let hits = ray_mesh_hits(mesh, origin, d, params.max_distance);
                            if hits.is_empty() {
                                params.max_distance
                            } else {
                                hits[0]
                            }
                        })
                        .sum();
                    sum / params.sample_count as f32
                }
            }
        })
        .collect();

    // Compute stats
    let min_thickness = raw_values.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_thickness = raw_values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mean_thickness = raw_values.iter().sum::<f32>() / n_verts as f32;

    // Normalize or invert if requested
    let values: Vec<f32> = raw_values
        .iter()
        .map(|&v| {
            let mut out = v;
            if params.normalize {
                let range = max_thickness - min_thickness;
                if range > f32::EPSILON {
                    out = (out - min_thickness) / range;
                } else {
                    out = 0.5;
                }
            }
            if params.invert {
                out = 1.0 - out;
            }
            out
        })
        .collect();

    ThicknessMap {
        values,
        min_thickness,
        max_thickness,
        mean_thickness,
        vertex_count: n_verts,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    /// Build a simple box-like mesh: two parallel triangles forming a slab.
    /// Front face at z=0, back face at z=1, normal pointing +Z.
    fn slab_mesh() -> MeshBuffers {
        // Front triangle (z=0), normal +Z
        // Back triangle (z=1), normal -Z (inverted winding so ray can hit from front)
        let positions = vec![
            // Front face
            [-1.0f32, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [0.0, 1.0, 0.0],
            // Back face
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        let normals = vec![
            [0.0f32, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
        ];
        // Front face: counter-clockwise from +Z view → normal is +Z
        // Back face: reversed winding so ray from front (-Z dir) hits it
        let indices = vec![0u32, 1, 2, 3, 5, 4];

        MeshBuffers::from_morph(MB {
            positions,
            normals,
            uvs: vec![[0.0, 0.0]; 6],
            indices,
            has_suit: false,
        })
    }

    fn triangle_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0f32, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0u32, 1, 2],
            has_suit: false,
        })
    }

    #[test]
    fn test_ray_triangle_hit_basic() {
        // Triangle in XY plane at z=0
        let v0 = [0.0f32, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.0, 1.0, 0.0];
        // Ray from above (z=1) pointing down (-z)
        let origin = [0.2f32, 0.2, 1.0];
        let dir = [0.0f32, 0.0, -1.0];
        let hit = ray_triangle_hit(origin, dir, v0, v1, v2);
        assert!(hit.is_some(), "expected hit");
        let t = hit.expect("should succeed");
        assert!((t - 1.0).abs() < 1e-5, "expected t~1.0, got {t}");
    }

    #[test]
    fn test_ray_triangle_miss_parallel() {
        let v0 = [0.0f32, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.0, 1.0, 0.0];
        // Ray parallel to triangle plane
        let origin = [0.2f32, 0.2, 0.0];
        let dir = [1.0f32, 0.0, 0.0];
        let hit = ray_triangle_hit(origin, dir, v0, v1, v2);
        assert!(hit.is_none(), "expected no hit for parallel ray");
    }

    #[test]
    fn test_ray_triangle_miss_behind() {
        let v0 = [0.0f32, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.0, 1.0, 0.0];
        // Ray pointing away from triangle
        let origin = [0.2f32, 0.2, 1.0];
        let dir = [0.0f32, 0.0, 1.0]; // pointing away (+z, triangle at z=0)
        let hit = ray_triangle_hit(origin, dir, v0, v1, v2);
        assert!(hit.is_none(), "expected no hit for ray pointing away");
    }

    #[test]
    fn test_ray_triangle_miss_outside() {
        let v0 = [0.0f32, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.0, 1.0, 0.0];
        // Ray misses triangle (hits plane but outside triangle)
        let origin = [2.0f32, 2.0, 1.0];
        let dir = [0.0f32, 0.0, -1.0];
        let hit = ray_triangle_hit(origin, dir, v0, v1, v2);
        assert!(hit.is_none(), "expected no hit outside triangle bounds");
    }

    #[test]
    fn test_ray_mesh_hits_simple() {
        let mesh = slab_mesh();
        // Origin in front of front face, shoot in -Z direction
        let origin = [0.0f32, 0.0, -0.5];
        let dir = [0.0f32, 0.0, 1.0];
        let hits = ray_mesh_hits(&mesh, origin, dir, 10.0);
        // Should hit the front face (z=0) at t~0.5, and possibly back face (z=1) at t~1.5
        assert!(!hits.is_empty(), "expected at least one hit");
        assert!(hits[0] > 0.0, "hit distance should be positive");
    }

    #[test]
    fn test_ray_mesh_hits_empty() {
        let mesh = triangle_mesh();
        // Ray that misses completely
        let origin = [10.0f32, 10.0, 1.0];
        let dir = [0.0f32, 0.0, -1.0];
        let hits = ray_mesh_hits(&mesh, origin, dir, 100.0);
        assert!(
            hits.is_empty(),
            "expected no hits for ray missing all triangles"
        );
    }

    #[test]
    fn test_sample_thickness_at() {
        let mesh = slab_mesh();
        let params = ThicknessParams {
            max_distance: 5.0,
            ray_offset: 0.001,
            normalize: false,
            invert: false,
            sample_count: 1,
        };
        // Sample from a point on the front face, normal +Z → ray goes -Z → hits back face
        let pos = [0.0f32, 0.0, 0.0];
        let normal = [0.0f32, 0.0, 1.0];
        let thickness = sample_thickness_at(&mesh, pos, normal, &params);
        // Should be approximately 1.0 (distance from z=0 to z=1)
        assert!(
            thickness > 0.0,
            "thickness should be positive, got {thickness}"
        );
        assert!(
            thickness <= params.max_distance,
            "thickness should not exceed max_distance"
        );
    }

    #[test]
    fn test_compute_thickness_basic() {
        let mesh = slab_mesh();
        let params = ThicknessParams {
            normalize: false,
            invert: false,
            ..Default::default()
        };
        let map = compute_thickness(&mesh, &params);
        assert_eq!(map.vertex_count, 6);
        assert_eq!(map.values.len(), 6);
        assert!(map.min_thickness >= 0.0);
        assert!(map.max_thickness >= map.min_thickness);
    }

    #[test]
    fn test_thickness_map_normalized() {
        let mesh = slab_mesh();
        let params = ThicknessParams {
            normalize: true,
            invert: false,
            max_distance: 5.0,
            ..Default::default()
        };
        let map = compute_thickness(&mesh, &params);
        for &v in &map.values {
            assert!(
                (0.0..=1.0).contains(&v),
                "normalized value out of range: {v}"
            );
        }
    }

    #[test]
    fn test_thickness_map_to_colors() {
        let mesh = slab_mesh();
        let params = ThicknessParams::default();
        let map = compute_thickness(&mesh, &params);
        let colors = map.to_vertex_colors();
        assert_eq!(colors.len(), map.vertex_count);
        for c in &colors {
            for &ch in c.iter() {
                assert!(
                    (0.0..=1.0).contains(&ch),
                    "color channel out of [0,1]: {ch}"
                );
            }
            assert!((c[3] - 1.0).abs() < 1e-6, "alpha should be 1.0");
        }
    }

    #[test]
    fn test_cone_samples_count() {
        let dir = [0.0f32, 0.0, -1.0];
        let samples = cone_samples(dir, 16, 0.3, 42);
        assert_eq!(samples.len(), 16);

        // Zero count returns empty
        let empty = cone_samples(dir, 0, 0.3, 42);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_cone_samples_angle() {
        let dir = [0.0f32, 1.0, 0.0];
        let spread = 0.3f32; // radians
        let samples = cone_samples(dir, 32, spread, 123);

        let n = normalize3(dir);
        for s in &samples {
            let s_norm = normalize3(*s);
            let cos_angle = dot3(n, s_norm).clamp(-1.0, 1.0);
            let angle = cos_angle.acos();
            assert!(
                angle <= spread + 1e-5,
                "sample angle {angle} exceeds spread {spread}"
            );
        }
    }

    #[test]
    fn test_thickness_params_default() {
        let p = ThicknessParams::default();
        assert!((p.max_distance - 2.0).abs() < f32::EPSILON);
        assert!((p.ray_offset - 0.001).abs() < f32::EPSILON);
        assert!(p.normalize);
        assert!(!p.invert);
        assert_eq!(p.sample_count, 1);
    }
}
