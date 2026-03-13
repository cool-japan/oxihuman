// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Simulates a conveyor belt surface that applies velocity to contacting bodies.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConveyorBelt {
    direction: [f32; 3],
    speed: f32,
    friction: f32,
    width: f32,
    length: f32,
    active: bool,
}

#[allow(dead_code)]
impl ConveyorBelt {
    pub fn new(direction: [f32; 3], speed: f32) -> Self {
        let len = (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2]).sqrt();
        let norm = if len > 1e-9 {
            [direction[0] / len, direction[1] / len, direction[2] / len]
        } else {
            [1.0, 0.0, 0.0]
        };
        Self {
            direction: norm,
            speed,
            friction: 0.8,
            width: 1.0,
            length: 5.0,
            active: true,
        }
    }

    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction.clamp(0.0, 1.0);
        self
    }

    pub fn with_dimensions(mut self, width: f32, length: f32) -> Self {
        self.width = width;
        self.length = length;
        self
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn belt_velocity(&self) -> [f32; 3] {
        if !self.active {
            return [0.0; 3];
        }
        [
            self.direction[0] * self.speed,
            self.direction[1] * self.speed,
            self.direction[2] * self.speed,
        ]
    }

    pub fn apply_to_body(&self, body_velocity: [f32; 3], dt: f32) -> [f32; 3] {
        if !self.active {
            return body_velocity;
        }
        let belt_vel = self.belt_velocity();
        let mut result = body_velocity;
        for i in 0..3 {
            let diff = belt_vel[i] - body_velocity[i];
            result[i] += diff * self.friction * dt;
        }
        result
    }

    pub fn is_on_belt(&self, position: [f32; 3], belt_center: [f32; 3]) -> bool {
        let dx = (position[0] - belt_center[0]).abs();
        let dz = (position[2] - belt_center[2]).abs();
        dx <= self.width * 0.5 && dz <= self.length * 0.5
    }

    pub fn area(&self) -> f32 {
        self.width * self.length
    }

    pub fn reverse(&mut self) {
        self.speed = -self.speed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let cb = ConveyorBelt::new([1.0, 0.0, 0.0], 5.0);
        assert!((cb.speed() - 5.0).abs() < 1e-9);
        assert!(cb.is_active());
    }

    #[test]
    fn test_belt_velocity() {
        let cb = ConveyorBelt::new([1.0, 0.0, 0.0], 3.0);
        let vel = cb.belt_velocity();
        assert!((vel[0] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_inactive_belt() {
        let mut cb = ConveyorBelt::new([1.0, 0.0, 0.0], 5.0);
        cb.set_active(false);
        let vel = cb.belt_velocity();
        assert_eq!(vel, [0.0; 3]);
    }

    #[test]
    fn test_apply_to_stationary_body() {
        let cb = ConveyorBelt::new([1.0, 0.0, 0.0], 10.0).with_friction(1.0);
        let result = cb.apply_to_body([0.0; 3], 1.0);
        assert!(result[0] > 0.0);
    }

    #[test]
    fn test_body_at_belt_speed_no_change() {
        let cb = ConveyorBelt::new([1.0, 0.0, 0.0], 5.0);
        let result = cb.apply_to_body([5.0, 0.0, 0.0], 1.0);
        assert!((result[0] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_is_on_belt() {
        let cb = ConveyorBelt::new([1.0, 0.0, 0.0], 1.0)
            .with_dimensions(2.0, 4.0);
        assert!(cb.is_on_belt([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]));
        assert!(!cb.is_on_belt([5.0, 0.0, 0.0], [0.0, 0.0, 0.0]));
    }

    #[test]
    fn test_area() {
        let cb = ConveyorBelt::new([1.0, 0.0, 0.0], 1.0)
            .with_dimensions(3.0, 5.0);
        assert!((cb.area() - 15.0).abs() < 1e-6);
    }

    #[test]
    fn test_reverse() {
        let mut cb = ConveyorBelt::new([1.0, 0.0, 0.0], 5.0);
        cb.reverse();
        assert!((cb.speed() - (-5.0)).abs() < 1e-9);
    }

    #[test]
    fn test_friction() {
        let cb_low = ConveyorBelt::new([1.0, 0.0, 0.0], 10.0).with_friction(0.1);
        let cb_high = ConveyorBelt::new([1.0, 0.0, 0.0], 10.0).with_friction(1.0);
        let r_low = cb_low.apply_to_body([0.0; 3], 1.0);
        let r_high = cb_high.apply_to_body([0.0; 3], 1.0);
        assert!(r_high[0] > r_low[0]);
    }

    #[test]
    fn test_set_speed() {
        let mut cb = ConveyorBelt::new([1.0, 0.0, 0.0], 1.0);
        cb.set_speed(20.0);
        assert!((cb.speed() - 20.0).abs() < 1e-9);
    }
}
