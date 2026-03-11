// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Blend mode for combining morph weights.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MorphBlendMode {
    Additive,
    Override,
    Multiply,
    Screen,
}

/// Blend two values using additive mode.
#[allow(dead_code)]
pub fn blend_additive(base: f32, layer: f32) -> f32 {
    (base + layer).clamp(0.0, 1.0)
}

/// Blend two values using override mode (layer replaces base).
#[allow(dead_code)]
pub fn blend_override(_base: f32, layer: f32) -> f32 {
    layer.clamp(0.0, 1.0)
}

/// Blend two values using multiply mode.
#[allow(dead_code)]
pub fn blend_multiply(base: f32, layer: f32) -> f32 {
    (base * layer).clamp(0.0, 1.0)
}

/// Blend two values using screen mode.
#[allow(dead_code)]
pub fn blend_screen(base: f32, layer: f32) -> f32 {
    (1.0 - (1.0 - base) * (1.0 - layer)).clamp(0.0, 1.0)
}

/// Return the name of a blend mode.
#[allow(dead_code)]
pub fn blend_mode_name(mode: MorphBlendMode) -> &'static str {
    match mode {
        MorphBlendMode::Additive => "additive",
        MorphBlendMode::Override => "override",
        MorphBlendMode::Multiply => "multiply",
        MorphBlendMode::Screen => "screen",
    }
}

/// Blend two values using the specified mode.
#[allow(dead_code)]
pub fn blend_two_values(mode: MorphBlendMode, base: f32, layer: f32) -> f32 {
    match mode {
        MorphBlendMode::Additive => blend_additive(base, layer),
        MorphBlendMode::Override => blend_override(base, layer),
        MorphBlendMode::Multiply => blend_multiply(base, layer),
        MorphBlendMode::Screen => blend_screen(base, layer),
    }
}

/// Serialize a blend mode to a JSON string.
#[allow(dead_code)]
pub fn blend_mode_to_json(mode: MorphBlendMode) -> String {
    format!("{{\"mode\":\"{}\"}}", blend_mode_name(mode))
}

/// Return the default blend mode (Additive).
#[allow(dead_code)]
pub fn default_blend_mode() -> MorphBlendMode {
    MorphBlendMode::Additive
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn additive_basic() {
        assert!((blend_additive(0.3, 0.4) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn additive_clamp() {
        assert!((blend_additive(0.8, 0.5) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn override_replaces() {
        assert!((blend_override(0.3, 0.9) - 0.9).abs() < 1e-6);
    }

    #[test]
    fn multiply_basic() {
        assert!((blend_multiply(0.5, 0.5) - 0.25).abs() < 1e-6);
    }

    #[test]
    fn screen_basic() {
        let r = blend_screen(0.5, 0.5);
        assert!((r - 0.75).abs() < 1e-6);
    }

    #[test]
    fn mode_name() {
        assert_eq!(blend_mode_name(MorphBlendMode::Additive), "additive");
        assert_eq!(blend_mode_name(MorphBlendMode::Override), "override");
    }

    #[test]
    fn blend_two() {
        let r = blend_two_values(MorphBlendMode::Multiply, 0.5, 0.8);
        assert!((r - 0.4).abs() < 1e-6);
    }

    #[test]
    fn to_json() {
        let j = blend_mode_to_json(MorphBlendMode::Screen);
        assert!(j.contains("screen"));
    }

    #[test]
    fn default_is_additive() {
        assert_eq!(default_blend_mode(), MorphBlendMode::Additive);
    }

    #[test]
    fn screen_zeros() {
        assert!(blend_screen(0.0, 0.0).abs() < 1e-6);
    }
}
