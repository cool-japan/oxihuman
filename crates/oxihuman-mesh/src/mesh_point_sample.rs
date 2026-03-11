// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Point sampling strategies for mesh surfaces.
#[allow(dead_code)]
pub struct SamplePoint {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub face_idx: usize,
    pub bary: [f32; 3],
}

#[allow(dead_code)]
pub struct PointSampleConfig {
    pub count: usize,
    pub include_normals: bool,
}

#[allow(dead_code)]
pub fn default_point_sample_config() -> PointSampleConfig {
    PointSampleConfig {
        count: 100,
        include_normals: true,
    }
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn vec_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn triangle_area(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    vec_len(cross3(ab, ac)) * 0.5
}

fn triangle_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = cross3(ab, ac);
    let len = vec_len(n).max(1e-10);
    [n[0] / len, n[1] / len, n[2] / len]
}

fn bary_point(a: [f32; 3], b: [f32; 3], c: [f32; 3], u: f32, v: f32) -> [f32; 3] {
    let w = 1.0 - u - v;
    [
        a[0] * w + b[0] * u + c[0] * v,
        a[1] * w + b[1] * u + c[1] * v,
        a[2] * w + b[2] * u + c[2] * v,
    ]
}

/// Simple LCG for deterministic sampling (no rand dependency).
struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(1),
        }
    }
    fn next_f32(&mut self) -> f32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.state >> 33) as f32) / (u32::MAX as f32)
    }
}

/// Sample points uniformly from mesh surface (area-weighted).
#[allow(dead_code)]
pub fn sample_mesh_surface(
    positions: &[[f32; 3]],
    indices: &[u32],
    cfg: &PointSampleConfig,
) -> Vec<SamplePoint> {
    let tri_count = indices.len() / 3;
    if tri_count == 0 || positions.is_empty() {
        return vec![];
    }

    let areas: Vec<f32> = (0..tri_count)
        .map(|t| {
            let ia = indices[t * 3] as usize;
            let ib = indices[t * 3 + 1] as usize;
            let ic = indices[t * 3 + 2] as usize;
            if ia < positions.len() && ib < positions.len() && ic < positions.len() {
                triangle_area(positions[ia], positions[ib], positions[ic])
            } else {
                0.0
            }
        })
        .collect();

    let total_area: f32 = areas.iter().sum();
    if total_area < 1e-10 {
        return vec![];
    }

    // CDF
    let mut cdf = Vec::with_capacity(tri_count);
    let mut acc = 0.0f32;
    for &a in &areas {
        acc += a / total_area;
        cdf.push(acc);
    }

    let mut rng = Lcg::new(42);
    let mut samples = Vec::with_capacity(cfg.count);

    for _ in 0..cfg.count {
        let r = rng.next_f32();
        let t = cdf.partition_point(|&c| c < r).min(tri_count - 1);
        let ia = indices[t * 3] as usize;
        let ib = indices[t * 3 + 1] as usize;
        let ic = indices[t * 3 + 2] as usize;
        if ia >= positions.len() || ib >= positions.len() || ic >= positions.len() {
            continue;
        }
        // Uniform random barycentric coords
        let r1 = rng.next_f32().sqrt();
        let r2 = rng.next_f32();
        let u = 1.0 - r1;
        let v = r1 * (1.0 - r2);
        let w = r1 * r2;
        let pos = bary_point(positions[ia], positions[ib], positions[ic], v, w);
        let normal = if cfg.include_normals {
            triangle_normal(positions[ia], positions[ib], positions[ic])
        } else {
            [0.0, 1.0, 0.0]
        };
        samples.push(SamplePoint {
            position: pos,
            normal,
            face_idx: t,
            bary: [u, v, w],
        });
    }
    samples
}

#[allow(dead_code)]
pub fn sample_count(samples: &[SamplePoint]) -> usize {
    samples.len()
}

#[allow(dead_code)]
pub fn sample_centroid(samples: &[SamplePoint]) -> [f32; 3] {
    if samples.is_empty() {
        return [0.0; 3];
    }
    let n = samples.len() as f32;
    let mut s = [0.0f32; 3];
    for sp in samples {
        s[0] += sp.position[0];
        s[1] += sp.position[1];
        s[2] += sp.position[2];
    }
    [s[0] / n, s[1] / n, s[2] / n]
}

#[allow(dead_code)]
pub fn samples_all_normals_unit(samples: &[SamplePoint]) -> bool {
    samples.iter().all(|s| {
        let l = (s.normal[0] * s.normal[0] + s.normal[1] * s.normal[1] + s.normal[2] * s.normal[2])
            .sqrt();
        (l - 1.0).abs() < 0.01
    })
}

#[allow(dead_code)]
pub fn samples_to_json(samples: &[SamplePoint]) -> String {
    format!("{{\"count\":{}}}", samples.len())
}

#[allow(dead_code)]
pub fn sample_bary_sum_one(s: &SamplePoint) -> bool {
    (s.bary[0] + s.bary[1] + s.bary[2] - 1.0).abs() < 0.01
}

/// Generate Poisson disk distribution by thinning samples.
#[allow(dead_code)]
pub fn poisson_disk_thin(samples: &[SamplePoint], min_dist: f32) -> Vec<&SamplePoint> {
    let mut result: Vec<&SamplePoint> = Vec::new();
    'outer: for s in samples {
        for r in &result {
            let dx = s.position[0] - r.position[0];
            let dy = s.position[1] - r.position[1];
            let dz = s.position[2] - r.position[2];
            if (dx * dx + dy * dy + dz * dz).sqrt() < min_dist {
                continue 'outer;
            }
        }
        result.push(s);
    }
    result
}

#[allow(dead_code)]
pub fn golden_angle_sphere_samples(n: usize) -> Vec<[f32; 3]> {
    let golden_ratio = (1.0 + 5.0f32.sqrt()) / 2.0;
    (0..n)
        .map(|i| {
            let theta = 2.0 * PI * i as f32 / golden_ratio;
            let phi = (1.0 - 2.0 * (i as f32 + 0.5) / n as f32).acos();
            [phi.sin() * theta.cos(), phi.sin() * theta.sin(), phi.cos()]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_triangle() -> (Vec<[f32; 3]>, Vec<u32>) {
        (
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 0.0, 1.0]],
            vec![0, 1, 2],
        )
    }

    #[test]
    fn test_sample_count() {
        let (pos, idx) = unit_triangle();
        let cfg = PointSampleConfig {
            count: 50,
            include_normals: true,
        };
        let s = sample_mesh_surface(&pos, &idx, &cfg);
        assert_eq!(s.len(), 50);
    }

    #[test]
    fn test_empty_mesh() {
        let cfg = default_point_sample_config();
        let s = sample_mesh_surface(&[], &[], &cfg);
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn test_normals_unit() {
        let (pos, idx) = unit_triangle();
        let cfg = PointSampleConfig {
            count: 10,
            include_normals: true,
        };
        let s = sample_mesh_surface(&pos, &idx, &cfg);
        assert!(samples_all_normals_unit(&s));
    }

    #[test]
    fn test_bary_sum_one() {
        let (pos, idx) = unit_triangle();
        let cfg = PointSampleConfig {
            count: 20,
            include_normals: false,
        };
        let s = sample_mesh_surface(&pos, &idx, &cfg);
        for sp in &s {
            assert!(sample_bary_sum_one(sp));
        }
    }

    #[test]
    fn test_centroid_on_triangle() {
        let (pos, idx) = unit_triangle();
        let cfg = PointSampleConfig {
            count: 200,
            include_normals: false,
        };
        let s = sample_mesh_surface(&pos, &idx, &cfg);
        let c = sample_centroid(&s);
        // should be near centroid [0.5, 0.0, ~0.33]
        assert!((c[0] - 0.5).abs() < 0.15);
    }

    #[test]
    fn test_poisson_disk_thin() {
        let (pos, idx) = unit_triangle();
        let cfg = PointSampleConfig {
            count: 100,
            include_normals: false,
        };
        let s = sample_mesh_surface(&pos, &idx, &cfg);
        let thinned = poisson_disk_thin(&s, 0.1);
        assert!(!thinned.is_empty());
        assert!(thinned.len() <= s.len());
    }

    #[test]
    fn test_golden_angle_count() {
        let s = golden_angle_sphere_samples(50);
        assert_eq!(s.len(), 50);
    }

    #[test]
    fn test_golden_angle_on_unit_sphere() {
        let s = golden_angle_sphere_samples(20);
        for p in s {
            let l = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            assert!((l - 1.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_samples_to_json() {
        let (pos, idx) = unit_triangle();
        let cfg = PointSampleConfig {
            count: 10,
            include_normals: false,
        };
        let s = sample_mesh_surface(&pos, &idx, &cfg);
        let j = samples_to_json(&s);
        assert!(j.contains("count"));
    }

    #[test]
    fn test_default_config() {
        let cfg = default_point_sample_config();
        assert_eq!(cfg.count, 100);
        assert!(cfg.include_normals);
    }
}
