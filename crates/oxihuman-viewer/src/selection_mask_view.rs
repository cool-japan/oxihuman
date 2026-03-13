// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub enum SelectionMaskMode {
    PerVertex,
    PerFace,
}

#[derive(Debug, Clone)]
pub struct SelectionMaskView {
    pub enabled: bool,
    pub mode: SelectionMaskMode,
    pub highlight_color: [f32; 3],
    pub alpha: f32,
}

pub fn new_selection_mask_view() -> SelectionMaskView {
    SelectionMaskView {
        enabled: false,
        mode: SelectionMaskMode::PerVertex,
        highlight_color: [0.2, 0.6, 1.0],
        alpha: 0.5,
    }
}

pub fn smkv_set_mode(v: &mut SelectionMaskView, mode: SelectionMaskMode) {
    v.mode = mode;
}

pub fn smkv_set_alpha(v: &mut SelectionMaskView, a: f32) {
    v.alpha = a.clamp(0.0, 1.0);
}

pub fn smkv_enable(v: &mut SelectionMaskView) {
    v.enabled = true;
}

pub fn smkv_overlay_color(v: &SelectionMaskView, selected: bool) -> [f32; 4] {
    if selected {
        [
            v.highlight_color[0],
            v.highlight_color[1],
            v.highlight_color[2],
            v.alpha,
        ]
    } else {
        [0.0, 0.0, 0.0, 0.0]
    }
}

pub fn smkv_is_enabled(v: &SelectionMaskView) -> bool {
    v.enabled
}

pub fn smkv_to_json(v: &SelectionMaskView) -> String {
    let mode_str = match v.mode {
        SelectionMaskMode::PerVertex => "per_vertex",
        SelectionMaskMode::PerFace => "per_face",
    };
    format!(
        r#"{{"enabled":{},"mode":"{}","alpha":{:.4}}}"#,
        v.enabled, mode_str, v.alpha
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* disabled, per-vertex mode */
        let v = new_selection_mask_view();
        assert!(!v.enabled);
        assert_eq!(v.mode, SelectionMaskMode::PerVertex);
        assert!((v.alpha - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_mode() {
        /* mode stored */
        let mut v = new_selection_mask_view();
        smkv_set_mode(&mut v, SelectionMaskMode::PerFace);
        assert_eq!(v.mode, SelectionMaskMode::PerFace);
    }

    #[test]
    fn test_set_alpha() {
        /* valid alpha */
        let mut v = new_selection_mask_view();
        smkv_set_alpha(&mut v, 0.8);
        assert!((v.alpha - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_alpha_clamp() {
        /* clamp to [0,1] */
        let mut v = new_selection_mask_view();
        smkv_set_alpha(&mut v, 2.0);
        assert_eq!(v.alpha, 1.0);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_selection_mask_view();
        smkv_enable(&mut v);
        assert!(smkv_is_enabled(&v));
    }

    #[test]
    fn test_overlay_color_selected() {
        /* selected returns highlight color with alpha */
        let v = new_selection_mask_view();
        let c = smkv_overlay_color(&v, true);
        assert!((c[3] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_overlay_color_unselected() {
        /* unselected returns transparent black */
        let v = new_selection_mask_view();
        let c = smkv_overlay_color(&v, false);
        assert_eq!(c, [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_to_json() {
        /* JSON has mode */
        let v = new_selection_mask_view();
        assert!(smkv_to_json(&v).contains("mode"));
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let v = new_selection_mask_view();
        let v2 = v.clone();
        assert_eq!(v.mode, v2.mode);
    }
}
