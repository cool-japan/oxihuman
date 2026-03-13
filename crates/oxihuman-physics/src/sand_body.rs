// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Sand simulation: angle of repose, avalanche propagation.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Sand configuration parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SandConfig {
    /// Angle of repose in radians.
    pub angle_of_repose: f32,
    /// Cell size of the height grid.
    pub cell_size: f32,
    /// Avalanche redistribution fraction per step.
    pub avalanche_rate: f32,
}

/// Default sand config (dry sand ~34 degrees).
#[allow(dead_code)]
pub fn default_sand_config() -> SandConfig {
    SandConfig {
        angle_of_repose: 34.0 * PI / 180.0,
        cell_size: 1.0,
        avalanche_rate: 0.5,
    }
}

/// Sand height grid (column-major, width x depth).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SandBody {
    pub width: usize,
    pub depth: usize,
    pub heights: Vec<f32>,
    pub config: SandConfig,
}

/// Create a flat sand body.
#[allow(dead_code)]
pub fn new_sand_body(width: usize, depth: usize, config: SandConfig) -> SandBody {
    SandBody {
        width,
        depth,
        heights: vec![0.0f32; width * depth],
        config,
    }
}

fn idx(body: &SandBody, x: usize, z: usize) -> usize {
    z * body.width + x
}

/// Get height at (x, z).
#[allow(dead_code)]
pub fn sand_get(body: &SandBody, x: usize, z: usize) -> f32 {
    body.heights[idx(body, x, z)]
}

/// Set height at (x, z).
#[allow(dead_code)]
pub fn sand_set(body: &mut SandBody, x: usize, z: usize, h: f32) {
    let i = idx(body, x, z);
    body.heights[i] = h.max(0.0);
}

/// Add material to a cell.
#[allow(dead_code)]
pub fn sand_deposit(body: &mut SandBody, x: usize, z: usize, amount: f32) {
    let i = idx(body, x, z);
    body.heights[i] = (body.heights[i] + amount).max(0.0);
}

/// Compute the maximum slope in the grid (for stability check).
#[allow(dead_code)]
pub fn sand_max_slope(body: &SandBody) -> f32 {
    let mut max_slope = 0.0f32;
    for z in 0..body.depth {
        for x in 0..body.width {
            let h = sand_get(body, x, z);
            if x + 1 < body.width {
                let dh = (h - sand_get(body, x + 1, z)).abs();
                let slope = dh / body.config.cell_size;
                if slope > max_slope {
                    max_slope = slope;
                }
            }
            if z + 1 < body.depth {
                let dh = (h - sand_get(body, x, z + 1)).abs();
                let slope = dh / body.config.cell_size;
                if slope > max_slope {
                    max_slope = slope;
                }
            }
        }
    }
    max_slope
}

/// Critical height difference above which avalanche occurs.
#[allow(dead_code)]
pub fn sand_critical_delta(config: &SandConfig) -> f32 {
    config.cell_size * config.angle_of_repose.tan()
}

/// Run one avalanche pass: redistribute height where slope exceeds angle of repose.
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn sand_avalanche_step(body: &mut SandBody) {
    let critical = sand_critical_delta(&body.config);
    let rate = body.config.avalanche_rate;
    let w = body.width;
    let d = body.depth;
    let mut deltas = vec![0.0f32; w * d];
    for z in 0..d {
        for x in 0..w {
            let h = body.heights[z * w + x];
            // Check x+1 neighbor
            if x + 1 < w {
                let hn = body.heights[z * w + x + 1];
                let diff = h - hn;
                if diff > critical {
                    let transfer = (diff - critical) * rate * 0.5;
                    deltas[z * w + x] -= transfer;
                    deltas[z * w + x + 1] += transfer;
                } else if diff < -critical {
                    let transfer = (-diff - critical) * rate * 0.5;
                    deltas[z * w + x] += transfer;
                    deltas[z * w + x + 1] -= transfer;
                }
            }
            // Check z+1 neighbor
            if z + 1 < d {
                let hn = body.heights[(z + 1) * w + x];
                let diff = h - hn;
                if diff > critical {
                    let transfer = (diff - critical) * rate * 0.5;
                    deltas[z * w + x] -= transfer;
                    deltas[(z + 1) * w + x] += transfer;
                } else if diff < -critical {
                    let transfer = (-diff - critical) * rate * 0.5;
                    deltas[z * w + x] += transfer;
                    deltas[(z + 1) * w + x] -= transfer;
                }
            }
        }
    }
    for i in 0..body.heights.len() {
        body.heights[i] = (body.heights[i] + deltas[i]).max(0.0);
    }
}

/// Total volume (sum of all heights * cell_size^2).
#[allow(dead_code)]
pub fn sand_total_volume(body: &SandBody) -> f32 {
    let cell_area = body.config.cell_size * body.config.cell_size;
    body.heights.iter().sum::<f32>() * cell_area
}

/// Check if the current sand is stable (no slope exceeds angle of repose).
#[allow(dead_code)]
pub fn sand_is_stable(body: &SandBody) -> bool {
    sand_max_slope(body) <= body.config.angle_of_repose.tan()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_repose() {
        let cfg = default_sand_config();
        // ~34 degrees = ~0.593 radians
        assert!(cfg.angle_of_repose > 0.5 && cfg.angle_of_repose < 0.7);
    }

    #[test]
    fn new_body_flat() {
        let body = new_sand_body(5, 5, default_sand_config());
        for h in &body.heights {
            assert_eq!(*h, 0.0);
        }
    }

    #[test]
    fn deposit_increases_height() {
        let mut body = new_sand_body(5, 5, default_sand_config());
        sand_deposit(&mut body, 2, 2, 3.0);
        assert!((sand_get(&body, 2, 2) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn flat_sand_is_stable() {
        let body = new_sand_body(5, 5, default_sand_config());
        assert!(sand_is_stable(&body));
    }

    #[test]
    fn avalanche_reduces_peak() {
        let mut body = new_sand_body(5, 5, default_sand_config());
        sand_set(&mut body, 2, 2, 10.0);
        let before = sand_get(&body, 2, 2);
        sand_avalanche_step(&mut body);
        assert!(sand_get(&body, 2, 2) < before);
    }

    #[test]
    fn volume_conserved_after_avalanche() {
        let mut body = new_sand_body(5, 5, default_sand_config());
        sand_set(&mut body, 2, 2, 5.0);
        let before = sand_total_volume(&body);
        sand_avalanche_step(&mut body);
        let after = sand_total_volume(&body);
        assert!((before - after).abs() < 0.1);
    }

    #[test]
    fn max_slope_zero_flat() {
        let body = new_sand_body(3, 3, default_sand_config());
        assert_eq!(sand_max_slope(&body), 0.0);
    }

    #[test]
    fn set_negative_clamped() {
        let mut body = new_sand_body(3, 3, default_sand_config());
        sand_set(&mut body, 1, 1, -5.0);
        assert_eq!(sand_get(&body, 1, 1), 0.0);
    }

    #[test]
    fn total_volume_correct() {
        let mut body = new_sand_body(3, 3, default_sand_config());
        sand_set(&mut body, 0, 0, 2.0);
        let vol = sand_total_volume(&body);
        assert!((vol - 2.0).abs() < 1e-5);
    }

    #[test]
    fn critical_delta_positive() {
        let cfg = default_sand_config();
        let cd = sand_critical_delta(&cfg);
        assert!(cd > 0.0);
    }
}
