// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Hair parting position morph control.

/// Hair part position style.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HairPartStyle {
    Center,
    Left,
    Right,
    NoParting,
}

/// Hair part morph configuration.
#[derive(Debug, Clone)]
pub struct HairPartMorph {
    pub style: HairPartStyle,
    pub offset: f32,
    pub depth: f32,
}

impl HairPartMorph {
    pub fn new() -> Self {
        Self {
            style: HairPartStyle::Center,
            offset: 0.0,
            depth: 0.5,
        }
    }
}

impl Default for HairPartMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new hair part morph.
pub fn new_hair_part_morph() -> HairPartMorph {
    HairPartMorph::new()
}

/// Set parting offset from center (negative = left, positive = right).
pub fn hair_part_set_offset(morph: &mut HairPartMorph, offset: f32) {
    morph.offset = offset.clamp(-1.0, 1.0);
}

/// Set depth/sharpness of the part line.
pub fn hair_part_set_depth(morph: &mut HairPartMorph, depth: f32) {
    morph.depth = depth.clamp(0.0, 1.0);
}

/// Set the parting style enum.
pub fn hair_part_set_style(morph: &mut HairPartMorph, style: HairPartStyle) {
    morph.style = style;
}

/// Infer style from offset magnitude.
pub fn hair_part_infer_style(morph: &HairPartMorph) -> HairPartStyle {
    if morph.offset < -0.1 {
        HairPartStyle::Left
    } else if morph.offset > 0.1 {
        HairPartStyle::Right
    } else {
        HairPartStyle::Center
    }
}

/// Serialize to JSON-like string.
pub fn hair_part_morph_to_json(morph: &HairPartMorph) -> String {
    let style_str = match morph.style {
        HairPartStyle::Center => "center",
        HairPartStyle::Left => "left",
        HairPartStyle::Right => "right",
        HairPartStyle::NoParting => "none",
    };
    format!(
        r#"{{"style":"{style_str}","offset":{:.4},"depth":{:.4}}}"#,
        morph.offset, morph.depth
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_hair_part_morph();
        assert_eq!(m.style, HairPartStyle::Center);
        assert_eq!(m.offset, 0.0);
    }

    #[test]
    fn test_offset_clamp_positive() {
        let mut m = new_hair_part_morph();
        hair_part_set_offset(&mut m, 2.0);
        assert_eq!(m.offset, 1.0);
    }

    #[test]
    fn test_offset_clamp_negative() {
        let mut m = new_hair_part_morph();
        hair_part_set_offset(&mut m, -2.0);
        assert_eq!(m.offset, -1.0);
    }

    #[test]
    fn test_depth_set() {
        let mut m = new_hair_part_morph();
        hair_part_set_depth(&mut m, 0.7);
        assert!((m.depth - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_style_set() {
        let mut m = new_hair_part_morph();
        hair_part_set_style(&mut m, HairPartStyle::Right);
        assert_eq!(m.style, HairPartStyle::Right);
    }

    #[test]
    fn test_infer_style_left() {
        let mut m = new_hair_part_morph();
        hair_part_set_offset(&mut m, -0.5);
        assert_eq!(hair_part_infer_style(&m), HairPartStyle::Left);
    }

    #[test]
    fn test_json_contains_style() {
        let m = new_hair_part_morph();
        let s = hair_part_morph_to_json(&m);
        assert!(s.contains("style"));
    }

    #[test]
    fn test_clone() {
        let m = new_hair_part_morph();
        let m2 = m.clone();
        assert_eq!(m2.style, m.style);
    }

    #[test]
    fn test_default_trait() {
        let m: HairPartMorph = Default::default();
        assert!((m.depth - 0.5).abs() < 1e-6);
    }
}
