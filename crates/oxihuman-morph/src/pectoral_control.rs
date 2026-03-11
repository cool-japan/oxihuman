// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Pectoral muscle morphology control.

/// Pectoral parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PectoralParams {
    pub size: f32,
    pub definition: f32,
    pub spread: f32,
    pub asymmetry: f32,
}

/// Pectoral result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PectoralResult {
    pub left_weight: f32,
    pub right_weight: f32,
    pub size_weight: f32,
    pub definition_weight: f32,
}

/// Default pectoral parameters.
#[allow(dead_code)]
pub fn default_pectoral() -> PectoralParams {
    PectoralParams {
        size: 0.5,
        definition: 0.3,
        spread: 0.5,
        asymmetry: 0.0,
    }
}

/// Evaluate pectoral morph.
#[allow(dead_code)]
pub fn evaluate_pectoral(params: &PectoralParams) -> PectoralResult {
    let s = params.size.clamp(0.0, 1.0);
    let d = params.definition.clamp(0.0, 1.0);
    let asym = params.asymmetry.clamp(-1.0, 1.0);
    let base = s * 0.6 + d * 0.4;
    PectoralResult {
        left_weight: (base + asym * 0.3).clamp(0.0, 1.0),
        right_weight: (base - asym * 0.3).clamp(0.0, 1.0),
        size_weight: s,
        definition_weight: d,
    }
}

/// Blend pectoral params.
#[allow(dead_code)]
pub fn blend_pectoral(a: &PectoralParams, b: &PectoralParams, t: f32) -> PectoralParams {
    let t = t.clamp(0.0, 1.0);
    PectoralParams {
        size: a.size + (b.size - a.size) * t,
        definition: a.definition + (b.definition - a.definition) * t,
        spread: a.spread + (b.spread - a.spread) * t,
        asymmetry: a.asymmetry + (b.asymmetry - a.asymmetry) * t,
    }
}

/// Set pectoral size.
#[allow(dead_code)]
pub fn set_pectoral_size(params: &mut PectoralParams, value: f32) {
    params.size = value.clamp(0.0, 1.0);
}

/// Validate params.
#[allow(dead_code)]
pub fn is_valid_pectoral(params: &PectoralParams) -> bool {
    (0.0..=1.0).contains(&params.size)
        && (0.0..=1.0).contains(&params.definition)
        && (0.0..=1.0).contains(&params.spread)
        && (-1.0..=1.0).contains(&params.asymmetry)
}

/// Reset to defaults.
#[allow(dead_code)]
pub fn reset_pectoral(params: &mut PectoralParams) {
    *params = default_pectoral();
}

/// Compute mass estimate.
#[allow(dead_code)]
pub fn pectoral_mass(params: &PectoralParams) -> f32 {
    (params.size * params.spread).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let p = default_pectoral();
        assert!((p.size - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate() {
        let p = default_pectoral();
        let r = evaluate_pectoral(&p);
        assert!((0.0..=1.0).contains(&r.left_weight));
    }

    #[test]
    fn test_blend() {
        let a = default_pectoral();
        let mut b = default_pectoral();
        b.size = 1.0;
        let c = blend_pectoral(&a, &b, 0.5);
        assert!((c.size - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set_size() {
        let mut p = default_pectoral();
        set_pectoral_size(&mut p, 0.8);
        assert!((p.size - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_valid() {
        assert!(is_valid_pectoral(&default_pectoral()));
    }

    #[test]
    fn test_invalid() {
        let p = PectoralParams { size: 2.0, definition: 0.5, spread: 0.5, asymmetry: 0.0 };
        assert!(!is_valid_pectoral(&p));
    }

    #[test]
    fn test_reset() {
        let mut p = PectoralParams { size: 0.9, definition: 0.1, spread: 0.2, asymmetry: 0.3 };
        reset_pectoral(&mut p);
        assert!((p.size - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_mass() {
        let p = PectoralParams { size: 1.0, definition: 0.5, spread: 1.0, asymmetry: 0.0 };
        assert!((pectoral_mass(&p) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_asymmetry() {
        let p = PectoralParams { size: 0.5, definition: 0.5, spread: 0.5, asymmetry: 1.0 };
        let r = evaluate_pectoral(&p);
        assert!(r.left_weight > r.right_weight);
    }

    #[test]
    fn test_blend_identity() {
        let a = default_pectoral();
        let c = blend_pectoral(&a, &a, 0.5);
        assert!((c.size - a.size).abs() < 1e-6);
    }
}
