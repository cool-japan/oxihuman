// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Projectile body with drag and gravity.
#[allow(dead_code)]
pub struct ProjectileBody {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub mass: f32,
    pub drag_coeff: f32,
    pub radius: f32,
    pub active: bool,
}

#[allow(dead_code)]
impl ProjectileBody {
    pub fn new(mass: f32, radius: f32, drag_coeff: f32) -> Self {
        Self {
            pos: [0.0; 3],
            vel: [0.0; 3],
            mass,
            drag_coeff,
            radius,
            active: true,
        }
    }
    pub fn launch(&mut self, pos: [f32; 3], vel: [f32; 3]) {
        self.pos = pos;
        self.vel = vel;
        self.active = true;
    }
    pub fn step(&mut self, dt: f32, gravity: f32) {
        if !self.active {
            return;
        }
        let speed = (self.vel[0].powi(2) + self.vel[1].powi(2) + self.vel[2].powi(2)).sqrt();
        let drag_mag = self.drag_coeff * speed * speed;
        let mut acc = [0.0f32; 3];
        acc[1] -= gravity;
        if speed > 1e-8 {
            #[allow(clippy::needless_range_loop)]
            for i in 0..3 {
                acc[i] -= drag_mag * self.vel[i] / speed / self.mass.max(1e-10);
            }
        }
        #[allow(clippy::needless_range_loop)]
        for i in 0..3 {
            self.vel[i] += acc[i] * dt;
            self.pos[i] += self.vel[i] * dt;
        }
        if self.pos[1] < 0.0 {
            self.pos[1] = 0.0;
            self.active = false;
        }
    }
    pub fn kinetic_energy(&self) -> f32 {
        let v2: f32 = self.vel.iter().map(|v| v * v).sum();
        0.5 * self.mass * v2
    }
    pub fn speed(&self) -> f32 {
        self.vel.iter().map(|v| v * v).sum::<f32>().sqrt()
    }
    pub fn horizontal_range(&self) -> f32 {
        (self.pos[0].powi(2) + self.pos[2].powi(2)).sqrt()
    }
    pub fn angle_of_elevation(&self) -> f32 {
        self.vel[1].atan2(self.horizontal_speed()) * 180.0 / PI
    }
    pub fn horizontal_speed(&self) -> f32 {
        (self.vel[0].powi(2) + self.vel[2].powi(2)).sqrt()
    }
    /// Theoretical max range (vacuum, flat ground).
    pub fn max_range_vacuum(launch_speed: f32, angle_deg: f32, g: f32) -> f32 {
        let a = angle_deg * PI / 180.0;
        launch_speed * launch_speed * (2.0 * a).sin() / g.max(1e-10)
    }
    pub fn time_of_flight_vacuum(launch_speed: f32, angle_deg: f32, g: f32) -> f32 {
        let a = angle_deg * PI / 180.0;
        2.0 * launch_speed * a.sin() / g.max(1e-10)
    }
}

#[allow(dead_code)]
pub fn new_projectile_body(mass: f32, radius: f32, drag: f32) -> ProjectileBody {
    ProjectileBody::new(mass, radius, drag)
}
#[allow(dead_code)]
pub fn proj_launch(b: &mut ProjectileBody, pos: [f32; 3], vel: [f32; 3]) {
    b.launch(pos, vel);
}
#[allow(dead_code)]
pub fn proj_step(b: &mut ProjectileBody, dt: f32, gravity: f32) {
    b.step(dt, gravity);
}
#[allow(dead_code)]
pub fn proj_kinetic_energy(b: &ProjectileBody) -> f32 {
    b.kinetic_energy()
}
#[allow(dead_code)]
pub fn proj_speed(b: &ProjectileBody) -> f32 {
    b.speed()
}
#[allow(dead_code)]
pub fn proj_is_active(b: &ProjectileBody) -> bool {
    b.active
}
#[allow(dead_code)]
pub fn proj_horizontal_range(b: &ProjectileBody) -> f32 {
    b.horizontal_range()
}
#[allow(dead_code)]
pub fn proj_max_range_vacuum(speed: f32, angle_deg: f32, g: f32) -> f32 {
    ProjectileBody::max_range_vacuum(speed, angle_deg, g)
}
#[allow(dead_code)]
pub fn proj_time_of_flight(speed: f32, angle_deg: f32, g: f32) -> f32 {
    ProjectileBody::time_of_flight_vacuum(speed, angle_deg, g)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_launch_and_step() {
        let mut b = new_projectile_body(1.0, 0.05, 0.0);
        proj_launch(&mut b, [0.0; 3], [10.0, 10.0, 0.0]);
        proj_step(&mut b, 0.1, 9.81);
        assert!(b.pos[0] > 0.0);
    }
    #[test]
    fn test_falls_with_gravity() {
        let mut b = new_projectile_body(1.0, 0.05, 0.0);
        proj_launch(&mut b, [0.0, 10.0, 0.0], [0.0, 0.0, 0.0]);
        for _ in 0..100 {
            proj_step(&mut b, 0.05, 9.81);
        }
        assert!(!proj_is_active(&b));
    }
    #[test]
    fn test_kinetic_energy_on_launch() {
        let mut b = new_projectile_body(2.0, 0.05, 0.0);
        proj_launch(&mut b, [0.0; 3], [3.0, 4.0, 0.0]);
        let ke = proj_kinetic_energy(&b);
        assert!((ke - 25.0).abs() < 0.1);
    }
    #[test]
    fn test_speed() {
        let mut b = new_projectile_body(1.0, 0.05, 0.0);
        proj_launch(&mut b, [0.0; 3], [3.0, 0.0, 4.0]);
        assert!((proj_speed(&b) - 5.0).abs() < 1e-4);
    }
    #[test]
    fn test_max_range_45deg() {
        let r = proj_max_range_vacuum(10.0, 45.0, 9.81);
        assert!((r - 10.0 * 10.0 / 9.81).abs() < 0.05);
    }
    #[test]
    fn test_time_of_flight_positive() {
        let t = proj_time_of_flight(20.0, 30.0, 9.81);
        assert!(t > 0.0);
    }
    #[test]
    fn test_drag_reduces_speed() {
        let mut b_drag = new_projectile_body(1.0, 0.1, 0.5);
        let mut b_free = new_projectile_body(1.0, 0.1, 0.0);
        let v = [20.0, 10.0, 0.0];
        proj_launch(&mut b_drag, [0.0; 3], v);
        proj_launch(&mut b_free, [0.0; 3], v);
        for _ in 0..10 {
            proj_step(&mut b_drag, 0.05, 9.81);
            proj_step(&mut b_free, 0.05, 9.81);
        }
        assert!(proj_speed(&b_drag) < proj_speed(&b_free));
    }
    #[test]
    fn test_inactive_doesnt_move() {
        let mut b = new_projectile_body(1.0, 0.05, 0.0);
        proj_launch(&mut b, [0.0, 5.0, 0.0], [0.0, 0.0, 0.0]);
        for _ in 0..200 {
            proj_step(&mut b, 0.05, 9.81);
        }
        let pos_after_land = b.pos;
        proj_step(&mut b, 1.0, 9.81);
        assert_eq!(b.pos, pos_after_land);
    }
    #[test]
    fn test_horizontal_range() {
        let mut b = new_projectile_body(1.0, 0.05, 0.0);
        proj_launch(&mut b, [0.0; 3], [3.0, 0.0, 4.0]);
        proj_step(&mut b, 1.0, 0.0);
        let r = proj_horizontal_range(&b);
        assert!(r > 0.0);
    }
    #[test]
    fn test_lands_on_ground() {
        let mut b = new_projectile_body(1.0, 0.05, 0.0);
        proj_launch(&mut b, [0.0, 50.0, 0.0], [5.0, 5.0, 0.0]);
        for _ in 0..500 {
            proj_step(&mut b, 0.02, 9.81);
        }
        assert_eq!(b.pos[1], 0.0);
    }
}
