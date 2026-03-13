// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Procedural body hair parameters and generation.

/// Simple LCG pseudo-random number generator (no external crate).
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_add(1))
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }

    /// Returns a value in [0, 1).
    fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 11) as f32 / (1u64 << 53) as f32
    }

    /// Returns a value in [lo, hi).
    fn range_f32(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.next_f32() * (hi - lo)
    }
}

#[allow(dead_code)]
pub struct HairRegion {
    pub name: String,
    pub density: f32,
    pub length: f32,
    pub length_variance: f32,
    pub curl: f32,
    pub color: [f32; 3],
    pub enabled: bool,
}

#[allow(dead_code)]
pub struct HairProfile {
    pub regions: Vec<HairRegion>,
    pub global_density_scale: f32,
    pub global_length_scale: f32,
}

#[allow(dead_code)]
pub struct HairStrand {
    pub root: [f32; 3],
    pub tip: [f32; 3],
    pub thickness: f32,
    pub color: [f32; 3],
}

#[allow(dead_code)]
pub struct HairGenerationParams {
    pub seed: u64,
    pub lod: u8,
}

#[allow(dead_code)]
pub fn default_hair_profile() -> HairProfile {
    HairProfile {
        regions: vec![
            HairRegion {
                name: "scalp".to_string(),
                density: 150.0,
                length: 120.0,
                length_variance: 30.0,
                curl: 0.1,
                color: [0.2, 0.15, 0.1],
                enabled: true,
            },
            HairRegion {
                name: "eyebrow_left".to_string(),
                density: 80.0,
                length: 10.0,
                length_variance: 2.0,
                curl: 0.05,
                color: [0.18, 0.12, 0.08],
                enabled: true,
            },
            HairRegion {
                name: "eyebrow_right".to_string(),
                density: 80.0,
                length: 10.0,
                length_variance: 2.0,
                curl: 0.05,
                color: [0.18, 0.12, 0.08],
                enabled: true,
            },
            HairRegion {
                name: "eyelash_upper".to_string(),
                density: 100.0,
                length: 12.0,
                length_variance: 2.5,
                curl: 0.3,
                color: [0.1, 0.08, 0.05],
                enabled: true,
            },
            HairRegion {
                name: "eyelash_lower".to_string(),
                density: 60.0,
                length: 8.0,
                length_variance: 1.5,
                curl: 0.2,
                color: [0.1, 0.08, 0.05],
                enabled: true,
            },
            HairRegion {
                name: "beard".to_string(),
                density: 40.0,
                length: 5.0,
                length_variance: 2.0,
                curl: 0.1,
                color: [0.2, 0.15, 0.1],
                enabled: false,
            },
            HairRegion {
                name: "armpit".to_string(),
                density: 20.0,
                length: 25.0,
                length_variance: 5.0,
                curl: 0.2,
                color: [0.22, 0.17, 0.12],
                enabled: true,
            },
            HairRegion {
                name: "pubic".to_string(),
                density: 30.0,
                length: 20.0,
                length_variance: 5.0,
                curl: 0.5,
                color: [0.2, 0.15, 0.1],
                enabled: true,
            },
        ],
        global_density_scale: 1.0,
        global_length_scale: 1.0,
    }
}

#[allow(dead_code)]
pub fn add_region(profile: &mut HairProfile, region: HairRegion) {
    profile.regions.push(region);
}

#[allow(dead_code)]
pub fn scale_density(profile: &mut HairProfile, factor: f32) {
    profile.global_density_scale *= factor;
}

#[allow(dead_code)]
pub fn hair_count_for_region(region: &HairRegion, area_cm2: f32) -> usize {
    if !region.enabled {
        return 0;
    }
    (region.density * area_cm2).round() as usize
}

#[allow(dead_code)]
pub fn generate_strands(
    region: &HairRegion,
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    params: &HairGenerationParams,
) -> Vec<HairStrand> {
    if !region.enabled || positions.is_empty() {
        return Vec::new();
    }
    let lod_factor = lod_density_factor(params.lod);
    let count = (positions.len() as f32 * lod_factor).round() as usize;
    let count = count.min(positions.len());

    let mut rng = Lcg::new(params.seed);
    let mut strands = Vec::with_capacity(count);
    for (i, root) in positions.iter().enumerate().take(count) {
        let normal = if i < normals.len() {
            normals[i]
        } else {
            [0.0, 1.0, 0.0]
        };
        let length_var = rng.range_f32(-region.length_variance, region.length_variance);
        let length = (region.length + length_var).max(0.1) * 0.001; // mm to m
        let tip = curl_tip(
            *root,
            normal,
            length,
            region.curl,
            params.seed.wrapping_add(i as u64),
        );
        let thickness = rng.range_f32(0.04, 0.08);
        strands.push(HairStrand {
            root: *root,
            tip,
            thickness,
            color: region.color,
        });
    }
    strands
}

#[allow(dead_code)]
pub fn curl_tip(root: [f32; 3], normal: [f32; 3], length: f32, curl: f32, seed: u64) -> [f32; 3] {
    let s = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let angle = (s as f32 / u64::MAX as f32) * std::f32::consts::TAU;
    let curl_offset = curl * length * 0.5;
    [
        root[0] + normal[0] * length + angle.cos() * curl_offset,
        root[1] + normal[1] * length + angle.sin() * curl_offset,
        root[2] + normal[2] * length,
    ]
}

#[allow(dead_code)]
pub fn total_strand_count(profile: &HairProfile, area_cm2: f32) -> usize {
    profile
        .regions
        .iter()
        .filter(|r| r.enabled)
        .map(|r| {
            let effective_density = r.density * profile.global_density_scale;
            (effective_density * area_cm2).round() as usize
        })
        .sum()
}

#[allow(dead_code)]
pub fn region_by_name<'a>(profile: &'a HairProfile, name: &str) -> Option<&'a HairRegion> {
    profile.regions.iter().find(|r| r.name == name)
}

#[allow(dead_code)]
pub fn blend_hair_profiles(a: &HairProfile, b: &HairProfile, t: f32) -> HairProfile {
    let t = t.clamp(0.0, 1.0);
    let count = a.regions.len().min(b.regions.len());
    let mut regions = Vec::with_capacity(count);
    for i in 0..count {
        let ra = &a.regions[i];
        let rb = &b.regions[i];
        regions.push(HairRegion {
            name: ra.name.clone(),
            density: ra.density * (1.0 - t) + rb.density * t,
            length: ra.length * (1.0 - t) + rb.length * t,
            length_variance: ra.length_variance * (1.0 - t) + rb.length_variance * t,
            curl: ra.curl * (1.0 - t) + rb.curl * t,
            color: [
                ra.color[0] * (1.0 - t) + rb.color[0] * t,
                ra.color[1] * (1.0 - t) + rb.color[1] * t,
                ra.color[2] * (1.0 - t) + rb.color[2] * t,
            ],
            enabled: if t < 0.5 { ra.enabled } else { rb.enabled },
        });
    }
    HairProfile {
        regions,
        global_density_scale: a.global_density_scale * (1.0 - t) + b.global_density_scale * t,
        global_length_scale: a.global_length_scale * (1.0 - t) + b.global_length_scale * t,
    }
}

#[allow(dead_code)]
pub fn hair_profile_to_params(profile: &HairProfile) -> Vec<(String, f32)> {
    let mut out = Vec::new();
    out.push((
        "global_density_scale".to_string(),
        profile.global_density_scale,
    ));
    out.push((
        "global_length_scale".to_string(),
        profile.global_length_scale,
    ));
    for r in &profile.regions {
        let prefix = r.name.clone();
        out.push((format!("{prefix}.density"), r.density));
        out.push((format!("{prefix}.length"), r.length));
        out.push((format!("{prefix}.length_variance"), r.length_variance));
        out.push((format!("{prefix}.curl"), r.curl));
        out.push((format!("{prefix}.color_r"), r.color[0]));
        out.push((format!("{prefix}.color_g"), r.color[1]));
        out.push((format!("{prefix}.color_b"), r.color[2]));
        out.push((
            format!("{prefix}.enabled"),
            if r.enabled { 1.0 } else { 0.0 },
        ));
    }
    out
}

#[allow(dead_code)]
pub fn lod_density_factor(lod: u8) -> f32 {
    match lod {
        0 => 1.0,
        1 => 0.4,
        _ => 0.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_profile_non_empty() {
        let p = default_hair_profile();
        assert!(!p.regions.is_empty());
        assert!(p.regions.len() >= 8);
    }

    #[test]
    fn test_default_profile_names() {
        let p = default_hair_profile();
        let names: Vec<&str> = p.regions.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"scalp"));
        assert!(names.contains(&"beard"));
    }

    #[test]
    fn test_hair_count_formula() {
        let region = HairRegion {
            name: "test".to_string(),
            density: 100.0,
            length: 10.0,
            length_variance: 1.0,
            curl: 0.0,
            color: [0.0; 3],
            enabled: true,
        };
        let count = hair_count_for_region(&region, 5.0);
        assert_eq!(count, 500);
    }

    #[test]
    fn test_hair_count_disabled() {
        let region = HairRegion {
            name: "test".to_string(),
            density: 100.0,
            length: 10.0,
            length_variance: 1.0,
            curl: 0.0,
            color: [0.0; 3],
            enabled: false,
        };
        assert_eq!(hair_count_for_region(&region, 10.0), 0);
    }

    #[test]
    fn test_generate_strands_length() {
        let region = HairRegion {
            name: "test".to_string(),
            density: 10.0,
            length: 20.0,
            length_variance: 2.0,
            curl: 0.0,
            color: [0.5, 0.4, 0.3],
            enabled: true,
        };
        let positions: Vec<[f32; 3]> = (0..10).map(|i| [i as f32, 0.0, 0.0]).collect();
        let normals: Vec<[f32; 3]> = (0..10).map(|_| [0.0, 1.0, 0.0]).collect();
        let params = HairGenerationParams { seed: 42, lod: 0 };
        let strands = generate_strands(&region, &positions, &normals, &params);
        assert_eq!(strands.len(), 10);
    }

    #[test]
    fn test_generate_strands_lod1() {
        let region = HairRegion {
            name: "test".to_string(),
            density: 10.0,
            length: 20.0,
            length_variance: 2.0,
            curl: 0.0,
            color: [0.5, 0.4, 0.3],
            enabled: true,
        };
        let positions: Vec<[f32; 3]> = (0..100).map(|i| [i as f32, 0.0, 0.0]).collect();
        let normals: Vec<[f32; 3]> = (0..100).map(|_| [0.0, 1.0, 0.0]).collect();
        let params = HairGenerationParams { seed: 42, lod: 1 };
        let strands = generate_strands(&region, &positions, &normals, &params);
        assert!(strands.len() < 100);
    }

    #[test]
    fn test_generate_strands_disabled() {
        let region = HairRegion {
            name: "test".to_string(),
            density: 10.0,
            length: 20.0,
            length_variance: 2.0,
            curl: 0.0,
            color: [0.5, 0.4, 0.3],
            enabled: false,
        };
        let positions = vec![[0.0_f32; 3]];
        let normals = vec![[0.0, 1.0, 0.0_f32]];
        let params = HairGenerationParams { seed: 1, lod: 0 };
        assert!(generate_strands(&region, &positions, &normals, &params).is_empty());
    }

    #[test]
    fn test_blend_profiles() {
        let a = default_hair_profile();
        let b = default_hair_profile();
        let blended = blend_hair_profiles(&a, &b, 0.5);
        assert!(!blended.regions.is_empty());
        assert!((blended.global_density_scale - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_blend_profiles_t0() {
        let a = default_hair_profile();
        let mut b = default_hair_profile();
        b.global_density_scale = 2.0;
        let blended = blend_hair_profiles(&a, &b, 0.0);
        assert!((blended.global_density_scale - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_blend_profiles_t1() {
        let a = default_hair_profile();
        let mut b = default_hair_profile();
        b.global_density_scale = 2.0;
        let blended = blend_hair_profiles(&a, &b, 1.0);
        assert!((blended.global_density_scale - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_region_lookup() {
        let p = default_hair_profile();
        let r = region_by_name(&p, "scalp");
        assert!(r.is_some());
        assert_eq!(r.expect("should succeed").name, "scalp");
    }

    #[test]
    fn test_region_lookup_missing() {
        let p = default_hair_profile();
        assert!(region_by_name(&p, "nonexistent").is_none());
    }

    #[test]
    fn test_scale_density() {
        let mut p = default_hair_profile();
        scale_density(&mut p, 2.0);
        assert!((p.global_density_scale - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_lod_factor() {
        assert!((lod_density_factor(0) - 1.0).abs() < 1e-5);
        assert!((lod_density_factor(1) - 0.4).abs() < 1e-5);
        assert!((lod_density_factor(2) - 0.1).abs() < 1e-5);
        assert!((lod_density_factor(255) - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_total_strand_count() {
        let p = default_hair_profile();
        let count = total_strand_count(&p, 1.0);
        assert!(count > 0);
    }

    #[test]
    fn test_hair_profile_to_params() {
        let p = default_hair_profile();
        let params = hair_profile_to_params(&p);
        assert!(!params.is_empty());
        let has_global = params.iter().any(|(k, _)| k == "global_density_scale");
        assert!(has_global);
    }

    #[test]
    fn test_curl_tip() {
        let root = [0.0_f32; 3];
        let normal = [0.0, 1.0, 0.0];
        let tip = curl_tip(root, normal, 0.1, 0.0, 42);
        assert!((tip[1] - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_add_region() {
        let mut p = default_hair_profile();
        let n = p.regions.len();
        add_region(
            &mut p,
            HairRegion {
                name: "extra".to_string(),
                density: 5.0,
                length: 5.0,
                length_variance: 1.0,
                curl: 0.0,
                color: [1.0; 3],
                enabled: true,
            },
        );
        assert_eq!(p.regions.len(), n + 1);
    }
}
