// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Top-down 2D car physics stub.

use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct Car2d {
    pub position: [f32; 2],
    pub heading: f32,
    pub speed: f32,
    pub steer_angle: f32,
    pub wheelbase: f32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub drag: f32,
}

impl Car2d {
    pub fn new(wheelbase: f32) -> Self {
        Car2d {
            position: [0.0; 2],
            heading: 0.0,
            speed: 0.0,
            steer_angle: 0.0,
            wheelbase,
            max_speed: 20.0,
            acceleration: 5.0,
            drag: 0.5,
        }
    }

    pub fn steer(&mut self, delta: f32) {
        let max_steer = PI / 4.0;
        self.steer_angle = (self.steer_angle + delta).clamp(-max_steer, max_steer);
    }

    pub fn throttle(&mut self, input: f32, dt: f32) {
        self.speed += input * self.acceleration * dt;
        self.speed = self.speed.clamp(-self.max_speed, self.max_speed);
    }

    pub fn apply_drag(&mut self, dt: f32) {
        self.speed -= self.speed * self.drag * dt;
    }

    pub fn integrate(&mut self, dt: f32) {
        if self.wheelbase > f32::EPSILON && self.steer_angle.abs() > f32::EPSILON {
            let turn_radius = self.wheelbase / self.steer_angle.tan();
            let angular_vel = self.speed / turn_radius;
            self.heading += angular_vel * dt;
        }
        self.position[0] += self.speed * self.heading.cos() * dt;
        self.position[1] += self.speed * self.heading.sin() * dt;
    }

    pub fn step(&mut self, throttle_input: f32, steer_input: f32, dt: f32) {
        self.throttle(throttle_input, dt);
        self.steer(steer_input);
        self.apply_drag(dt);
        self.integrate(dt);
    }

    pub fn velocity(&self) -> [f32; 2] {
        [
            self.speed * self.heading.cos(),
            self.speed * self.heading.sin(),
        ]
    }
}

pub fn car_kinetic_energy(car: &Car2d, mass: f32) -> f32 {
    0.5 * mass * car.speed * car.speed
}

pub fn car_heading_deg(car: &Car2d) -> f32 {
    car.heading.to_degrees().rem_euclid(360.0)
}

pub fn car_distance_from_origin(car: &Car2d) -> f32 {
    (car.position[0].powi(2) + car.position[1].powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let car = Car2d::new(2.5);
        assert!((car.wheelbase - 2.5).abs() < 1e-5);
    }

    #[test]
    fn test_throttle_increases_speed() {
        let mut car = Car2d::new(2.5);
        car.throttle(1.0, 1.0);
        assert!(car.speed > 0.0 /* throttle increases speed */,);
    }

    #[test]
    fn test_max_speed_clamp() {
        let mut car = Car2d::new(2.5);
        for _ in 0..1000 {
            car.throttle(1.0, 0.1);
        }
        assert!(car.speed <= car.max_speed + 1e-3, /* speed clamped at max */);
    }

    #[test]
    fn test_drag_reduces_speed() {
        let mut car = Car2d::new(2.5);
        car.speed = 10.0;
        car.apply_drag(1.0);
        assert!(car.speed < 10.0 /* drag reduces speed */,);
    }

    #[test]
    fn test_integrate_moves_forward() {
        let mut car = Car2d::new(2.5);
        car.speed = 5.0;
        car.heading = 0.0;
        car.integrate(1.0);
        assert!(car.position[0] > 0.0 /* car moves in x direction */,);
    }

    #[test]
    fn test_steer_clamp() {
        let mut car = Car2d::new(2.5);
        car.steer(100.0);
        assert!(car.steer_angle <= PI / 4.0 + 1e-5, /* clamped at max steer */);
    }

    #[test]
    fn test_velocity_at_zero_speed() {
        let car = Car2d::new(2.5);
        let v = car.velocity();
        assert!(v[0].abs() < 1e-6 && v[1].abs() < 1e-6, /* zero speed = zero velocity */);
    }

    #[test]
    fn test_kinetic_energy() {
        let mut car = Car2d::new(2.5);
        car.speed = 10.0;
        let ke = car_kinetic_energy(&car, 1000.0);
        assert!((ke - 50000.0).abs() < 1.0 /* 0.5*1000*100 = 50000 */,);
    }

    #[test]
    fn test_heading_deg() {
        let mut car = Car2d::new(2.5);
        car.heading = PI;
        let deg = car_heading_deg(&car);
        assert!((deg - 180.0).abs() < 0.01 /* PI rad = 180 deg */,);
    }
}
