// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Alpha blending utilities for compositing transparent layers.

/// Blend mode for alpha compositing.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlphaBlendMode {
    Normal,
    Additive,
    Multiply,
    Screen,
    Overlay,
}

/// A single RGBA color with alpha.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[allow(dead_code)]
pub fn new_rgba(r: f32, g: f32, b: f32, a: f32) -> Rgba {
    Rgba {
        r: r.clamp(0.0, 1.0),
        g: g.clamp(0.0, 1.0),
        b: b.clamp(0.0, 1.0),
        a: a.clamp(0.0, 1.0),
    }
}

#[allow(dead_code)]
pub fn blend_normal(src: &Rgba, dst: &Rgba) -> Rgba {
    let out_a = src.a + dst.a * (1.0 - src.a);
    if out_a < 1e-9 {
        return new_rgba(0.0, 0.0, 0.0, 0.0);
    }
    let inv = 1.0 / out_a;
    Rgba {
        r: (src.r * src.a + dst.r * dst.a * (1.0 - src.a)) * inv,
        g: (src.g * src.a + dst.g * dst.a * (1.0 - src.a)) * inv,
        b: (src.b * src.a + dst.b * dst.a * (1.0 - src.a)) * inv,
        a: out_a,
    }
}

#[allow(dead_code)]
pub fn blend_additive(src: &Rgba, dst: &Rgba) -> Rgba {
    Rgba {
        r: (src.r * src.a + dst.r).min(1.0),
        g: (src.g * src.a + dst.g).min(1.0),
        b: (src.b * src.a + dst.b).min(1.0),
        a: (src.a + dst.a).min(1.0),
    }
}

#[allow(dead_code)]
pub fn blend_multiply(src: &Rgba, dst: &Rgba) -> Rgba {
    Rgba {
        r: src.r * dst.r,
        g: src.g * dst.g,
        b: src.b * dst.b,
        a: src.a * dst.a,
    }
}

#[allow(dead_code)]
pub fn blend_screen(src: &Rgba, dst: &Rgba) -> Rgba {
    Rgba {
        r: 1.0 - (1.0 - src.r) * (1.0 - dst.r),
        g: 1.0 - (1.0 - src.g) * (1.0 - dst.g),
        b: 1.0 - (1.0 - src.b) * (1.0 - dst.b),
        a: (src.a + dst.a).min(1.0),
    }
}

#[allow(dead_code)]
pub fn apply_blend(src: &Rgba, dst: &Rgba, mode: AlphaBlendMode) -> Rgba {
    match mode {
        AlphaBlendMode::Normal => blend_normal(src, dst),
        AlphaBlendMode::Additive => blend_additive(src, dst),
        AlphaBlendMode::Multiply => blend_multiply(src, dst),
        AlphaBlendMode::Screen => blend_screen(src, dst),
        AlphaBlendMode::Overlay => {
            // Simplified overlay
            let r = if dst.r < 0.5 {
                2.0 * src.r * dst.r
            } else {
                1.0 - 2.0 * (1.0 - src.r) * (1.0 - dst.r)
            };
            let g = if dst.g < 0.5 {
                2.0 * src.g * dst.g
            } else {
                1.0 - 2.0 * (1.0 - src.g) * (1.0 - dst.g)
            };
            let b = if dst.b < 0.5 {
                2.0 * src.b * dst.b
            } else {
                1.0 - 2.0 * (1.0 - src.b) * (1.0 - dst.b)
            };
            Rgba {
                r,
                g,
                b,
                a: (src.a + dst.a).min(1.0),
            }
        }
    }
}

#[allow(dead_code)]
pub fn premultiply_alpha(c: &Rgba) -> Rgba {
    Rgba {
        r: c.r * c.a,
        g: c.g * c.a,
        b: c.b * c.a,
        a: c.a,
    }
}

#[allow(dead_code)]
pub fn alpha_blend_mode_name(mode: AlphaBlendMode) -> &'static str {
    match mode {
        AlphaBlendMode::Normal => "normal",
        AlphaBlendMode::Additive => "additive",
        AlphaBlendMode::Multiply => "multiply",
        AlphaBlendMode::Screen => "screen",
        AlphaBlendMode::Overlay => "overlay",
    }
}

#[allow(dead_code)]
pub fn rgba_to_json(c: &Rgba) -> String {
    format!(
        r#"{{"r":{:.4},"g":{:.4},"b":{:.4},"a":{:.4}}}"#,
        c.r, c.g, c.b, c.a
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_rgba_clamps() {
        let c = new_rgba(2.0, -1.0, 0.5, 0.8);
        assert!((c.r - 1.0).abs() < 1e-6);
        assert!(c.g.abs() < 1e-6);
    }

    #[test]
    fn test_blend_normal_opaque() {
        let src = new_rgba(1.0, 0.0, 0.0, 1.0);
        let dst = new_rgba(0.0, 1.0, 0.0, 1.0);
        let out = blend_normal(&src, &dst);
        assert!((out.r - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_normal_transparent() {
        let src = new_rgba(1.0, 0.0, 0.0, 0.0);
        let dst = new_rgba(0.0, 1.0, 0.0, 1.0);
        let out = blend_normal(&src, &dst);
        assert!((out.g - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_additive() {
        let src = new_rgba(0.5, 0.0, 0.0, 1.0);
        let dst = new_rgba(0.3, 0.0, 0.0, 1.0);
        let out = blend_additive(&src, &dst);
        assert!((out.r - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_blend_multiply() {
        let src = new_rgba(0.5, 0.5, 0.5, 1.0);
        let dst = new_rgba(0.5, 0.5, 0.5, 1.0);
        let out = blend_multiply(&src, &dst);
        assert!((out.r - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_blend_screen() {
        let src = new_rgba(0.5, 0.0, 0.0, 1.0);
        let dst = new_rgba(0.5, 0.0, 0.0, 1.0);
        let out = blend_screen(&src, &dst);
        assert!((out.r - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_apply_blend_dispatch() {
        let src = new_rgba(1.0, 0.0, 0.0, 1.0);
        let dst = new_rgba(0.0, 1.0, 0.0, 1.0);
        let out = apply_blend(&src, &dst, AlphaBlendMode::Normal);
        assert!((out.r - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_premultiply_alpha() {
        let c = new_rgba(1.0, 1.0, 1.0, 0.5);
        let pm = premultiply_alpha(&c);
        assert!((pm.r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_mode_name() {
        assert_eq!(alpha_blend_mode_name(AlphaBlendMode::Additive), "additive");
    }

    #[test]
    fn test_rgba_to_json() {
        let c = new_rgba(1.0, 0.0, 0.5, 1.0);
        let j = rgba_to_json(&c);
        assert!(j.contains("\"r\":"));
    }
}
