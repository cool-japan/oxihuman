// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Models bounce behavior using coefficient of restitution.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct BounceModel {
    restitution: f32,
    min_velocity: f32,
    friction: f32,
}

#[allow(dead_code)]
impl BounceModel {
    pub fn new(restitution: f32) -> Self {
        Self {
            restitution: restitution.clamp(0.0, 1.0),
            min_velocity: 0.01,
            friction: 0.3,
        }
    }

    pub fn perfectly_elastic() -> Self {
        Self::new(1.0)
    }

    pub fn perfectly_inelastic() -> Self {
        Self::new(0.0)
    }

    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction.clamp(0.0, 1.0);
        self
    }

    pub fn with_min_velocity(mut self, min_vel: f32) -> Self {
        self.min_velocity = min_vel;
        self
    }

    pub fn restitution(&self) -> f32 {
        self.restitution
    }

    pub fn compute_bounce_velocity(&self, incoming_speed: f32, normal: [f32; 3], velocity: [f32; 3]) -> [f32; 3] {
        let dot: f32 = velocity.iter().zip(normal.iter()).map(|(v, n)| v * n).sum();
        if dot >= 0.0 {
            return velocity;
        }
        let mut result = [0.0f32; 3];
        for i in 0..3 {
            let normal_component = dot * normal[i];
            let tangent_component = velocity[i] - normal_component;
            result[i] = tangent_component * (1.0 - self.friction) - normal_component * self.restitution;
        }
        let speed_sq: f32 = result.iter().map(|&v| v * v).sum();
        if speed_sq < self.min_velocity * self.min_velocity {
            return [0.0; 3];
        }
        let _ = incoming_speed;
        result
    }

    pub fn energy_retained(&self) -> f32 {
        self.restitution * self.restitution
    }

    pub fn energy_lost(&self) -> f32 {
        1.0 - self.energy_retained()
    }

    pub fn combine_restitution(a: f32, b: f32) -> f32 {
        (a * b).sqrt()
    }

    pub fn will_bounce(&self, speed: f32) -> bool {
        speed > self.min_velocity && self.restitution > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let b = BounceModel::new(0.5);
        assert!((b.restitution() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_clamp() {
        let b = BounceModel::new(1.5);
        assert!((b.restitution() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_perfectly_elastic() {
        let b = BounceModel::perfectly_elastic();
        assert!((b.energy_retained() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_perfectly_inelastic() {
        let b = BounceModel::perfectly_inelastic();
        assert!(b.energy_retained().abs() < 1e-6);
    }

    #[test]
    fn test_energy_conservation() {
        let b = BounceModel::new(0.8);
        let retained = b.energy_retained();
        let lost = b.energy_lost();
        assert!((retained + lost - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_bounce_velocity_normal_hit() {
        let b = BounceModel::new(1.0).with_friction(0.0);
        let vel = [0.0, -10.0, 0.0];
        let normal = [0.0, 1.0, 0.0];
        let result = b.compute_bounce_velocity(10.0, normal, vel);
        assert!(result[1] > 0.0);
    }

    #[test]
    fn test_no_bounce_when_moving_away() {
        let b = BounceModel::new(1.0);
        let vel = [0.0, 10.0, 0.0]; // moving away from ground
        let normal = [0.0, 1.0, 0.0];
        let result = b.compute_bounce_velocity(10.0, normal, vel);
        assert_eq!(result, vel);
    }

    #[test]
    fn test_combine_restitution() {
        let combined = BounceModel::combine_restitution(0.5, 0.5);
        assert!((combined - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_will_bounce() {
        let b = BounceModel::new(0.5).with_min_velocity(1.0);
        assert!(b.will_bounce(2.0));
        assert!(!b.will_bounce(0.5));
    }

    #[test]
    fn test_below_min_velocity_stops() {
        let b = BounceModel::new(0.01).with_min_velocity(1.0).with_friction(0.0);
        let vel = [0.0, -0.5, 0.0];
        let normal = [0.0, 1.0, 0.0];
        let result = b.compute_bounce_velocity(0.5, normal, vel);
        assert_eq!(result, [0.0, 0.0, 0.0]);
    }
}
