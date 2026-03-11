// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Phase space representation (position, momentum) of a 1-DOF system.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct PhasePoint {
    pub q: f32,
    pub p: f32,
}

#[allow(dead_code)]
pub struct PhaseSpace {
    pub state: PhasePoint,
    pub mass: f32,
    pub trajectory: Vec<PhasePoint>,
    pub max_trajectory: usize,
}

#[allow(dead_code)]
impl PhaseSpace {
    pub fn new(q0: f32, p0: f32, mass: f32, max_trajectory: usize) -> Self {
        Self {
            state: PhasePoint { q: q0, p: p0 },
            mass,
            trajectory: Vec::new(),
            max_trajectory,
        }
    }
    pub fn velocity(&self) -> f32 {
        self.state.p / self.mass.max(1e-10)
    }
    pub fn kinetic_energy(&self) -> f32 {
        self.state.p * self.state.p / (2.0 * self.mass.max(1e-10))
    }
    pub fn record(&mut self) {
        if self.trajectory.len() >= self.max_trajectory {
            self.trajectory.remove(0);
        }
        self.trajectory.push(self.state);
    }
    pub fn apply_force(&mut self, force: f32, dt: f32) {
        self.state.p += force * dt;
        self.state.q += self.velocity() * dt;
        self.record();
    }
    pub fn apply_harmonic(&mut self, k: f32, dt: f32) {
        let force = -k * self.state.q;
        self.apply_force(force, dt);
    }
    pub fn trajectory_len(&self) -> usize {
        self.trajectory.len()
    }
    pub fn max_q(&self) -> f32 {
        self.trajectory
            .iter()
            .map(|p| p.q.abs())
            .fold(0.0f32, f32::max)
    }
    pub fn max_p(&self) -> f32 {
        self.trajectory
            .iter()
            .map(|p| p.p.abs())
            .fold(0.0f32, f32::max)
    }
    pub fn angular_frequency(&self, k: f32) -> f32 {
        (k / self.mass.max(1e-10)).sqrt()
    }
    pub fn period(&self, k: f32) -> f32 {
        2.0 * PI / self.angular_frequency(k).max(1e-10)
    }
    pub fn reset(&mut self, q0: f32, p0: f32) {
        self.state = PhasePoint { q: q0, p: p0 };
        self.trajectory.clear();
    }
}

#[allow(dead_code)]
pub fn new_phase_space(q0: f32, p0: f32, mass: f32, max_traj: usize) -> PhaseSpace {
    PhaseSpace::new(q0, p0, mass, max_traj)
}
#[allow(dead_code)]
pub fn ps_apply_force(ps: &mut PhaseSpace, f: f32, dt: f32) {
    ps.apply_force(f, dt);
}
#[allow(dead_code)]
pub fn ps_apply_harmonic(ps: &mut PhaseSpace, k: f32, dt: f32) {
    ps.apply_harmonic(k, dt);
}
#[allow(dead_code)]
pub fn ps_velocity(ps: &PhaseSpace) -> f32 {
    ps.velocity()
}
#[allow(dead_code)]
pub fn ps_kinetic_energy(ps: &PhaseSpace) -> f32 {
    ps.kinetic_energy()
}
#[allow(dead_code)]
pub fn ps_traj_len(ps: &PhaseSpace) -> usize {
    ps.trajectory_len()
}
#[allow(dead_code)]
pub fn ps_max_q(ps: &PhaseSpace) -> f32 {
    ps.max_q()
}
#[allow(dead_code)]
pub fn ps_max_p(ps: &PhaseSpace) -> f32 {
    ps.max_p()
}
#[allow(dead_code)]
pub fn ps_period(ps: &PhaseSpace, k: f32) -> f32 {
    ps.period(k)
}
#[allow(dead_code)]
pub fn ps_reset(ps: &mut PhaseSpace, q0: f32, p0: f32) {
    ps.reset(q0, p0);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_initial_velocity() {
        let ps = new_phase_space(0.0, 2.0, 1.0, 100);
        assert!((ps_velocity(&ps) - 2.0).abs() < 1e-5);
    }
    #[test]
    fn test_kinetic_energy() {
        let ps = new_phase_space(0.0, 2.0, 1.0, 100);
        assert!((ps_kinetic_energy(&ps) - 2.0).abs() < 1e-5);
    }
    #[test]
    fn test_apply_force_changes_state() {
        let mut ps = new_phase_space(0.0, 0.0, 1.0, 100);
        ps_apply_force(&mut ps, 10.0, 0.1);
        assert!(ps.state.p > 0.0);
    }
    #[test]
    fn test_trajectory_records() {
        let mut ps = new_phase_space(1.0, 0.0, 1.0, 10);
        for _ in 0..5 {
            ps_apply_harmonic(&mut ps, 1.0, 0.01);
        }
        assert_eq!(ps_traj_len(&ps), 5);
    }
    #[test]
    fn test_max_trajectory_limit() {
        let mut ps = new_phase_space(1.0, 0.0, 1.0, 5);
        for _ in 0..20 {
            ps_apply_harmonic(&mut ps, 1.0, 0.01);
        }
        assert!(ps_traj_len(&ps) <= 5);
    }
    #[test]
    fn test_harmonic_oscillates() {
        let mut ps = new_phase_space(1.0, 0.0, 1.0, 1000);
        for _ in 0..100 {
            ps_apply_harmonic(&mut ps, 1.0, 0.01);
        }
        let mq = ps_max_q(&ps);
        assert!(mq > 0.0);
    }
    #[test]
    fn test_period_positive() {
        let ps = new_phase_space(0.0, 0.0, 1.0, 10);
        assert!(ps_period(&ps, 1.0) > 0.0);
    }
    #[test]
    fn test_reset() {
        let mut ps = new_phase_space(1.0, 2.0, 1.0, 10);
        ps_apply_force(&mut ps, 5.0, 0.1);
        ps_reset(&mut ps, 0.0, 0.0);
        assert_eq!(ps.state.q, 0.0);
        assert_eq!(ps.state.p, 0.0);
        assert_eq!(ps_traj_len(&ps), 0);
    }
    #[test]
    fn test_period_formula() {
        let ps = new_phase_space(0.0, 0.0, 1.0, 10);
        let omega = (1.0f32 / 1.0f32).sqrt();
        let expected = 2.0 * PI / omega;
        assert!((ps_period(&ps, 1.0) - expected).abs() < 1e-4);
    }
    #[test]
    fn test_max_p_after_force() {
        let mut ps = new_phase_space(0.0, 0.0, 1.0, 100);
        ps_apply_force(&mut ps, 10.0, 1.0);
        assert!(ps_max_p(&ps) > 0.0);
    }
}
