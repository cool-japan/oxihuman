// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Various numerical integration methods for position/velocity update.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntegratorKind {
    Euler,
    SemiImplicit,
    Verlet,
    RK2,
}

#[allow(dead_code)]
pub struct PositionIntegrator {
    pub kind: IntegratorKind,
    pub pos: f32,
    pub vel: f32,
    pub prev_pos: f32,
    pub steps: u64,
}

#[allow(dead_code)]
impl PositionIntegrator {
    pub fn new(kind: IntegratorKind, pos0: f32, vel0: f32) -> Self {
        Self {
            kind,
            pos: pos0,
            vel: vel0,
            prev_pos: pos0,
            steps: 0,
        }
    }
    pub fn step(&mut self, acc: f32, dt: f32) {
        match self.kind {
            IntegratorKind::Euler => {
                let new_pos = self.pos + self.vel * dt;
                self.prev_pos = self.pos;
                self.vel += acc * dt;
                self.pos = new_pos;
            }
            IntegratorKind::SemiImplicit => {
                self.vel += acc * dt;
                self.prev_pos = self.pos;
                self.pos += self.vel * dt;
            }
            IntegratorKind::Verlet => {
                let new_pos = 2.0 * self.pos - self.prev_pos + acc * dt * dt;
                self.vel = (new_pos - self.prev_pos) / (2.0 * dt);
                self.prev_pos = self.pos;
                self.pos = new_pos;
            }
            IntegratorKind::RK2 => {
                let v1 = self.vel;
                let p1 = self.pos;
                let v_mid = v1 + acc * dt * 0.5;
                let p_mid = p1 + v1 * dt * 0.5;
                self.prev_pos = self.pos;
                self.vel += acc * dt;
                self.pos = p_mid + v_mid * dt * 0.5;
                let _ = p_mid;
            }
        }
        self.steps += 1;
    }
    pub fn kinetic_energy(&self, mass: f32) -> f32 {
        0.5 * mass * self.vel * self.vel
    }
    pub fn displacement(&self) -> f32 {
        self.pos - self.prev_pos
    }
    pub fn speed(&self) -> f32 {
        self.vel.abs()
    }
    pub fn reset(&mut self, pos0: f32, vel0: f32) {
        self.pos = pos0;
        self.vel = vel0;
        self.prev_pos = pos0;
        self.steps = 0;
    }
}

#[allow(dead_code)]
pub fn new_position_integrator(kind: IntegratorKind, pos0: f32, vel0: f32) -> PositionIntegrator {
    PositionIntegrator::new(kind, pos0, vel0)
}
#[allow(dead_code)]
pub fn pi_step(i: &mut PositionIntegrator, acc: f32, dt: f32) {
    i.step(acc, dt);
}
#[allow(dead_code)]
pub fn pi_kinetic_energy(i: &PositionIntegrator, mass: f32) -> f32 {
    i.kinetic_energy(mass)
}
#[allow(dead_code)]
pub fn pi_displacement(i: &PositionIntegrator) -> f32 {
    i.displacement()
}
#[allow(dead_code)]
pub fn pi_speed(i: &PositionIntegrator) -> f32 {
    i.speed()
}
#[allow(dead_code)]
pub fn pi_steps(i: &PositionIntegrator) -> u64 {
    i.steps
}
#[allow(dead_code)]
pub fn pi_reset(i: &mut PositionIntegrator, pos: f32, vel: f32) {
    i.reset(pos, vel);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_euler_free_fall() {
        let mut i = new_position_integrator(IntegratorKind::Euler, 0.0, 0.0);
        pi_step(&mut i, -9.81, 0.1);
        assert!(i.vel < 0.0);
    }
    #[test]
    fn test_semi_implicit_stability() {
        let mut i = new_position_integrator(IntegratorKind::SemiImplicit, 1.0, 0.0);
        for _ in 0..100 {
            let pos = i.pos;
            pi_step(&mut i, -pos, 0.05);
        }
        assert!(i.pos.abs() < 10.0);
    }
    #[test]
    fn test_verlet_step() {
        // Verlet uses prev_pos; apply a force to get motion
        let mut i = new_position_integrator(IntegratorKind::Verlet, 0.0, 0.0);
        pi_step(&mut i, 10.0, 0.1);
        assert!(i.pos.abs() > 0.0);
    }
    #[test]
    fn test_rk2_step() {
        let mut i = new_position_integrator(IntegratorKind::RK2, 0.0, 2.0);
        pi_step(&mut i, 0.0, 0.1);
        assert!(i.pos > 0.0);
    }
    #[test]
    fn test_step_count() {
        let mut i = new_position_integrator(IntegratorKind::Euler, 0.0, 0.0);
        pi_step(&mut i, 0.0, 0.1);
        pi_step(&mut i, 0.0, 0.1);
        assert_eq!(pi_steps(&i), 2);
    }
    #[test]
    fn test_kinetic_energy() {
        let i = new_position_integrator(IntegratorKind::Euler, 0.0, 2.0);
        assert!((pi_kinetic_energy(&i, 1.0) - 2.0).abs() < 1e-5);
    }
    #[test]
    fn test_speed() {
        let mut i = new_position_integrator(IntegratorKind::Euler, 0.0, -3.0);
        assert!((pi_speed(&i) - 3.0).abs() < 1e-5);
        pi_step(&mut i, 0.0, 0.1);
    }
    #[test]
    fn test_reset() {
        let mut i = new_position_integrator(IntegratorKind::Euler, 0.0, 1.0);
        pi_step(&mut i, 0.0, 0.5);
        pi_reset(&mut i, 0.0, 0.0);
        assert_eq!(i.pos, 0.0);
        assert_eq!(i.vel, 0.0);
        assert_eq!(pi_steps(&i), 0);
    }
    #[test]
    fn test_constant_velocity_euler() {
        let mut i = new_position_integrator(IntegratorKind::Euler, 0.0, 2.0);
        pi_step(&mut i, 0.0, 1.0);
        assert!((i.pos - 2.0).abs() < 1e-5);
    }
    #[test]
    fn test_displacement() {
        let mut i = new_position_integrator(IntegratorKind::SemiImplicit, 0.0, 1.0);
        pi_step(&mut i, 0.0, 0.5);
        let d = pi_displacement(&i);
        assert!(d.abs() > 0.0);
    }
}
