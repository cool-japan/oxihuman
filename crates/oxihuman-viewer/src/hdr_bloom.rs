// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! HDR bloom post-process combining threshold extraction with multi-pass blur.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HdrBloomConfig {
    pub threshold: f32,
    pub knee: f32,
    pub intensity: f32,
    pub passes: u32,
    pub downsample_factor: u32,
    pub enabled: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HdrBloomBuffer {
    pub width: u32,
    pub height: u32,
    pub data: Vec<[f32; 4]>,
}

#[allow(dead_code)]
pub fn default_hdr_bloom_config() -> HdrBloomConfig {
    HdrBloomConfig {
        threshold: 1.0,
        knee: 0.1,
        intensity: 0.8,
        passes: 5,
        downsample_factor: 2,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn new_hdr_bloom_buffer(w: u32, h: u32) -> HdrBloomBuffer {
    HdrBloomBuffer {
        width: w,
        height: h,
        data: vec![[0.0, 0.0, 0.0, 1.0]; (w as usize) * (h as usize)],
    }
}

#[allow(dead_code)]
pub fn soft_threshold(luminance: f32, threshold: f32, knee: f32) -> f32 {
    let soft = luminance - threshold + knee;
    let soft = soft.clamp(0.0, 2.0 * knee);
    let contrib = soft * soft / (4.0 * knee + 1e-8);
    if luminance > threshold + knee {
        luminance - threshold
    } else if luminance > threshold - knee {
        contrib
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn pixel_luminance_hdr(px: [f32; 4]) -> f32 {
    0.2126 * px[0] + 0.7152 * px[1] + 0.0722 * px[2]
}

#[allow(dead_code)]
pub fn extract_bright_hdr(input: &HdrBloomBuffer, threshold: f32, knee: f32) -> HdrBloomBuffer {
    let mut out = new_hdr_bloom_buffer(input.width, input.height);
    for (i, &px) in input.data.iter().enumerate() {
        let lum = pixel_luminance_hdr(px);
        let factor = soft_threshold(lum, threshold, knee);
        let scale = if lum > 1e-8 { factor / lum } else { 0.0 };
        out.data[i] = [px[0] * scale, px[1] * scale, px[2] * scale, 1.0];
    }
    out
}

#[allow(dead_code)]
pub fn downsample_2x(input: &HdrBloomBuffer) -> HdrBloomBuffer {
    let nw = (input.width / 2).max(1);
    let nh = (input.height / 2).max(1);
    let mut out = new_hdr_bloom_buffer(nw, nh);
    for y in 0..nh {
        for x in 0..nw {
            let sx = (x * 2) as usize;
            let sy = (y * 2) as usize;
            let w = input.width as usize;
            let p00 = input.data[sy * w + sx];
            let oi = (y as usize) * (nw as usize) + (x as usize);
            out.data[oi] = p00;
        }
    }
    out
}

#[allow(dead_code)]
pub fn bloom_combine(base: &HdrBloomBuffer, bloom: &HdrBloomBuffer, intensity: f32) -> HdrBloomBuffer {
    let len = base.data.len().min(bloom.data.len());
    let mut out = new_hdr_bloom_buffer(base.width, base.height);
    for i in 0..len {
        let b = base.data[i];
        let bl = bloom.data[i];
        out.data[i] = [
            b[0] + bl[0] * intensity,
            b[1] + bl[1] * intensity,
            b[2] + bl[2] * intensity,
            b[3],
        ];
    }
    out
}

#[allow(dead_code)]
pub fn hdr_bloom_to_json(cfg: &HdrBloomConfig) -> String {
    format!(
        r#"{{"threshold":{},"knee":{},"intensity":{},"passes":{},"enabled":{}}}"#,
        cfg.threshold, cfg.knee, cfg.intensity, cfg.passes, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_hdr_bloom_config();
        assert!(c.enabled);
        assert_eq!(c.passes, 5);
    }

    #[test]
    fn test_new_buffer() {
        let b = new_hdr_bloom_buffer(8, 8);
        assert_eq!(b.data.len(), 64);
    }

    #[test]
    fn test_soft_threshold_below() {
        let v = soft_threshold(0.5, 1.0, 0.1);
        assert!(v.abs() < 1e-6);
    }

    #[test]
    fn test_soft_threshold_above() {
        let v = soft_threshold(2.0, 1.0, 0.1);
        assert!((v - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_luminance() {
        let lum = pixel_luminance_hdr([1.0, 1.0, 1.0, 1.0]);
        assert!((lum - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_extract_bright() {
        let mut buf = new_hdr_bloom_buffer(2, 2);
        buf.data[0] = [2.0, 2.0, 2.0, 1.0];
        let bright = extract_bright_hdr(&buf, 1.0, 0.1);
        assert!(bright.data[0][0] > 0.0);
    }

    #[test]
    fn test_downsample() {
        let buf = new_hdr_bloom_buffer(4, 4);
        let ds = downsample_2x(&buf);
        assert_eq!(ds.width, 2);
        assert_eq!(ds.height, 2);
    }

    #[test]
    fn test_combine() {
        let base = new_hdr_bloom_buffer(2, 2);
        let bloom = new_hdr_bloom_buffer(2, 2);
        let result = bloom_combine(&base, &bloom, 1.0);
        assert_eq!(result.data.len(), 4);
    }

    #[test]
    fn test_to_json() {
        let c = default_hdr_bloom_config();
        let j = hdr_bloom_to_json(&c);
        assert!(j.contains("threshold"));
    }

    #[test]
    fn test_soft_threshold_knee_zone() {
        let v = soft_threshold(0.95, 1.0, 0.1);
        assert!(v > 0.0);
    }
}
