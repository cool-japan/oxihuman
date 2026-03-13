// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Depth peeling transparency visualization.
#[derive(Debug, Clone)]
pub struct DepthPeelView {
    pub enabled: bool,
    /// Number of depth layers to peel.
    pub layer_count: u32,
    /// Overall alpha for the depth peel compositing.
    pub alpha: f32,
}

pub fn new_depth_peel_view() -> DepthPeelView {
    DepthPeelView {
        enabled: false,
        layer_count: 4,
        alpha: 0.5,
    }
}

pub fn dpv_enable(v: &mut DepthPeelView) {
    v.enabled = true;
}

pub fn dpv_set_layer_count(v: &mut DepthPeelView, n: u32) {
    v.layer_count = n.clamp(1, 16);
}

pub fn dpv_set_alpha(v: &mut DepthPeelView, a: f32) {
    v.alpha = a.clamp(0.0, 1.0);
}

/// Returns the alpha for a given peel layer (layers further back are more transparent).
pub fn dpv_layer_alpha(v: &DepthPeelView, layer: u32) -> f32 {
    let t = if v.layer_count <= 1 {
        1.0
    } else {
        1.0 - layer as f32 / (v.layer_count - 1) as f32
    };
    (v.alpha * t).clamp(0.0, 1.0)
}

/// Returns colour for a peel layer for visual debugging.
pub fn dpv_layer_color(layer: u32, total: u32) -> [f32; 3] {
    let t = if total == 0 {
        0.0
    } else {
        layer as f32 / total as f32
    };
    [t, 0.5, 1.0 - t]
}

pub fn dpv_to_json(v: &DepthPeelView) -> String {
    format!(
        r#"{{"enabled":{},"layer_count":{},"alpha":{:.4}}}"#,
        v.enabled, v.layer_count, v.alpha
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        /* layer_count=4, alpha=0.5 */
        let v = new_depth_peel_view();
        assert_eq!(v.layer_count, 4);
        assert!((v.alpha - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_enable() {
        /* enable */
        let mut v = new_depth_peel_view();
        dpv_enable(&mut v);
        assert!(v.enabled);
    }

    #[test]
    fn test_set_layer_count() {
        /* valid count */
        let mut v = new_depth_peel_view();
        dpv_set_layer_count(&mut v, 8);
        assert_eq!(v.layer_count, 8);
    }

    #[test]
    fn test_layer_count_clamp_max() {
        /* clamped at 16 */
        let mut v = new_depth_peel_view();
        dpv_set_layer_count(&mut v, 100);
        assert_eq!(v.layer_count, 16);
    }

    #[test]
    fn test_layer_count_clamp_min() {
        /* clamped at 1 */
        let mut v = new_depth_peel_view();
        dpv_set_layer_count(&mut v, 0);
        assert_eq!(v.layer_count, 1);
    }

    #[test]
    fn test_alpha_set() {
        /* alpha stored */
        let mut v = new_depth_peel_view();
        dpv_set_alpha(&mut v, 0.7);
        assert!((v.alpha - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_layer_alpha_first() {
        /* first layer has full relative alpha */
        let v = new_depth_peel_view();
        let a = dpv_layer_alpha(&v, 0);
        assert!((a - v.alpha).abs() < 1e-6);
    }

    #[test]
    fn test_layer_color_range() {
        /* colour components in [0,1] */
        let c = dpv_layer_color(2, 4);
        for ch in c {
            assert!((0.0..=1.0).contains(&ch));
        }
    }

    #[test]
    fn test_to_json() {
        /* JSON has layer_count */
        assert!(dpv_to_json(&new_depth_peel_view()).contains("layer_count"));
    }
}
