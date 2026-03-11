// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Philtrum depth control for facial morphing.

/// Philtrum depth parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PhiltrumDepthParams {
    pub depth: f32,
    pub width: f32,
    pub length: f32,
    pub definition: f32,
}

/// Philtrum depth result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PhiltrumDepthResult {
    pub depth_weight: f32,
    pub width_weight: f32,
    pub definition_weight: f32,
    pub combined_weight: f32,
}

/// Default philtrum depth parameters.
#[allow(dead_code)]
pub fn default_philtrum_depth() -> PhiltrumDepthParams {
    PhiltrumDepthParams {
        depth: 0.5,
        width: 0.5,
        length: 0.5,
        definition: 0.5,
    }
}

/// Evaluate philtrum depth morph.
#[allow(dead_code)]
pub fn evaluate_philtrum_depth(params: &PhiltrumDepthParams) -> PhiltrumDepthResult {
    let d = params.depth.clamp(0.0, 1.0);
    let w = params.width.clamp(0.0, 1.0);
    let def = params.definition.clamp(0.0, 1.0);
    PhiltrumDepthResult {
        depth_weight: d,
        width_weight: w,
        definition_weight: def,
        combined_weight: d * 0.5 + w * 0.2 + def * 0.3,
    }
}

/// Blend philtrum depth params.
#[allow(dead_code)]
pub fn blend_philtrum_depth(a: &PhiltrumDepthParams, b: &PhiltrumDepthParams, t: f32) -> PhiltrumDepthParams {
    let t = t.clamp(0.0, 1.0);
    PhiltrumDepthParams {
        depth: a.depth + (b.depth - a.depth) * t,
        width: a.width + (b.width - a.width) * t,
        length: a.length + (b.length - a.length) * t,
        definition: a.definition + (b.definition - a.definition) * t,
    }
}

/// Set philtrum depth.
#[allow(dead_code)]
pub fn set_philtrum_depth(params: &mut PhiltrumDepthParams, value: f32) {
    params.depth = value.clamp(0.0, 1.0);
}

/// Validate params.
#[allow(dead_code)]
pub fn is_valid_philtrum_depth(params: &PhiltrumDepthParams) -> bool {
    (0.0..=1.0).contains(&params.depth)
        && (0.0..=1.0).contains(&params.width)
        && (0.0..=1.0).contains(&params.length)
        && (0.0..=1.0).contains(&params.definition)
}

/// Reset to defaults.
#[allow(dead_code)]
pub fn reset_philtrum_depth(params: &mut PhiltrumDepthParams) {
    *params = default_philtrum_depth();
}

/// Compute groove prominence.
#[allow(dead_code)]
pub fn groove_prominence(params: &PhiltrumDepthParams) -> f32 {
    (params.depth * params.definition).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let p = default_philtrum_depth();
        assert!((p.depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate() {
        let p = default_philtrum_depth();
        let r = evaluate_philtrum_depth(&p);
        assert!((0.0..=1.0).contains(&r.combined_weight));
    }

    #[test]
    fn test_blend() {
        let a = default_philtrum_depth();
        let mut b = default_philtrum_depth();
        b.depth = 1.0;
        let c = blend_philtrum_depth(&a, &b, 0.5);
        assert!((c.depth - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set() {
        let mut p = default_philtrum_depth();
        set_philtrum_depth(&mut p, 0.8);
        assert!((p.depth - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_valid() {
        assert!(is_valid_philtrum_depth(&default_philtrum_depth()));
    }

    #[test]
    fn test_invalid() {
        let p = PhiltrumDepthParams { depth: 2.0, width: 0.5, length: 0.5, definition: 0.5 };
        assert!(!is_valid_philtrum_depth(&p));
    }

    #[test]
    fn test_reset() {
        let mut p = PhiltrumDepthParams { depth: 0.9, width: 0.1, length: 0.2, definition: 0.3 };
        reset_philtrum_depth(&mut p);
        assert!((p.depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_groove() {
        let p = PhiltrumDepthParams { depth: 1.0, width: 0.5, length: 0.5, definition: 1.0 };
        assert!((groove_prominence(&p) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_zero_groove() {
        let p = PhiltrumDepthParams { depth: 0.0, width: 0.5, length: 0.5, definition: 1.0 };
        assert!(groove_prominence(&p).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = default_philtrum_depth();
        let c = blend_philtrum_depth(&a, &a, 0.5);
        assert!((c.depth - a.depth).abs() < 1e-6);
    }
}
