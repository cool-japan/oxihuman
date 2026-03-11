// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Lens flare, vignette, and chromatic aberration overlay effects.
//!
//! Pure-data module; no GPU calls.  UV coordinates are normalised `[0, 1]`
//! with `(0.5, 0.5)` at screen centre.

// ── Types ──────────────────────────────────────────────────────────────────

/// Combined lens-effect configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LensEffectConfig {
    /// Vignette configuration.
    pub vignette: VignetteConfig,
    /// List of active lens flares.
    pub flares: Vec<LensFlare>,
    /// Chromatic aberration magnitude (0 = disabled).
    pub chromatic_aberration_strength: f32,
    /// Minimum luminance to trigger a flare (0–1 normalised).
    pub flare_threshold: f32,
    /// Whether any lens overlay effects are active.
    pub enabled: bool,
}

/// A single lens flare element.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LensFlare {
    /// Unique identifier.
    pub id: u32,
    /// Position in screen UV space `[u, v]`.
    pub position: [f32; 2],
    /// Peak intensity multiplier.
    pub intensity: f32,
    /// Flare tint colour `[r, g, b, a]`.
    pub color: [f32; 4],
    /// Number of radial streak arms.
    pub streak_count: u32,
    /// Length of each streak arm (UV space).
    pub streak_length: f32,
    /// Whether this flare is visible.
    pub visible: bool,
}

/// Screen-edge darkening (vignette) configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VignetteConfig {
    /// Darkening strength (0 = none, 1 = full black at edges).
    pub strength: f32,
    /// Radial softness of the vignette fall-off.
    pub softness: f32,
    /// Inner radius at which darkening begins (UV distance from centre).
    pub inner_radius: f32,
    /// Outer radius at which darkening reaches maximum.
    pub outer_radius: f32,
}

// ── Type aliases ───────────────────────────────────────────────────────────

/// Screen-space RGB channel offsets `(red_uv, green_uv, blue_uv)`.
pub type ChromaticOffsets = ([f32; 2], [f32; 2], [f32; 2]);

/// A collection of streak endpoint positions in UV space.
pub type StreakPositions = Vec<([f32; 2], [f32; 2])>;

// ── Default constructors ───────────────────────────────────────────────────

/// Return a sensible default [`LensEffectConfig`].
#[allow(dead_code)]
pub fn default_lens_effect_config() -> LensEffectConfig {
    LensEffectConfig {
        vignette: VignetteConfig {
            strength: 0.4,
            softness: 0.6,
            inner_radius: 0.4,
            outer_radius: 0.9,
        },
        flares: Vec::new(),
        chromatic_aberration_strength: 0.002,
        flare_threshold: 0.8,
        enabled: true,
    }
}

/// Construct a new [`LensFlare`] with explicit parameters.
#[allow(dead_code)]
pub fn new_lens_flare(
    id: u32,
    position: [f32; 2],
    intensity: f32,
    color: [f32; 4],
    streak_count: u32,
    streak_length: f32,
) -> LensFlare {
    LensFlare {
        id,
        position,
        intensity: intensity.max(0.0),
        color,
        streak_count,
        streak_length: streak_length.max(0.0),
        visible: true,
    }
}

// ── Vignette ───────────────────────────────────────────────────────────────

/// Compute the vignette darkening alpha at the given UV coordinate.
///
/// Returns a value in `[0, 1]` where `0` means no darkening and `1` means
/// fully black.
#[allow(dead_code)]
pub fn vignette_alpha(cfg: &VignetteConfig, uv: [f32; 2]) -> f32 {
    let dx = uv[0] - 0.5;
    let dy = uv[1] - 0.5;
    let dist = (dx * dx + dy * dy).sqrt();
    let inner = cfg.inner_radius.max(0.0);
    let outer = cfg.outer_radius.max(inner + f32::EPSILON);
    if dist <= inner {
        return 0.0;
    }
    if dist >= outer {
        return cfg.strength.clamp(0.0, 1.0);
    }
    let t = (dist - inner) / (outer - inner);
    // Smooth-step fall-off
    let smooth = t * t * (3.0 - 2.0 * t);
    (smooth * cfg.strength).clamp(0.0, 1.0)
}

/// Compute the darkened pixel value `pixel_value * (1 - vignette_alpha)`.
#[allow(dead_code)]
pub fn vignette_mask_pixel(cfg: &VignetteConfig, uv: [f32; 2], pixel: [f32; 4]) -> [f32; 4] {
    let alpha = vignette_alpha(cfg, uv);
    let scale = 1.0 - alpha;
    [
        pixel[0] * scale,
        pixel[1] * scale,
        pixel[2] * scale,
        pixel[3],
    ]
}

/// Update the vignette strength in a [`LensEffectConfig`].
#[allow(dead_code)]
pub fn set_vignette_strength(cfg: &mut LensEffectConfig, strength: f32) {
    cfg.vignette.strength = strength.clamp(0.0, 1.0);
}

// ── Lens flare ─────────────────────────────────────────────────────────────

/// Compute the effective intensity of a flare given the angle between the
/// light direction and the camera forward vector (radians).
#[allow(dead_code)]
pub fn lens_flare_intensity(flare: &LensFlare, light_angle_rad: f32) -> f32 {
    // Intensity falls off as cos²(angle) — full at 0, zero at π/2
    let cos_a = light_angle_rad.cos().max(0.0);
    flare.intensity * cos_a * cos_a
}

/// Return `true` if the flare is both marked visible and its intensity
/// exceeds the config threshold.
#[allow(dead_code)]
pub fn is_flare_visible(cfg: &LensEffectConfig, flare: &LensFlare) -> bool {
    flare.visible && flare.intensity >= cfg.flare_threshold
}

/// Compute evenly-spaced radial streak start/end positions for a flare.
///
/// Each tuple `(start_uv, end_uv)` represents one streak arm.
#[allow(dead_code)]
pub fn flare_streak_positions(flare: &LensFlare) -> StreakPositions {
    let n = flare.streak_count.max(1);
    let mut streaks = Vec::with_capacity(n as usize);
    for i in 0..n {
        let angle = (i as f32) * std::f32::consts::TAU / (n as f32);
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let start = flare.position;
        let end = [
            flare.position[0] + cos_a * flare.streak_length,
            flare.position[1] + sin_a * flare.streak_length,
        ];
        streaks.push((start, end));
    }
    streaks
}

/// Return the number of flares in the config.
#[allow(dead_code)]
pub fn lens_flare_count(cfg: &LensEffectConfig) -> usize {
    cfg.flares.len()
}

/// Update the flare visibility threshold.
#[allow(dead_code)]
pub fn set_flare_threshold(cfg: &mut LensEffectConfig, threshold: f32) {
    cfg.flare_threshold = threshold.clamp(0.0, 1.0);
}

// ── Chromatic aberration ───────────────────────────────────────────────────

/// Compute RGB channel UV offsets for chromatic aberration at the given UV.
///
/// Returns `(red_uv, green_uv, blue_uv)` where each channel is shifted
/// radially outward by a fraction of `strength`.
#[allow(dead_code)]
pub fn chromatic_aberration_offset(cfg: &LensEffectConfig, uv: [f32; 2]) -> ChromaticOffsets {
    let dx = uv[0] - 0.5;
    let dy = uv[1] - 0.5;
    let dist = (dx * dx + dy * dy).sqrt().max(f32::EPSILON);
    let dir = [dx / dist, dy / dist];
    let s = cfg.chromatic_aberration_strength;

    let red = [uv[0] + dir[0] * s * 1.0, uv[1] + dir[1] * s * 1.0];
    let green = [uv[0], uv[1]]; // green channel un-shifted
    let blue = [uv[0] - dir[0] * s * 1.0, uv[1] - dir[1] * s * 1.0];
    (red, green, blue)
}

// ── Serialization ──────────────────────────────────────────────────────────

/// Serialize the config to a compact JSON string.
#[allow(dead_code)]
pub fn lens_overlay_to_json(cfg: &LensEffectConfig) -> String {
    let flare_strs: Vec<String> = cfg
        .flares
        .iter()
        .map(|f| {
            format!(
                r#"{{"id":{},"position":[{:.4},{:.4}],"intensity":{:.4},"streak_count":{},"streak_length":{:.4},"visible":{}}}"#,
                f.id, f.position[0], f.position[1], f.intensity, f.streak_count, f.streak_length, f.visible
            )
        })
        .collect();
    format!(
        r#"{{"enabled":{},"chromatic_aberration_strength":{:.6},"flare_threshold":{:.4},"vignette":{{"strength":{:.4},"softness":{:.4},"inner_radius":{:.4},"outer_radius":{:.4}}},"flares":[{}]}}"#,
        cfg.enabled,
        cfg.chromatic_aberration_strength,
        cfg.flare_threshold,
        cfg.vignette.strength,
        cfg.vignette.softness,
        cfg.vignette.inner_radius,
        cfg.vignette.outer_radius,
        flare_strs.join(","),
    )
}

/// Blend two [`LensEffectConfig`]s by averaging their scalar parameters.
/// Flare lists are concatenated.
#[allow(dead_code)]
pub fn blend_lens_configs(a: &LensEffectConfig, b: &LensEffectConfig) -> LensEffectConfig {
    let mut flares = a.flares.clone();
    flares.extend(b.flares.iter().cloned());
    LensEffectConfig {
        vignette: VignetteConfig {
            strength: (a.vignette.strength + b.vignette.strength) * 0.5,
            softness: (a.vignette.softness + b.vignette.softness) * 0.5,
            inner_radius: (a.vignette.inner_radius + b.vignette.inner_radius) * 0.5,
            outer_radius: (a.vignette.outer_radius + b.vignette.outer_radius) * 0.5,
        },
        flares,
        chromatic_aberration_strength: (a.chromatic_aberration_strength
            + b.chromatic_aberration_strength)
            * 0.5,
        flare_threshold: (a.flare_threshold + b.flare_threshold) * 0.5,
        enabled: a.enabled || b.enabled,
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_lens_effect_config() {
        let cfg = default_lens_effect_config();
        assert!(cfg.enabled);
        assert!(cfg.vignette.strength > 0.0);
        assert!(cfg.flares.is_empty());
    }

    #[test]
    fn test_new_lens_flare() {
        let flare = new_lens_flare(1, [0.5, 0.5], 1.0, [1.0, 1.0, 1.0, 1.0], 6, 0.1);
        assert_eq!(flare.id, 1);
        assert_eq!(flare.streak_count, 6);
        assert!(flare.visible);
    }

    #[test]
    fn test_vignette_alpha_center_is_zero() {
        let cfg = VignetteConfig {
            strength: 1.0,
            softness: 0.5,
            inner_radius: 0.4,
            outer_radius: 0.9,
        };
        let alpha = vignette_alpha(&cfg, [0.5, 0.5]);
        assert!(alpha < 1e-5, "centre alpha should be ~0, got {alpha}");
    }

    #[test]
    fn test_vignette_alpha_corner_is_max() {
        let cfg = VignetteConfig {
            strength: 1.0,
            softness: 0.5,
            inner_radius: 0.0,
            outer_radius: 0.1,
        };
        let alpha = vignette_alpha(&cfg, [0.0, 0.0]);
        assert!(
            (alpha - 1.0).abs() < 1e-4,
            "corner alpha should be 1.0, got {alpha}"
        );
    }

    #[test]
    fn test_vignette_alpha_between_zero_and_one() {
        let cfg = VignetteConfig {
            strength: 0.8,
            softness: 0.5,
            inner_radius: 0.2,
            outer_radius: 0.8,
        };
        for uv in [[0.1f32, 0.1], [0.5, 0.8], [0.9, 0.9]] {
            let alpha = vignette_alpha(&cfg, uv);
            assert!((0.0..=1.0).contains(&alpha), "alpha={alpha} out of [0,1]");
        }
    }

    #[test]
    fn test_vignette_mask_pixel_dims() {
        let cfg = VignetteConfig {
            strength: 0.5,
            softness: 0.5,
            inner_radius: 0.3,
            outer_radius: 0.7,
        };
        let out = vignette_mask_pixel(&cfg, [0.5, 0.5], [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(out.len(), 4);
    }

    #[test]
    fn test_vignette_mask_pixel_darkens() {
        let cfg = VignetteConfig {
            strength: 1.0,
            softness: 0.5,
            inner_radius: 0.0,
            outer_radius: 0.1,
        };
        let out = vignette_mask_pixel(&cfg, [0.0, 0.0], [1.0, 1.0, 1.0, 1.0]);
        assert!(out[0] < 1.0, "pixel should be darkened, got {}", out[0]);
    }

    #[test]
    fn test_set_vignette_strength() {
        let mut cfg = default_lens_effect_config();
        set_vignette_strength(&mut cfg, 0.75);
        assert!((cfg.vignette.strength - 0.75).abs() < 1e-5);
    }

    #[test]
    fn test_lens_flare_intensity_zero_angle() {
        let flare = new_lens_flare(0, [0.5, 0.5], 2.0, [1.0, 1.0, 1.0, 1.0], 4, 0.1);
        let intensity = lens_flare_intensity(&flare, 0.0);
        assert!((intensity - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_lens_flare_intensity_ninety_degrees() {
        let flare = new_lens_flare(0, [0.5, 0.5], 2.0, [1.0, 1.0, 1.0, 1.0], 4, 0.1);
        let intensity = lens_flare_intensity(&flare, std::f32::consts::FRAC_PI_2);
        assert!(intensity < 1e-4);
    }

    #[test]
    fn test_is_flare_visible_true() {
        let mut cfg = default_lens_effect_config();
        cfg.flare_threshold = 0.5;
        let flare = new_lens_flare(0, [0.5, 0.5], 1.0, [1.0, 1.0, 1.0, 1.0], 4, 0.1);
        assert!(is_flare_visible(&cfg, &flare));
    }

    #[test]
    fn test_is_flare_visible_below_threshold() {
        let mut cfg = default_lens_effect_config();
        cfg.flare_threshold = 2.0; // very high threshold
        let flare = new_lens_flare(0, [0.5, 0.5], 1.0, [1.0, 1.0, 1.0, 1.0], 4, 0.1);
        assert!(!is_flare_visible(&cfg, &flare));
    }

    #[test]
    fn test_flare_streak_positions_count() {
        let flare = new_lens_flare(0, [0.5, 0.5], 1.0, [1.0, 1.0, 1.0, 1.0], 6, 0.1);
        let streaks = flare_streak_positions(&flare);
        assert_eq!(streaks.len(), 6);
    }

    #[test]
    fn test_flare_streak_start_equals_flare_position() {
        let flare = new_lens_flare(0, [0.3, 0.7], 1.0, [1.0, 1.0, 1.0, 1.0], 4, 0.2);
        let streaks = flare_streak_positions(&flare);
        for (start, _) in &streaks {
            assert!((start[0] - 0.3).abs() < 1e-5);
            assert!((start[1] - 0.7).abs() < 1e-5);
        }
    }

    #[test]
    fn test_chromatic_aberration_offset_center() {
        let cfg = default_lens_effect_config();
        // At centre the direction is degenerate but function must not panic
        let (r, g, b) = chromatic_aberration_offset(&cfg, [0.5, 0.5]);
        assert_eq!(g, [0.5, 0.5]);
        let _ = (r, b);
    }

    #[test]
    fn test_chromatic_aberration_green_unshifted() {
        let cfg = default_lens_effect_config();
        let uv = [0.8, 0.2];
        let (_r, g, _b) = chromatic_aberration_offset(&cfg, uv);
        assert!((g[0] - uv[0]).abs() < 1e-6);
        assert!((g[1] - uv[1]).abs() < 1e-6);
    }

    #[test]
    fn test_set_flare_threshold() {
        let mut cfg = default_lens_effect_config();
        set_flare_threshold(&mut cfg, 0.3);
        assert!((cfg.flare_threshold - 0.3).abs() < 1e-5);
    }

    #[test]
    fn test_lens_overlay_to_json_nonempty() {
        let cfg = default_lens_effect_config();
        let json = lens_overlay_to_json(&cfg);
        assert!(!json.is_empty());
        assert!(json.contains("vignette"));
        assert!(json.contains("enabled"));
    }

    #[test]
    fn test_blend_lens_configs_flare_count() {
        let mut a = default_lens_effect_config();
        a.flares.push(new_lens_flare(
            0,
            [0.5, 0.5],
            1.0,
            [1.0, 1.0, 1.0, 1.0],
            4,
            0.1,
        ));
        let mut b = default_lens_effect_config();
        b.flares.push(new_lens_flare(
            1,
            [0.3, 0.3],
            0.5,
            [1.0, 0.8, 0.0, 1.0],
            6,
            0.2,
        ));
        let blended = blend_lens_configs(&a, &b);
        assert_eq!(lens_flare_count(&blended), 2);
    }

    #[test]
    fn test_lens_flare_count_empty() {
        let cfg = default_lens_effect_config();
        assert_eq!(lens_flare_count(&cfg), 0);
    }
}
