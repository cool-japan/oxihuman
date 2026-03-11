// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Color grading LUT support for OxiHuman viewer.
//!
//! Provides identity and contrast LUT construction, trilinear LUT lookup, and
//! a full grading pipeline (exposure, contrast, saturation, hue shift).

// ── Types ─────────────────────────────────────────────────────────────────────

/// Configuration for a colour LUT.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct LutConfig {
    /// Number of entries per axis (e.g. 16 → 16×16×16 = 4096 entries).
    pub size: u32,
    /// Blend strength from original to graded colour (0.0 = no effect, 1.0 = full).
    pub strength: f32,
    /// Gamma applied before LUT lookup.
    pub gamma: f32,
}

/// A 3D colour LUT with `size^3` entries.
///
/// Entry order: R-major → G → B  (i.e. index = `r + g*size + b*size*size`).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ColorLut {
    /// Flat storage of `[R, G, B]` output values in `[0, 1]`.
    pub data: Vec<[f32; 3]>,
    /// Number of entries per axis.
    pub size: u32,
}

/// Parameters for the per-pixel colour grading operation.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct GradingParams {
    /// Exposure adjustment in EV stops.
    pub exposure: f32,
    /// Contrast multiplier (1.0 = no change, >1.0 increases contrast).
    pub contrast: f32,
    /// Saturation multiplier (1.0 = no change, 0.0 = greyscale).
    pub saturation: f32,
    /// Hue rotation in degrees.
    pub hue_shift: f32,
}

/// Result produced by [`apply_grading`].
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GradingResult {
    /// Graded pixels as `[R, G, B, A]` in `[0, 1]`.
    pub output_pixels: Vec<[f32; 4]>,
    /// Parameters used for this grading pass.
    pub params: GradingParams,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return a sensible default [`LutConfig`].
#[allow(dead_code)]
pub fn default_lut_config() -> LutConfig {
    LutConfig {
        size: 16,
        strength: 1.0,
        gamma: 2.2,
    }
}

/// Return a default [`GradingParams`] (identity — no colour change).
#[allow(dead_code)]
pub fn default_grading_params() -> GradingParams {
    GradingParams {
        exposure: 0.0,
        contrast: 1.0,
        saturation: 1.0,
        hue_shift: 0.0,
    }
}

/// Build an identity [`ColorLut`] of the given `size`.
///
/// Each entry maps the input colour to itself.
#[allow(dead_code)]
pub fn new_identity_lut(size: u32) -> ColorLut {
    let n = size as usize;
    let total = n * n * n;
    let mut data = Vec::with_capacity(total);
    let scale = if size > 1 { 1.0 / (size - 1) as f32 } else { 0.0 };
    for b in 0..n {
        for g in 0..n {
            for r in 0..n {
                data.push([r as f32 * scale, g as f32 * scale, b as f32 * scale]);
            }
        }
    }
    ColorLut { data, size }
}

/// Look up `color` in `lut` using trilinear interpolation.
///
/// `color` components should be in `[0, 1]`.
#[allow(dead_code)]
pub fn apply_lut(lut: &ColorLut, color: [f32; 3]) -> [f32; 3] {
    if lut.data.is_empty() || lut.size == 0 {
        return color;
    }
    let n = lut.size as usize;
    let max_idx = (lut.size - 1) as f32;

    // Scale to LUT space
    let r_f = (color[0].clamp(0.0, 1.0) * max_idx).clamp(0.0, max_idx);
    let g_f = (color[1].clamp(0.0, 1.0) * max_idx).clamp(0.0, max_idx);
    let b_f = (color[2].clamp(0.0, 1.0) * max_idx).clamp(0.0, max_idx);

    let r0 = r_f.floor() as usize;
    let g0 = g_f.floor() as usize;
    let b0 = b_f.floor() as usize;
    let r1 = (r0 + 1).min(n - 1);
    let g1 = (g0 + 1).min(n - 1);
    let b1 = (b0 + 1).min(n - 1);

    let tr = r_f - r0 as f32;
    let tg = g_f - g0 as f32;
    let tb = b_f - b0 as f32;

    // Trilinear interpolation
    let c000 = lut_entry(lut, r0, g0, b0, n);
    let c100 = lut_entry(lut, r1, g0, b0, n);
    let c010 = lut_entry(lut, r0, g1, b0, n);
    let c110 = lut_entry(lut, r1, g1, b0, n);
    let c001 = lut_entry(lut, r0, g0, b1, n);
    let c101 = lut_entry(lut, r1, g0, b1, n);
    let c011 = lut_entry(lut, r0, g1, b1, n);
    let c111 = lut_entry(lut, r1, g1, b1, n);

    let mut out = [0.0_f32; 3];
    for i in 0..3 {
        let c00 = lerp(c000[i], c100[i], tr);
        let c10 = lerp(c010[i], c110[i], tr);
        let c01 = lerp(c001[i], c101[i], tr);
        let c11 = lerp(c011[i], c111[i], tr);
        let c0 = lerp(c00, c10, tg);
        let c1 = lerp(c01, c11, tg);
        out[i] = lerp(c0, c1, tb).clamp(0.0, 1.0);
    }
    out
}

/// Build a contrast LUT of the given `size`.
///
/// `contrast == 1.0` produces an identity LUT; `> 1.0` increases contrast.
#[allow(dead_code)]
pub fn build_contrast_lut(size: u32, contrast: f32) -> ColorLut {
    let n = size as usize;
    let total = n * n * n;
    let mut data = Vec::with_capacity(total);
    let scale = if size > 1 { 1.0 / (size - 1) as f32 } else { 0.0 };
    for b in 0..n {
        for g in 0..n {
            for r in 0..n {
                let rv = apply_contrast(r as f32 * scale, contrast);
                let gv = apply_contrast(g as f32 * scale, contrast);
                let bv = apply_contrast(b as f32 * scale, contrast);
                data.push([rv, gv, bv]);
            }
        }
    }
    ColorLut { data, size }
}

/// Apply `params` to each pixel in `pixels` and return a [`GradingResult`].
///
/// `pixels` is a slice of `[R, G, B, A]` values in `[0, 1]`.
#[allow(dead_code)]
pub fn apply_grading(pixels: &[[f32; 4]], params: &GradingParams) -> GradingResult {
    let exposure_scale = 2.0_f32.powf(params.exposure);

    let output_pixels: Vec<[f32; 4]> = pixels
        .iter()
        .map(|&[r, g, b, a]| {
            // Exposure
            let r = (r * exposure_scale).clamp(0.0, 1.0);
            let g = (g * exposure_scale).clamp(0.0, 1.0);
            let b = (b * exposure_scale).clamp(0.0, 1.0);

            // Contrast (pivot at 0.5)
            let r = apply_contrast(r, params.contrast);
            let g = apply_contrast(g, params.contrast);
            let b = apply_contrast(b, params.contrast);

            // Saturation
            let lum = 0.299 * r + 0.587 * g + 0.114 * b;
            let r = lerp(lum, r, params.saturation).clamp(0.0, 1.0);
            let g = lerp(lum, g, params.saturation).clamp(0.0, 1.0);
            let b = lerp(lum, b, params.saturation).clamp(0.0, 1.0);

            // Hue shift (via RGB→HSV→RGB)
            let [r, g, b] = hue_rotate([r, g, b], params.hue_shift);

            [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), a]
        })
        .collect();

    GradingResult {
        output_pixels,
        params: params.clone(),
    }
}

/// Serialise a [`ColorLut`] summary to a compact JSON string.
#[allow(dead_code)]
pub fn lut_to_json(lut: &ColorLut) -> String {
    format!(
        r#"{{"size":{},"entry_count":{}}}"#,
        lut.size,
        lut_entry_count(lut),
    )
}

/// Serialise a [`GradingResult`] summary to a compact JSON string.
#[allow(dead_code)]
pub fn grading_result_to_json(r: &GradingResult) -> String {
    format!(
        r#"{{"pixel_count":{},"exposure":{},"contrast":{},"saturation":{},"hue_shift":{}}}"#,
        r.output_pixels.len(),
        r.params.exposure,
        r.params.contrast,
        r.params.saturation,
        r.params.hue_shift,
    )
}

/// Return the total number of entries in `lut` (`size^3`).
#[allow(dead_code)]
pub fn lut_entry_count(lut: &ColorLut) -> usize {
    lut.data.len()
}

// ── Private helpers ───────────────────────────────────────────────────────────

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline]
fn apply_contrast(v: f32, contrast: f32) -> f32 {
    ((v - 0.5) * contrast + 0.5).clamp(0.0, 1.0)
}

#[inline]
fn lut_entry(lut: &ColorLut, r: usize, g: usize, b: usize, n: usize) -> [f32; 3] {
    let idx = r + g * n + b * n * n;
    lut.data.get(idx).copied().unwrap_or([0.0; 3])
}

/// Rotate hue of an RGB colour by `degrees`.
fn hue_rotate(rgb: [f32; 3], degrees: f32) -> [f32; 3] {
    if degrees == 0.0 {
        return rgb;
    }
    let [r, g, b] = rgb;
    let (h, s, v) = rgb_to_hsv(r, g, b);
    let h2 = (h + degrees / 360.0).rem_euclid(1.0);
    hsv_to_rgb(h2, s, v)
}

fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let cmax = r.max(g).max(b);
    let cmin = r.min(g).min(b);
    let delta = cmax - cmin;

    let h = if delta < 1e-6 {
        0.0
    } else if (cmax - r).abs() < 1e-6 {
        ((g - b) / delta).rem_euclid(6.0) / 6.0
    } else if (cmax - g).abs() < 1e-6 {
        ((b - r) / delta + 2.0) / 6.0
    } else {
        ((r - g) / delta + 4.0) / 6.0
    };

    let s = if cmax < 1e-6 { 0.0 } else { delta / cmax };
    (h, s, cmax)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    if s < 1e-6 {
        return [v, v, v];
    }
    let h6 = h * 6.0;
    let i = h6.floor() as u32 % 6;
    let f = h6 - h6.floor();
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    match i {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_lut_config_fields() {
        let cfg = default_lut_config();
        assert_eq!(cfg.size, 16);
        assert!((cfg.strength - 1.0).abs() < f32::EPSILON);
        assert!((cfg.gamma - 2.2).abs() < 1e-4);
    }

    #[test]
    fn default_grading_params_identity() {
        let p = default_grading_params();
        assert_eq!(p.exposure, 0.0);
        assert_eq!(p.contrast, 1.0);
        assert_eq!(p.saturation, 1.0);
        assert_eq!(p.hue_shift, 0.0);
    }

    #[test]
    fn identity_lut_size() {
        let lut = new_identity_lut(8);
        assert_eq!(lut_entry_count(&lut), 8 * 8 * 8);
        assert_eq!(lut.size, 8);
    }

    #[test]
    fn identity_lut_maps_to_self() {
        let lut = new_identity_lut(16);
        let color = [0.25, 0.5, 0.75];
        let out = apply_lut(&lut, color);
        assert!((out[0] - color[0]).abs() < 0.02, "R: {} vs {}", out[0], color[0]);
        assert!((out[1] - color[1]).abs() < 0.02, "G: {} vs {}", out[1], color[1]);
        assert!((out[2] - color[2]).abs() < 0.02, "B: {} vs {}", out[2], color[2]);
    }

    #[test]
    fn apply_lut_clamps_out_of_range() {
        let lut = new_identity_lut(8);
        let out = apply_lut(&lut, [2.0, -1.0, 0.5]);
        assert!(out[0] <= 1.0);
        assert!(out[1] >= 0.0);
    }

    #[test]
    fn contrast_lut_size() {
        let lut = build_contrast_lut(8, 1.5);
        assert_eq!(lut_entry_count(&lut), 8 * 8 * 8);
    }

    #[test]
    fn contrast_lut_identity_at_one() {
        let lut = build_contrast_lut(16, 1.0);
        let id = new_identity_lut(16);
        // Compare a few entries
        for i in [0, 100, 500, 4000] {
            for ch in 0..3 {
                assert!((lut.data[i][ch] - id.data[i][ch]).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn grading_identity_params_no_change() {
        let params = default_grading_params();
        let pixels: Vec<[f32; 4]> = vec![[0.5, 0.3, 0.8, 1.0]];
        let result = apply_grading(&pixels, &params);
        assert_eq!(result.output_pixels.len(), 1);
        let out = result.output_pixels[0];
        // With identity params (no hue shift, contrast=1, sat=1, exposure=0)
        assert!((out[0] - 0.5).abs() < 1e-4, "R: {}", out[0]);
        assert!((out[1] - 0.3).abs() < 1e-4, "G: {}", out[1]);
        assert!((out[2] - 0.8).abs() < 1e-4, "B: {}", out[2]);
        assert!((out[3] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn grading_exposure_brightens() {
        let params = GradingParams {
            exposure: 1.0, // +1 EV = ×2
            contrast: 1.0,
            saturation: 1.0,
            hue_shift: 0.0,
        };
        let pixels: Vec<[f32; 4]> = vec![[0.25, 0.25, 0.25, 1.0]];
        let result = apply_grading(&pixels, &params);
        let out = result.output_pixels[0];
        // 0.25 * 2 = 0.5, then contrast pivot gives 0.5
        assert!((out[0] - 0.5).abs() < 1e-4);
    }

    #[test]
    fn grading_saturation_zero_produces_gray() {
        let params = GradingParams {
            exposure: 0.0,
            contrast: 1.0,
            saturation: 0.0,
            hue_shift: 0.0,
        };
        let pixels: Vec<[f32; 4]> = vec![[1.0, 0.0, 0.0, 1.0]];
        let result = apply_grading(&pixels, &params);
        let out = result.output_pixels[0];
        // All channels should equal luminance of red (≈ 0.299)
        let lum = out[0];
        assert!((out[1] - lum).abs() < 1e-4);
        assert!((out[2] - lum).abs() < 1e-4);
    }

    #[test]
    fn lut_to_json_has_size() {
        let lut = new_identity_lut(4);
        let json = lut_to_json(&lut);
        assert!(json.contains("size"));
        assert!(json.contains("entry_count"));
        assert!(json.contains("64")); // 4^3
    }

    #[test]
    fn grading_result_to_json_has_pixel_count() {
        let params = default_grading_params();
        let pixels: Vec<[f32; 4]> = vec![[0.1, 0.2, 0.3, 1.0]; 10];
        let r = apply_grading(&pixels, &params);
        let json = grading_result_to_json(&r);
        assert!(json.contains("pixel_count"));
        assert!(json.contains("10"));
    }

    #[test]
    fn lut_entry_count_matches_size_cubed() {
        let lut = new_identity_lut(6);
        assert_eq!(lut_entry_count(&lut), 216);
    }

    #[test]
    fn apply_lut_empty_returns_input() {
        let lut = ColorLut { data: vec![], size: 0 };
        let color = [0.3, 0.5, 0.7];
        assert_eq!(apply_lut(&lut, color), color);
    }
}
