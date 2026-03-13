// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Eyelash morphology control for character faces.

/// Eyelash parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyelashParams {
    pub length: f32,
    pub curl: f32,
    pub density: f32,
    pub thickness: f32,
}

/// Result of eyelash evaluation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyelashResult {
    pub length_weight: f32,
    pub curl_weight: f32,
    pub density_weight: f32,
    pub overall_weight: f32,
}

/// Default eyelash parameters.
#[allow(dead_code)]
pub fn default_eyelash() -> EyelashParams {
    EyelashParams {
        length: 0.5,
        curl: 0.3,
        density: 0.5,
        thickness: 0.4,
    }
}

/// Evaluate eyelash morph weights.
#[allow(dead_code)]
pub fn evaluate_eyelash(params: &EyelashParams) -> EyelashResult {
    let l = params.length.clamp(0.0, 1.0);
    let c = params.curl.clamp(0.0, 1.0);
    let d = params.density.clamp(0.0, 1.0);
    let overall = l * 0.4 + c * 0.3 + d * 0.3;
    EyelashResult {
        length_weight: l,
        curl_weight: c,
        density_weight: d,
        overall_weight: overall,
    }
}

/// Blend eyelash params.
#[allow(dead_code)]
pub fn blend_eyelash(a: &EyelashParams, b: &EyelashParams, t: f32) -> EyelashParams {
    let t = t.clamp(0.0, 1.0);
    EyelashParams {
        length: a.length + (b.length - a.length) * t,
        curl: a.curl + (b.curl - a.curl) * t,
        density: a.density + (b.density - a.density) * t,
        thickness: a.thickness + (b.thickness - a.thickness) * t,
    }
}

/// Set eyelash length.
#[allow(dead_code)]
pub fn set_eyelash_length(params: &mut EyelashParams, value: f32) {
    params.length = value.clamp(0.0, 1.0);
}

/// Set eyelash curl.
#[allow(dead_code)]
pub fn set_eyelash_curl(params: &mut EyelashParams, value: f32) {
    params.curl = value.clamp(0.0, 1.0);
}

/// Validate params.
#[allow(dead_code)]
pub fn is_valid_eyelash(params: &EyelashParams) -> bool {
    (0.0..=1.0).contains(&params.length)
        && (0.0..=1.0).contains(&params.curl)
        && (0.0..=1.0).contains(&params.density)
        && (0.0..=1.0).contains(&params.thickness)
}

/// Reset to defaults.
#[allow(dead_code)]
pub fn reset_eyelash(params: &mut EyelashParams) {
    *params = default_eyelash();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let p = default_eyelash();
        assert!((p.length - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate() {
        let p = default_eyelash();
        let r = evaluate_eyelash(&p);
        assert!((0.0..=1.0).contains(&r.overall_weight));
    }

    #[test]
    fn test_blend() {
        let a = default_eyelash();
        let mut b = default_eyelash();
        b.length = 1.0;
        let c = blend_eyelash(&a, &b, 0.5);
        assert!((c.length - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set_length() {
        let mut p = default_eyelash();
        set_eyelash_length(&mut p, 0.8);
        assert!((p.length - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_curl() {
        let mut p = default_eyelash();
        set_eyelash_curl(&mut p, 0.9);
        assert!((p.curl - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_valid() {
        assert!(is_valid_eyelash(&default_eyelash()));
    }

    #[test]
    fn test_invalid() {
        let p = EyelashParams { length: 2.0, curl: 0.5, density: 0.5, thickness: 0.5 };
        assert!(!is_valid_eyelash(&p));
    }

    #[test]
    fn test_reset() {
        let mut p = EyelashParams { length: 0.9, curl: 0.9, density: 0.9, thickness: 0.9 };
        reset_eyelash(&mut p);
        assert!((p.length - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_full_weight() {
        let p = EyelashParams { length: 1.0, curl: 1.0, density: 1.0, thickness: 1.0 };
        let r = evaluate_eyelash(&p);
        assert!((r.overall_weight - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_zero_weight() {
        let p = EyelashParams { length: 0.0, curl: 0.0, density: 0.0, thickness: 0.0 };
        let r = evaluate_eyelash(&p);
        assert!(r.overall_weight.abs() < 1e-6);
    }
}
