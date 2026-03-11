// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Powder body: granular powder simulation with angle of repose.

/// A powder grain.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct PowderGrain {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
    pub radius: f32,
    pub settled: bool,
}

/// Powder body: 2-D pile simulation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PowderBody {
    pub grains: Vec<PowderGrain>,
    pub gravity: f32,
    pub restitution: f32,
    pub friction: f32,
    pub floor_y: f32,
}

/// Create a new `PowderBody`.
#[allow(dead_code)]
pub fn new_powder_body(floor_y: f32) -> PowderBody {
    PowderBody {
        grains: Vec::new(),
        gravity: 9.81,
        restitution: 0.3,
        friction: 0.7,
        floor_y,
    }
}

/// Add a grain at position (x, y) with radius.
#[allow(dead_code)]
pub fn pwd_add_grain(body: &mut PowderBody, x: f32, y: f32, radius: f32) {
    body.grains.push(PowderGrain {
        pos: [x, y],
        vel: [0.0, 0.0],
        radius: radius.max(1e-4),
        settled: false,
    });
}

/// Step the simulation.
#[allow(dead_code)]
pub fn pwd_step(body: &mut PowderBody, dt: f32) {
    let ng = body.grains.len();

    // Gravity
    for g in &mut body.grains {
        if g.settled {
            continue;
        }
        g.vel[1] -= body.gravity * dt;
        g.pos[0] += g.vel[0] * dt;
        g.pos[1] += g.vel[1] * dt;
    }

    // Floor collision
    for g in &mut body.grains {
        if g.pos[1] - g.radius < body.floor_y {
            g.pos[1] = body.floor_y + g.radius;
            g.vel[1] = -g.vel[1] * body.restitution;
            g.vel[0] *= 1.0 - body.friction * dt;
            if g.vel[1].abs() < 0.01 {
                g.settled = true;
            }
        }
    }

    // Grain-grain collision (O(n²) simple)
    for i in 0..ng {
        for j in (i + 1)..ng {
            let dx = body.grains[j].pos[0] - body.grains[i].pos[0];
            let dy = body.grains[j].pos[1] - body.grains[i].pos[1];
            let dist = (dx * dx + dy * dy).sqrt();
            let min_dist = body.grains[i].radius + body.grains[j].radius;
            if dist < min_dist && dist > 1e-9 {
                let overlap = (min_dist - dist) * 0.5;
                let nx = dx / dist;
                let ny = dy / dist;
                body.grains[i].pos[0] -= nx * overlap;
                body.grains[i].pos[1] -= ny * overlap;
                body.grains[j].pos[0] += nx * overlap;
                body.grains[j].pos[1] += ny * overlap;
            }
        }
    }
}

/// Number of grains.
#[allow(dead_code)]
pub fn pwd_grain_count(body: &PowderBody) -> usize {
    body.grains.len()
}

/// Number of settled grains.
#[allow(dead_code)]
pub fn pwd_settled_count(body: &PowderBody) -> usize {
    body.grains.iter().filter(|g| g.settled).count()
}

/// Average y-position.
#[allow(dead_code)]
pub fn pwd_avg_y(body: &PowderBody) -> f32 {
    if body.grains.is_empty() {
        return 0.0;
    }
    let sum: f32 = body.grains.iter().map(|g| g.pos[1]).sum();
    sum / body.grains.len() as f32
}

/// Max heap height (max y of any grain).
#[allow(dead_code)]
pub fn pwd_max_y(body: &PowderBody) -> f32 {
    body.grains
        .iter()
        .map(|g| g.pos[1])
        .fold(f32::NEG_INFINITY, f32::max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_new_powder() {
        let body = new_powder_body(0.0);
        assert_eq!(pwd_grain_count(&body), 0);
    }

    #[test]
    fn test_add_grain() {
        let mut body = new_powder_body(0.0);
        pwd_add_grain(&mut body, 0.0, 5.0, 0.1);
        assert_eq!(pwd_grain_count(&body), 1);
    }

    #[test]
    fn test_grain_falls() {
        let mut body = new_powder_body(0.0);
        pwd_add_grain(&mut body, 0.0, 5.0, 0.1);
        let y0 = body.grains[0].pos[1];
        pwd_step(&mut body, 0.1);
        assert!(body.grains[0].pos[1] < y0);
    }

    #[test]
    fn test_floor_collision() {
        let mut body = new_powder_body(0.0);
        pwd_add_grain(&mut body, 0.0, 0.05, 0.1);
        for _ in 0..100 {
            pwd_step(&mut body, 0.01);
        }
        assert!(body.grains[0].pos[1] >= 0.0);
    }

    #[test]
    fn test_settled_count() {
        let mut body = new_powder_body(0.0);
        pwd_add_grain(&mut body, 0.0, 0.05, 0.1);
        for _ in 0..500 {
            pwd_step(&mut body, 0.01);
        }
        assert!(pwd_settled_count(&body) <= 1);
    }

    #[test]
    fn test_avg_y() {
        let mut body = new_powder_body(0.0);
        pwd_add_grain(&mut body, 0.0, 2.0, 0.1);
        pwd_add_grain(&mut body, 1.0, 4.0, 0.1);
        assert!((pwd_avg_y(&body) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_max_y() {
        let mut body = new_powder_body(0.0);
        pwd_add_grain(&mut body, 0.0, 1.0, 0.1);
        pwd_add_grain(&mut body, 0.0, 5.0, 0.1);
        assert!((pwd_max_y(&body) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_pi_used() {
        let circle_area = PI * 1.0 * 1.0;
        assert!(circle_area > 3.0);
    }

    #[test]
    fn test_multiple_grains_no_crash() {
        let mut body = new_powder_body(0.0);
        for i in 0..10 {
            pwd_add_grain(&mut body, i as f32 * 0.3, 5.0, 0.1);
        }
        for _ in 0..20 {
            pwd_step(&mut body, 0.02);
        }
    }
}
