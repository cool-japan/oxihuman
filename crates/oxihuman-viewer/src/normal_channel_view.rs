// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct NormalChannelView {
    pub channel_mask: [bool; 3],
    pub enabled: bool,
    pub reconstruct_z: bool,
}

pub fn new_normal_channel_view() -> NormalChannelView {
    NormalChannelView {
        channel_mask: [true, true, true],
        enabled: false,
        reconstruct_z: false,
    }
}

pub fn ncv_set_mask(v: &mut NormalChannelView, mask: [bool; 3]) {
    v.channel_mask = mask;
}

pub fn ncv_enable(v: &mut NormalChannelView) {
    v.enabled = true;
}

pub fn ncv_active_channels(v: &NormalChannelView) -> u32 {
    v.channel_mask.iter().filter(|&&b| b).count() as u32
}

pub fn ncv_is_enabled(v: &NormalChannelView) -> bool {
    v.enabled
}

pub fn ncv_to_json(v: &NormalChannelView) -> String {
    format!(
        r#"{{"channel_mask":[{},{},{}],"enabled":{},"reconstruct_z":{}}}"#,
        v.channel_mask[0], v.channel_mask[1], v.channel_mask[2], v.enabled, v.reconstruct_z
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* all channels active, disabled */
        let v = new_normal_channel_view();
        assert_eq!(ncv_active_channels(&v), 3);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_mask_all_false() {
        /* all channels off => 0 active */
        let mut v = new_normal_channel_view();
        ncv_set_mask(&mut v, [false, false, false]);
        assert_eq!(ncv_active_channels(&v), 0);
    }

    #[test]
    fn test_set_mask_partial() {
        /* one channel off => 2 active */
        let mut v = new_normal_channel_view();
        ncv_set_mask(&mut v, [true, false, true]);
        assert_eq!(ncv_active_channels(&v), 2);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_normal_channel_view();
        ncv_enable(&mut v);
        assert!(ncv_is_enabled(&v));
    }

    #[test]
    fn test_reconstruct_z_default() {
        /* reconstruct_z off by default */
        let v = new_normal_channel_view();
        assert!(!v.reconstruct_z);
    }

    #[test]
    fn test_to_json_has_channel_mask() {
        /* JSON has channel_mask */
        let v = new_normal_channel_view();
        assert!(ncv_to_json(&v).contains("channel_mask"));
    }

    #[test]
    fn test_to_json_has_reconstruct_z() {
        /* JSON has reconstruct_z */
        let v = new_normal_channel_view();
        assert!(ncv_to_json(&v).contains("reconstruct_z"));
    }

    #[test]
    fn test_active_channels_all_on() {
        /* default: 3 active */
        let v = new_normal_channel_view();
        assert_eq!(ncv_active_channels(&v), 3);
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let v = new_normal_channel_view();
        let v2 = v.clone();
        assert_eq!(v.channel_mask, v2.channel_mask);
    }
}
