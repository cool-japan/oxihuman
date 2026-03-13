// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use crate::mesh::MeshBuffers;

// ---------------------------------------------------------------------------
// Helper math
// ---------------------------------------------------------------------------

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l > 0.0 {
        scale3(v, 1.0 / l)
    } else {
        [0.0, 0.0, 1.0]
    }
}

fn lerp2(a: [f32; 2], b: [f32; 2], t: f32) -> [f32; 2] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    len3(sub3(a, b))
}

// ---------------------------------------------------------------------------
// SurfacePoint
// ---------------------------------------------------------------------------

/// A sampled point on a mesh surface.
#[derive(Debug, Clone)]
pub struct SurfacePoint {
    /// World position.
    pub position: [f32; 3],
    /// Interpolated normal at this point.
    pub normal: [f32; 3],
    /// Interpolated UV at this point.
    pub uv: [f32; 2],
    /// Index of the face this point is on.
    pub face_index: usize,
    /// Barycentric coordinates (u, v, w) where u+v+w=1.
    pub barycentric: [f32; 3],
}

impl SurfacePoint {
    /// Compute a surface point from barycentric coordinates and a face index.
    pub fn from_barycentric(mesh: &MeshBuffers, face_index: usize, bary: [f32; 3]) -> Self {
        let base = face_index * 3;
        let i0 = mesh.indices[base] as usize;
        let i1 = mesh.indices[base + 1] as usize;
        let i2 = mesh.indices[base + 2] as usize;

        let p0 = mesh.positions[i0];
        let p1 = mesh.positions[i1];
        let p2 = mesh.positions[i2];

        let n0 = mesh.normals[i0];
        let n1 = mesh.normals[i1];
        let n2 = mesh.normals[i2];

        let uv0 = mesh.uvs[i0];
        let uv1 = mesh.uvs[i1];
        let uv2 = mesh.uvs[i2];

        let [u, v, w] = bary;

        // Barycentric interpolation
        let position = add3(add3(scale3(p0, u), scale3(p1, v)), scale3(p2, w));

        let normal_raw = add3(add3(scale3(n0, u), scale3(n1, v)), scale3(n2, w));
        let normal = normalize3(normal_raw);

        // UV interpolation
        let uv = [
            uv0[0] * u + uv1[0] * v + uv2[0] * w,
            uv0[1] * u + uv1[1] * v + uv2[1] * w,
        ];

        // Suppress unused lerp2 lint by using it here - actually just inline
        let _ = lerp2; // keep it available for tests if needed

        SurfacePoint {
            position,
            normal,
            uv,
            face_index,
            barycentric: bary,
        }
    }
}

// ---------------------------------------------------------------------------
// LCG random number generator
// ---------------------------------------------------------------------------

/// Simple LCG (Linear Congruential Generator) — no external dependencies.
#[allow(dead_code)]
pub struct Lcg {
    state: u64,
}

#[allow(dead_code)]
impl Lcg {
    /// Create a new LCG with the given seed.
    pub fn new(seed: u64) -> Self {
        // Use a non-zero initial state by mixing the seed.
        let state = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        Self {
            state: if state == 0 { 1 } else { state },
        }
    }

    /// Advance the state and return the new raw u64.
    pub fn next_u64(&mut self) -> u64 {
        // Knuth's multiplicative LCG with Newlib constants.
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    /// Return a float in [0, 1).
    pub fn next_f32(&mut self) -> f32 {
        let bits = self.next_u64();
        // Use top 24 bits for mantissa precision.
        (bits >> 40) as f32 / (1u64 << 24) as f32
    }
}

// ---------------------------------------------------------------------------
// Area helpers
// ---------------------------------------------------------------------------

/// Compute the area of a single triangular face.
pub fn face_area(mesh: &MeshBuffers, face_index: usize) -> f32 {
    let base = face_index * 3;
    let p0 = mesh.positions[mesh.indices[base] as usize];
    let p1 = mesh.positions[mesh.indices[base + 1] as usize];
    let p2 = mesh.positions[mesh.indices[base + 2] as usize];
    let e1 = sub3(p1, p0);
    let e2 = sub3(p2, p0);
    len3(cross3(e1, e2)) * 0.5
}

/// Compute per-face areas. Returns `Vec<f32>` of length `face_count`.
pub fn face_areas(mesh: &MeshBuffers) -> Vec<f32> {
    (0..mesh.face_count())
        .map(|fi| face_area(mesh, fi))
        .collect()
}

/// Total surface area of the mesh.
pub fn total_surface_area(mesh: &MeshBuffers) -> f32 {
    face_areas(mesh).iter().sum()
}

// ---------------------------------------------------------------------------
// Sampling internals
// ---------------------------------------------------------------------------

/// Sample a random point in a triangle using uniform barycentric sampling.
/// Uses the sqrt-based formula for a uniform distribution.
/// Returns barycentric coordinates (u, v, w).
fn sample_triangle(r1: f32, r2: f32) -> [f32; 3] {
    let sqrt_r1 = r1.sqrt();
    let u = 1.0 - sqrt_r1;
    let v = sqrt_r1 * (1.0 - r2);
    let w = sqrt_r1 * r2;
    [u, v, w]
}

/// Build a cumulative distribution function from face areas (for weighted sampling).
/// The returned CDF is normalised to [0, 1].
fn build_area_cdf(areas: &[f32]) -> Vec<f32> {
    let total: f32 = areas.iter().sum();
    if total <= 0.0 {
        return vec![1.0f32 / areas.len() as f32; areas.len()]
            .iter()
            .scan(0.0f32, |acc, &x| {
                *acc += x;
                Some(*acc)
            })
            .collect();
    }
    let mut cdf = Vec::with_capacity(areas.len());
    let mut running = 0.0f32;
    for &a in areas {
        running += a / total;
        cdf.push(running);
    }
    // Clamp the last entry to exactly 1.0 to avoid floating-point drift.
    if let Some(last) = cdf.last_mut() {
        *last = 1.0;
    }
    cdf
}

/// Binary-search the CDF to find which face a random value `r` falls in.
fn sample_face_from_cdf(cdf: &[f32], r: f32) -> usize {
    // Find the first index where cdf[i] >= r.
    match cdf.partition_point(|&c| c < r) {
        idx if idx < cdf.len() => idx,
        _ => cdf.len() - 1,
    }
}

// ---------------------------------------------------------------------------
// Public sampling functions
// ---------------------------------------------------------------------------

/// Sample `n` random points on the mesh surface, distributed proportionally
/// to face areas (larger faces receive proportionally more samples).
pub fn sample_surface(mesh: &MeshBuffers, n: usize, seed: u64) -> Vec<SurfacePoint> {
    if mesh.face_count() == 0 || n == 0 {
        return Vec::new();
    }
    let areas = face_areas(mesh);
    let cdf = build_area_cdf(&areas);
    let mut rng = Lcg::new(seed);
    let mut points = Vec::with_capacity(n);

    for _ in 0..n {
        let r = rng.next_f32();
        let fi = sample_face_from_cdf(&cdf, r);
        let r1 = rng.next_f32();
        let r2 = rng.next_f32();
        let bary = sample_triangle(r1, r2);
        points.push(SurfacePoint::from_barycentric(mesh, fi, bary));
    }
    points
}

/// Sample one random point per face.
pub fn sample_one_per_face(mesh: &MeshBuffers, seed: u64) -> Vec<SurfacePoint> {
    let fc = mesh.face_count();
    if fc == 0 {
        return Vec::new();
    }
    let mut rng = Lcg::new(seed);
    let mut points = Vec::with_capacity(fc);
    for fi in 0..fc {
        let r1 = rng.next_f32();
        let r2 = rng.next_f32();
        let bary = sample_triangle(r1, r2);
        points.push(SurfacePoint::from_barycentric(mesh, fi, bary));
    }
    points
}

/// Sample points using Poisson disk sampling on the mesh surface.
///
/// Generates candidate points and rejects those closer than `min_distance`
/// to any already-accepted point. Uses `max_attempts` rejection tries per
/// candidate before giving up. Pass `max_attempts = 30` as a sensible default.
pub fn sample_poisson_disk(
    mesh: &MeshBuffers,
    min_distance: f32,
    max_attempts: usize,
    seed: u64,
) -> Vec<SurfacePoint> {
    if mesh.face_count() == 0 {
        return Vec::new();
    }
    let areas = face_areas(mesh);
    let cdf = build_area_cdf(&areas);
    let mut rng = Lcg::new(seed);
    let mut accepted: Vec<SurfacePoint> = Vec::new();

    let attempts = max_attempts.max(1);
    // Estimate a reasonable upper bound on total samples.
    let total_area = total_surface_area(mesh);
    let min_samples = if min_distance > 0.0 {
        // Rough bound: area / (π r²) × 4 headroom
        let cap =
            (total_area / (std::f32::consts::PI * min_distance * min_distance) * 4.0) as usize;
        cap.clamp(1, 100_000)
    } else {
        10_000
    };

    let mut generated = 0usize;

    'outer: loop {
        if generated >= min_samples * attempts {
            break;
        }
        // Generate a candidate point.
        let r = rng.next_f32();
        let fi = sample_face_from_cdf(&cdf, r);
        let r1 = rng.next_f32();
        let r2 = rng.next_f32();
        let bary = sample_triangle(r1, r2);
        let candidate = SurfacePoint::from_barycentric(mesh, fi, bary);
        generated += 1;

        // Check against all accepted points.
        for existing in &accepted {
            if dist3(candidate.position, existing.position) < min_distance {
                continue 'outer;
            }
        }
        accepted.push(candidate);

        // If we've reached the density cap, stop.
        if min_distance > 0.0 && accepted.len() >= min_samples {
            break;
        }
    }
    accepted
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // Test helpers
    // ------------------------------------------------------------------

    /// Unit right-angle triangle in the XY plane.
    fn unit_triangle_mesh() -> MeshBuffers {
        MeshBuffers {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            tangents: vec![],
            colors: None,
            indices: vec![0, 1, 2],
            has_suit: false,
        }
    }

    /// Two right-angle unit triangles forming a 1×1 square.
    fn two_triangle_mesh() -> MeshBuffers {
        MeshBuffers {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            tangents: vec![],
            colors: None,
            indices: vec![0, 1, 2, 0, 2, 3],
            has_suit: false,
        }
    }

    /// A degenerate face (all vertices at the same position).
    fn degenerate_mesh() -> MeshBuffers {
        MeshBuffers {
            positions: vec![[0.5, 0.5, 0.5]; 3],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            tangents: vec![],
            colors: None,
            indices: vec![0, 1, 2],
            has_suit: false,
        }
    }

    // ------------------------------------------------------------------
    // LCG tests
    // ------------------------------------------------------------------

    #[test]
    fn lcg_new_advances_state() {
        let mut a = Lcg::new(42);
        let v1 = a.next_u64();
        let v2 = a.next_u64();
        assert_ne!(v1, v2, "consecutive LCG values should differ");
    }

    #[test]
    fn lcg_next_f32_in_range() {
        let mut rng = Lcg::new(12345);
        for _ in 0..1000 {
            let v = rng.next_f32();
            assert!((0.0..1.0).contains(&v), "f32 out of [0,1): {v}");
        }
    }

    // ------------------------------------------------------------------
    // Area tests
    // ------------------------------------------------------------------

    #[test]
    fn face_area_known_triangle() {
        let mesh = unit_triangle_mesh();
        let area = face_area(&mesh, 0);
        assert!((area - 0.5).abs() < 1e-5, "expected 0.5, got {area}");
    }

    #[test]
    fn face_areas_length_matches_face_count() {
        let mesh = two_triangle_mesh();
        let areas = face_areas(&mesh);
        assert_eq!(areas.len(), mesh.face_count());
    }

    #[test]
    fn total_surface_area_positive() {
        let mesh = two_triangle_mesh();
        let area = total_surface_area(&mesh);
        assert!(area > 0.0, "total area must be positive, got {area}");
        assert!((area - 1.0).abs() < 1e-5, "expected 1.0, got {area}");
    }

    #[test]
    fn face_area_zero_for_degenerate() {
        let mesh = degenerate_mesh();
        let area = face_area(&mesh, 0);
        assert!(
            area.abs() < 1e-6,
            "degenerate face area should be 0, got {area}"
        );
    }

    // ------------------------------------------------------------------
    // sample_surface tests
    // ------------------------------------------------------------------

    #[test]
    fn sample_surface_correct_count() {
        let mesh = two_triangle_mesh();
        let pts = sample_surface(&mesh, 100, 7);
        assert_eq!(pts.len(), 100);
    }

    #[test]
    fn sample_surface_positions_finite() {
        let mesh = two_triangle_mesh();
        let pts = sample_surface(&mesh, 50, 99);
        for p in &pts {
            assert!(
                p.position.iter().all(|x| x.is_finite()),
                "non-finite position: {:?}",
                p.position
            );
        }
    }

    #[test]
    fn sample_surface_normals_unit_length() {
        let mesh = two_triangle_mesh();
        let pts = sample_surface(&mesh, 50, 13);
        for p in &pts {
            let len = len3(p.normal);
            assert!((len - 1.0).abs() < 1e-4, "normal length={len}");
        }
    }

    #[test]
    fn sample_surface_uvs_in_range() {
        let mesh = two_triangle_mesh();
        let pts = sample_surface(&mesh, 100, 31);
        for p in &pts {
            assert!(
                p.uv[0] >= -1e-5 && p.uv[0] <= 1.0 + 1e-5,
                "u out of range: {}",
                p.uv[0]
            );
            assert!(
                p.uv[1] >= -1e-5 && p.uv[1] <= 1.0 + 1e-5,
                "v out of range: {}",
                p.uv[1]
            );
        }
    }

    // ------------------------------------------------------------------
    // sample_one_per_face tests
    // ------------------------------------------------------------------

    #[test]
    fn sample_one_per_face_count_equals_face_count() {
        let mesh = two_triangle_mesh();
        let pts = sample_one_per_face(&mesh, 55);
        assert_eq!(pts.len(), mesh.face_count());
    }

    // ------------------------------------------------------------------
    // Poisson disk tests
    // ------------------------------------------------------------------

    #[test]
    fn sample_poisson_disk_min_distance_respected() {
        let mesh = two_triangle_mesh();
        let min_dist = 0.1;
        let pts = sample_poisson_disk(&mesh, min_dist, 30, 77);
        for i in 0..pts.len() {
            for j in (i + 1)..pts.len() {
                let d = dist3(pts[i].position, pts[j].position);
                assert!(
                    d >= min_dist - 1e-5,
                    "points {i} and {j} are too close: {d} < {min_dist}"
                );
            }
        }
    }

    // ------------------------------------------------------------------
    // SurfacePoint::from_barycentric test
    // ------------------------------------------------------------------

    #[test]
    fn surface_point_from_barycentric_on_face() {
        let mesh = unit_triangle_mesh();
        // Centroid barycentric coords.
        let bary = [1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0];
        let sp = SurfacePoint::from_barycentric(&mesh, 0, bary);
        // Centroid of [(0,0,0),(1,0,0),(0,1,0)] = (1/3, 1/3, 0).
        assert!(
            (sp.position[0] - 1.0 / 3.0).abs() < 1e-5,
            "x={}",
            sp.position[0]
        );
        assert!(
            (sp.position[1] - 1.0 / 3.0).abs() < 1e-5,
            "y={}",
            sp.position[1]
        );
        assert!(sp.position[2].abs() < 1e-5, "z={}", sp.position[2]);
    }
}
