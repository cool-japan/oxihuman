// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Shadow map generation and PCF sampling for soft shadows.

// ── Enums ────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ShadowMapFilter {
    None,
    Pcf3x3,
    Pcf5x5,
    Vsm,
}

// ── Structs ──────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShadowMapConfig {
    pub resolution: u32,
    pub near_plane: f32,
    pub far_plane: f32,
    pub bias: f32,
    pub filter: ShadowMapFilter,
    pub light_direction: [f32; 3],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShadowMap {
    pub config: ShadowMapConfig,
    pub depth_data: Vec<f32>,
    pub width: u32,
    pub height: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShadowSampleResult {
    pub in_shadow: bool,
    pub shadow_factor: f32,
    pub depth_diff: f32,
}

// ── Functions ────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_shadow_map_config() -> ShadowMapConfig {
    ShadowMapConfig {
        resolution: 1024,
        near_plane: 0.1,
        far_plane: 100.0,
        bias: 0.005,
        filter: ShadowMapFilter::Pcf3x3,
        light_direction: [0.0, -1.0, 0.0],
    }
}

#[allow(dead_code)]
pub fn new_shadow_map(cfg: ShadowMapConfig) -> ShadowMap {
    let w = cfg.resolution;
    let h = cfg.resolution;
    let pixel_count = (w * h) as usize;
    ShadowMap {
        config: cfg,
        depth_data: vec![1.0f32; pixel_count],
        width: w,
        height: h,
    }
}

#[allow(dead_code)]
pub fn shadow_map_pixel_count(sm: &ShadowMap) -> usize {
    (sm.width * sm.height) as usize
}

#[allow(dead_code)]
pub fn sample_shadow_map(sm: &ShadowMap, uv: [f32; 2], receiver_depth: f32) -> ShadowSampleResult {
    let map_depth = bilinear_depth(sm, uv);
    let depth_diff = receiver_depth - map_depth - sm.config.bias;
    let in_shadow = depth_diff > 0.0;
    let shadow_factor = if in_shadow { 0.0 } else { 1.0 };
    ShadowSampleResult {
        in_shadow,
        shadow_factor,
        depth_diff,
    }
}

#[allow(dead_code)]
pub fn pcf_sample(sm: &ShadowMap, uv: [f32; 2], depth: f32, kernel: usize) -> f32 {
    if kernel == 0 {
        return if sample_shadow_map(sm, uv, depth).in_shadow {
            0.0
        } else {
            1.0
        };
    }
    let half = kernel as f32 / 2.0;
    let texel_u = 1.0 / sm.width as f32;
    let texel_v = 1.0 / sm.height as f32;
    let mut lit = 0.0f32;
    let mut total = 0.0f32;
    for dy in 0..kernel {
        for dx in 0..kernel {
            let ou = uv[0] + (dx as f32 - half) * texel_u;
            let ov = uv[1] + (dy as f32 - half) * texel_v;
            let result = sample_shadow_map(sm, [ou, ov], depth);
            lit += result.shadow_factor;
            total += 1.0;
        }
    }
    if total > 0.0 { lit / total } else { 1.0 }
}

#[allow(dead_code)]
pub fn shadow_map_depth_at(sm: &ShadowMap, x: u32, y: u32) -> f32 {
    if x >= sm.width || y >= sm.height {
        return 1.0;
    }
    let idx = (y * sm.width + x) as usize;
    sm.depth_data.get(idx).copied().unwrap_or(1.0)
}

#[allow(dead_code)]
pub fn clear_shadow_map(sm: &mut ShadowMap) {
    sm.depth_data.iter_mut().for_each(|d| *d = 1.0);
}

#[allow(dead_code)]
pub fn write_depth(sm: &mut ShadowMap, x: u32, y: u32, depth: f32) {
    if x >= sm.width || y >= sm.height {
        return;
    }
    let idx = (y * sm.width + x) as usize;
    if let Some(d) = sm.depth_data.get_mut(idx) {
        *d = depth.clamp(0.0, 1.0);
    }
}

#[allow(dead_code)]
pub fn shadow_map_to_json(sm: &ShadowMap) -> String {
    format!(
        r#"{{"width":{},"height":{},"filter":"{}","bias":{}}}"#,
        sm.width,
        sm.height,
        filter_name(sm),
        sm.config.bias
    )
}

#[allow(dead_code)]
pub fn light_direction_normalized(sm: &ShadowMap) -> [f32; 3] {
    let d = sm.config.light_direction;
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if len < 1e-9 {
        [0.0, -1.0, 0.0]
    } else {
        [d[0] / len, d[1] / len, d[2] / len]
    }
}

#[allow(dead_code)]
pub fn filter_name(sm: &ShadowMap) -> &'static str {
    match sm.config.filter {
        ShadowMapFilter::None => "none",
        ShadowMapFilter::Pcf3x3 => "pcf3x3",
        ShadowMapFilter::Pcf5x5 => "pcf5x5",
        ShadowMapFilter::Vsm => "vsm",
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn bilinear_depth(sm: &ShadowMap, uv: [f32; 2]) -> f32 {
    let u = uv[0].clamp(0.0, 1.0);
    let v = uv[1].clamp(0.0, 1.0);
    let fx = u * (sm.width as f32 - 1.0);
    let fy = v * (sm.height as f32 - 1.0);
    let x0 = fx.floor() as u32;
    let y0 = fy.floor() as u32;
    let x1 = (x0 + 1).min(sm.width - 1);
    let y1 = (y0 + 1).min(sm.height - 1);
    let tx = fx - x0 as f32;
    let ty = fy - y0 as f32;
    let d00 = shadow_map_depth_at(sm, x0, y0);
    let d10 = shadow_map_depth_at(sm, x1, y0);
    let d01 = shadow_map_depth_at(sm, x0, y1);
    let d11 = shadow_map_depth_at(sm, x1, y1);
    let top = d00 * (1.0 - tx) + d10 * tx;
    let bot = d01 * (1.0 - tx) + d11 * tx;
    top * (1.0 - ty) + bot * ty
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_resolution() {
        let cfg = default_shadow_map_config();
        assert_eq!(cfg.resolution, 1024);
        assert_eq!(cfg.filter, ShadowMapFilter::Pcf3x3);
    }

    #[test]
    fn new_shadow_map_pixel_count() {
        let cfg = default_shadow_map_config();
        let sm = new_shadow_map(cfg);
        assert_eq!(shadow_map_pixel_count(&sm), 1024 * 1024);
    }

    #[test]
    fn write_and_read_depth() {
        let cfg = default_shadow_map_config();
        let mut sm = new_shadow_map(cfg);
        write_depth(&mut sm, 10, 10, 0.5);
        assert!((shadow_map_depth_at(&sm, 10, 10) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn clear_shadow_map_resets_to_one() {
        let cfg = default_shadow_map_config();
        let mut sm = new_shadow_map(cfg);
        write_depth(&mut sm, 0, 0, 0.3);
        clear_shadow_map(&mut sm);
        assert!((shadow_map_depth_at(&sm, 0, 0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn sample_not_in_shadow_when_receiver_is_closer() {
        let cfg = default_shadow_map_config();
        let mut sm = new_shadow_map(cfg);
        // depth_data = 0.9 at center, receiver at 0.5 => not in shadow
        write_depth(&mut sm, 512, 512, 0.9);
        let result = sample_shadow_map(&sm, [0.5, 0.5], 0.5);
        assert!(!result.in_shadow);
        assert!((result.shadow_factor - 1.0).abs() < 1e-4);
    }

    #[test]
    fn sample_in_shadow_when_receiver_is_farther() {
        let cfg = default_shadow_map_config();
        let mut sm = new_shadow_map(cfg);
        // Fill a 2x2 block around center so bilinear sample returns ~0.3
        for dy in 0..2u32 {
            for dx in 0..2u32 {
                write_depth(&mut sm, 511 + dx, 511 + dy, 0.3);
            }
        }
        // receiver at 0.8 => farther than 0.3 + bias => in shadow
        let result = sample_shadow_map(&sm, [0.5, 0.5], 0.8);
        assert!(result.in_shadow);
    }

    #[test]
    fn filter_name_all_variants() {
        let mut cfg = default_shadow_map_config();
        cfg.filter = ShadowMapFilter::None;
        let sm = new_shadow_map(cfg.clone());
        assert_eq!(filter_name(&sm), "none");

        cfg.filter = ShadowMapFilter::Pcf5x5;
        let sm = new_shadow_map(cfg.clone());
        assert_eq!(filter_name(&sm), "pcf5x5");

        cfg.filter = ShadowMapFilter::Vsm;
        let sm = new_shadow_map(cfg);
        assert_eq!(filter_name(&sm), "vsm");
    }

    #[test]
    fn light_direction_normalized_unit_length() {
        let cfg = default_shadow_map_config();
        let sm = new_shadow_map(cfg);
        let n = light_direction_normalized(&sm);
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn shadow_map_to_json_contains_fields() {
        let cfg = default_shadow_map_config();
        let sm = new_shadow_map(cfg);
        let json = shadow_map_to_json(&sm);
        assert!(json.contains("width"));
        assert!(json.contains("filter"));
    }

    #[test]
    fn pcf_sample_fully_lit_unoccluded() {
        let cfg = default_shadow_map_config();
        // all depth = 1.0, receiver at 0.5 => fully lit
        let sm = new_shadow_map(cfg);
        let factor = pcf_sample(&sm, [0.5, 0.5], 0.5, 3);
        assert!(factor > 0.9);
    }
}
