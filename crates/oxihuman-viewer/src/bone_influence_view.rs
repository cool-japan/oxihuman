// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct BoneInfluenceView {
    pub bone_index: u32,
    pub enabled: bool,
    pub max_bones: u32,
}

pub fn new_bone_influence_view() -> BoneInfluenceView {
    BoneInfluenceView {
        bone_index: 0,
        enabled: false,
        max_bones: 256,
    }
}

pub fn biv_set_bone(v: &mut BoneInfluenceView, idx: u32) {
    v.bone_index = idx.min(v.max_bones.saturating_sub(1));
}

pub fn biv_enable(v: &mut BoneInfluenceView) {
    v.enabled = true;
}

pub fn biv_influence_color(_v: &BoneInfluenceView, weight: f32) -> [f32; 3] {
    let w = weight.clamp(0.0, 1.0);
    /* blue -> red heatmap */
    let r = w;
    let g = 0.0;
    let b = 1.0 - w;
    [r, g, b]
}

pub fn biv_is_valid_bone(v: &BoneInfluenceView) -> bool {
    v.bone_index < v.max_bones
}

pub fn biv_is_enabled(v: &BoneInfluenceView) -> bool {
    v.enabled
}

pub fn biv_to_json(v: &BoneInfluenceView) -> String {
    format!(
        r#"{{"bone_index":{},"enabled":{},"max_bones":{}}}"#,
        v.bone_index, v.enabled, v.max_bones
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* bone 0, disabled */
        let v = new_bone_influence_view();
        assert_eq!(v.bone_index, 0);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_bone() {
        /* bone index stored */
        let mut v = new_bone_influence_view();
        biv_set_bone(&mut v, 10);
        assert_eq!(v.bone_index, 10);
    }

    #[test]
    fn test_set_bone_clamped() {
        /* bone clamped to max-1 */
        let mut v = new_bone_influence_view();
        biv_set_bone(&mut v, 9999);
        assert!(v.bone_index < v.max_bones);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_bone_influence_view();
        biv_enable(&mut v);
        assert!(biv_is_enabled(&v));
    }

    #[test]
    fn test_influence_color_zero() {
        /* weight=0 => blue */
        let v = new_bone_influence_view();
        let c = biv_influence_color(&v, 0.0);
        assert!(c[2] > c[0]);
    }

    #[test]
    fn test_influence_color_one() {
        /* weight=1 => red */
        let v = new_bone_influence_view();
        let c = biv_influence_color(&v, 1.0);
        assert!(c[0] > c[2]);
    }

    #[test]
    fn test_is_valid_bone() {
        /* bone_index < max_bones => valid */
        let v = new_bone_influence_view();
        assert!(biv_is_valid_bone(&v));
    }

    #[test]
    fn test_to_json() {
        /* JSON has bone_index */
        let v = new_bone_influence_view();
        assert!(biv_to_json(&v).contains("bone_index"));
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let v = new_bone_influence_view();
        let v2 = v.clone();
        assert_eq!(v.max_bones, v2.max_bones);
    }
}
