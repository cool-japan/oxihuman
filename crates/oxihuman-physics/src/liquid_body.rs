// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Liquid body: height-field shallow-water simulation.

/// A single cell in the height field.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct LiquidCell {
    pub height: f32,
    pub velocity_x: f32,
    pub velocity_z: f32,
}

/// 2-D liquid body (NxN height field).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LiquidBody {
    pub cells: Vec<LiquidCell>,
    pub grid_n: usize,
    pub cell_size: f32,
    pub gravity: f32,
    pub damping: f32,
}

/// Create a new `LiquidBody` of `n x n` cells.
#[allow(dead_code)]
pub fn new_liquid_body(n: usize, cell_size: f32) -> LiquidBody {
    LiquidBody {
        cells: (0..n * n).map(|_| LiquidCell::default()).collect(),
        grid_n: n,
        cell_size,
        gravity: 9.81,
        damping: 0.98,
    }
}

fn idx(body: &LiquidBody, x: usize, z: usize) -> usize {
    x * body.grid_n + z
}

/// Set height at cell (x, z).
#[allow(dead_code)]
pub fn lb_set_height(body: &mut LiquidBody, x: usize, z: usize, h: f32) {
    if x < body.grid_n && z < body.grid_n {
        let i = idx(body, x, z);
        body.cells[i].height = h.max(0.0);
    }
}

/// Get height at cell (x, z).
#[allow(dead_code)]
pub fn lb_get_height(body: &LiquidBody, x: usize, z: usize) -> f32 {
    if x < body.grid_n && z < body.grid_n {
        body.cells[idx(body, x, z)].height
    } else {
        0.0
    }
}

/// Average height across all cells.
#[allow(dead_code)]
pub fn lb_avg_height(body: &LiquidBody) -> f32 {
    if body.cells.is_empty() {
        return 0.0;
    }
    let sum: f32 = body.cells.iter().map(|c| c.height).sum();
    sum / body.cells.len() as f32
}

/// Simple explicit wave step (central differences).
#[allow(dead_code)]
pub fn lb_step(body: &mut LiquidBody, dt: f32) {
    let n = body.grid_n;
    let c = body.gravity * body.cell_size;
    let new_heights: Vec<f32> = (0..n * n)
        .map(|i| {
            let x = i / n;
            let z = i % n;
            let h = body.cells[i].height;
            let left = if x > 0 {
                body.cells[(x - 1) * n + z].height
            } else {
                h
            };
            let right = if x + 1 < n {
                body.cells[(x + 1) * n + z].height
            } else {
                h
            };
            let back = if z > 0 {
                body.cells[x * n + z - 1].height
            } else {
                h
            };
            let front = if z + 1 < n {
                body.cells[x * n + z + 1].height
            } else {
                h
            };
            let lap = left + right + back + front - 4.0 * h;
            let new_vx = (body.cells[i].velocity_x + c * lap * dt) * body.damping;
            h + new_vx * dt
        })
        .collect();
    for (i, &nh) in new_heights.iter().enumerate() {
        body.cells[i].height = nh.max(0.0);
    }
}

/// Total volume (sum of all heights * cell area).
#[allow(dead_code)]
pub fn lb_total_volume(body: &LiquidBody) -> f32 {
    let area = body.cell_size * body.cell_size;
    body.cells.iter().map(|c| c.height * area).sum()
}

/// Number of cells.
#[allow(dead_code)]
pub fn lb_cell_count(body: &LiquidBody) -> usize {
    body.cells.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_liquid_body() {
        let body = new_liquid_body(4, 1.0);
        assert_eq!(lb_cell_count(&body), 16);
    }

    #[test]
    fn test_set_height() {
        let mut body = new_liquid_body(4, 1.0);
        lb_set_height(&mut body, 1, 1, 2.5);
        assert!((lb_get_height(&body, 1, 1) - 2.5).abs() < 1e-5);
    }

    #[test]
    fn test_height_clamped_to_zero() {
        let mut body = new_liquid_body(4, 1.0);
        lb_set_height(&mut body, 0, 0, -1.0);
        assert!((lb_get_height(&body, 0, 0)).abs() < 1e-9);
    }

    #[test]
    fn test_avg_height_zero() {
        let body = new_liquid_body(4, 1.0);
        assert!((lb_avg_height(&body)).abs() < 1e-9);
    }

    #[test]
    fn test_total_volume() {
        let mut body = new_liquid_body(2, 1.0);
        lb_set_height(&mut body, 0, 0, 1.0);
        lb_set_height(&mut body, 0, 1, 1.0);
        lb_set_height(&mut body, 1, 0, 1.0);
        lb_set_height(&mut body, 1, 1, 1.0);
        assert!((lb_total_volume(&body) - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_step_does_not_crash() {
        let mut body = new_liquid_body(4, 0.5);
        lb_set_height(&mut body, 2, 2, 5.0);
        lb_step(&mut body, 0.016);
    }

    #[test]
    fn test_out_of_bounds_get() {
        let body = new_liquid_body(4, 1.0);
        assert!((lb_get_height(&body, 100, 100)).abs() < 1e-9);
    }

    #[test]
    fn test_avg_height_uniform() {
        let mut body = new_liquid_body(2, 1.0);
        for x in 0..2 {
            for z in 0..2 {
                lb_set_height(&mut body, x, z, 3.0);
            }
        }
        assert!((lb_avg_height(&body) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_cell_size_affects_volume() {
        let mut body = new_liquid_body(2, 2.0);
        for x in 0..2 {
            for z in 0..2 {
                lb_set_height(&mut body, x, z, 1.0);
            }
        }
        assert!((lb_total_volume(&body) - 16.0).abs() < 1e-4);
    }
}
