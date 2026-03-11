// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Simple kinematic character controller.
#[allow(dead_code)]
pub struct KinematicController {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub max_speed: f32,
    pub on_ground: bool,
    pub jump_speed: f32,
}

#[allow(dead_code)]
impl KinematicController {
    pub fn new(max_speed: f32, jump_speed: f32) -> Self {
        Self {
            position: [0.0; 3],
            velocity: [0.0; 3],
            max_speed,
            on_ground: false,
            jump_speed,
        }
    }
    pub fn move_horizontal(&mut self, dx: f32, dz: f32) {
        let spd = (dx * dx + dz * dz).sqrt();
        if spd > 0.0 {
            let scale = (self.max_speed / spd).min(1.0);
            self.velocity[0] = dx * scale;
            self.velocity[2] = dz * scale;
        } else {
            self.velocity[0] = 0.0;
            self.velocity[2] = 0.0;
        }
    }
    pub fn jump(&mut self) {
        if self.on_ground {
            self.velocity[1] = self.jump_speed;
            self.on_ground = false;
        }
    }
    pub fn integrate(&mut self, dt: f32, gravity: f32) {
        self.velocity[1] -= gravity * dt;
        for i in 0..3 {
            self.position[i] += self.velocity[i] * dt;
        }
        if self.position[1] <= 0.0 {
            self.position[1] = 0.0;
            self.velocity[1] = 0.0f32.max(self.velocity[1]);
            self.on_ground = true;
        }
    }
    pub fn speed_horizontal(&self) -> f32 {
        (self.velocity[0].powi(2) + self.velocity[2].powi(2)).sqrt()
    }
    pub fn heading_rad(&self) -> f32 {
        self.velocity[2].atan2(self.velocity[0])
    }
    pub fn heading_deg(&self) -> f32 {
        self.heading_rad() * 180.0 / PI
    }
    pub fn set_position(&mut self, p: [f32; 3]) {
        self.position = p;
    }
    pub fn kinetic_energy(&self, mass: f32) -> f32 {
        let v2 = self.velocity.iter().map(|v| v * v).sum::<f32>();
        0.5 * mass * v2
    }
    pub fn stop(&mut self) {
        self.velocity = [0.0; 3];
    }
}

#[allow(dead_code)]
pub fn new_kinematic_controller(max_speed: f32, jump_speed: f32) -> KinematicController {
    KinematicController::new(max_speed, jump_speed)
}
#[allow(dead_code)]
pub fn kc_move(c: &mut KinematicController, dx: f32, dz: f32) {
    c.move_horizontal(dx, dz);
}
#[allow(dead_code)]
pub fn kc_jump(c: &mut KinematicController) {
    c.jump();
}
#[allow(dead_code)]
pub fn kc_integrate(c: &mut KinematicController, dt: f32, gravity: f32) {
    c.integrate(dt, gravity);
}
#[allow(dead_code)]
pub fn kc_speed(c: &KinematicController) -> f32 {
    c.speed_horizontal()
}
#[allow(dead_code)]
pub fn kc_heading_deg(c: &KinematicController) -> f32 {
    c.heading_deg()
}
#[allow(dead_code)]
pub fn kc_kinetic_energy(c: &KinematicController, mass: f32) -> f32 {
    c.kinetic_energy(mass)
}
#[allow(dead_code)]
pub fn kc_stop(c: &mut KinematicController) {
    c.stop();
}
#[allow(dead_code)]
pub fn kc_on_ground(c: &KinematicController) -> bool {
    c.on_ground
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_move_clamps_to_max_speed() {
        let mut c = new_kinematic_controller(2.0, 5.0);
        kc_move(&mut c, 100.0, 0.0);
        let spd = kc_speed(&c);
        assert!((spd - 2.0).abs() < 1e-5);
    }
    #[test]
    fn test_integrate_falls() {
        let mut c = new_kinematic_controller(5.0, 5.0);
        c.position = [0.0, 10.0, 0.0];
        kc_integrate(&mut c, 1.0, 9.81);
        assert!(c.position[1] < 10.0);
    }
    #[test]
    fn test_lands_on_ground() {
        let mut c = new_kinematic_controller(5.0, 5.0);
        c.position = [0.0, 0.5, 0.0];
        for _ in 0..20 {
            kc_integrate(&mut c, 0.1, 9.81);
        }
        assert_eq!(c.position[1], 0.0);
        assert!(kc_on_ground(&c));
    }
    #[test]
    fn test_jump() {
        let mut c = new_kinematic_controller(5.0, 8.0);
        c.on_ground = true;
        kc_jump(&mut c);
        assert!(c.velocity[1] > 0.0);
        assert!(!kc_on_ground(&c));
    }
    #[test]
    fn test_stop() {
        let mut c = new_kinematic_controller(5.0, 5.0);
        kc_move(&mut c, 1.0, 1.0);
        kc_stop(&mut c);
        assert_eq!(kc_speed(&c), 0.0);
    }
    #[test]
    fn test_kinetic_energy_zero_when_still() {
        let c = new_kinematic_controller(5.0, 5.0);
        assert_eq!(kc_kinetic_energy(&c, 70.0), 0.0);
    }
    #[test]
    fn test_heading() {
        let mut c = new_kinematic_controller(5.0, 5.0);
        c.velocity = [1.0, 0.0, 0.0];
        let h = kc_heading_deg(&c);
        assert!(
            (0.0..=1.0).contains(&h.abs()) || (359.0..=360.0).contains(&h.abs()) || h.abs() < 1.0
        );
        let _ = h; // just verify no panic
    }
    #[test]
    fn test_set_position() {
        let mut c = new_kinematic_controller(5.0, 5.0);
        c.set_position([3.0, 4.0, 5.0]);
        assert_eq!(c.position, [3.0, 4.0, 5.0]);
    }
    #[test]
    fn test_horizontal_move_zero_input() {
        let mut c = new_kinematic_controller(5.0, 5.0);
        kc_move(&mut c, 0.0, 0.0);
        assert_eq!(c.velocity[0], 0.0);
        assert_eq!(c.velocity[2], 0.0);
    }
    #[test]
    fn test_kinetic_energy_positive_when_moving() {
        let mut c = new_kinematic_controller(5.0, 5.0);
        kc_move(&mut c, 1.0, 0.0);
        let ke = kc_kinetic_energy(&c, 70.0);
        assert!(ke > 0.0);
    }
}
