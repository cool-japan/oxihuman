// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Bloom post-processing effect using Gaussian blur and threshold extraction.

// ── Structs ───────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BloomEffectConfig {
    pub threshold: f32,
    pub intensity: f32,
    pub radius: f32,
    pub iterations: u32,
    pub enabled: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BloomBuffer {
    pub width: u32,
    pub height: u32,
    pub data: Vec<[f32; 4]>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BloomResult {
    pub output: BloomBuffer,
    pub bright_pixel_count: usize,
    pub peak_luminance: f32,
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_bloom_effect_config() -> BloomEffectConfig {
    BloomEffectConfig {
        threshold: 1.0,
        intensity: 0.5,
        radius: 2.0,
        iterations: 4,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn new_bloom_buffer(w: u32, h: u32) -> BloomBuffer {
    let size = (w as usize) * (h as usize);
    BloomBuffer {
        width: w,
        height: h,
        data: vec![[0.0, 0.0, 0.0, 1.0]; size],
    }
}

#[allow(dead_code)]
pub fn extract_bright(input: &BloomBuffer, threshold: f32) -> BloomBuffer {
    let mut out = new_bloom_buffer(input.width, input.height);
    for (i, &px) in input.data.iter().enumerate() {
        let lum = pixel_luminance(px);
        if lum > threshold {
            out.data[i] = [px[0], px[1], px[2], px[3]];
        } else {
            out.data[i] = [0.0, 0.0, 0.0, px[3]];
        }
    }
    out
}

#[allow(dead_code)]
pub fn gaussian_blur_h(buf: &BloomBuffer, radius: f32) -> BloomBuffer {
    let w = buf.width as usize;
    let h = buf.height as usize;
    let mut out = new_bloom_buffer(buf.width, buf.height);
    let sigma = radius.max(0.01);
    let half = (sigma * 3.0).ceil() as i32;

    for y in 0..h {
        for x in 0..w {
            let mut acc = [0.0f32; 4];
            let mut weight_sum = 0.0f32;
            for dx in -half..=half {
                let sx = (x as i32 + dx).clamp(0, w as i32 - 1) as usize;
                let w_val = gauss_weight(dx as f32, sigma);
                let src = buf.data[y * w + sx];
                for c in 0..4 {
                    acc[c] += src[c] * w_val;
                }
                weight_sum += w_val;
            }
            if weight_sum > 1e-9 {
                for item in &mut acc {
                    *item /= weight_sum;
                }
            }
            out.data[y * w + x] = acc;
        }
    }
    out
}

#[allow(dead_code)]
pub fn gaussian_blur_v(buf: &BloomBuffer, radius: f32) -> BloomBuffer {
    let w = buf.width as usize;
    let h = buf.height as usize;
    let mut out = new_bloom_buffer(buf.width, buf.height);
    let sigma = radius.max(0.01);
    let half = (sigma * 3.0).ceil() as i32;

    for y in 0..h {
        for x in 0..w {
            let mut acc = [0.0f32; 4];
            let mut weight_sum = 0.0f32;
            for dy in -half..=half {
                let sy = (y as i32 + dy).clamp(0, h as i32 - 1) as usize;
                let w_val = gauss_weight(dy as f32, sigma);
                let src = buf.data[sy * w + x];
                for c in 0..4 {
                    acc[c] += src[c] * w_val;
                }
                weight_sum += w_val;
            }
            if weight_sum > 1e-9 {
                for item in &mut acc {
                    *item /= weight_sum;
                }
            }
            out.data[y * w + x] = acc;
        }
    }
    out
}

#[allow(dead_code)]
pub fn composite_bloom(base: &BloomBuffer, bloom: &BloomBuffer, intensity: f32) -> BloomBuffer {
    let len = base.data.len().min(bloom.data.len());
    let mut out = new_bloom_buffer(base.width, base.height);
    for i in 0..len {
        let b = base.data[i];
        let bl = bloom.data[i];
        out.data[i] = [
            (b[0] + bl[0] * intensity).clamp(0.0, 1.0),
            (b[1] + bl[1] * intensity).clamp(0.0, 1.0),
            (b[2] + bl[2] * intensity).clamp(0.0, 1.0),
            b[3],
        ];
    }
    out
}

#[allow(dead_code)]
pub fn apply_bloom(input: &BloomBuffer, cfg: &BloomEffectConfig) -> BloomResult {
    if !cfg.enabled {
        return BloomResult {
            output: input.clone(),
            bright_pixel_count: 0,
            peak_luminance: 0.0,
        };
    }

    let bright = extract_bright(input, cfg.threshold);
    let bright_pixel_count = bright.data.iter().filter(|&&px| pixel_luminance(px) > 0.0).count();
    let peak_luminance = input
        .data
        .iter()
        .map(|&px| pixel_luminance(px))
        .fold(0.0f32, f32::max);

    let mut blurred = bright.clone();
    for _ in 0..cfg.iterations {
        blurred = gaussian_blur_h(&blurred, cfg.radius);
        blurred = gaussian_blur_v(&blurred, cfg.radius);
    }

    let output = composite_bloom(input, &blurred, cfg.intensity);
    BloomResult {
        output,
        bright_pixel_count,
        peak_luminance,
    }
}

#[allow(dead_code)]
pub fn buffer_pixel_count(buf: &BloomBuffer) -> usize {
    buf.data.len()
}

#[allow(dead_code)]
pub fn pixel_luminance(pixel: [f32; 4]) -> f32 {
    0.2126 * pixel[0] + 0.7152 * pixel[1] + 0.0722 * pixel[2]
}

#[allow(dead_code)]
pub fn bloom_effect_config_to_json(cfg: &BloomEffectConfig) -> String {
    format!(
        r#"{{"threshold":{},"intensity":{},"radius":{},"iterations":{},"enabled":{}}}"#,
        cfg.threshold, cfg.intensity, cfg.radius, cfg.iterations, cfg.enabled
    )
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn gauss_weight(x: f32, sigma: f32) -> f32 {
    let exp = -(x * x) / (2.0 * sigma * sigma);
    exp.exp()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_bloom_config() {
        let cfg = default_bloom_effect_config();
        assert!(cfg.enabled);
        assert!((cfg.threshold - 1.0).abs() < 1e-6);
        assert!(cfg.iterations > 0);
    }

    #[test]
    fn test_new_bloom_buffer() {
        let buf = new_bloom_buffer(4, 4);
        assert_eq!(buf.width, 4);
        assert_eq!(buf.height, 4);
        assert_eq!(buffer_pixel_count(&buf), 16);
    }

    #[test]
    fn test_pixel_luminance() {
        // Pure white should be ~1.0
        let lum = pixel_luminance([1.0, 1.0, 1.0, 1.0]);
        assert!((lum - 1.0).abs() < 1e-4);
        // Black should be 0.0
        assert!((pixel_luminance([0.0, 0.0, 0.0, 1.0])).abs() < 1e-9);
    }

    #[test]
    fn test_extract_bright_filters() {
        let mut buf = new_bloom_buffer(2, 1);
        buf.data[0] = [2.0, 2.0, 2.0, 1.0]; // above threshold
        buf.data[1] = [0.1, 0.1, 0.1, 1.0]; // below threshold
        let bright = extract_bright(&buf, 1.0);
        assert!(pixel_luminance(bright.data[0]) > 0.0);
        assert!((pixel_luminance(bright.data[1])).abs() < 1e-6);
    }

    #[test]
    fn test_gaussian_blur_h_preserves_size() {
        let buf = new_bloom_buffer(8, 8);
        let blurred = gaussian_blur_h(&buf, 1.5);
        assert_eq!(blurred.width, 8);
        assert_eq!(blurred.height, 8);
    }

    #[test]
    fn test_gaussian_blur_v_preserves_size() {
        let buf = new_bloom_buffer(8, 8);
        let blurred = gaussian_blur_v(&buf, 1.5);
        assert_eq!(blurred.width, 8);
        assert_eq!(blurred.height, 8);
    }

    #[test]
    fn test_composite_bloom_clamps() {
        let mut base = new_bloom_buffer(1, 1);
        base.data[0] = [0.9, 0.9, 0.9, 1.0];
        let mut bloom = new_bloom_buffer(1, 1);
        bloom.data[0] = [0.5, 0.5, 0.5, 1.0];
        let result = composite_bloom(&base, &bloom, 2.0);
        // Should be clamped to 1.0
        assert!(result.data[0][0] <= 1.0);
        assert!(result.data[0][1] <= 1.0);
    }

    #[test]
    fn test_apply_bloom_disabled() {
        let mut cfg = default_bloom_effect_config();
        cfg.enabled = false;
        let buf = new_bloom_buffer(4, 4);
        let result = apply_bloom(&buf, &cfg);
        assert_eq!(result.bright_pixel_count, 0);
        assert_eq!(result.output.width, 4);
    }

    #[test]
    fn test_apply_bloom_bright_pixels_detected() {
        let cfg = default_bloom_effect_config();
        let mut buf = new_bloom_buffer(2, 2);
        // Make two pixels bright
        buf.data[0] = [2.0, 2.0, 2.0, 1.0];
        buf.data[2] = [1.5, 1.5, 1.5, 1.0];
        let result = apply_bloom(&buf, &cfg);
        assert!(result.bright_pixel_count > 0);
        assert!(result.peak_luminance > 1.0);
    }

    #[test]
    fn test_bloom_config_to_json() {
        let cfg = default_bloom_effect_config();
        let json = bloom_effect_config_to_json(&cfg);
        assert!(json.contains("threshold"));
        assert!(json.contains("intensity"));
        assert!(json.contains("enabled"));
    }
}
