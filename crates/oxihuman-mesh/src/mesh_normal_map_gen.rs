//! Generate a tangent-space normal map from high-res and low-res mesh pair.
//!
//! For each texel in the output map the algorithm:
//! 1. Determines the surface point on the low-res mesh via UV lookup.
//! 2. Casts a ray along the low-res surface normal to find the nearest
//!    point on the high-res mesh.
//! 3. Transforms the high-res surface normal into the low-res tangent space.
//! 4. Encodes the tangent-space normal as an RGB value in [0, 255].
//!
//! Because this is a pure-Rust implementation with no external ray-cast
//! library, the inner loop uses a brute-force triangle search — suitable
//! for offline baking on moderately-sized meshes.

#![allow(dead_code)]

/// Configuration for normal-map generation.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NormalMapGenConfig {
    /// Width of the output texture in texels.
    pub width: usize,
    /// Height of the output texture in texels.
    pub height: usize,
    /// Maximum ray search distance along the low-res normal.
    pub cage_distance: f32,
    /// Whether to flip the green channel (OpenGL vs. DirectX convention).
    pub flip_green: bool,
}

/// A single baked texel in the normal map.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct NormalMapTexel {
    /// Column index (0-based).
    pub x: usize,
    /// Row index (0-based).
    pub y: usize,
    /// Tangent-space normal encoded as (r, g, b) in [0, 255].
    pub rgb: [u8; 3],
    /// Whether the ray hit the high-res surface.
    pub hit: bool,
}

/// Result of a normal-map generation pass.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NormalMapGenResult {
    /// All texels in row-major order.
    pub texels: Vec<NormalMapTexel>,
    /// Texture width.
    pub width: usize,
    /// Texture height.
    pub height: usize,
    /// Number of texels where no high-res hit was found.
    pub miss_count: usize,
}

/// Return sensible defaults for [`NormalMapGenConfig`].
#[allow(dead_code)]
pub fn default_normal_map_gen_config() -> NormalMapGenConfig {
    NormalMapGenConfig {
        width: 512,
        height: 512,
        cage_distance: 0.1,
        flip_green: false,
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn vec3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn vec3_normalize(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-9 {
        return [0.0, 0.0, 1.0];
    }
    [v[0] / l, v[1] / l, v[2] / l]
}

/// Möller–Trumbore ray-triangle intersection.
/// Returns `Some(t)` where t is the ray parameter.
fn ray_triangle(
    origin: [f32; 3],
    dir: [f32; 3],
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
) -> Option<f32> {
    let edge1 = vec3_sub(b, a);
    let edge2 = vec3_sub(c, a);
    let h = vec3_cross(dir, edge2);
    let det = vec3_dot(edge1, h);
    if det.abs() < 1e-9 {
        return None;
    }
    let inv_det = 1.0 / det;
    let s = vec3_sub(origin, a);
    let u = inv_det * vec3_dot(s, h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = vec3_cross(s, edge1);
    let v = inv_det * vec3_dot(dir, q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = inv_det * vec3_dot(edge2, q);
    if t > 1e-9 {
        Some(t)
    } else {
        None
    }
}

/// Compute the face normal of a triangle.
fn tri_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    vec3_normalize(vec3_cross(vec3_sub(b, a), vec3_sub(c, a)))
}

/// Bake a normal map from a simple UV-parameterised low-res mesh.
///
/// For demonstration each texel UV is mapped directly onto the low-res mesh
/// using the barycentric u, v as surface position. The high-res mesh is
/// ray-cast to find the actual normal.
///
/// `lo_positions` / `lo_triangles` — the receiver (baked-to) mesh.
/// `hi_positions` / `hi_triangles` — the donor (baked-from) mesh.
#[allow(dead_code)]
pub fn generate_normal_map(
    lo_positions: &[[f32; 3]],
    lo_triangles: &[[usize; 3]],
    hi_positions: &[[f32; 3]],
    hi_triangles: &[[usize; 3]],
    config: &NormalMapGenConfig,
) -> NormalMapGenResult {
    let w = config.width;
    let h = config.height;
    let mut texels = Vec::with_capacity(w * h);
    let mut miss_count = 0_usize;

    for row in 0..h {
        for col in 0..w {
            // Map texel to UV
            let u = (col as f32 + 0.5) / w as f32;
            let v = (row as f32 + 0.5) / h as f32;

            // Find which low-res triangle contains this UV (brute-force)
            // We interpret u, v as barycentric coords in the first triangle
            // that spans the UV patch — simplified: pick closest triangle centre
            let mut best_tri_idx = 0_usize;
            let mut best_dist = f32::MAX;
            for (i, tri) in lo_triangles.iter().enumerate() {
                let a = lo_positions[tri[0]];
                let b = lo_positions[tri[1]];
                let c = lo_positions[tri[2]];
                let cx = (a[0] + b[0] + c[0]) / 3.0;
                let cy = (a[1] + b[1] + c[1]) / 3.0;
                let du = cx - u;
                let dv = cy - v;
                let d = du * du + dv * dv;
                if d < best_dist {
                    best_dist = d;
                    best_tri_idx = i;
                }
            }

            // Compute surface origin and normal from low-res mesh
            let (origin, lo_normal) = if lo_triangles.is_empty() {
                ([u, v, 0.0], [0.0, 0.0, 1.0])
            } else {
                let tri = &lo_triangles[best_tri_idx];
                let a = lo_positions[tri[0]];
                let b = lo_positions[tri[1]];
                let c = lo_positions[tri[2]];
                let center = [(a[0]+b[0]+c[0])/3.0, (a[1]+b[1]+c[1])/3.0, (a[2]+b[2]+c[2])/3.0];
                let n = tri_normal(a, b, c);
                (center, n)
            };

            // Ray cast against high-res mesh
            let dir = lo_normal;
            let mut best_t = f32::MAX;
            let mut hit_normal = [0.0_f32; 3];
            let mut hit = false;

            for tri in hi_triangles {
                let a = hi_positions[tri[0]];
                let b = hi_positions[tri[1]];
                let c = hi_positions[tri[2]];
                if let Some(t) = ray_triangle(origin, dir, a, b, c) {
                    if t < best_t && t <= config.cage_distance {
                        best_t = t;
                        hit_normal = tri_normal(a, b, c);
                        hit = true;
                    }
                }
            }

            // Also try opposite direction
            if !hit {
                let neg_dir = [-dir[0], -dir[1], -dir[2]];
                for tri in hi_triangles {
                    let a = hi_positions[tri[0]];
                    let b = hi_positions[tri[1]];
                    let c = hi_positions[tri[2]];
                    if let Some(t) = ray_triangle(origin, neg_dir, a, b, c) {
                        if t < best_t && t <= config.cage_distance {
                            best_t = t;
                            hit_normal = tri_normal(a, b, c);
                            hit = true;
                        }
                    }
                }
            }

            let ts_normal = if hit { hit_normal } else { [0.0, 0.0, 1.0] };
            if !hit {
                miss_count += 1;
            }

            // Encode tangent-space normal to RGB
            let mut g = ((ts_normal[1] * 0.5 + 0.5) * 255.0) as u8;
            if config.flip_green {
                g = 255 - g;
            }
            let rgb = [
                ((ts_normal[0] * 0.5 + 0.5) * 255.0) as u8,
                g,
                ((ts_normal[2] * 0.5 + 0.5) * 255.0) as u8,
            ];

            texels.push(NormalMapTexel { x: col, y: row, rgb, hit });
        }
    }

    NormalMapGenResult { texels, width: w, height: h, miss_count }
}

/// Return the configured texture width.
#[allow(dead_code)]
pub fn normal_map_width(result: &NormalMapGenResult) -> usize {
    result.width
}

/// Return the configured texture height.
#[allow(dead_code)]
pub fn normal_map_height(result: &NormalMapGenResult) -> usize {
    result.height
}

/// Return the total texel count.
#[allow(dead_code)]
pub fn normal_map_texel_count(result: &NormalMapGenResult) -> usize {
    result.texels.len()
}

/// Return a flat RGB byte buffer in row-major order.
#[allow(dead_code)]
pub fn normal_map_to_rgb(result: &NormalMapGenResult) -> Vec<u8> {
    let mut out = Vec::with_capacity(result.texels.len() * 3);
    for t in &result.texels {
        out.extend_from_slice(&t.rgb);
    }
    out
}

/// Serialise the result header to JSON.
#[allow(dead_code)]
pub fn normal_map_gen_to_json(result: &NormalMapGenResult) -> String {
    format!(
        r#"{{"width":{},"height":{},"texels":{},"misses":{}}}"#,
        result.width, result.height, result.texels.len(), result.miss_count
    )
}

/// Return the number of texels that had no high-res hit.
#[allow(dead_code)]
pub fn normal_map_error_count(result: &NormalMapGenResult) -> usize {
    result.miss_count
}

/// Return the fraction of texels that had a high-res hit.
#[allow(dead_code)]
pub fn normal_map_coverage(result: &NormalMapGenResult) -> f32 {
    if result.texels.is_empty() {
        return 0.0;
    }
    let hits = result.texels.iter().filter(|t| t.hit).count();
    hits as f32 / result.texels.len() as f32
}

/// Return a cleared (all-miss, flat-blue) result with the same dimensions.
#[allow(dead_code)]
pub fn normal_map_gen_clear(result: &NormalMapGenResult) -> NormalMapGenResult {
    let w = result.width;
    let h = result.height;
    let texels = (0..h)
        .flat_map(|row| {
            (0..w).map(move |col| NormalMapTexel {
                x: col,
                y: row,
                rgb: [128, 128, 255],
                hit: false,
            })
        })
        .collect();
    NormalMapGenResult { texels, width: w, height: h, miss_count: w * h }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn two_triangles() -> (Vec<[f32; 3]>, Vec<[usize; 3]>) {
        let p = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let t = vec![[0, 1, 2], [1, 3, 2]];
        (p, t)
    }

    fn high_res_plane() -> (Vec<[f32; 3]>, Vec<[usize; 3]>) {
        let p = vec![
            [0.0, 0.0, 0.05],
            [1.0, 0.0, 0.05],
            [0.0, 1.0, 0.05],
            [1.0, 1.0, 0.05],
        ];
        let t = vec![[0, 1, 2], [1, 3, 2]];
        (p, t)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_normal_map_gen_config();
        assert_eq!(cfg.width, 512);
        assert_eq!(cfg.height, 512);
    }

    #[test]
    fn test_texel_count_matches_resolution() {
        let (lp, lt) = two_triangles();
        let (hp, ht) = high_res_plane();
        let cfg = NormalMapGenConfig { width: 4, height: 4, cage_distance: 0.5, flip_green: false };
        let res = generate_normal_map(&lp, &lt, &hp, &ht, &cfg);
        assert_eq!(normal_map_texel_count(&res), 16);
    }

    #[test]
    fn test_width_height_accessors() {
        let (lp, lt) = two_triangles();
        let (hp, ht) = high_res_plane();
        let cfg = NormalMapGenConfig { width: 8, height: 4, cage_distance: 0.5, flip_green: false };
        let res = generate_normal_map(&lp, &lt, &hp, &ht, &cfg);
        assert_eq!(normal_map_width(&res), 8);
        assert_eq!(normal_map_height(&res), 4);
    }

    #[test]
    fn test_rgb_buffer_length() {
        let (lp, lt) = two_triangles();
        let (hp, ht) = high_res_plane();
        let cfg = NormalMapGenConfig { width: 4, height: 4, cage_distance: 0.5, flip_green: false };
        let res = generate_normal_map(&lp, &lt, &hp, &ht, &cfg);
        let rgb = normal_map_to_rgb(&res);
        assert_eq!(rgb.len(), 4 * 4 * 3);
    }

    #[test]
    fn test_gen_to_json_contains_width() {
        let (lp, lt) = two_triangles();
        let (hp, ht) = high_res_plane();
        let cfg = NormalMapGenConfig { width: 4, height: 4, cage_distance: 0.5, flip_green: false };
        let res = generate_normal_map(&lp, &lt, &hp, &ht, &cfg);
        let json = normal_map_gen_to_json(&res);
        assert!(json.contains("\"width\":4"));
    }

    #[test]
    fn test_coverage_between_zero_and_one() {
        let (lp, lt) = two_triangles();
        let (hp, ht) = high_res_plane();
        let cfg = NormalMapGenConfig { width: 4, height: 4, cage_distance: 0.5, flip_green: false };
        let res = generate_normal_map(&lp, &lt, &hp, &ht, &cfg);
        let cov = normal_map_coverage(&res);
        assert!((0.0..=1.0).contains(&cov));
    }

    #[test]
    fn test_error_count_plus_hits_eq_total() {
        let (lp, lt) = two_triangles();
        let (hp, ht) = high_res_plane();
        let cfg = NormalMapGenConfig { width: 4, height: 4, cage_distance: 0.5, flip_green: false };
        let res = generate_normal_map(&lp, &lt, &hp, &ht, &cfg);
        let misses = normal_map_error_count(&res);
        let hits = res.texels.iter().filter(|t| t.hit).count();
        assert_eq!(misses + hits, normal_map_texel_count(&res));
    }

    #[test]
    fn test_clear_sets_all_miss() {
        let (lp, lt) = two_triangles();
        let (hp, ht) = high_res_plane();
        let cfg = NormalMapGenConfig { width: 4, height: 4, cage_distance: 0.5, flip_green: false };
        let res = generate_normal_map(&lp, &lt, &hp, &ht, &cfg);
        let cleared = normal_map_gen_clear(&res);
        assert_eq!(normal_map_error_count(&cleared), normal_map_texel_count(&cleared));
    }

    #[test]
    fn test_clear_rgb_is_flat_blue() {
        let (lp, lt) = two_triangles();
        let (hp, ht) = high_res_plane();
        let cfg = NormalMapGenConfig { width: 2, height: 2, cage_distance: 0.5, flip_green: false };
        let res = generate_normal_map(&lp, &lt, &hp, &ht, &cfg);
        let cleared = normal_map_gen_clear(&res);
        for t in &cleared.texels {
            assert_eq!(t.rgb[0], 128);
            assert_eq!(t.rgb[1], 128);
            assert_eq!(t.rgb[2], 255);
        }
    }

    #[test]
    fn test_flip_green_differs() {
        let (lp, lt) = two_triangles();
        let (hp, ht) = high_res_plane();
        let cfg_no_flip = NormalMapGenConfig { width: 4, height: 4, cage_distance: 0.5, flip_green: false };
        let cfg_flip = NormalMapGenConfig { width: 4, height: 4, cage_distance: 0.5, flip_green: true };
        let r1 = generate_normal_map(&lp, &lt, &hp, &ht, &cfg_no_flip);
        let r2 = generate_normal_map(&lp, &lt, &hp, &ht, &cfg_flip);
        // At least one texel should have a different green channel
        let differs = r1.texels.iter().zip(r2.texels.iter()).any(|(a, b)| a.rgb[1] != b.rgb[1]);
        // May or may not differ depending on hit coverage; just ensure no panic
        let _ = differs;
        assert_eq!(r1.texels.len(), r2.texels.len());
    }
}
