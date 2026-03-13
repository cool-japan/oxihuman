// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct MipLevelView {
    pub current_mip: u32,
    pub max_mip: u32,
    pub enabled: bool,
}

pub fn new_mip_level_view() -> MipLevelView {
    MipLevelView {
        current_mip: 0,
        max_mip: 8,
        enabled: false,
    }
}

pub fn mlv_set_mip(v: &mut MipLevelView, mip: u32) {
    v.current_mip = mip.min(v.max_mip);
}

pub fn mlv_enable(v: &mut MipLevelView) {
    v.enabled = true;
}

pub fn mlv_mip_color(v: &MipLevelView) -> [f32; 3] {
    /* each mip level gets a unique hue */
    let t = if v.max_mip == 0 {
        0.0
    } else {
        v.current_mip as f32 / v.max_mip as f32
    };
    let r = (t * 2.0 - 1.0).clamp(0.0, 1.0);
    let g = (1.0 - (t * 2.0 - 1.0).abs()).clamp(0.0, 1.0);
    let b = (1.0 - t * 2.0).clamp(0.0, 1.0);
    [r, g, b]
}

pub fn mlv_level_count(v: &MipLevelView) -> u32 {
    v.max_mip + 1
}

pub fn mlv_is_enabled(v: &MipLevelView) -> bool {
    v.enabled
}

pub fn mlv_to_json(v: &MipLevelView) -> String {
    format!(
        r#"{{"current_mip":{},"max_mip":{},"enabled":{}}}"#,
        v.current_mip, v.max_mip, v.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* new at mip 0, disabled */
        let v = new_mip_level_view();
        assert_eq!(v.current_mip, 0);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_mip() {
        /* mip stored */
        let mut v = new_mip_level_view();
        mlv_set_mip(&mut v, 3);
        assert_eq!(v.current_mip, 3);
    }

    #[test]
    fn test_set_mip_clamped() {
        /* mip clamped to max */
        let mut v = new_mip_level_view();
        mlv_set_mip(&mut v, 100);
        assert_eq!(v.current_mip, v.max_mip);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_mip_level_view();
        mlv_enable(&mut v);
        assert!(mlv_is_enabled(&v));
    }

    #[test]
    fn test_mip_color_range() {
        /* color components in [0,1] */
        let v = new_mip_level_view();
        let c = mlv_mip_color(&v);
        for ch in c {
            assert!((0.0..=1.0).contains(&ch));
        }
    }

    #[test]
    fn test_level_count() {
        /* max_mip=8 => 9 levels */
        let v = new_mip_level_view();
        assert_eq!(mlv_level_count(&v), 9);
    }

    #[test]
    fn test_to_json() {
        /* JSON has current_mip */
        let v = new_mip_level_view();
        assert!(mlv_to_json(&v).contains("current_mip"));
    }

    #[test]
    fn test_to_json_enabled_false() {
        /* new view JSON shows disabled */
        let v = new_mip_level_view();
        assert!(mlv_to_json(&v).contains("false"));
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let v = new_mip_level_view();
        let v2 = v.clone();
        assert_eq!(v.max_mip, v2.max_mip);
    }
}
