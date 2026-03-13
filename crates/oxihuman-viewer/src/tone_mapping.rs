// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! HDR tone mapping operators (Reinhard, ACES, Uncharted2).

// ── Enums ─────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ToneMappingOp {
    Reinhard,
    Aces,
    Uncharted2,
    Filmic,
    Linear,
}

// ── Structs ───────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ToneMappingParams {
    pub op: ToneMappingOp,
    pub exposure: f32,
    pub gamma: f32,
    pub white_point: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ToneMappingResult {
    pub mapped_pixels: Vec<[f32; 4]>,
    pub min_luminance: f32,
    pub max_luminance: f32,
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_tone_mapping_params() -> ToneMappingParams {
    ToneMappingParams {
        op: ToneMappingOp::Reinhard,
        exposure: 1.0,
        gamma: 2.2,
        white_point: 1.0,
    }
}

#[allow(dead_code)]
pub fn reinhard_tonemap(hdr: [f32; 3], exposure: f32) -> [f32; 3] {
    let scale = |v: f32| -> f32 {
        let x = v * exposure;
        x / (1.0 + x)
    };
    [scale(hdr[0]), scale(hdr[1]), scale(hdr[2])]
}

#[allow(dead_code)]
pub fn aces_tonemap(hdr: [f32; 3], exposure: f32) -> [f32; 3] {
    // ACES filmic curve approximation
    let aces = |v: f32| -> f32 {
        let x = v * exposure;
        let a = 2.51;
        let b = 0.03;
        let c = 2.43;
        let d = 0.59;
        let e = 0.14;
        ((x * (a * x + b)) / (x * (c * x + d) + e)).clamp(0.0, 1.0)
    };
    [aces(hdr[0]), aces(hdr[1]), aces(hdr[2])]
}

#[allow(dead_code)]
pub fn uncharted2_tonemap(hdr: [f32; 3], exposure: f32) -> [f32; 3] {
    let uc2 = |v: f32| -> f32 {
        let x = v * exposure;
        let a = 0.15_f32;
        let b = 0.50_f32;
        let c = 0.10_f32;
        let d = 0.20_f32;
        let e = 0.02_f32;
        let f = 0.30_f32;
        (x * (a * x + c * b) + d * e) / (x * (a * x + b) + d * f) - e / f
    };
    let white = uc2(11.2) + 1e-9;
    let map = |v: f32| (uc2(v) / white).clamp(0.0, 1.0);
    [map(hdr[0]), map(hdr[1]), map(hdr[2])]
}

#[allow(dead_code)]
pub fn apply_gamma(color: [f32; 3], gamma: f32) -> [f32; 3] {
    let inv = 1.0 / gamma.max(1e-9);
    [
        color[0].max(0.0).powf(inv),
        color[1].max(0.0).powf(inv),
        color[2].max(0.0).powf(inv),
    ]
}

#[allow(dead_code)]
pub fn tonemap_pixel(hdr: [f32; 4], cfg: &ToneMappingParams) -> [f32; 4] {
    let rgb = [hdr[0], hdr[1], hdr[2]];
    let mapped = match cfg.op {
        ToneMappingOp::Reinhard => reinhard_tonemap(rgb, cfg.exposure),
        ToneMappingOp::Aces => aces_tonemap(rgb, cfg.exposure),
        ToneMappingOp::Uncharted2 => uncharted2_tonemap(rgb, cfg.exposure),
        ToneMappingOp::Filmic => aces_tonemap(rgb, cfg.exposure),
        ToneMappingOp::Linear => {
            let e = cfg.exposure;
            [(rgb[0] * e).clamp(0.0, 1.0), (rgb[1] * e).clamp(0.0, 1.0), (rgb[2] * e).clamp(0.0, 1.0)]
        }
    };
    let gamma_corrected = apply_gamma(mapped, cfg.gamma);
    [gamma_corrected[0], gamma_corrected[1], gamma_corrected[2], hdr[3]]
}

#[allow(dead_code)]
pub fn tonemap_buffer(pixels: &[[f32; 4]], cfg: &ToneMappingParams) -> ToneMappingResult {
    let mapped_pixels: Vec<[f32; 4]> = pixels.iter().map(|&px| tonemap_pixel(px, cfg)).collect();

    let luminances: Vec<f32> = pixels.iter().map(|&px| luminance_hdr([px[0], px[1], px[2]])).collect();
    let min_luminance = luminances.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_luminance = luminances.iter().cloned().fold(0.0f32, f32::max);

    ToneMappingResult {
        mapped_pixels,
        min_luminance: if min_luminance.is_infinite() { 0.0 } else { min_luminance },
        max_luminance,
    }
}

#[allow(dead_code)]
pub fn op_name(cfg: &ToneMappingParams) -> &'static str {
    match cfg.op {
        ToneMappingOp::Reinhard => "reinhard",
        ToneMappingOp::Aces => "aces",
        ToneMappingOp::Uncharted2 => "uncharted2",
        ToneMappingOp::Filmic => "filmic",
        ToneMappingOp::Linear => "linear",
    }
}

#[allow(dead_code)]
pub fn luminance_hdr(color: [f32; 3]) -> f32 {
    0.2126 * color[0] + 0.7152 * color[1] + 0.0722 * color[2]
}

#[allow(dead_code)]
pub fn tone_mapping_params_to_json(cfg: &ToneMappingParams) -> String {
    format!(
        r#"{{"op":"{}","exposure":{},"gamma":{},"white_point":{}}}"#,
        op_name(cfg),
        cfg.exposure,
        cfg.gamma,
        cfg.white_point
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let cfg = default_tone_mapping_params();
        assert_eq!(cfg.op, ToneMappingOp::Reinhard);
        assert!((cfg.exposure - 1.0).abs() < 1e-6);
        assert!((cfg.gamma - 2.2).abs() < 1e-6);
    }

    #[test]
    fn test_reinhard_black() {
        let result = reinhard_tonemap([0.0, 0.0, 0.0], 1.0);
        assert_eq!(result, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_reinhard_bright_below_one() {
        let result = reinhard_tonemap([10.0, 10.0, 10.0], 1.0);
        for c in result {
            assert!((0.0..1.0).contains(&c), "reinhard should stay below 1.0 for any finite input");
        }
    }

    #[test]
    fn test_aces_clamps() {
        let result = aces_tonemap([100.0, 100.0, 100.0], 1.0);
        for c in result {
            assert!((0.0..=1.0).contains(&c), "aces output must be in [0,1]");
        }
    }

    #[test]
    fn test_uncharted2_clamps() {
        let result = uncharted2_tonemap([5.0, 5.0, 5.0], 1.0);
        for c in result {
            assert!((0.0..=1.0).contains(&c), "uncharted2 output must be in [0,1]");
        }
    }

    #[test]
    fn test_apply_gamma_identity() {
        let color = [0.5, 0.25, 0.75];
        // gamma=1.0 should be identity
        let result = apply_gamma(color, 1.0);
        for i in 0..3 {
            assert!((result[i] - color[i]).abs() < 1e-5);
        }
    }

    #[test]
    fn test_luminance_hdr_white() {
        let lum = luminance_hdr([1.0, 1.0, 1.0]);
        assert!((lum - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_luminance_hdr_black() {
        assert!((luminance_hdr([0.0, 0.0, 0.0])).abs() < 1e-9);
    }

    #[test]
    fn test_tonemap_pixel_alpha_preserved() {
        let cfg = default_tone_mapping_params();
        let px = [1.0, 0.5, 0.2, 0.75];
        let result = tonemap_pixel(px, &cfg);
        assert!((result[3] - 0.75).abs() < 1e-6, "alpha must be preserved");
    }

    #[test]
    fn test_tonemap_buffer_count() {
        let cfg = default_tone_mapping_params();
        let pixels = vec![[1.0f32; 4]; 10];
        let result = tonemap_buffer(&pixels, &cfg);
        assert_eq!(result.mapped_pixels.len(), 10);
    }

    #[test]
    fn test_tonemap_buffer_luminance_range() {
        let cfg = default_tone_mapping_params();
        let pixels = vec![[0.5, 0.5, 0.5, 1.0], [2.0, 2.0, 2.0, 1.0]];
        let result = tonemap_buffer(&pixels, &cfg);
        assert!(result.max_luminance >= result.min_luminance);
    }

    #[test]
    fn test_op_name() {
        let mut cfg = default_tone_mapping_params();
        assert_eq!(op_name(&cfg), "reinhard");
        cfg.op = ToneMappingOp::Aces;
        assert_eq!(op_name(&cfg), "aces");
        cfg.op = ToneMappingOp::Uncharted2;
        assert_eq!(op_name(&cfg), "uncharted2");
        cfg.op = ToneMappingOp::Linear;
        assert_eq!(op_name(&cfg), "linear");
    }

    #[test]
    fn test_to_json() {
        let cfg = default_tone_mapping_params();
        let json = tone_mapping_params_to_json(&cfg);
        assert!(json.contains("reinhard"));
        assert!(json.contains("exposure"));
        assert!(json.contains("gamma"));
    }
}
