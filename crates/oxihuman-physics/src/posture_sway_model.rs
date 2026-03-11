// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Quiet standing postural sway model (spring-damper with noise).

pub struct PostureSway {
    pub cop_x: f32,
    pub cop_y: f32,
    pub vx: f32,
    pub vy: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub noise_level: f32,
}

pub fn new_posture_sway() -> PostureSway {
    PostureSway {
        cop_x: 0.0,
        cop_y: 0.0,
        vx: 0.0,
        vy: 0.0,
        stiffness: 100.0,
        damping: 10.0,
        noise_level: 0.1,
    }
}

pub fn sway_step(s: &mut PostureSway, dt: f32, noise: f32) {
    /* spring-damper + stochastic noise input */
    let ax = -s.stiffness * s.cop_x - s.damping * s.vx + s.noise_level * (noise - 0.5);
    let ay = -s.stiffness * s.cop_y - s.damping * s.vy + s.noise_level * (noise - 0.5) * 0.7;
    s.vx += ax * dt;
    s.vy += ay * dt;
    s.cop_x += s.vx * dt;
    s.cop_y += s.vy * dt;
}

pub fn sway_cop_position(s: &PostureSway) -> [f32; 2] {
    [s.cop_x, s.cop_y]
}

pub fn sway_path_length(positions: &[[f32; 2]]) -> f32 {
    if positions.len() < 2 {
        return 0.0;
    }
    positions
        .windows(2)
        .map(|w| {
            let dx = w[1][0] - w[0][0];
            let dy = w[1][1] - w[0][1];
            (dx * dx + dy * dy).sqrt()
        })
        .sum()
}

pub fn sway_mean_displacement(positions: &[[f32; 2]]) -> f32 {
    if positions.is_empty() {
        return 0.0;
    }
    let n = positions.len() as f32;
    let sum: f32 = positions
        .iter()
        .map(|p| (p[0] * p[0] + p[1] * p[1]).sqrt())
        .sum();
    sum / n
}

pub fn sway_rms(positions: &[[f32; 2]]) -> f32 {
    if positions.is_empty() {
        return 0.0;
    }
    let n = positions.len() as f32;
    let sum_sq: f32 = positions.iter().map(|p| p[0] * p[0] + p[1] * p[1]).sum();
    (sum_sq / n).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_posture_sway() {
        /* new sway starts at origin */
        let s = new_posture_sway();
        assert_eq!(s.cop_x, 0.0);
        assert_eq!(s.cop_y, 0.0);
    }

    #[test]
    fn test_sway_step_moves() {
        /* noise drives cop away from origin */
        let mut s = new_posture_sway();
        sway_step(&mut s, 0.01, 1.0);
        let pos = sway_cop_position(&s);
        assert!(pos[0] != 0.0 || pos[1] != 0.0);
    }

    #[test]
    fn test_sway_cop_position() {
        /* cop position matches internal state */
        let s = new_posture_sway();
        let pos = sway_cop_position(&s);
        assert_eq!(pos, [0.0, 0.0]);
    }

    #[test]
    fn test_sway_path_length_zero() {
        /* single position has zero path length */
        let positions = vec![[0.0f32, 0.0]];
        assert_eq!(sway_path_length(&positions), 0.0);
    }

    #[test]
    fn test_sway_path_length_nonzero() {
        /* path length is sum of segment lengths */
        let positions = vec![[0.0f32, 0.0], [3.0, 4.0], [3.0, 4.0]];
        assert!((sway_path_length(&positions) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_sway_mean_displacement() {
        /* mean displacement of fixed point */
        let positions = vec![[3.0f32, 4.0], [3.0, 4.0]];
        let md = sway_mean_displacement(&positions);
        assert!((md - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_sway_rms() {
        /* rms of fixed point equals its distance from origin */
        let positions = vec![[3.0f32, 4.0], [3.0, 4.0]];
        let rms = sway_rms(&positions);
        assert!((rms - 5.0).abs() < 1e-4);
    }
}
