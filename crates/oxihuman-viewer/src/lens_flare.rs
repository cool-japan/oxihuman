// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Procedural lens flare and starburst generation.

/// The visual element type of a flare sprite.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum FlareElement {
    Halo,
    Ring,
    Streak,
    Ghost,
}

/// Configuration for procedural lens-flare generation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LensFlareConfig {
    pub intensity: f32,
    pub threshold: f32,
    pub element_count: usize,
    pub enabled: bool,
}

/// A single rendered flare sprite.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FlareSprite {
    pub element: FlareElement,
    pub position: [f32; 2],
    pub size: f32,
    pub brightness: f32,
    pub color: [f32; 4],
}

/// Result of generating a lens flare.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LensFlareResult {
    pub sprites: Vec<FlareSprite>,
    pub total_brightness: f32,
}

/// Returns a default lens flare configuration.
#[allow(dead_code)]
pub fn default_lens_flare_config() -> LensFlareConfig {
    LensFlareConfig {
        intensity: 1.0,
        threshold: 0.8,
        element_count: 4,
        enabled: true,
    }
}

/// Generates a procedural lens flare given a light position and screen centre.
/// Produces up to `cfg.element_count` sprites along the axis from the light
/// to the screen centre.
#[allow(dead_code)]
pub fn generate_lens_flare(
    light_pos: [f32; 2],
    screen_center: [f32; 2],
    cfg: &LensFlareConfig,
) -> LensFlareResult {
    if !cfg.enabled {
        return LensFlareResult { sprites: Vec::new(), total_brightness: 0.0 };
    }

    let elements = [FlareElement::Halo, FlareElement::Ring, FlareElement::Streak, FlareElement::Ghost];
    let count = cfg.element_count.min(elements.len());
    let mut sprites = Vec::with_capacity(count);
    let mut total_brightness = 0.0_f32;

    for (i, elem) in elements.iter().enumerate().take(count) {
        let t = if count > 1 { i as f32 / (count - 1) as f32 } else { 0.5 };
        let pos = flare_along_axis(light_pos, screen_center, t);
        let brightness = cfg.intensity * (1.0 - 0.2 * i as f32).max(0.1);
        total_brightness += brightness;
        sprites.push(FlareSprite {
            element: elem.clone(),
            position: pos,
            size: 0.1 + 0.05 * i as f32,
            brightness,
            color: [1.0, 0.95, 0.8, 1.0],
        });
    }

    LensFlareResult { sprites, total_brightness }
}

/// Returns the display name of a `FlareElement` variant.
#[allow(dead_code)]
pub fn flare_element_name(e: &FlareElement) -> &'static str {
    match e {
        FlareElement::Halo => "halo",
        FlareElement::Ring => "ring",
        FlareElement::Streak => "streak",
        FlareElement::Ghost => "ghost",
    }
}

/// Returns the number of sprites in a `LensFlareResult`.
#[allow(dead_code)]
pub fn flare_sprite_count(result: &LensFlareResult) -> usize {
    result.sprites.len()
}

/// Scales the brightness of every sprite in a result by `scale`.
#[allow(dead_code)]
pub fn scale_flare_intensity(result: &mut LensFlareResult, scale: f32) {
    for s in &mut result.sprites {
        s.brightness *= scale;
    }
    result.total_brightness *= scale;
}

/// Linearly interpolates a point along the axis from `src` to `dst` at parameter `t`.
#[allow(dead_code)]
pub fn flare_along_axis(src: [f32; 2], dst: [f32; 2], t: f32) -> [f32; 2] {
    [src[0] + (dst[0] - src[0]) * t, src[1] + (dst[1] - src[1]) * t]
}

/// Serialises a `LensFlareConfig` to a JSON string.
#[allow(dead_code)]
pub fn lens_flare_config_to_json(cfg: &LensFlareConfig) -> String {
    format!(
        r#"{{"intensity":{i:.4},"threshold":{t:.4},"element_count":{e},"enabled":{en}}}"#,
        i = cfg.intensity,
        t = cfg.threshold,
        e = cfg.element_count,
        en = cfg.enabled,
    )
}

/// Serialises a `LensFlareResult` to a JSON string.
#[allow(dead_code)]
pub fn lens_flare_result_to_json(r: &LensFlareResult) -> String {
    format!(
        r#"{{"sprite_count":{s},"total_brightness":{b:.4}}}"#,
        s = r.sprites.len(),
        b = r.total_brightness,
    )
}

/// Returns `true` when the total brightness exceeds `threshold`.
#[allow(dead_code)]
pub fn is_lens_flare_visible(result: &LensFlareResult, threshold: f32) -> bool {
    result.total_brightness > threshold
}

/// Creates a new `FlareSprite` with default white colour and zero brightness.
#[allow(dead_code)]
pub fn new_flare_sprite(element: FlareElement, pos: [f32; 2], size: f32) -> FlareSprite {
    FlareSprite {
        element,
        position: pos,
        size,
        brightness: 0.0,
        color: [1.0, 1.0, 1.0, 1.0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_lens_flare_config();
        assert!((cfg.intensity - 1.0).abs() < 1e-6);
        assert_eq!(cfg.element_count, 4);
        assert!(cfg.enabled);
    }

    #[test]
    fn generate_disabled_returns_empty() {
        let mut cfg = default_lens_flare_config();
        cfg.enabled = false;
        let result = generate_lens_flare([0.8, 0.3], [0.5, 0.5], &cfg);
        assert_eq!(flare_sprite_count(&result), 0);
    }

    #[test]
    fn generate_produces_correct_count() {
        let cfg = default_lens_flare_config();
        let result = generate_lens_flare([0.8, 0.3], [0.5, 0.5], &cfg);
        assert_eq!(flare_sprite_count(&result), 4);
    }

    #[test]
    fn flare_element_names() {
        assert_eq!(flare_element_name(&FlareElement::Halo), "halo");
        assert_eq!(flare_element_name(&FlareElement::Ring), "ring");
        assert_eq!(flare_element_name(&FlareElement::Streak), "streak");
        assert_eq!(flare_element_name(&FlareElement::Ghost), "ghost");
    }

    #[test]
    fn scale_flare_intensity_halves_brightness() {
        let cfg = default_lens_flare_config();
        let mut result = generate_lens_flare([0.8, 0.3], [0.5, 0.5], &cfg);
        let before = result.total_brightness;
        scale_flare_intensity(&mut result, 0.5);
        assert!((result.total_brightness - before * 0.5).abs() < 1e-5);
    }

    #[test]
    fn flare_along_axis_midpoint() {
        let mid = flare_along_axis([0.0, 0.0], [1.0, 1.0], 0.5);
        assert!((mid[0] - 0.5).abs() < 1e-6);
        assert!((mid[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn is_lens_flare_visible_above_threshold() {
        let cfg = default_lens_flare_config();
        let result = generate_lens_flare([0.8, 0.3], [0.5, 0.5], &cfg);
        assert!(is_lens_flare_visible(&result, 0.0));
    }

    #[test]
    fn config_to_json_contains_intensity() {
        let cfg = default_lens_flare_config();
        let json = lens_flare_config_to_json(&cfg);
        assert!(json.contains("\"intensity\""));
        assert!(json.contains("\"enabled\":true"));
    }

    #[test]
    fn new_flare_sprite_zero_brightness() {
        let s = new_flare_sprite(FlareElement::Halo, [0.5, 0.5], 0.2);
        assert!((s.brightness).abs() < 1e-6);
        assert_eq!(s.element, FlareElement::Halo);
    }
}
