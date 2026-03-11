// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hair curl/wave pattern strength morph parameters.

/// Hair curl pattern type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurlPattern {
    Straight,
    Wavy,
    Curly,
    Coily,
    Kinky,
}

impl CurlPattern {
    #[allow(dead_code)]
    pub fn name(self) -> &'static str {
        match self {
            CurlPattern::Straight => "straight",
            CurlPattern::Wavy => "wavy",
            CurlPattern::Curly => "curly",
            CurlPattern::Coily => "coily",
            CurlPattern::Kinky => "kinky",
        }
    }

    #[allow(dead_code)]
    pub fn curl_index(self) -> f32 {
        match self {
            CurlPattern::Straight => 0.0,
            CurlPattern::Wavy => 0.25,
            CurlPattern::Curly => 0.5,
            CurlPattern::Coily => 0.75,
            CurlPattern::Kinky => 1.0,
        }
    }
}

/// Parameters for hair curl morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HairCurlParams {
    pub pattern: CurlPattern,
    pub strength: f32,
    pub frequency: f32,
    pub random_seed: u32,
}

impl Default for HairCurlParams {
    fn default() -> Self {
        HairCurlParams {
            pattern: CurlPattern::Straight,
            strength: 0.0,
            frequency: 1.0,
            random_seed: 42,
        }
    }
}

#[allow(dead_code)]
pub fn default_hair_curl_params() -> HairCurlParams {
    HairCurlParams::default()
}

#[allow(dead_code)]
pub fn hc_set_pattern(p: &mut HairCurlParams, pat: CurlPattern) {
    p.pattern = pat;
}

#[allow(dead_code)]
pub fn hc_set_strength(p: &mut HairCurlParams, v: f32) {
    p.strength = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn hc_set_frequency(p: &mut HairCurlParams, v: f32) {
    p.frequency = v.clamp(0.1, 10.0);
}

#[allow(dead_code)]
pub fn hc_reset(p: &mut HairCurlParams) {
    *p = HairCurlParams::default();
}

#[allow(dead_code)]
pub fn hc_is_straight(p: &HairCurlParams) -> bool {
    p.strength < 1e-6 || p.pattern == CurlPattern::Straight
}

#[allow(dead_code)]
pub fn hc_effective_curl(p: &HairCurlParams) -> f32 {
    p.pattern.curl_index() * p.strength
}

#[allow(dead_code)]
pub fn hc_blend(a: &HairCurlParams, b: &HairCurlParams, t: f32) -> HairCurlParams {
    let t = t.clamp(0.0, 1.0);
    let pattern = if t < 0.5 { a.pattern } else { b.pattern };
    HairCurlParams {
        pattern,
        strength: a.strength + (b.strength - a.strength) * t,
        frequency: a.frequency + (b.frequency - a.frequency) * t,
        random_seed: a.random_seed,
    }
}

#[allow(dead_code)]
pub fn hc_to_json(p: &HairCurlParams) -> String {
    format!(
        r#"{{"pattern":"{}","strength":{:.4},"frequency":{:.4}}}"#,
        p.pattern.name(),
        p.strength,
        p.frequency
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_straight() {
        assert!(hc_is_straight(&default_hair_curl_params()));
    }

    #[test]
    fn curl_index_range() {
        assert!((CurlPattern::Straight.curl_index()).abs() < 1e-6);
        assert!((CurlPattern::Kinky.curl_index() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_strength_clamps() {
        let mut p = default_hair_curl_params();
        hc_set_strength(&mut p, 5.0);
        assert!((p.strength - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_frequency_clamps_low() {
        let mut p = default_hair_curl_params();
        hc_set_frequency(&mut p, 0.0);
        assert!((p.frequency - 0.1).abs() < 1e-6);
    }

    #[test]
    fn reset_to_straight() {
        let mut p = default_hair_curl_params();
        hc_set_pattern(&mut p, CurlPattern::Kinky);
        hc_set_strength(&mut p, 1.0);
        hc_reset(&mut p);
        assert!(hc_is_straight(&p));
    }

    #[test]
    fn effective_curl_zero_when_straight() {
        let p = default_hair_curl_params();
        assert!(hc_effective_curl(&p).abs() < 1e-6);
    }

    #[test]
    fn effective_curl_with_kinky_full() {
        let mut p = default_hair_curl_params();
        hc_set_pattern(&mut p, CurlPattern::Kinky);
        hc_set_strength(&mut p, 1.0);
        assert!((hc_effective_curl(&p) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn blend_strength_midpoint() {
        let a = default_hair_curl_params();
        let mut b = default_hair_curl_params();
        hc_set_strength(&mut b, 1.0);
        let m = hc_blend(&a, &b, 0.5);
        assert!((m.strength - 0.5).abs() < 1e-5);
    }

    #[test]
    fn to_json_contains_pattern() {
        assert!(hc_to_json(&default_hair_curl_params()).contains("pattern"));
    }
}
