// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Vignette lens darkening post-processing effect.

/// Configuration for the vignette effect.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VignetteEffectConfig {
    pub strength: f32,
    pub radius: f32,
    pub softness: f32,
    pub color: [f32; 3],
    pub enabled: bool,
}

/// A pixel buffer used as input/output for the vignette pass.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VignetteBuffer {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<[f32; 4]>,
}

/// Result of applying the vignette effect.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VignetteResult {
    pub output: VignetteBuffer,
    pub avg_darkening: f32,
}

/// Returns a default vignette configuration.
#[allow(dead_code)]
pub fn default_vignette_config() -> VignetteEffectConfig {
    VignetteEffectConfig {
        strength: 0.5,
        radius: 0.75,
        softness: 0.45,
        color: [0.0, 0.0, 0.0],
        enabled: true,
    }
}

/// Creates a new pixel buffer with all pixels initialised to opaque white.
#[allow(dead_code)]
pub fn new_vignette_buffer(w: u32, h: u32) -> VignetteBuffer {
    let n = (w as usize) * (h as usize);
    VignetteBuffer {
        width: w,
        height: h,
        pixels: vec![[1.0, 1.0, 1.0, 1.0]; n],
    }
}

/// Applies the vignette effect to `input`, returning a new `VignetteResult`.
/// When `cfg.enabled` is false the input is passed through unchanged.
#[allow(dead_code)]
pub fn apply_vignette(input: &VignetteBuffer, cfg: &VignetteEffectConfig) -> VignetteResult {
    let mut output = input.clone();
    let w = input.width as f32;
    let h = input.height as f32;
    let mut total_darkening = 0.0_f32;

    if cfg.enabled {
        for y in 0..input.height {
            for x in 0..input.width {
                let uv = [
                    (x as f32 + 0.5) / w,
                    (y as f32 + 0.5) / h,
                ];
                let factor = vignette_factor_at(uv, cfg);
                let idx = (y as usize) * (input.width as usize) + (x as usize);
                let p = &mut output.pixels[idx];
                p[0] = (p[0] * factor).clamp(0.0, 1.0);
                p[1] = (p[1] * factor).clamp(0.0, 1.0);
                p[2] = (p[2] * factor).clamp(0.0, 1.0);
                total_darkening += 1.0 - factor;
            }
        }
    }

    let pixel_count = (input.width * input.height) as f32;
    let avg = if pixel_count > 0.0 { total_darkening / pixel_count } else { 0.0 };
    VignetteResult { output, avg_darkening: avg }
}

/// Computes the vignette attenuation factor [0..1] at a UV coordinate.
/// UV is expected in [0,1]×[0,1] with (0.5, 0.5) being the centre.
#[allow(dead_code)]
pub fn vignette_factor_at(uv: [f32; 2], cfg: &VignetteEffectConfig) -> f32 {
    if !cfg.enabled {
        return 1.0;
    }
    let dist = uv_distance_from_center(uv);
    let edge = (dist - cfg.radius) / cfg.softness.max(1e-6);
    let fade = smoothstep(edge);
    1.0 - fade * cfg.strength
}

/// Returns the RGBA pixel at position (x, y) in the buffer.
/// Returns transparent black if the coordinates are out of bounds.
#[allow(dead_code)]
pub fn vignette_pixel_at(buf: &VignetteBuffer, x: u32, y: u32) -> [f32; 4] {
    if x >= buf.width || y >= buf.height {
        return [0.0; 4];
    }
    buf.pixels[(y as usize) * (buf.width as usize) + (x as usize)]
}

/// Writes an RGBA pixel to position (x, y) in the buffer.
/// Does nothing if the coordinates are out of bounds.
#[allow(dead_code)]
pub fn write_vignette_pixel(buf: &mut VignetteBuffer, x: u32, y: u32, p: [f32; 4]) {
    if x >= buf.width || y >= buf.height {
        return;
    }
    buf.pixels[(y as usize) * (buf.width as usize) + (x as usize)] = p;
}

/// Returns the total number of pixels in a buffer.
#[allow(dead_code)]
pub fn vignette_pixel_count(buf: &VignetteBuffer) -> usize {
    buf.pixels.len()
}

/// Serialises a vignette configuration to a JSON string.
#[allow(dead_code)]
pub fn vignette_config_to_json(cfg: &VignetteEffectConfig) -> String {
    let [cr, cg, cb] = cfg.color;
    format!(
        r#"{{"strength":{s:.4},"radius":{r:.4},"softness":{sf:.4},"color":[{cr:.4},{cg:.4},{cb:.4}],"enabled":{e}}}"#,
        s = cfg.strength,
        r = cfg.radius,
        sf = cfg.softness,
        e = cfg.enabled,
    )
}

/// Returns the Euclidean distance of a UV coordinate from the screen centre (0.5, 0.5).
#[allow(dead_code)]
pub fn uv_distance_from_center(uv: [f32; 2]) -> f32 {
    let dx = uv[0] - 0.5;
    let dy = uv[1] - 0.5;
    (dx * dx + dy * dy).sqrt()
}

/// Returns the average darkening value from a `VignetteResult`.
#[allow(dead_code)]
pub fn vignette_avg_darkening(result: &VignetteResult) -> f32 {
    result.avg_darkening
}

// Internal smooth-step helper (Hermite, clamped).
fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_enabled() {
        let cfg = default_vignette_config();
        assert!(cfg.enabled);
        assert!((cfg.strength - 0.5).abs() < 1e-6);
    }

    #[test]
    fn new_buffer_correct_size() {
        let buf = new_vignette_buffer(4, 4);
        assert_eq!(vignette_pixel_count(&buf), 16);
    }

    #[test]
    fn pixel_at_bounds_check() {
        let buf = new_vignette_buffer(2, 2);
        let p = vignette_pixel_at(&buf, 5, 5);
        assert_eq!(p, [0.0; 4]);
    }

    #[test]
    fn write_and_read_pixel() {
        let mut buf = new_vignette_buffer(4, 4);
        write_vignette_pixel(&mut buf, 1, 2, [0.1, 0.2, 0.3, 1.0]);
        let p = vignette_pixel_at(&buf, 1, 2);
        assert!((p[0] - 0.1).abs() < 1e-6);
        assert!((p[1] - 0.2).abs() < 1e-6);
    }

    #[test]
    fn center_uv_distance_is_zero() {
        let d = uv_distance_from_center([0.5, 0.5]);
        assert!(d.abs() < 1e-6);
    }

    #[test]
    fn corner_uv_distance_nonzero() {
        let d = uv_distance_from_center([0.0, 0.0]);
        assert!(d > 0.5);
    }

    #[test]
    fn vignette_factor_center_near_one() {
        let cfg = default_vignette_config();
        let f = vignette_factor_at([0.5, 0.5], &cfg);
        assert!(f > 0.9, "center should be nearly unaffected");
    }

    #[test]
    fn apply_vignette_disabled_passthrough() {
        let mut cfg = default_vignette_config();
        cfg.enabled = false;
        let input = new_vignette_buffer(4, 4);
        let result = apply_vignette(&input, &cfg);
        assert!((result.avg_darkening).abs() < 1e-6);
    }

    #[test]
    fn apply_vignette_enabled_darkens() {
        // Use a tight radius so corner pixels are darkened.
        let mut cfg = default_vignette_config();
        cfg.radius = 0.2;
        cfg.softness = 0.1;
        let input = new_vignette_buffer(16, 16);
        let result = apply_vignette(&input, &cfg);
        assert!(vignette_avg_darkening(&result) > 0.0);
    }

    #[test]
    fn config_to_json_contains_strength() {
        let cfg = default_vignette_config();
        let json = vignette_config_to_json(&cfg);
        assert!(json.contains("\"strength\""));
        assert!(json.contains("\"enabled\":true"));
    }
}
