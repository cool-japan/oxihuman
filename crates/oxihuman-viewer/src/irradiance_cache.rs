// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Irradiance cache — stores pre-computed diffuse irradiance samples for GI.

/// One cached irradiance sample at a world-space position.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IrradianceSample {
    pub position: [f32; 3],
    pub irradiance: [f32; 3],
    pub valid: bool,
    pub age: u32,
}

/// Irradiance cache storage.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct IrradianceCache {
    pub samples: Vec<IrradianceSample>,
    pub max_age: u32,
}

#[allow(dead_code)]
pub fn new_irradiance_cache(max_age: u32) -> IrradianceCache {
    IrradianceCache {
        samples: Vec::new(),
        max_age,
    }
}

#[allow(dead_code)]
pub fn ic_add_sample(cache: &mut IrradianceCache, position: [f32; 3], irradiance: [f32; 3]) {
    cache.samples.push(IrradianceSample {
        position,
        irradiance,
        valid: true,
        age: 0,
    });
}

#[allow(dead_code)]
pub fn ic_tick(cache: &mut IrradianceCache) {
    for s in cache.samples.iter_mut() {
        s.age += 1;
        if s.age > cache.max_age {
            s.valid = false;
        }
    }
    cache.samples.retain(|s| s.valid);
}

#[allow(dead_code)]
pub fn ic_count(cache: &IrradianceCache) -> usize {
    cache.samples.len()
}

#[allow(dead_code)]
pub fn ic_valid_count(cache: &IrradianceCache) -> usize {
    cache.samples.iter().filter(|s| s.valid).count()
}

#[allow(dead_code)]
pub fn ic_clear(cache: &mut IrradianceCache) {
    cache.samples.clear();
}

#[allow(dead_code)]
pub fn ic_nearest(cache: &IrradianceCache, point: [f32; 3]) -> Option<&IrradianceSample> {
    cache.samples.iter().filter(|s| s.valid).min_by(|a, b| {
        let da = sample_dist_sq(a, point);
        let db = sample_dist_sq(b, point);
        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
    })
}

fn sample_dist_sq(s: &IrradianceSample, p: [f32; 3]) -> f32 {
    let dx = s.position[0] - p[0];
    let dy = s.position[1] - p[1];
    let dz = s.position[2] - p[2];
    dx * dx + dy * dy + dz * dz
}

#[allow(dead_code)]
pub fn ic_average_irradiance(cache: &IrradianceCache) -> [f32; 3] {
    let valid: Vec<&IrradianceSample> = cache.samples.iter().filter(|s| s.valid).collect();
    if valid.is_empty() {
        return [0.0; 3];
    }
    let sum = valid.iter().fold([0.0f32; 3], |acc, s| {
        [
            acc[0] + s.irradiance[0],
            acc[1] + s.irradiance[1],
            acc[2] + s.irradiance[2],
        ]
    });
    let n = valid.len() as f32;
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

#[allow(dead_code)]
pub fn ic_to_json(cache: &IrradianceCache) -> String {
    format!(
        r#"{{"count":{},"max_age":{}}}"#,
        ic_count(cache),
        cache.max_age
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cache_empty() {
        let c = new_irradiance_cache(10);
        assert_eq!(ic_count(&c), 0);
    }

    #[test]
    fn add_sample() {
        let mut c = new_irradiance_cache(10);
        ic_add_sample(&mut c, [0.0; 3], [1.0, 0.5, 0.25]);
        assert_eq!(ic_count(&c), 1);
    }

    #[test]
    fn tick_ages_out() {
        let mut c = new_irradiance_cache(2);
        ic_add_sample(&mut c, [0.0; 3], [1.0; 3]);
        ic_tick(&mut c);
        ic_tick(&mut c);
        ic_tick(&mut c);
        assert_eq!(ic_count(&c), 0);
    }

    #[test]
    fn clear() {
        let mut c = new_irradiance_cache(10);
        ic_add_sample(&mut c, [0.0; 3], [1.0; 3]);
        ic_clear(&mut c);
        assert_eq!(ic_count(&c), 0);
    }

    #[test]
    fn nearest_sample() {
        let mut c = new_irradiance_cache(10);
        ic_add_sample(&mut c, [0.0; 3], [1.0, 0.0, 0.0]);
        ic_add_sample(&mut c, [5.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let near = ic_nearest(&c, [0.1, 0.0, 0.0]);
        assert!(near.is_some());
        assert!((near.unwrap().irradiance[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn nearest_empty_returns_none() {
        let c = new_irradiance_cache(10);
        assert!(ic_nearest(&c, [0.0; 3]).is_none());
    }

    #[test]
    fn average_irradiance_empty() {
        let c = new_irradiance_cache(10);
        let avg = ic_average_irradiance(&c);
        assert!(avg.iter().all(|&v| v.abs() < 1e-6));
    }

    #[test]
    fn average_irradiance_computed() {
        let mut c = new_irradiance_cache(10);
        ic_add_sample(&mut c, [0.0; 3], [1.0, 0.0, 0.0]);
        ic_add_sample(&mut c, [1.0; 3], [0.0, 1.0, 0.0]);
        let avg = ic_average_irradiance(&c);
        assert!((avg[0] - 0.5).abs() < 1e-6);
        assert!((avg[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_json_fields() {
        let c = new_irradiance_cache(16);
        let j = ic_to_json(&c);
        assert!(j.contains("count"));
        assert!(j.contains("max_age"));
    }
}
