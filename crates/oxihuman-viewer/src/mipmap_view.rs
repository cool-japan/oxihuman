// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Mipmap level visualization for texture debugging.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MipmapViewConfig {
    pub force_level: Option<u32>,
    pub show_level_colors: bool,
    pub max_level: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MipmapLevel {
    pub level: u32,
    pub width: u32,
    pub height: u32,
}

#[allow(dead_code)]
pub fn default_mipmap_view_config() -> MipmapViewConfig {
    MipmapViewConfig {
        force_level: None,
        show_level_colors: true,
        max_level: 12,
    }
}

#[allow(dead_code)]
pub fn compute_mip_levels(width: u32, height: u32) -> u32 {
    let max_dim = width.max(height);
    if max_dim == 0 {
        return 0;
    }
    (max_dim as f32).log2().floor() as u32 + 1
}

#[allow(dead_code)]
pub fn mip_dimensions(base_width: u32, base_height: u32, level: u32) -> (u32, u32) {
    let w = (base_width >> level).max(1);
    let h = (base_height >> level).max(1);
    (w, h)
}

#[allow(dead_code)]
pub fn level_color(level: u32) -> [f32; 3] {
    match level {
        0 => [0.0, 1.0, 0.0],
        1 => [0.5, 1.0, 0.0],
        2 => [1.0, 1.0, 0.0],
        3 => [1.0, 0.5, 0.0],
        4 => [1.0, 0.0, 0.0],
        5 => [1.0, 0.0, 0.5],
        6 => [1.0, 0.0, 1.0],
        7 => [0.5, 0.0, 1.0],
        _ => [0.3, 0.3, 0.3],
    }
}

#[allow(dead_code)]
pub fn estimate_mip_level(texel_density: f32) -> u32 {
    if texel_density <= 0.0 {
        return 0;
    }
    texel_density.log2().max(0.0).floor() as u32
}

#[allow(dead_code)]
pub fn total_mip_texels(base_width: u32, base_height: u32) -> u64 {
    let levels = compute_mip_levels(base_width, base_height);
    let mut total = 0u64;
    for l in 0..levels {
        let (w, h) = mip_dimensions(base_width, base_height, l);
        total += (w as u64) * (h as u64);
    }
    total
}

#[allow(dead_code)]
pub fn generate_mip_chain(base_width: u32, base_height: u32) -> Vec<MipmapLevel> {
    let levels = compute_mip_levels(base_width, base_height);
    (0..levels)
        .map(|l| {
            let (w, h) = mip_dimensions(base_width, base_height, l);
            MipmapLevel { level: l, width: w, height: h }
        })
        .collect()
}

#[allow(dead_code)]
pub fn mipmap_view_to_json(cfg: &MipmapViewConfig) -> String {
    let force = cfg.force_level.map_or("null".to_string(), |l| l.to_string());
    format!(
        r#"{{"force_level":{},"show_colors":{},"max_level":{}}}"#,
        force, cfg.show_level_colors, cfg.max_level
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_mipmap_view_config();
        assert!(c.force_level.is_none());
    }

    #[test]
    fn test_compute_levels_256() {
        let levels = compute_mip_levels(256, 256);
        assert_eq!(levels, 9);
    }

    #[test]
    fn test_compute_levels_1() {
        let levels = compute_mip_levels(1, 1);
        assert_eq!(levels, 1);
    }

    #[test]
    fn test_compute_levels_zero() {
        let levels = compute_mip_levels(0, 0);
        assert_eq!(levels, 0);
    }

    #[test]
    fn test_mip_dimensions() {
        let (w, h) = mip_dimensions(256, 256, 2);
        assert_eq!(w, 64);
        assert_eq!(h, 64);
    }

    #[test]
    fn test_mip_dimensions_min() {
        let (w, h) = mip_dimensions(256, 256, 20);
        assert_eq!(w, 1);
        assert_eq!(h, 1);
    }

    #[test]
    fn test_level_color() {
        let c = level_color(0);
        assert!((c[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_estimate_level() {
        assert_eq!(estimate_mip_level(1.0), 0);
        assert_eq!(estimate_mip_level(2.0), 1);
        assert_eq!(estimate_mip_level(4.0), 2);
    }

    #[test]
    fn test_total_texels() {
        let t = total_mip_texels(4, 4);
        assert!(t > 16);
    }

    #[test]
    fn test_generate_chain() {
        let chain = generate_mip_chain(64, 64);
        assert_eq!(chain.len(), 7);
        assert_eq!(chain[0].width, 64);
        assert_eq!(chain[6].width, 1);
    }
}
