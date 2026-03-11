// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! F-Curve / graph editor view.

/// Interpolation mode for a curve key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveInterp {
    Constant,
    Linear,
    Bezier,
}

/// A key on an f-curve.
#[derive(Debug, Clone)]
pub struct CurveKey {
    pub frame: f32,
    pub value: f32,
    pub interp: CurveInterp,
}

/// A single f-curve track.
#[derive(Debug, Clone)]
pub struct FCurve {
    pub id: u32,
    pub name: String,
    pub keys: Vec<CurveKey>,
    pub visible: bool,
}

/// State for the curve editor view.
#[derive(Debug, Clone)]
pub struct CurveEditorView {
    pub curves: Vec<FCurve>,
    pub current_frame: f32,
    pub enabled: bool,
}

/// Create a new curve editor view.
pub fn new_curve_editor_view() -> CurveEditorView {
    CurveEditorView {
        curves: Vec::new(),
        current_frame: 0.0,
        enabled: true,
    }
}

/// Add an f-curve.
pub fn cev_add_curve(v: &mut CurveEditorView, id: u32, name: &str) {
    v.curves.push(FCurve {
        id,
        name: name.to_string(),
        keys: Vec::new(),
        visible: true,
    });
}

/// Add a key to a curve.
pub fn cev_add_key(
    v: &mut CurveEditorView,
    curve_id: u32,
    frame: f32,
    value: f32,
    interp: CurveInterp,
) {
    if let Some(c) = v.curves.iter_mut().find(|c| c.id == curve_id) {
        c.keys.push(CurveKey {
            frame,
            value,
            interp,
        });
        c.keys.sort_by(|a, b| {
            a.frame
                .partial_cmp(&b.frame)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
}

/// Total key count across all curves.
pub fn cev_total_keys(v: &CurveEditorView) -> usize {
    v.curves.iter().map(|c| c.keys.len()).sum()
}

/// Set current frame.
pub fn cev_set_frame(v: &mut CurveEditorView, frame: f32) {
    v.current_frame = frame;
}

/// Serialise to JSON.
pub fn cev_to_json(v: &CurveEditorView) -> String {
    format!(
        r#"{{"curve_count":{},"total_keys":{},"enabled":{}}}"#,
        v.curves.len(),
        cev_total_keys(v),
        v.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_empty() {
        let v = new_curve_editor_view();
        assert!(v.curves.is_empty() /* no curves */);
    }

    #[test]
    fn add_curve() {
        let mut v = new_curve_editor_view();
        cev_add_curve(&mut v, 1, "X Rotation");
        assert_eq!(v.curves.len(), 1 /* one curve */);
    }

    #[test]
    fn add_key_to_curve() {
        let mut v = new_curve_editor_view();
        cev_add_curve(&mut v, 1, "Scale");
        cev_add_key(&mut v, 1, 0.0, 1.0, CurveInterp::Bezier);
        assert_eq!(cev_total_keys(&v), 1 /* one key */);
    }

    #[test]
    fn keys_sorted_by_frame() {
        let mut v = new_curve_editor_view();
        cev_add_curve(&mut v, 1, "Pos");
        cev_add_key(&mut v, 1, 50.0, 1.0, CurveInterp::Linear);
        cev_add_key(&mut v, 1, 10.0, 0.0, CurveInterp::Linear);
        assert!((v.curves[0].keys[0].frame - 10.0).abs() < 1e-6 /* sorted */);
    }

    #[test]
    fn add_key_to_missing_curve_is_noop() {
        let mut v = new_curve_editor_view();
        cev_add_key(&mut v, 99, 0.0, 0.0, CurveInterp::Constant);
        assert_eq!(cev_total_keys(&v), 0 /* no keys added */);
    }

    #[test]
    fn set_frame() {
        let mut v = new_curve_editor_view();
        cev_set_frame(&mut v, 15.5);
        assert!((v.current_frame - 15.5).abs() < 1e-6 /* frame set */);
    }

    #[test]
    fn json_has_curve_count() {
        let v = new_curve_editor_view();
        assert!(cev_to_json(&v).contains("curve_count") /* json field */);
    }

    #[test]
    fn enabled_default() {
        let v = new_curve_editor_view();
        assert!(v.enabled /* enabled */);
    }
}
