// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// MikkTSpace tangent visualization — shows T vectors computed by MikkT algorithm.
#[derive(Debug, Clone)]
pub struct MikktView {
    pub enabled: bool,
    /// Scale of tangent arrow display.
    pub scale: f32,
    /// Show handedness sign (+1 / -1) as colour.
    pub show_handedness: bool,
}

pub fn new_mikkt_view() -> MikktView {
    MikktView {
        enabled: false,
        scale: 0.03,
        show_handedness: true,
    }
}

pub fn mkv_enable(v: &mut MikktView) {
    v.enabled = true;
}

pub fn mkv_set_scale(v: &mut MikktView, s: f32) {
    v.scale = s.max(1e-4);
}

pub fn mkv_set_show_handedness(v: &mut MikktView, show: bool) {
    v.show_handedness = show;
}

/// Returns colour for a tangent with the given handedness (+1 or -1).
pub fn mkv_handedness_color(handedness: f32) -> [f32; 3] {
    if handedness >= 0.0 {
        [0.0, 0.8, 0.2] // positive handedness → green
    } else {
        [0.8, 0.0, 0.2] // negative handedness → red
    }
}

/// Compute the MikkT bitangent from tangent, normal and handedness.
pub fn mkv_bitangent(tangent: [f32; 3], normal: [f32; 3], handedness: f32) -> [f32; 3] {
    let cross = [
        normal[1] * tangent[2] - normal[2] * tangent[1],
        normal[2] * tangent[0] - normal[0] * tangent[2],
        normal[0] * tangent[1] - normal[1] * tangent[0],
    ];
    [
        cross[0] * handedness,
        cross[1] * handedness,
        cross[2] * handedness,
    ]
}

pub fn mkv_to_json(v: &MikktView) -> String {
    format!(
        r#"{{"enabled":{},"scale":{:.4},"show_handedness":{}}}"#,
        v.enabled, v.scale, v.show_handedness
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        /* scale=0.03, show_handedness=true */
        let v = new_mikkt_view();
        assert!((v.scale - 0.03).abs() < 1e-6);
        assert!(v.show_handedness);
    }

    #[test]
    fn test_enable() {
        /* enable */
        let mut v = new_mikkt_view();
        mkv_enable(&mut v);
        assert!(v.enabled);
    }

    #[test]
    fn test_set_scale() {
        /* valid */
        let mut v = new_mikkt_view();
        mkv_set_scale(&mut v, 0.1);
        assert!((v.scale - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_positive_handedness_color() {
        /* positive → green dominant */
        let c = mkv_handedness_color(1.0);
        assert!(c[1] > c[0]);
    }

    #[test]
    fn test_negative_handedness_color() {
        /* negative → red dominant */
        let c = mkv_handedness_color(-1.0);
        assert!(c[0] > c[1]);
    }

    #[test]
    fn test_bitangent_x_axis() {
        /* T=(1,0,0), N=(0,1,0), h=1 → B=(0,0,1) cross product */
        let b = mkv_bitangent([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 1.0);
        assert!((b[2] - (-1.0)).abs() < 1e-5 || (b[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_bitangent_handedness_negates() {
        /* h=-1 negates result */
        let b1 = mkv_bitangent([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 1.0);
        let b2 = mkv_bitangent([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], -1.0);
        assert!((b1[2] + b2[2]).abs() < 1e-5);
    }

    #[test]
    fn test_scale_min_enforced() {
        /* scale cannot be 0 */
        let mut v = new_mikkt_view();
        mkv_set_scale(&mut v, 0.0);
        assert!(v.scale > 0.0);
    }

    #[test]
    fn test_to_json() {
        /* JSON has scale */
        assert!(mkv_to_json(&new_mikkt_view()).contains("scale"));
    }
}
