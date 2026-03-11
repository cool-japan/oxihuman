// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct RayMarchDebug {
    pub show_steps: bool,
    pub show_distance: bool,
    pub max_steps: u32,
}

pub fn new_ray_march_debug(max_steps: u32) -> RayMarchDebug {
    RayMarchDebug {
        show_steps: true,
        show_distance: false,
        max_steps,
    }
}

pub fn ray_march_step_color(steps: u32, max_steps: u32) -> [f32; 3] {
    let t = if max_steps == 0 {
        0.0
    } else {
        (steps as f32 / max_steps as f32).clamp(0.0, 1.0)
    };
    [t, 1.0 - t, 0.0]
}

pub fn ray_march_hit_color(hit: bool) -> [f32; 3] {
    if hit {
        [0.0, 1.0, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    }
}

pub fn ray_march_distance_color(dist: f32, max_dist: f32) -> [f32; 3] {
    let t = if max_dist < 1e-9 {
        0.0
    } else {
        (dist / max_dist).clamp(0.0, 1.0)
    };
    [t, t, 1.0 - t]
}

pub fn ray_march_efficiency(steps: u32, max_steps: u32) -> f32 {
    if max_steps == 0 {
        0.0
    } else {
        1.0 - steps as f32 / max_steps as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ray_march_debug() {
        /* max_steps stored correctly */
        let r = new_ray_march_debug(64);
        assert_eq!(r.max_steps, 64);
    }

    #[test]
    fn test_ray_march_step_color_full() {
        /* max steps -> red */
        let c = ray_march_step_color(64, 64);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_ray_march_hit_color() {
        /* hit -> green, miss -> red */
        let h = ray_march_hit_color(true);
        let m = ray_march_hit_color(false);
        assert!((h[1] - 1.0).abs() < 1e-6);
        assert!((m[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_ray_march_distance_color() {
        /* dist=max -> [1,1,0] */
        let c = ray_march_distance_color(10.0, 10.0);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_ray_march_efficiency() {
        /* half steps -> 0.5 efficiency */
        let e = ray_march_efficiency(32, 64);
        assert!((e - 0.5).abs() < 1e-6);
    }
}
