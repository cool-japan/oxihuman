// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Chroma key matte debug view for green/blue screen compositing.

/// Chroma key view configuration.
#[derive(Debug, Clone)]
pub struct ChromaKeyView {
    pub key_color: [f32; 3],
    pub tolerance: f32,
    pub softness: f32,
    pub spill_suppression: f32,
    pub invert: bool,
    pub enabled: bool,
}

impl ChromaKeyView {
    pub fn new() -> Self {
        Self {
            key_color: [0.0, 1.0, 0.0], /* green screen default */
            tolerance: 0.3,
            softness: 0.1,
            spill_suppression: 0.5,
            invert: false,
            enabled: false,
        }
    }
}

impl Default for ChromaKeyView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new chroma key view.
pub fn new_chroma_key_view() -> ChromaKeyView {
    ChromaKeyView::new()
}

/// Set the key color as RGB.
pub fn ckv_set_key_color(view: &mut ChromaKeyView, r: f32, g: f32, b: f32) {
    view.key_color = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)];
}

/// Set tolerance for chroma matching.
pub fn ckv_set_tolerance(view: &mut ChromaKeyView, tolerance: f32) {
    view.tolerance = tolerance.clamp(0.0, 1.0);
}

/// Set edge softness for smooth keying.
pub fn ckv_set_softness(view: &mut ChromaKeyView, softness: f32) {
    view.softness = softness.clamp(0.0, 1.0);
}

/// Set spill suppression level.
pub fn ckv_set_spill_suppression(view: &mut ChromaKeyView, level: f32) {
    view.spill_suppression = level.clamp(0.0, 1.0);
}

/// Toggle matte inversion.
pub fn ckv_set_invert(view: &mut ChromaKeyView, invert: bool) {
    view.invert = invert;
}

/// Compute chroma distance between two RGB colors.
pub fn ckv_chroma_distance(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    let dr = a[0] - b[0];
    let dg = a[1] - b[1];
    let db = a[2] - b[2];
    (dr * dr + dg * dg + db * db).sqrt()
}

/// Evaluate matte alpha for a given pixel color.
pub fn ckv_evaluate(view: &ChromaKeyView, pixel: &[f32; 3]) -> f32 {
    let dist = ckv_chroma_distance(pixel, &view.key_color);
    let soft_lo = view.tolerance;
    let soft_hi = (view.tolerance + view.softness).min(1.0);
    let alpha = if dist < soft_lo {
        0.0
    } else if dist < soft_hi {
        let t = (dist - soft_lo) / (soft_hi - soft_lo + 1e-6);
        t.clamp(0.0, 1.0)
    } else {
        1.0
    };
    if view.invert {
        1.0 - alpha
    } else {
        alpha
    }
}

/// Serialize to JSON-like string.
pub fn chroma_key_view_to_json(view: &ChromaKeyView) -> String {
    format!(
        r#"{{"key_color":[{:.4},{:.4},{:.4}],"tolerance":{:.4},"enabled":{}}}"#,
        view.key_color[0], view.key_color[1], view.key_color[2], view.tolerance, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_chroma_key_view();
        assert!((v.key_color[1] - 1.0).abs() < 1e-6); /* green */
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_key_color() {
        let mut v = new_chroma_key_view();
        ckv_set_key_color(&mut v, 0.0, 0.0, 1.0);
        assert!((v.key_color[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_tolerance_clamp() {
        let mut v = new_chroma_key_view();
        ckv_set_tolerance(&mut v, 2.0);
        assert_eq!(v.tolerance, 1.0);
    }

    #[test]
    fn test_softness_set() {
        let mut v = new_chroma_key_view();
        ckv_set_softness(&mut v, 0.2);
        assert!((v.softness - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_keyed_out() {
        let v = new_chroma_key_view();
        /* pure green == key color, distance 0, alpha 0 */
        let alpha = ckv_evaluate(&v, &[0.0, 1.0, 0.0]);
        assert_eq!(alpha, 0.0);
    }

    #[test]
    fn test_evaluate_not_keyed() {
        let v = new_chroma_key_view();
        /* red is far from green key */
        let alpha = ckv_evaluate(&v, &[1.0, 0.0, 0.0]);
        assert!(alpha > 0.5);
    }

    #[test]
    fn test_spill_suppression_set() {
        let mut v = new_chroma_key_view();
        ckv_set_spill_suppression(&mut v, 0.8);
        assert!((v.spill_suppression - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_json() {
        let v = new_chroma_key_view();
        let s = chroma_key_view_to_json(&v);
        assert!(s.contains("key_color"));
    }

    #[test]
    fn test_clone() {
        let v = new_chroma_key_view();
        let v2 = v.clone();
        assert!((v2.tolerance - v.tolerance).abs() < 1e-6);
    }
}
