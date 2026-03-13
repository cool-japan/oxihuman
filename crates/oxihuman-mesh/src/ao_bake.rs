// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

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
// Simple LCG (same constants as sampling.rs — no external deps)
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
// AoBakeConfig
// ---------------------------------------------------------------------------

/// Configuration for AO baking.
#[derive(Debug, Clone)]
pub struct AoBakeConfig {
    /// Number of hemisphere samples per vertex.
    pub sample_count: usize,
    /// Maximum ray distance for occlusion testing.
    pub max_distance: f32,
    /// Self-occlusion bias (offset from surface to avoid self-intersection).
    pub bias: f32,
    /// Random seed for sample generation.
    pub seed: u64,
}

impl Default for AoBakeConfig {
    fn default() -> Self {
        Self {
            sample_count: 64,
            max_distance: 10.0,
            bias: 1e-3,
            seed: 42,
        }
    }
}

impl AoBakeConfig {
    /// Fast preset: fewer samples, lower quality.
    pub fn fast() -> Self {
        Self {
            sample_count: 32,
            max_distance: 10.0,
            bias: 1e-3,
            seed: 42,
        }
    }

    /// Quality preset: more samples, higher quality.
    pub fn quality() -> Self {
        Self {
            sample_count: 128,
            max_distance: 10.0,
            bias: 1e-3,
            seed: 42,
        }
    }
}

// ---------------------------------------------------------------------------
// hemisphere_samples
// ---------------------------------------------------------------------------

/// Generate hemisphere sample directions using cosine-weighted distribution.
///
/// Returns N unit vectors in the hemisphere around `(0, 1, 0)`.
/// Uses spherical coordinates: θ = arccos(sqrt(1-r1)), φ = 2π·r2.
/// Sample = [sin(θ)·cos(φ), cos(θ), sin(θ)·sin(φ)].
pub fn hemisphere_samples(n: usize, seed: u64) -> Vec<[f32; 3]> {
    let mut rng = Lcg::new(seed);
    let mut samples = Vec::with_capacity(n);
    for _ in 0..n {
        let r1 = rng.next_f32();
        let r2 = rng.next_f32();
        // Cosine-weighted: theta = arccos(sqrt(1 - r1))
        let cos_theta = (1.0 - r1).sqrt();
        let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
        let phi = 2.0 * std::f32::consts::PI * r2;
        let x = sin_theta * phi.cos();
        let y = cos_theta; // hemisphere axis is +Y
        let z = sin_theta * phi.sin();
        samples.push([x, y, z]);
    }
    samples
}

// ---------------------------------------------------------------------------
// tangent_to_world
// ---------------------------------------------------------------------------

/// Transform a sample direction from tangent space (hemisphere around +Y)
/// to world space (hemisphere around `normal`).
pub fn tangent_to_world(normal: [f32; 3], sample: [f32; 3]) -> [f32; 3] {
    let n = normalize3(normal);
    // Build an orthonormal basis (tangent, bitangent, normal).
    // Choose a helper vector not parallel to n.
    let helper = if n[0].abs() < 0.9 {
        [1.0f32, 0.0, 0.0]
    } else {
        [0.0f32, 1.0, 0.0]
    };
    let tangent = normalize3(cross3(helper, n));
    let bitangent = cross3(n, tangent);

    // Rotate sample from tangent space to world space:
    // world = sample.x * tangent + sample.y * normal + sample.z * bitangent
    let [sx, sy, sz] = sample;
    let v = add3(
        add3(scale3(tangent, sx), scale3(n, sy)),
        scale3(bitangent, sz),
    );
    normalize3(v)
}

// ---------------------------------------------------------------------------
// ray_triangle_intersect  (Möller-Trumbore)
// ---------------------------------------------------------------------------

/// Ray-triangle intersection test using the Möller-Trumbore algorithm.
///
/// Returns `Some(t)` where `t` is the distance along the ray, or `None` if
/// there is no intersection (including back-faces and parallel rays).
pub fn ray_triangle_intersect(
    ray_origin: [f32; 3],
    ray_dir: [f32; 3],
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
) -> Option<f32> {
    const EPSILON: f32 = 1e-8;
    let e1 = sub3(v1, v0);
    let e2 = sub3(v2, v0);
    let h = cross3(ray_dir, e2);
    let det = dot3(e1, h);
    if det.abs() < EPSILON {
        return None; // Ray is parallel to the triangle.
    }
    let inv_det = 1.0 / det;
    let s = sub3(ray_origin, v0);
    let u = inv_det * dot3(s, h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross3(s, e1);
    let v = inv_det * dot3(ray_dir, q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = inv_det * dot3(e2, q);
    if t > EPSILON {
        Some(t)
    } else {
        None // Intersection is behind the ray origin.
    }
}

// ---------------------------------------------------------------------------
// ray_hits_mesh
// ---------------------------------------------------------------------------

/// Check if a ray hits any triangle in the mesh within `max_dist`.
pub fn ray_hits_mesh(
    mesh: &MeshBuffers,
    ray_origin: [f32; 3],
    ray_dir: [f32; 3],
    max_dist: f32,
) -> bool {
    let fc = mesh.face_count();
    for fi in 0..fc {
        let base = fi * 3;
        let v0 = mesh.positions[mesh.indices[base] as usize];
        let v1 = mesh.positions[mesh.indices[base + 1] as usize];
        let v2 = mesh.positions[mesh.indices[base + 2] as usize];
        if let Some(t) = ray_triangle_intersect(ray_origin, ray_dir, v0, v1, v2) {
            if t < max_dist {
                return true;
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// bake_vertex_ao
// ---------------------------------------------------------------------------

/// Bake per-vertex ambient occlusion values.
///
/// Returns `Vec<f32>` in `[0, 1]` where `0` = fully occluded and `1` = fully lit.
///
/// **Algorithm**:
/// 1. Generate `config.sample_count` cosine-weighted hemisphere samples.
/// 2. For each vertex `v` with normal `N`:
///    - For each sample: cast ray from `v + bias*N` in world-space sample direction.
///    - Count hits against the mesh.
///    - `AO = 1 - (hits / sample_count)`.
#[allow(clippy::too_many_arguments)]
pub fn bake_vertex_ao(mesh: &MeshBuffers, config: &AoBakeConfig) -> Vec<f32> {
    let vc = mesh.vertex_count();
    if vc == 0 || config.sample_count == 0 {
        return vec![1.0f32; vc];
    }

    let samples_ts = hemisphere_samples(config.sample_count, config.seed);
    let mut ao = Vec::with_capacity(vc);

    for vi in 0..vc {
        let pos = mesh.positions[vi];
        let nor = mesh.normals[vi];
        let origin = add3(pos, scale3(nor, config.bias));

        let mut hits = 0usize;
        for &sample_ts in &samples_ts {
            let dir = tangent_to_world(nor, sample_ts);
            if ray_hits_mesh(mesh, origin, dir, config.max_distance) {
                hits += 1;
            }
        }
        let occ = hits as f32 / config.sample_count as f32;
        ao.push((1.0 - occ).clamp(0.0, 1.0));
    }
    ao
}

// ---------------------------------------------------------------------------
// ao_to_vertex_colors
// ---------------------------------------------------------------------------

/// Convert per-vertex AO values to vertex colors (grayscale).
///
/// Returns `Vec<[u8; 3]>` RGB where dark = occluded, light = open.
pub fn ao_to_vertex_colors(ao_values: &[f32]) -> Vec<[u8; 3]> {
    ao_values
        .iter()
        .map(|&v| {
            let c = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
            [c, c, c]
        })
        .collect()
}

// ---------------------------------------------------------------------------
// fast_vertex_ao
// ---------------------------------------------------------------------------

/// Simple per-vertex AO based on normal deviation from neighbors (fast approximation).
///
/// No ray casting — measures how "sheltered" a vertex is by its geometry.
/// `AO ≈ clamp(dot(N, avg_neighbor_N) * 0.5 + 0.5, 0, 1)`.
pub fn fast_vertex_ao(mesh: &MeshBuffers) -> Vec<f32> {
    let vc = mesh.vertex_count();
    if vc == 0 {
        return Vec::new();
    }

    // Accumulate neighbor normals for each vertex.
    let mut accum: Vec<[f32; 3]> = vec![[0.0f32; 3]; vc];
    let mut counts: Vec<u32> = vec![0u32; vc];

    let fc = mesh.face_count();
    for fi in 0..fc {
        let base = fi * 3;
        let i0 = mesh.indices[base] as usize;
        let i1 = mesh.indices[base + 1] as usize;
        let i2 = mesh.indices[base + 2] as usize;

        // Each vertex accumulates its triangle neighbors' normals (not itself).
        for &(vi, na, nb) in &[(i0, i1, i2), (i1, i0, i2), (i2, i0, i1)] {
            let na_n = mesh.normals[na];
            let nb_n = mesh.normals[nb];
            accum[vi] = add3(accum[vi], add3(na_n, nb_n));
            counts[vi] += 2;
        }
    }

    let mut ao = Vec::with_capacity(vc);
    for vi in 0..vc {
        let n = normalize3(mesh.normals[vi]);
        let avg_neighbor = if counts[vi] > 0 {
            normalize3(accum[vi])
        } else {
            n
        };
        let d = dot3(n, avg_neighbor);
        let v = (d * 0.5 + 0.5).clamp(0.0, 1.0);
        ao.push(v);
    }
    ao
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Test mesh helpers
    // -----------------------------------------------------------------------

    /// Single triangle in the XY plane, normal pointing +Z.
    fn single_triangle_mesh() -> MeshBuffers {
        MeshBuffers {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            tangents: vec![],
            colors: None,
            indices: vec![0, 1, 2],
            has_suit: false,
        }
    }

    /// Tetrahedron: 4 vertices, 4 triangular faces.
    fn tetrahedron_mesh() -> MeshBuffers {
        // Vertices of a regular tetrahedron centred near origin.
        let positions = vec![
            [1.0f32, 1.0, 1.0],
            [-1.0, -1.0, 1.0],
            [-1.0, 1.0, -1.0],
            [1.0, -1.0, -1.0],
        ];
        // Approximate outward normals (normalised centroid directions).
        let normals: Vec<[f32; 3]> = positions
            .iter()
            .map(|&p| {
                let l = len3(p);
                if l > 0.0 {
                    [p[0] / l, p[1] / l, p[2] / l]
                } else {
                    [0.0, 1.0, 0.0]
                }
            })
            .collect();
        MeshBuffers {
            positions,
            normals,
            uvs: vec![[0.0, 0.0]; 4],
            tangents: vec![],
            colors: None,
            // 4 faces of a tetrahedron.
            indices: vec![0, 1, 2, 0, 2, 3, 0, 3, 1, 1, 3, 2],
            has_suit: false,
        }
    }

    // -----------------------------------------------------------------------
    // AoBakeConfig tests
    // -----------------------------------------------------------------------

    #[test]
    fn ao_bake_config_default() {
        let cfg = AoBakeConfig::default();
        assert_eq!(cfg.sample_count, 64);
        assert!(cfg.max_distance > 0.0);
        assert!(cfg.bias > 0.0);
    }

    #[test]
    fn ao_bake_config_fast() {
        let cfg = AoBakeConfig::fast();
        assert_eq!(cfg.sample_count, 32);
    }

    #[test]
    fn ao_bake_config_quality() {
        let cfg = AoBakeConfig::quality();
        assert_eq!(cfg.sample_count, 128);
    }

    // -----------------------------------------------------------------------
    // hemisphere_samples tests
    // -----------------------------------------------------------------------

    #[test]
    fn hemisphere_samples_count_correct() {
        let samples = hemisphere_samples(64, 0);
        assert_eq!(samples.len(), 64);
    }

    #[test]
    fn hemisphere_samples_all_unit_length() {
        let samples = hemisphere_samples(32, 1);
        for s in &samples {
            let l = len3(*s);
            assert!((l - 1.0).abs() < 1e-5, "sample is not unit length: {l}");
        }
    }

    #[test]
    fn hemisphere_samples_all_positive_y() {
        // Cosine-weighted hemisphere around +Y: all samples must have y >= 0.
        let samples = hemisphere_samples(256, 7);
        for s in &samples {
            assert!(s[1] >= 0.0, "sample y={} is not non-negative", s[1]);
        }
    }

    // -----------------------------------------------------------------------
    // tangent_to_world tests
    // -----------------------------------------------------------------------

    #[test]
    fn tangent_to_world_preserves_length() {
        let normal = normalize3([0.0, 1.0, 0.0]);
        let sample = [0.5f32, 0.8660254, 0.0]; // roughly unit vec
        let world = tangent_to_world(normal, sample);
        let l = len3(world);
        assert!((l - 1.0).abs() < 1e-5, "length={l}");
    }

    #[test]
    fn tangent_to_world_identity_normal_preserves_y() {
        // When normal = +Y and sample = [0,1,0], the world-space result should
        // also point in the +Y direction (i.e., dot > 0.9).
        let normal = [0.0f32, 1.0, 0.0];
        let sample = [0.0f32, 1.0, 0.0];
        let world = tangent_to_world(normal, sample);
        let d = dot3(world, normal);
        assert!(d > 0.9, "dot={d}");
    }

    // -----------------------------------------------------------------------
    // ray_triangle_intersect tests
    // -----------------------------------------------------------------------

    #[test]
    fn ray_triangle_intersect_hits() {
        // Triangle in XY plane at z=0, ray coming from +Z pointing -Z.
        let v0 = [-1.0f32, -1.0, 0.0];
        let v1 = [1.0, -1.0, 0.0];
        let v2 = [0.0, 1.0, 0.0];
        let origin = [0.0f32, 0.0, 2.0];
        let dir = [0.0f32, 0.0, -1.0];
        let t = ray_triangle_intersect(origin, dir, v0, v1, v2);
        assert!(t.is_some(), "expected hit, got None");
        let t = t.expect("should succeed");
        assert!((t - 2.0).abs() < 1e-5, "expected t≈2.0, got {t}");
    }

    #[test]
    fn ray_triangle_intersect_miss_parallel() {
        // Ray parallel to the triangle plane.
        let v0 = [-1.0f32, -1.0, 0.0];
        let v1 = [1.0, -1.0, 0.0];
        let v2 = [0.0, 1.0, 0.0];
        let origin = [0.0f32, 0.0, 1.0];
        let dir = [1.0f32, 0.0, 0.0]; // parallel to XY plane
        let t = ray_triangle_intersect(origin, dir, v0, v1, v2);
        assert!(t.is_none(), "expected miss (parallel), got {t:?}");
    }

    #[test]
    fn ray_triangle_intersect_miss_behind() {
        // Triangle in XY plane at z=0, ray at z=-1 pointing -Z (away from triangle).
        let v0 = [-1.0f32, -1.0, 0.0];
        let v1 = [1.0, -1.0, 0.0];
        let v2 = [0.0, 1.0, 0.0];
        let origin = [0.0f32, 0.0, -1.0];
        let dir = [0.0f32, 0.0, -1.0]; // pointing away
        let t = ray_triangle_intersect(origin, dir, v0, v1, v2);
        assert!(t.is_none(), "expected miss (behind), got {t:?}");
    }

    // -----------------------------------------------------------------------
    // ray_hits_mesh tests
    // -----------------------------------------------------------------------

    #[test]
    fn ray_hits_mesh_returns_true_for_hit() {
        let mesh = single_triangle_mesh();
        // Triangle in XY plane, normal +Z. Ray from (0.2, 0.2, 1) pointing -Z.
        let origin = [0.2f32, 0.2, 1.0];
        let dir = [0.0f32, 0.0, -1.0];
        assert!(
            ray_hits_mesh(&mesh, origin, dir, 10.0),
            "expected ray to hit the triangle"
        );
    }

    #[test]
    fn ray_hits_mesh_returns_false_for_miss() {
        let mesh = single_triangle_mesh();
        // Ray entirely outside the triangle footprint.
        let origin = [5.0f32, 5.0, 1.0];
        let dir = [0.0f32, 0.0, -1.0];
        assert!(
            !ray_hits_mesh(&mesh, origin, dir, 10.0),
            "expected ray to miss the triangle"
        );
    }

    // -----------------------------------------------------------------------
    // bake_vertex_ao tests
    // -----------------------------------------------------------------------

    #[test]
    fn bake_vertex_ao_length_matches_vertices() {
        let mesh = tetrahedron_mesh();
        let cfg = AoBakeConfig {
            sample_count: 8,
            max_distance: 5.0,
            bias: 1e-3,
            seed: 0,
        };
        let ao = bake_vertex_ao(&mesh, &cfg);
        assert_eq!(ao.len(), mesh.vertex_count());
    }

    #[test]
    fn bake_vertex_ao_values_in_range() {
        let mesh = tetrahedron_mesh();
        let cfg = AoBakeConfig {
            sample_count: 8,
            max_distance: 5.0,
            bias: 1e-3,
            seed: 1,
        };
        let ao = bake_vertex_ao(&mesh, &cfg);
        for &v in &ao {
            assert!((0.0..=1.0).contains(&v), "ao value out of range: {v}");
        }
    }

    #[test]
    fn bake_vertex_ao_empty_mesh() {
        let mesh = MeshBuffers {
            positions: vec![],
            normals: vec![],
            uvs: vec![],
            tangents: vec![],
            colors: None,
            indices: vec![],
            has_suit: false,
        };
        let cfg = AoBakeConfig::fast();
        let ao = bake_vertex_ao(&mesh, &cfg);
        assert!(ao.is_empty());
    }

    // -----------------------------------------------------------------------
    // ao_to_vertex_colors tests
    // -----------------------------------------------------------------------

    #[test]
    fn ao_to_vertex_colors_length_matches() {
        let ao = vec![0.0f32, 0.5, 1.0];
        let colors = ao_to_vertex_colors(&ao);
        assert_eq!(colors.len(), 3);
    }

    #[test]
    fn ao_to_vertex_colors_range_correct() {
        let ao = vec![0.0f32, 1.0];
        let colors = ao_to_vertex_colors(&ao);
        assert_eq!(colors[0], [0u8, 0, 0]);
        assert_eq!(colors[1], [255u8, 255, 255]);
    }

    // -----------------------------------------------------------------------
    // fast_vertex_ao tests
    // -----------------------------------------------------------------------

    #[test]
    fn fast_vertex_ao_values_in_range() {
        let mesh = tetrahedron_mesh();
        let ao = fast_vertex_ao(&mesh);
        for &v in &ao {
            assert!((0.0..=1.0).contains(&v), "fast_vertex_ao out of range: {v}");
        }
    }

    #[test]
    fn fast_vertex_ao_length_matches() {
        let mesh = tetrahedron_mesh();
        let ao = fast_vertex_ao(&mesh);
        assert_eq!(ao.len(), mesh.vertex_count());
    }

    #[test]
    fn fast_vertex_ao_single_triangle() {
        // A flat mesh: all vertices face the same direction, so neighbour
        // normals are identical — AO should be close to 1.0.
        let mesh = single_triangle_mesh();
        let ao = fast_vertex_ao(&mesh);
        assert_eq!(ao.len(), 3);
        for &v in &ao {
            assert!((0.0..=1.0).contains(&v), "value out of range: {v}");
        }
    }
}
