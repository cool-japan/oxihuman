// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Ambient occlusion preview data (CPU-side AO computation).

#[allow(dead_code)]
pub struct AoConfig {
    pub sample_count: u32,
    pub radius: f32,
    pub bias: f32,
    pub power: f32, // contrast boost
    pub seed: u64,
}

#[allow(dead_code)]
pub struct AoBuffer {
    pub values: Vec<f32>, // per-vertex AO [0, 1], 1 = fully lit
    pub vertex_count: usize,
}

#[allow(dead_code)]
pub fn default_ao_config() -> AoConfig {
    AoConfig {
        sample_count: 32,
        radius: 0.1,
        bias: 0.001,
        power: 1.0,
        seed: 42,
    }
}

#[allow(dead_code)]
pub fn new_ao_buffer(vertex_count: usize) -> AoBuffer {
    AoBuffer {
        values: vec![1.0f32; vertex_count],
        vertex_count,
    }
}

// LCG random number generator producing [0, 1)
fn lcg_next(state: &mut u64) -> f32 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    let bits = ((*state >> 33) as u32) as f32;
    bits / (u32::MAX as f32)
}

/// Cosine-weighted hemisphere sample using LCG RNG.
#[allow(dead_code)]
pub fn ao_hemisphere_sample(normal: [f32; 3], rng_state: &mut u64) -> [f32; 3] {
    // Sample using the rejection method to get uniform sphere direction in hemisphere
    loop {
        let x = lcg_next(rng_state) * 2.0 - 1.0;
        let y = lcg_next(rng_state) * 2.0 - 1.0;
        let z = lcg_next(rng_state) * 2.0 - 1.0;
        let len2 = x * x + y * y + z * z;
        if !(1e-8..=1.0).contains(&len2) {
            continue;
        }
        let inv_len = len2.sqrt().recip();
        let sx = x * inv_len;
        let sy = y * inv_len;
        let sz = z * inv_len;
        // flip to be in hemisphere of normal
        let dot = sx * normal[0] + sy * normal[1] + sz * normal[2];
        if dot >= 0.0 {
            return [sx, sy, sz];
        } else {
            return [-sx, -sy, -sz];
        }
    }
}

/// Möller–Trumbore ray-triangle intersection.
/// Returns Some(t) where t > 0 if intersection found, else None.
#[allow(dead_code)]
pub fn ray_intersects_triangle(
    ray_origin: [f32; 3],
    ray_dir: [f32; 3],
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
) -> Option<f32> {
    const EPSILON: f32 = 1e-8;
    let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

    let h = cross3(ray_dir, edge2);
    let a = dot3(edge1, h);
    if a.abs() < EPSILON {
        return None; // ray is parallel
    }
    let f = 1.0 / a;
    let s = [
        ray_origin[0] - v0[0],
        ray_origin[1] - v0[1],
        ray_origin[2] - v0[2],
    ];
    let u = f * dot3(s, h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross3(s, edge1);
    let v = f * dot3(ray_dir, q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = f * dot3(edge2, q);
    if t > EPSILON {
        Some(t)
    } else {
        None
    }
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

/// Compute per-vertex ambient occlusion via brute-force hemisphere sampling.
/// For each vertex, casts `sample_count` rays in the hemisphere, counts fraction not occluded.
#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
pub fn compute_ao_cpu(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    indices: &[u32],
    cfg: &AoConfig,
) -> AoBuffer {
    let n = positions.len();
    let mut ao = new_ao_buffer(n);
    let face_count = indices.len() / 3;

    for (vi, value) in ao.values.iter_mut().enumerate() {
        if vi >= normals.len() {
            break;
        }
        let pos = positions[vi];
        let normal = normals[vi];
        let nlen = len3(normal);
        if nlen < 1e-8 {
            *value = 1.0;
            continue;
        }

        let mut rng_state = cfg.seed.wrapping_add(vi as u64 * 1_000_003);
        let mut unoccluded = 0u32;

        for _ in 0..cfg.sample_count {
            let dir = ao_hemisphere_sample(normal, &mut rng_state);
            // Offset ray origin slightly along normal to avoid self-intersection
            let ray_origin = [
                pos[0] + normal[0] * cfg.bias,
                pos[1] + normal[1] * cfg.bias,
                pos[2] + normal[2] * cfg.bias,
            ];

            let mut hit = false;
            'face_loop: for fi in 0..face_count {
                let base = fi * 3;
                let (ai, bi, ci) = (
                    indices[base] as usize,
                    indices[base + 1] as usize,
                    indices[base + 2] as usize,
                );
                if ai >= n || bi >= n || ci >= n {
                    continue;
                }
                if ai == vi || bi == vi || ci == vi {
                    continue; // skip own triangle
                }
                if let Some(t) = ray_intersects_triangle(
                    ray_origin,
                    dir,
                    positions[ai],
                    positions[bi],
                    positions[ci],
                ) {
                    if t < cfg.radius {
                        hit = true;
                        break 'face_loop;
                    }
                }
            }
            if !hit {
                unoccluded += 1;
            }
        }

        *value = if cfg.sample_count > 0 {
            unoccluded as f32 / cfg.sample_count as f32
        } else {
            1.0
        };
    }
    ao
}

/// Convert AO buffer to per-vertex RGBA colors (white * ao).
#[allow(dead_code)]
pub fn ao_to_vertex_color(ao: &AoBuffer) -> Vec<[f32; 4]> {
    ao.values.iter().map(|&v| [v, v, v, 1.0]).collect()
}

/// Apply power curve for contrast enhancement: ao^power.
#[allow(dead_code)]
pub fn apply_ao_power(ao: &mut AoBuffer, power: f32) {
    for v in ao.values.iter_mut() {
        *v = v.powf(power).clamp(0.0, 1.0);
    }
}

/// Smooth AO values by averaging neighbors for given iterations.
#[allow(dead_code)]
pub fn smooth_ao(ao: &mut AoBuffer, adjacency: &[Vec<usize>], iterations: u32) {
    for _ in 0..iterations {
        let old = ao.values.clone();
        for (i, neighbors) in adjacency.iter().enumerate() {
            if i >= ao.vertex_count {
                break;
            }
            if neighbors.is_empty() {
                continue;
            }
            let sum: f32 = neighbors
                .iter()
                .filter(|&&n| n < ao.vertex_count)
                .map(|&n| old[n])
                .sum();
            let valid_count = neighbors.iter().filter(|&&n| n < ao.vertex_count).count();
            if valid_count > 0 {
                ao.values[i] = (old[i] + sum) / (1 + valid_count) as f32;
            }
        }
    }
}

#[allow(dead_code)]
pub fn ao_min(ao: &AoBuffer) -> f32 {
    ao.values.iter().cloned().fold(f32::MAX, f32::min)
}

#[allow(dead_code)]
pub fn ao_max(ao: &AoBuffer) -> f32 {
    ao.values.iter().cloned().fold(f32::MIN, f32::max)
}

#[allow(dead_code)]
pub fn ao_average(ao: &AoBuffer) -> f32 {
    if ao.values.is_empty() {
        return 0.0;
    }
    let sum: f32 = ao.values.iter().sum();
    sum / ao.values.len() as f32
}

/// Scale AO values to [0, 255] as u8 grayscale.
#[allow(dead_code)]
pub fn ao_to_grayscale(ao: &AoBuffer) -> Vec<u8> {
    ao.values
        .iter()
        .map(|&v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)
        .collect()
}

/// Weighted blend of two AO buffers.
#[allow(dead_code)]
pub fn combine_ao_buffers(a: &AoBuffer, b: &AoBuffer, weight_a: f32, weight_b: f32) -> AoBuffer {
    let n = a.vertex_count.min(b.vertex_count);
    let mut values = Vec::with_capacity(n);
    let total = weight_a + weight_b;
    let (wa, wb) = if total < 1e-8 {
        (0.5, 0.5)
    } else {
        (weight_a / total, weight_b / total)
    };
    for i in 0..n {
        values.push((a.values[i] * wa + b.values[i] * wb).clamp(0.0, 1.0));
    }
    AoBuffer {
        values,
        vertex_count: n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ao_buffer() {
        let buf = new_ao_buffer(5);
        assert_eq!(buf.vertex_count, 5);
        assert_eq!(buf.values.len(), 5);
        assert!(buf.values.iter().all(|&v| (v - 1.0).abs() < 1e-6));
    }

    #[test]
    fn test_ao_hemisphere_sample_is_unit_length() {
        let mut rng = 12345u64;
        let normal = [0.0f32, 1.0, 0.0];
        for _ in 0..20 {
            let s = ao_hemisphere_sample(normal, &mut rng);
            let len = (s[0] * s[0] + s[1] * s[1] + s[2] * s[2]).sqrt();
            assert!(
                (len - 1.0).abs() < 1e-4,
                "sample should be unit length, got {}",
                len
            );
        }
    }

    #[test]
    fn test_ao_hemisphere_sample_in_correct_hemisphere() {
        let mut rng = 99999u64;
        let normal = [0.0f32, 0.0, 1.0];
        for _ in 0..20 {
            let s = ao_hemisphere_sample(normal, &mut rng);
            let dot = s[0] * normal[0] + s[1] * normal[1] + s[2] * normal[2];
            assert!(
                dot >= 0.0,
                "sample should be in normal hemisphere, dot={}",
                dot
            );
        }
    }

    #[test]
    fn test_ray_intersects_triangle_hit() {
        // Triangle in XY plane at z=1
        let v0 = [0.0, 0.0, 1.0];
        let v1 = [1.0, 0.0, 1.0];
        let v2 = [0.0, 1.0, 1.0];
        let origin = [0.2, 0.2, 0.0];
        let dir = [0.0, 0.0, 1.0];
        let result = ray_intersects_triangle(origin, dir, v0, v1, v2);
        assert!(result.is_some(), "ray should hit triangle");
        let t = result.expect("should succeed");
        assert!((t - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_ray_intersects_triangle_miss() {
        let v0 = [0.0, 0.0, 1.0];
        let v1 = [1.0, 0.0, 1.0];
        let v2 = [0.0, 1.0, 1.0];
        let origin = [5.0, 5.0, 0.0]; // far outside the triangle
        let dir = [0.0, 0.0, 1.0];
        let result = ray_intersects_triangle(origin, dir, v0, v1, v2);
        assert!(result.is_none(), "ray should miss triangle");
    }

    #[test]
    fn test_ray_intersects_triangle_back_face() {
        // ray going in -z direction, triangle at z=1 should not be hit from behind
        let v0 = [0.0, 0.0, 1.0];
        let v1 = [1.0, 0.0, 1.0];
        let v2 = [0.0, 1.0, 1.0];
        let origin = [0.2, 0.2, 2.0];
        let dir = [0.0, 0.0, 1.0]; // going away
        let result = ray_intersects_triangle(origin, dir, v0, v1, v2);
        assert!(result.is_none(), "ray going away should miss");
    }

    #[test]
    fn test_ao_to_vertex_color() {
        let ao = AoBuffer {
            values: vec![0.0, 0.5, 1.0],
            vertex_count: 3,
        };
        let colors = ao_to_vertex_color(&ao);
        assert_eq!(colors.len(), 3);
        assert_eq!(colors[0], [0.0, 0.0, 0.0, 1.0]);
        assert!((colors[1][0] - 0.5).abs() < 1e-6);
        assert_eq!(colors[2], [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_apply_ao_power() {
        let mut ao = AoBuffer {
            values: vec![0.5, 1.0, 0.0],
            vertex_count: 3,
        };
        apply_ao_power(&mut ao, 2.0);
        assert!((ao.values[0] - 0.25).abs() < 1e-5);
        assert!((ao.values[1] - 1.0).abs() < 1e-5);
        assert!((ao.values[2] - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_ao_min() {
        let ao = AoBuffer {
            values: vec![0.3, 0.7, 0.1, 0.9],
            vertex_count: 4,
        };
        assert!((ao_min(&ao) - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_ao_max() {
        let ao = AoBuffer {
            values: vec![0.3, 0.7, 0.1, 0.9],
            vertex_count: 4,
        };
        assert!((ao_max(&ao) - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_ao_average() {
        let ao = AoBuffer {
            values: vec![0.0, 0.5, 1.0],
            vertex_count: 3,
        };
        let avg = ao_average(&ao);
        assert!((avg - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_ao_to_grayscale() {
        let ao = AoBuffer {
            values: vec![0.0, 0.5, 1.0],
            vertex_count: 3,
        };
        let gs = ao_to_grayscale(&ao);
        assert_eq!(gs.len(), 3);
        assert_eq!(gs[0], 0);
        assert_eq!(gs[2], 255);
        assert!(gs[1] > 100 && gs[1] < 150, "0.5 should be ~128");
    }

    #[test]
    fn test_ao_to_grayscale_clamped() {
        let ao = AoBuffer {
            values: vec![1.5, -0.5],
            vertex_count: 2,
        };
        let gs = ao_to_grayscale(&ao);
        assert_eq!(gs[0], 255, "values > 1.0 should clamp to 255");
        assert_eq!(gs[1], 0, "values < 0.0 should clamp to 0");
    }

    #[test]
    fn test_combine_ao_buffers() {
        let a = AoBuffer {
            values: vec![1.0, 0.0],
            vertex_count: 2,
        };
        let b = AoBuffer {
            values: vec![0.0, 1.0],
            vertex_count: 2,
        };
        let combined = combine_ao_buffers(&a, &b, 0.5, 0.5);
        assert_eq!(combined.vertex_count, 2);
        assert!((combined.values[0] - 0.5).abs() < 1e-5);
        assert!((combined.values[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_default_ao_config() {
        let cfg = default_ao_config();
        assert_eq!(cfg.sample_count, 32);
        assert!(cfg.radius > 0.0);
        assert!(cfg.bias > 0.0);
        assert!((cfg.power - 1.0).abs() < 1e-6);
    }
}
