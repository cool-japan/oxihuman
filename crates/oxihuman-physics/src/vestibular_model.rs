// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Inner ear semicircular canal vestibular dynamics (first-order model).

pub struct SemicircularCanal {
    pub angular_velocity: f32,
    pub cupula_displacement: f32,
    pub time_constant: f32,
}

pub fn new_semicircular_canal(tc: f32) -> SemicircularCanal {
    SemicircularCanal {
        angular_velocity: 0.0,
        cupula_displacement: 0.0,
        time_constant: tc.max(0.001),
    }
}

pub fn canal_step(c: &mut SemicircularCanal, head_angular_vel: f32, dt: f32) {
    /* first-order dynamics: cupula approaches head velocity with time constant */
    let alpha = dt / (c.time_constant + dt);
    c.cupula_displacement += alpha * (head_angular_vel - c.cupula_displacement);
    c.angular_velocity = head_angular_vel;
}

pub fn canal_perceived_rotation(c: &SemicircularCanal) -> f32 {
    c.cupula_displacement
}

pub fn canal_is_stimulated(c: &SemicircularCanal, threshold: f32) -> bool {
    c.cupula_displacement.abs() > threshold
}

pub fn canal_reset(c: &mut SemicircularCanal) {
    c.angular_velocity = 0.0;
    c.cupula_displacement = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_semicircular_canal() {
        /* new canal starts at rest */
        let c = new_semicircular_canal(7.0);
        assert_eq!(canal_perceived_rotation(&c), 0.0);
    }

    #[test]
    fn test_canal_step_responds_to_rotation() {
        /* cupula responds to head rotation */
        let mut c = new_semicircular_canal(7.0);
        canal_step(&mut c, 1.0, 1.0);
        assert!(canal_perceived_rotation(&c) > 0.0);
    }

    #[test]
    fn test_canal_decays_after_constant_rotation() {
        /* at constant rotation, perceived rotation approaches head velocity */
        let mut c = new_semicircular_canal(1.0);
        for _ in 0..100 {
            canal_step(&mut c, 1.0, 0.1);
        }
        assert!((canal_perceived_rotation(&c) - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_canal_is_stimulated() {
        /* canal is stimulated when cupula displacement exceeds threshold */
        let mut c = new_semicircular_canal(7.0);
        canal_step(&mut c, 10.0, 1.0);
        assert!(canal_is_stimulated(&c, 0.1));
    }

    #[test]
    fn test_canal_not_stimulated() {
        /* canal is not stimulated below threshold */
        let c = new_semicircular_canal(7.0);
        assert!(!canal_is_stimulated(&c, 0.01));
    }

    #[test]
    fn test_canal_reset() {
        /* reset clears all state */
        let mut c = new_semicircular_canal(7.0);
        canal_step(&mut c, 5.0, 1.0);
        canal_reset(&mut c);
        assert_eq!(canal_perceived_rotation(&c), 0.0);
    }

    #[test]
    fn test_canal_time_constant() {
        /* time constant is stored correctly */
        let c = new_semicircular_canal(12.5);
        assert!((c.time_constant - 12.5).abs() < 1e-5);
    }
}
