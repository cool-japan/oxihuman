// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! 2D ballistic projectile simulation stub.

#[derive(Debug, Clone)]
pub struct Projectile2d {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub gravity: f32,
    pub drag: f32,
    pub active: bool,
}

impl Projectile2d {
    pub fn new(pos: [f32; 2], vel: [f32; 2], gravity: f32) -> Self {
        Projectile2d {
            position: pos,
            velocity: vel,
            gravity,
            drag: 0.0,
            active: true,
        }
    }

    pub fn with_drag(mut self, drag: f32) -> Self {
        self.drag = drag;
        self
    }

    pub fn step(&mut self, dt: f32) {
        if !self.active {
            return;
        }
        let speed = (self.velocity[0].powi(2) + self.velocity[1].powi(2)).sqrt();
        if self.drag > 0.0 && speed > f32::EPSILON {
            let drag_force = self.drag * speed * speed;
            self.velocity[0] -= drag_force * (self.velocity[0] / speed) * dt;
            self.velocity[1] -= drag_force * (self.velocity[1] / speed) * dt;
        }
        self.velocity[1] -= self.gravity * dt;
        self.position[0] += self.velocity[0] * dt;
        self.position[1] += self.velocity[1] * dt;
    }

    pub fn is_below_ground(&self, ground_y: f32) -> bool {
        self.position[1] < ground_y
    }

    pub fn speed(&self) -> f32 {
        (self.velocity[0].powi(2) + self.velocity[1].powi(2)).sqrt()
    }
}

/// Launch angle that maximizes range (45 degrees in vacuum).
pub fn optimal_launch_angle() -> f32 {
    std::f32::consts::FRAC_PI_4
}

/// Compute range of a projectile launched at angle and speed.
pub fn range(speed: f32, angle_rad: f32, gravity: f32) -> f32 {
    if gravity < f32::EPSILON {
        return f32::INFINITY;
    }
    speed * speed * (2.0 * angle_rad).sin() / gravity
}

/// Time of flight for a projectile launched at angle and speed.
pub fn time_of_flight(speed: f32, angle_rad: f32, gravity: f32) -> f32 {
    if gravity < f32::EPSILON {
        return f32::INFINITY;
    }
    2.0 * speed * angle_rad.sin() / gravity
}

/// Maximum height reached by a projectile.
pub fn max_height(speed: f32, angle_rad: f32, gravity: f32) -> f32 {
    let vy = speed * angle_rad.sin();
    vy * vy / (2.0 * gravity)
}

pub fn simulate_trajectory(
    pos: [f32; 2],
    vel: [f32; 2],
    g: f32,
    dt: f32,
    steps: usize,
) -> Vec<[f32; 2]> {
    let mut proj = Projectile2d::new(pos, vel, g);
    let mut positions = vec![pos];
    for _ in 0..steps {
        proj.step(dt);
        positions.push(proj.position);
    }
    positions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_horizontal_motion() {
        let mut p = Projectile2d::new([0.0, 0.0], [10.0, 0.0], 0.0);
        p.step(1.0);
        assert!((p.position[0] - 10.0).abs() < 1e-4 /* x = vx*t */,);
    }

    #[test]
    fn test_gravity_applies() {
        let mut p = Projectile2d::new([0.0, 0.0], [0.0, 0.0], 9.81);
        p.step(1.0);
        assert!(p.position[1] < 0.0 /* gravity pulls down */,);
    }

    #[test]
    fn test_optimal_angle_is_45_deg() {
        use std::f32::consts::FRAC_PI_4;
        assert!((optimal_launch_angle() - FRAC_PI_4).abs() < 1e-5, /* 45 degrees */);
    }

    #[test]
    fn test_range_formula() {
        use std::f32::consts::FRAC_PI_4;
        let r = range(10.0, FRAC_PI_4, 9.81);
        assert!(r > 0.0 /* positive range */,);
    }

    #[test]
    fn test_time_of_flight() {
        use std::f32::consts::FRAC_PI_2;
        let t = time_of_flight(10.0, FRAC_PI_2, 9.81);
        assert!((t - 2.0 * 10.0 / 9.81).abs() < 0.01 /* t = 2vy/g */,);
    }

    #[test]
    fn test_max_height() {
        use std::f32::consts::FRAC_PI_2;
        let h = max_height(10.0, FRAC_PI_2, 9.81);
        assert!(h > 0.0 /* positive height */,);
    }

    #[test]
    fn test_is_below_ground() {
        let mut p = Projectile2d::new([0.0, -1.0], [0.0, 0.0], 0.0);
        assert!(p.is_below_ground(0.0) /* starts below ground */,);
        p.position[1] = 1.0;
        assert!(!p.is_below_ground(0.0) /* now above ground */,);
    }

    #[test]
    fn test_trajectory_length() {
        let traj = simulate_trajectory([0.0, 0.0], [5.0, 5.0], 9.81, 0.1, 10);
        assert_eq!(traj.len(), 11 /* initial + 10 steps */,);
    }

    #[test]
    fn test_inactive_projectile() {
        let mut p = Projectile2d::new([0.0, 0.0], [10.0, 10.0], 9.81);
        p.active = false;
        p.step(1.0);
        assert!((p.position[0] - 0.0).abs() < 1e-6, /* inactive: no movement */);
    }
}
