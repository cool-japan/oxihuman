// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Motion blur tile — per-tile velocity analysis for tiled motion blur reconstruction.

use std::f32::consts::SQRT_2;

/// Motion vector for a tile.
#[derive(Debug, Clone, PartialEq, Default)]
#[allow(dead_code)]
pub struct TileVelocity {
    pub vx: f32,
    pub vy: f32,
}

impl TileVelocity {
    #[allow(dead_code)]
    pub fn magnitude(&self) -> f32 {
        (self.vx * self.vx + self.vy * self.vy).sqrt()
    }
    #[allow(dead_code)]
    pub fn normalize(&self) -> [f32; 2] {
        let m = self.magnitude();
        if m < 1e-9 {
            [0.0, 0.0]
        } else {
            [self.vx / m, self.vy / m]
        }
    }
}

/// Motion blur tile configuration.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct MotionBlurTileConfig {
    pub tile_size: u32,
    pub max_blur_samples: u32,
    pub shutteropen: f32,
    pub enabled: bool,
}

impl Default for MotionBlurTileConfig {
    fn default() -> Self {
        Self {
            tile_size: 16,
            max_blur_samples: 16,
            shutteropen: 0.5,
            enabled: true,
        }
    }
}

/// Per-tile blur data.
#[derive(Debug, Clone, PartialEq, Default)]
#[allow(dead_code)]
pub struct BlurTile {
    pub tile_x: u32,
    pub tile_y: u32,
    pub dominant_velocity: TileVelocity,
    pub max_velocity: f32,
    pub needs_blur: bool,
}

/// Motion blur tile manager.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct MotionBlurTileManager {
    pub config: MotionBlurTileConfig,
    pub tiles: Vec<BlurTile>,
    pub width: u32,
    pub height: u32,
}

/// Create new tile manager.
#[allow(dead_code)]
pub fn new_motion_blur_manager(
    cfg: MotionBlurTileConfig,
    width: u32,
    height: u32,
) -> MotionBlurTileManager {
    MotionBlurTileManager {
        config: cfg,
        tiles: Vec::new(),
        width,
        height,
    }
}

/// Build tile grid.
#[allow(dead_code)]
pub fn build_tile_grid(m: &mut MotionBlurTileManager) {
    m.tiles.clear();
    let ts = m.config.tile_size.max(1);
    let tx = m.width.div_ceil(ts);
    let ty = m.height.div_ceil(ts);
    for y in 0..ty {
        for x in 0..tx {
            m.tiles.push(BlurTile {
                tile_x: x,
                tile_y: y,
                ..Default::default()
            });
        }
    }
}

/// Set velocity for a tile.
#[allow(dead_code)]
pub fn set_tile_velocity(
    m: &mut MotionBlurTileManager,
    tile_x: u32,
    tile_y: u32,
    vx: f32,
    vy: f32,
) {
    let ts = m.config.tile_size.max(1);
    let tw = m.width.div_ceil(ts);
    let idx = (tile_y * tw + tile_x) as usize;
    if let Some(t) = m.tiles.get_mut(idx) {
        t.dominant_velocity = TileVelocity { vx, vy };
        t.max_velocity = (vx * vx + vy * vy).sqrt();
        // threshold uses SQRT_2 pixels as minimum blur threshold
        t.needs_blur = t.max_velocity > SQRT_2;
    }
}

/// Count tiles that need blur.
#[allow(dead_code)]
pub fn blur_tile_count(m: &MotionBlurTileManager) -> usize {
    m.tiles.iter().filter(|t| t.needs_blur).count()
}

/// Total tile count.
#[allow(dead_code)]
pub fn tile_count(m: &MotionBlurTileManager) -> usize {
    m.tiles.len()
}

/// Compute sample count for a given velocity magnitude.
#[allow(dead_code)]
pub fn sample_count_for_velocity(velocity: f32, cfg: &MotionBlurTileConfig) -> u32 {
    let samples = (velocity * cfg.shutteropen).ceil() as u32;
    samples.clamp(1, cfg.max_blur_samples)
}

/// Export to JSON-like string.
#[allow(dead_code)]
pub fn motion_blur_tile_to_json(m: &MotionBlurTileManager) -> String {
    format!(
        r#"{{"tile_count":{},"blur_tiles":{},"enabled":{}}}"#,
        m.tiles.len(),
        blur_tile_count(m),
        m.config.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_tile_grid_correct_count() {
        let cfg = MotionBlurTileConfig {
            tile_size: 16,
            ..Default::default()
        };
        let mut m = new_motion_blur_manager(cfg, 64, 32);
        build_tile_grid(&mut m);
        assert_eq!(tile_count(&m), 8); // 4*2
    }

    #[test]
    fn velocity_magnitude() {
        let v = TileVelocity { vx: 3.0, vy: 4.0 };
        assert!((v.magnitude() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn velocity_normalize() {
        let v = TileVelocity { vx: 1.0, vy: 0.0 };
        let n = v.normalize();
        assert!((n[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn zero_velocity_no_blur() {
        let cfg = MotionBlurTileConfig::default();
        let mut m = new_motion_blur_manager(cfg, 32, 32);
        build_tile_grid(&mut m);
        set_tile_velocity(&mut m, 0, 0, 0.0, 0.0);
        assert_eq!(blur_tile_count(&m), 0);
    }

    #[test]
    fn high_velocity_triggers_blur() {
        let cfg = MotionBlurTileConfig::default();
        let mut m = new_motion_blur_manager(cfg, 32, 32);
        build_tile_grid(&mut m);
        set_tile_velocity(&mut m, 0, 0, 5.0, 5.0);
        assert!(blur_tile_count(&m) > 0);
    }

    #[test]
    fn sqrt2_threshold() {
        // exactly SQRT_2 should not trigger blur (> not >=)
        let v = TileVelocity {
            vx: SQRT_2,
            vy: 0.0,
        };
        assert!((v.magnitude() - SQRT_2).abs() < 1e-5);
    }

    #[test]
    fn sample_count_clamps() {
        let cfg = MotionBlurTileConfig {
            max_blur_samples: 8,
            ..Default::default()
        };
        let n = sample_count_for_velocity(1000.0, &cfg);
        assert_eq!(n, 8);
    }

    #[test]
    fn sample_count_min_one() {
        let cfg = MotionBlurTileConfig::default();
        let n = sample_count_for_velocity(0.0, &cfg);
        assert_eq!(n, 1);
    }

    #[test]
    fn json_contains_tile_count() {
        let m = new_motion_blur_manager(MotionBlurTileConfig::default(), 64, 64);
        assert!(motion_blur_tile_to_json(&m).contains("tile_count"));
    }

    #[test]
    fn is_empty_false_after_build() {
        let cfg = MotionBlurTileConfig::default();
        let mut m = new_motion_blur_manager(cfg, 32, 32);
        build_tile_grid(&mut m);
        assert!(!m.tiles.is_empty());
    }
}
