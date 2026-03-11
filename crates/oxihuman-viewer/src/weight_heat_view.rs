// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct WeightHeatView {
    pub enabled: bool,
    pub bone_index: usize,
    pub show_all_bones: bool,
}

pub fn new_weight_heat_view() -> WeightHeatView {
    WeightHeatView {
        enabled: false,
        bone_index: 0,
        show_all_bones: false,
    }
}

pub fn whv_set_bone(v: &mut WeightHeatView, idx: usize) {
    v.bone_index = idx;
}

pub fn whv_enable(v: &mut WeightHeatView) {
    v.enabled = true;
}

pub fn whv_toggle_all_bones(v: &mut WeightHeatView) {
    v.show_all_bones = !v.show_all_bones;
}

pub fn whv_weight_color(weight: f32) -> [f32; 3] {
    let t = weight.clamp(0.0, 1.0);
    /* blue (no influence) -> red (full influence) */
    [t, 0.0, 1.0 - t]
}

pub fn whv_is_enabled(v: &WeightHeatView) -> bool {
    v.enabled
}

pub fn whv_to_json(v: &WeightHeatView) -> String {
    format!(
        r#"{{"enabled":{},"bone_index":{},"show_all_bones":{}}}"#,
        v.enabled, v.bone_index, v.show_all_bones
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* disabled, bone 0 */
        let v = new_weight_heat_view();
        assert!(!v.enabled);
        assert_eq!(v.bone_index, 0);
        assert!(!v.show_all_bones);
    }

    #[test]
    fn test_set_bone() {
        /* bone index stored */
        let mut v = new_weight_heat_view();
        whv_set_bone(&mut v, 5);
        assert_eq!(v.bone_index, 5);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_weight_heat_view();
        whv_enable(&mut v);
        assert!(whv_is_enabled(&v));
    }

    #[test]
    fn test_toggle_all_bones() {
        /* toggle flips flag */
        let mut v = new_weight_heat_view();
        whv_toggle_all_bones(&mut v);
        assert!(v.show_all_bones);
    }

    #[test]
    fn test_weight_color_zero() {
        /* zero weight -> blue */
        let c = whv_weight_color(0.0);
        assert_eq!(c, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_weight_color_full() {
        /* full weight -> red */
        let c = whv_weight_color(1.0);
        assert_eq!(c, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_weight_color_range() {
        /* mid weight in range */
        let c = whv_weight_color(0.5);
        for ch in c {
            assert!((0.0..=1.0).contains(&ch));
        }
    }

    #[test]
    fn test_to_json() {
        /* JSON has bone_index */
        let v = new_weight_heat_view();
        assert!(whv_to_json(&v).contains("bone_index"));
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let v = new_weight_heat_view();
        let v2 = v.clone();
        assert_eq!(v.bone_index, v2.bone_index);
    }
}
