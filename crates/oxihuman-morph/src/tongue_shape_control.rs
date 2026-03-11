// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Tongue shape morph controls.

/// Available tongue shape presets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TongueShape {
    Flat,
    Tip,
    Grooved,
    Curled,
    Widened,
}

/// Weight set describing a blended tongue shape.
#[derive(Debug, Clone)]
pub struct TongueShapeWeights {
    pub flat: f32,
    pub tip: f32,
    pub grooved: f32,
    pub curled: f32,
    pub widened: f32,
}

impl Default for TongueShapeWeights {
    fn default() -> Self {
        Self { flat: 1.0, tip: 0.0, grooved: 0.0, curled: 0.0, widened: 0.0 }
    }
}

impl TongueShapeWeights {
    /// Create weights from a single dominant shape.
    pub fn from_shape(shape: TongueShape) -> Self {
        let (flat, tip, grooved, curled, widened) = match shape {
            TongueShape::Flat => (1.0, 0.0, 0.0, 0.0, 0.0),
            TongueShape::Tip => (0.0, 1.0, 0.0, 0.0, 0.0),
            TongueShape::Grooved => (0.0, 0.0, 1.0, 0.0, 0.0),
            TongueShape::Curled => (0.0, 0.0, 0.0, 1.0, 0.0),
            TongueShape::Widened => (0.0, 0.0, 0.0, 0.0, 1.0),
        };
        Self { flat, tip, grooved, curled, widened }
    }

    /// Normalise so all weights sum to 1.0.
    pub fn normalize(&mut self) {
        let sum = self.flat + self.tip + self.grooved + self.curled + self.widened;
        if sum > 1e-6 {
            let inv = 1.0 / sum;
            self.flat *= inv;
            self.tip *= inv;
            self.grooved *= inv;
            self.curled *= inv;
            self.widened *= inv;
        }
    }
}

/// Set the tongue protrusion (0 = retracted, 1 = fully extended).
pub fn tongue_set_protrusion(weights: &mut TongueShapeWeights, amount: f32) {
    let _ = amount.clamp(0.0, 1.0);
    /* protrusion is a meta-control — stored indirectly via tip weight */
    weights.tip = amount.clamp(0.0, 1.0);
}

/// Blend two tongue shape weight sets.
pub fn blend_tongue_shapes(a: &TongueShapeWeights, b: &TongueShapeWeights, t: f32) -> TongueShapeWeights {
    let t = t.clamp(0.0, 1.0);
    TongueShapeWeights {
        flat: a.flat + (b.flat - a.flat) * t,
        tip: a.tip + (b.tip - a.tip) * t,
        grooved: a.grooved + (b.grooved - a.grooved) * t,
        curled: a.curled + (b.curled - a.curled) * t,
        widened: a.widened + (b.widened - a.widened) * t,
    }
}

/// Return the dominant shape for a weight set.
pub fn dominant_tongue_shape(w: &TongueShapeWeights) -> TongueShape {
    let arr = [
        (w.flat, TongueShape::Flat),
        (w.tip, TongueShape::Tip),
        (w.grooved, TongueShape::Grooved),
        (w.curled, TongueShape::Curled),
        (w.widened, TongueShape::Widened),
    ];
    arr.iter().max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)).map(|p| p.1).unwrap_or(TongueShape::Flat)
}

/// Reset tongue weights to neutral flat.
pub fn tongue_reset(w: &mut TongueShapeWeights) {
    *w = TongueShapeWeights::default();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_flat() {
        /* default shape should be flat */
        let w = TongueShapeWeights::default();
        assert!((w.flat - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_from_shape_tip() {
        /* from_shape(Tip) should have tip=1 and rest=0 */
        let w = TongueShapeWeights::from_shape(TongueShape::Tip);
        assert!((w.tip - 1.0).abs() < 1e-6);
        assert_eq!(w.flat, 0.0);
    }

    #[test]
    fn test_normalize() {
        /* normalize should bring sum to 1 */
        let mut w = TongueShapeWeights { flat: 2.0, tip: 2.0, grooved: 0.0, curled: 0.0, widened: 0.0 };
        w.normalize();
        let sum = w.flat + w.tip;
        assert!((sum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_blend_midpoint() {
        /* midpoint blend should average values */
        let a = TongueShapeWeights::from_shape(TongueShape::Flat);
        let b = TongueShapeWeights::from_shape(TongueShape::Curled);
        let m = blend_tongue_shapes(&a, &b, 0.5);
        assert!((m.flat - 0.5).abs() < 1e-6);
        assert!((m.curled - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_dominant_shape() {
        /* dominant should return the highest weight shape */
        let w = TongueShapeWeights::from_shape(TongueShape::Grooved);
        assert_eq!(dominant_tongue_shape(&w), TongueShape::Grooved);
    }

    #[test]
    fn test_protrusion_sets_tip() {
        /* protrusion control should update tip weight */
        let mut w = TongueShapeWeights::default();
        tongue_set_protrusion(&mut w, 0.7);
        assert!((w.tip - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        /* reset should restore default flat state */
        let mut w = TongueShapeWeights::from_shape(TongueShape::Curled);
        tongue_reset(&mut w);
        assert!((w.flat - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity_at_zero() {
        /* blend at t=0 should return first arg unchanged */
        let a = TongueShapeWeights::from_shape(TongueShape::Widened);
        let b = TongueShapeWeights::from_shape(TongueShape::Tip);
        let m = blend_tongue_shapes(&a, &b, 0.0);
        assert!((m.widened - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity_at_one() {
        /* blend at t=1 should return second arg unchanged */
        let a = TongueShapeWeights::from_shape(TongueShape::Grooved);
        let b = TongueShapeWeights::from_shape(TongueShape::Tip);
        let m = blend_tongue_shapes(&a, &b, 1.0);
        assert!((m.tip - 1.0).abs() < 1e-6);
    }
}
