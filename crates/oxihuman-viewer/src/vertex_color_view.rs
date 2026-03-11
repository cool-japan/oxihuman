// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct VertexColorView {
    pub channel: u32,
    pub enabled: bool,
    pub alpha_as_mask: bool,
}

pub fn new_vertex_color_view() -> VertexColorView {
    VertexColorView {
        channel: 0,
        enabled: false,
        alpha_as_mask: false,
    }
}

pub fn vcv_set_channel(v: &mut VertexColorView, ch: u32) {
    v.channel = ch.min(4);
}

pub fn vcv_enable(v: &mut VertexColorView) {
    v.enabled = true;
}

pub fn vcv_channel_name(v: &VertexColorView) -> &str {
    match v.channel {
        0 => "RGBA",
        1 => "R",
        2 => "G",
        3 => "B",
        4 => "A",
        _ => "RGBA",
    }
}

pub fn vcv_is_enabled(v: &VertexColorView) -> bool {
    v.enabled
}

pub fn vcv_to_json(v: &VertexColorView) -> String {
    format!(
        r#"{{"channel":{},"enabled":{},"alpha_as_mask":{}}}"#,
        v.channel, v.enabled, v.alpha_as_mask
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* channel 0, disabled */
        let v = new_vertex_color_view();
        assert_eq!(v.channel, 0);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_channel() {
        /* channel stored */
        let mut v = new_vertex_color_view();
        vcv_set_channel(&mut v, 2);
        assert_eq!(v.channel, 2);
    }

    #[test]
    fn test_set_channel_clamped() {
        /* channel clamped to 4 */
        let mut v = new_vertex_color_view();
        vcv_set_channel(&mut v, 10);
        assert_eq!(v.channel, 4);
    }

    #[test]
    fn test_channel_name_rgba() {
        /* channel 0 => RGBA */
        let v = new_vertex_color_view();
        assert_eq!(vcv_channel_name(&v), "RGBA");
    }

    #[test]
    fn test_channel_names() {
        /* all channel names */
        let mut v = new_vertex_color_view();
        vcv_set_channel(&mut v, 1);
        assert_eq!(vcv_channel_name(&v), "R");
        vcv_set_channel(&mut v, 3);
        assert_eq!(vcv_channel_name(&v), "B");
        vcv_set_channel(&mut v, 4);
        assert_eq!(vcv_channel_name(&v), "A");
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_vertex_color_view();
        vcv_enable(&mut v);
        assert!(vcv_is_enabled(&v));
    }

    #[test]
    fn test_to_json_channel() {
        /* JSON has channel */
        let v = new_vertex_color_view();
        assert!(vcv_to_json(&v).contains("channel"));
    }

    #[test]
    fn test_to_json_alpha_mask() {
        /* JSON has alpha_as_mask */
        let v = new_vertex_color_view();
        assert!(vcv_to_json(&v).contains("alpha_as_mask"));
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let v = new_vertex_color_view();
        let v2 = v.clone();
        assert_eq!(v.channel, v2.channel);
    }
}
