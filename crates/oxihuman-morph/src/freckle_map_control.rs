// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Freckle placement map parameters.

/// Freckle distribution type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FreckleDistribution {
    Sparse,
    Moderate,
    Dense,
    Clustered,
    Uniform,
}

impl FreckleDistribution {
    #[allow(dead_code)]
    pub fn name(self) -> &'static str {
        match self {
            FreckleDistribution::Sparse => "sparse",
            FreckleDistribution::Moderate => "moderate",
            FreckleDistribution::Dense => "dense",
            FreckleDistribution::Clustered => "clustered",
            FreckleDistribution::Uniform => "uniform",
        }
    }

    #[allow(dead_code)]
    pub fn density_index(self) -> f32 {
        match self {
            FreckleDistribution::Sparse => 0.1,
            FreckleDistribution::Moderate => 0.4,
            FreckleDistribution::Dense => 0.7,
            FreckleDistribution::Clustered => 0.6,
            FreckleDistribution::Uniform => 0.5,
        }
    }
}

/// Freckle map parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FreckleParams {
    pub distribution: FreckleDistribution,
    pub density: f32,
    pub size: f32,
    pub darkness: f32,
    pub sun_exposure: f32,
    pub face_coverage: f32,
    pub body_coverage: f32,
}

impl Default for FreckleParams {
    fn default() -> Self {
        FreckleParams {
            distribution: FreckleDistribution::Sparse,
            density: 0.0,
            size: 0.0,
            darkness: 0.0,
            sun_exposure: 0.0,
            face_coverage: 0.0,
            body_coverage: 0.0,
        }
    }
}

#[allow(dead_code)]
pub fn default_freckle_params() -> FreckleParams {
    FreckleParams::default()
}

#[allow(dead_code)]
pub fn fm_set_density(p: &mut FreckleParams, v: f32) {
    p.density = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fm_set_size(p: &mut FreckleParams, v: f32) {
    p.size = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fm_set_darkness(p: &mut FreckleParams, v: f32) {
    p.darkness = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fm_set_sun_exposure(p: &mut FreckleParams, v: f32) {
    p.sun_exposure = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fm_set_face_coverage(p: &mut FreckleParams, v: f32) {
    p.face_coverage = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fm_set_body_coverage(p: &mut FreckleParams, v: f32) {
    p.body_coverage = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fm_set_distribution(p: &mut FreckleParams, d: FreckleDistribution) {
    p.distribution = d;
}

#[allow(dead_code)]
pub fn fm_reset(p: &mut FreckleParams) {
    *p = FreckleParams::default();
}

#[allow(dead_code)]
pub fn fm_is_neutral(p: &FreckleParams) -> bool {
    p.density < 1e-6 && p.face_coverage < 1e-6 && p.body_coverage < 1e-6
}

#[allow(dead_code)]
pub fn fm_effective_density(p: &FreckleParams) -> f32 {
    let sun_boost = p.sun_exposure * 0.3;
    (p.density + sun_boost).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn fm_blend(a: &FreckleParams, b: &FreckleParams, t: f32) -> FreckleParams {
    let t = t.clamp(0.0, 1.0);
    FreckleParams {
        distribution: if t < 0.5 {
            a.distribution
        } else {
            b.distribution
        },
        density: a.density + (b.density - a.density) * t,
        size: a.size + (b.size - a.size) * t,
        darkness: a.darkness + (b.darkness - a.darkness) * t,
        sun_exposure: a.sun_exposure + (b.sun_exposure - a.sun_exposure) * t,
        face_coverage: a.face_coverage + (b.face_coverage - a.face_coverage) * t,
        body_coverage: a.body_coverage + (b.body_coverage - a.body_coverage) * t,
    }
}

#[allow(dead_code)]
pub fn fm_to_json(p: &FreckleParams) -> String {
    format!(
        r#"{{"distribution":"{}","density":{:.4},"size":{:.4},"darkness":{:.4}}}"#,
        p.distribution.name(),
        p.density,
        p.size,
        p.darkness
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_neutral() {
        assert!(fm_is_neutral(&default_freckle_params()));
    }

    #[test]
    fn set_density_clamps() {
        let mut p = default_freckle_params();
        fm_set_density(&mut p, 5.0);
        assert!((p.density - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_sun_exposure() {
        let mut p = default_freckle_params();
        fm_set_sun_exposure(&mut p, 0.7);
        assert!((p.sun_exposure - 0.7).abs() < 1e-5);
    }

    #[test]
    fn effective_density_boosted_by_sun() {
        let mut p = default_freckle_params();
        fm_set_density(&mut p, 0.5);
        fm_set_sun_exposure(&mut p, 1.0);
        assert!(fm_effective_density(&p) > 0.5);
    }

    #[test]
    fn reset_clears() {
        let mut p = default_freckle_params();
        fm_set_density(&mut p, 0.8);
        fm_reset(&mut p);
        assert!(fm_is_neutral(&p));
    }

    #[test]
    fn distribution_density_index() {
        assert!((FreckleDistribution::Sparse.density_index() - 0.1).abs() < 1e-6);
        assert!((FreckleDistribution::Dense.density_index() - 0.7).abs() < 1e-6);
    }

    #[test]
    fn blend_midpoint() {
        let a = default_freckle_params();
        let mut b = default_freckle_params();
        fm_set_density(&mut b, 1.0);
        let m = fm_blend(&a, &b, 0.5);
        assert!((m.density - 0.5).abs() < 1e-5);
    }

    #[test]
    fn set_distribution() {
        let mut p = default_freckle_params();
        fm_set_distribution(&mut p, FreckleDistribution::Dense);
        assert_eq!(p.distribution, FreckleDistribution::Dense);
    }

    #[test]
    fn to_json_has_distribution() {
        assert!(fm_to_json(&default_freckle_params()).contains("distribution"));
    }
}
