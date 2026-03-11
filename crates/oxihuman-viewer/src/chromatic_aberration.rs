//! Chromatic aberration lens distortion post-processing effect.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Configuration for the chromatic aberration effect.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChromaticConfig {
    /// UV-space offset applied to the red channel.
    pub red_offset: [f32; 2],
    /// UV-space offset applied to the green channel.
    pub green_offset: [f32; 2],
    /// UV-space offset applied to the blue channel.
    pub blue_offset: [f32; 2],
    /// Radial distortion strength (0 = none, higher = more distortion).
    pub radial_strength: f32,
    /// Whether the effect is active.
    pub enabled: bool,
}

/// RGBA pixel buffer used as input/output for the chromatic aberration pass.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChromaticBuffer {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<[f32; 4]>,
}

/// Result of applying chromatic aberration to a buffer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChromaticResult {
    pub output: ChromaticBuffer,
    /// Maximum per-channel shift in pixels observed during the pass.
    pub max_shift_px: f32,
}

// ── Functions ──────────────────────────────────────────────────────────────────

/// Return a default [`ChromaticConfig`] with subtle offsets disabled.
#[allow(dead_code)]
pub fn default_chromatic_config() -> ChromaticConfig {
    ChromaticConfig {
        red_offset: [0.002, 0.0],
        green_offset: [0.0, 0.0],
        blue_offset: [-0.002, 0.0],
        radial_strength: 0.0,
        enabled: false,
    }
}

/// Allocate a new [`ChromaticBuffer`] with all pixels set to opaque black.
#[allow(dead_code)]
pub fn new_chromatic_buffer(w: u32, h: u32) -> ChromaticBuffer {
    let size = (w as usize) * (h as usize);
    ChromaticBuffer {
        width: w,
        height: h,
        pixels: vec![[0.0, 0.0, 0.0, 1.0]; size],
    }
}

/// Apply chromatic aberration to `input`, producing a [`ChromaticResult`].
#[allow(dead_code)]
pub fn apply_chromatic_aberration(
    input: &ChromaticBuffer,
    cfg: &ChromaticConfig,
) -> ChromaticResult {
    let mut output = new_chromatic_buffer(input.width, input.height);
    let w = input.width as usize;
    let h = input.height as usize;

    let max_shift_px = max_channel_shift(cfg)
        * (w.min(h) as f32)
        + cfg.radial_strength * (w.min(h) as f32) * 0.1;

    if !cfg.enabled {
        output.pixels = input.pixels.clone();
        return ChromaticResult {
            output,
            max_shift_px: 0.0,
        };
    }

    for y in 0..h {
        for x in 0..w {
            let uv = [
                x as f32 / w as f32,
                y as f32 / h as f32,
            ];
            // Apply radial distortion to each channel's UV
            let uv_r = if cfg.radial_strength.abs() > 1e-9 {
                let distorted = radial_distort_uv(uv, cfg.radial_strength);
                [distorted[0] + cfg.red_offset[0], distorted[1] + cfg.red_offset[1]]
            } else {
                [uv[0] + cfg.red_offset[0], uv[1] + cfg.red_offset[1]]
            };
            let uv_g = if cfg.radial_strength.abs() > 1e-9 {
                let distorted = radial_distort_uv(uv, cfg.radial_strength * 0.5);
                [distorted[0] + cfg.green_offset[0], distorted[1] + cfg.green_offset[1]]
            } else {
                [uv[0] + cfg.green_offset[0], uv[1] + cfg.green_offset[1]]
            };
            let uv_b = if cfg.radial_strength.abs() > 1e-9 {
                let distorted = radial_distort_uv(uv, cfg.radial_strength * 0.25);
                [distorted[0] + cfg.blue_offset[0], distorted[1] + cfg.blue_offset[1]]
            } else {
                [uv[0] + cfg.blue_offset[0], uv[1] + cfg.blue_offset[1]]
            };

            let r = sample_channel(input, uv_r[0] * w as f32, uv_r[1] * h as f32, 0);
            let g = sample_channel(input, uv_g[0] * w as f32, uv_g[1] * h as f32, 1);
            let b = sample_channel(input, uv_b[0] * w as f32, uv_b[1] * h as f32, 2);
            let a = sample_channel(input, x as f32, y as f32, 3);

            let out_idx = y * w + x;
            if let Some(px) = output.pixels.get_mut(out_idx) {
                *px = [r, g, b, a];
            }
        }
    }

    ChromaticResult {
        output,
        max_shift_px,
    }
}

/// Bilinear sample of channel `ch` from a buffer at floating-point position `(x, y)`.
#[allow(dead_code)]
pub fn sample_channel(buf: &ChromaticBuffer, x: f32, y: f32, ch: usize) -> f32 {
    let w = buf.width as usize;
    let h = buf.height as usize;
    if w == 0 || h == 0 {
        return 0.0;
    }
    let cx = x.clamp(0.0, (w - 1) as f32) as usize;
    let cy = y.clamp(0.0, (h - 1) as f32) as usize;
    let idx = cy * w + cx;
    buf.pixels
        .get(idx)
        .and_then(|p| p.get(ch))
        .copied()
        .unwrap_or(0.0)
}

/// Apply barrel/pincushion radial distortion to a UV coordinate.
/// The `strength` value bends the UV toward (positive) or away (negative) from centre.
#[allow(dead_code)]
pub fn radial_distort_uv(uv: [f32; 2], strength: f32) -> [f32; 2] {
    let cx = uv[0] - 0.5;
    let cy = uv[1] - 0.5;
    let r2 = cx * cx + cy * cy;
    let factor = 1.0 + strength * r2;
    [0.5 + cx * factor, 0.5 + cy * factor]
}

/// Read the RGBA pixel at `(x, y)`, returning transparent black if out of bounds.
#[allow(dead_code)]
pub fn buffer_pixel_at(buf: &ChromaticBuffer, x: u32, y: u32) -> [f32; 4] {
    let idx = (y as usize) * (buf.width as usize) + (x as usize);
    buf.pixels.get(idx).copied().unwrap_or([0.0, 0.0, 0.0, 0.0])
}

/// Write an RGBA pixel at `(x, y)`, each component clamped to `[0, 1]`.
#[allow(dead_code)]
pub fn write_pixel_chromatic(buf: &mut ChromaticBuffer, x: u32, y: u32, pixel: [f32; 4]) {
    let idx = (y as usize) * (buf.width as usize) + (x as usize);
    if let Some(slot) = buf.pixels.get_mut(idx) {
        *slot = [
            pixel[0].clamp(0.0, 1.0),
            pixel[1].clamp(0.0, 1.0),
            pixel[2].clamp(0.0, 1.0),
            pixel[3].clamp(0.0, 1.0),
        ];
    }
}

/// Serialise the chromatic config to a JSON string.
#[allow(dead_code)]
pub fn chromatic_config_to_json(cfg: &ChromaticConfig) -> String {
    format!(
        r#"{{"red_offset":[{:.4},{:.4}],"green_offset":[{:.4},{:.4}],"blue_offset":[{:.4},{:.4}],"radial_strength":{:.4},"enabled":{}}}"#,
        cfg.red_offset[0],
        cfg.red_offset[1],
        cfg.green_offset[0],
        cfg.green_offset[1],
        cfg.blue_offset[0],
        cfg.blue_offset[1],
        cfg.radial_strength,
        cfg.enabled,
    )
}

/// Return the total pixel count of a [`ChromaticBuffer`].
#[allow(dead_code)]
pub fn chromatic_pixel_count(buf: &ChromaticBuffer) -> usize {
    buf.pixels.len()
}

/// Return the maximum UV-space offset across all three channel offsets.
#[allow(dead_code)]
pub fn max_channel_shift(cfg: &ChromaticConfig) -> f32 {
    let shifts = [
        (cfg.red_offset[0].powi(2) + cfg.red_offset[1].powi(2)).sqrt(),
        (cfg.green_offset[0].powi(2) + cfg.green_offset[1].powi(2)).sqrt(),
        (cfg.blue_offset[0].powi(2) + cfg.blue_offset[1].powi(2)).sqrt(),
    ];
    shifts.iter().cloned().fold(0.0_f32, f32::max)
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_disabled() {
        let cfg = default_chromatic_config();
        assert!(!cfg.enabled);
    }

    #[test]
    fn new_buffer_all_opaque_black() {
        let buf = new_chromatic_buffer(4, 4);
        assert_eq!(buf.pixels.len(), 16);
        for px in &buf.pixels {
            assert!((px[0]).abs() < 1e-6);
            assert!((px[3] - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn write_and_read_pixel() {
        let mut buf = new_chromatic_buffer(8, 8);
        write_pixel_chromatic(&mut buf, 2, 3, [0.5, 0.25, 0.1, 1.0]);
        let px = buffer_pixel_at(&buf, 2, 3);
        assert!((px[0] - 0.5).abs() < 1e-6);
        assert!((px[1] - 0.25).abs() < 1e-6);
    }

    #[test]
    fn pixel_count_matches_dimensions() {
        let buf = new_chromatic_buffer(16, 8);
        assert_eq!(chromatic_pixel_count(&buf), 128);
    }

    #[test]
    fn disabled_effect_passes_through() {
        let mut input = new_chromatic_buffer(4, 4);
        write_pixel_chromatic(&mut input, 1, 1, [0.8, 0.6, 0.4, 1.0]);
        let cfg = default_chromatic_config(); // enabled=false
        let result = apply_chromatic_aberration(&input, &cfg);
        let px = buffer_pixel_at(&result.output, 1, 1);
        assert!((px[0] - 0.8).abs() < 1e-5);
        assert!((result.max_shift_px).abs() < 1e-6);
    }

    #[test]
    fn max_channel_shift_non_zero() {
        let cfg = ChromaticConfig {
            red_offset: [0.01, 0.0],
            green_offset: [0.0, 0.0],
            blue_offset: [-0.01, 0.0],
            radial_strength: 0.0,
            enabled: true,
        };
        let shift = max_channel_shift(&cfg);
        assert!(shift > 0.0);
    }

    #[test]
    fn radial_distort_center_unchanged() {
        let uv = [0.5, 0.5];
        let distorted = radial_distort_uv(uv, 1.0);
        // Centre is the fixed point of radial distortion
        assert!((distorted[0] - 0.5).abs() < 1e-6);
        assert!((distorted[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn config_to_json_contains_keys() {
        let cfg = default_chromatic_config();
        let json = chromatic_config_to_json(&cfg);
        assert!(json.contains("red_offset"));
        assert!(json.contains("radial_strength"));
        assert!(json.contains("enabled"));
    }
}
