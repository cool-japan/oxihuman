// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Muscle fiber activation model (Hill-type muscle stub).

/// Muscle fiber state.
#[derive(Debug, Clone)]
pub struct Muscle {
    /// Maximum isometric force (N).
    pub f_max: f32,
    /// Current activation level [0, 1].
    pub activation: f32,
    /// Normalized fiber length (1 = optimal).
    pub fiber_length: f32,
    /// Contraction velocity (negative = shortening).
    pub velocity: f32,
}

impl Muscle {
    pub fn new(f_max: f32) -> Self {
        Muscle {
            f_max,
            activation: 0.0,
            fiber_length: 1.0,
            velocity: 0.0,
        }
    }
}

/// Create a new muscle with given max force.
pub fn new_muscle(f_max: f32) -> Muscle {
    Muscle::new(f_max)
}

/// Set activation level, clamped to [0, 1].
pub fn muscle_set_activation(m: &mut Muscle, a: f32) {
    m.activation = a.clamp(0.0, 1.0);
}

/// Force-length relationship (Gaussian approximation).
pub fn force_length(fiber_length: f32) -> f32 {
    let x = fiber_length - 1.0;
    (-x * x / 0.18).exp()
}

/// Force-velocity relationship (Hill's equation stub).
pub fn force_velocity(v: f32) -> f32 {
    /* concentric: v < 0; eccentric: v > 0 */
    if v <= 0.0 {
        /* shortening: (v_max - v) / (v_max + k*v) */
        let v_max = 10.0;
        let k = 0.25;
        (v_max + v) / (v_max - k * v)
    } else {
        /* lengthening: slightly above 1.0 */
        1.0 + 1.5 * v / (v + 10.0)
    }
    .clamp(0.0, 2.0)
}

/// Compute current muscle force.
pub fn muscle_force(m: &Muscle) -> f32 {
    m.f_max * m.activation * force_length(m.fiber_length) * force_velocity(m.velocity)
}

/// Advance activation toward target with first-order dynamics.
pub fn muscle_activate(m: &mut Muscle, target: f32, dt: f32, tau: f32) {
    let target = target.clamp(0.0, 1.0);
    let tau = tau.max(1e-6);
    m.activation += (target - m.activation) * (dt / tau);
    m.activation = m.activation.clamp(0.0, 1.0);
}

/// Return the passive elastic force at given fiber length.
pub fn passive_force(fiber_length: f32) -> f32 {
    if fiber_length <= 1.0 {
        0.0
    } else {
        (fiber_length - 1.0).powi(2) * 50.0
    }
}

/// Return total muscle force (active + passive).
pub fn total_muscle_force(m: &Muscle) -> f32 {
    muscle_force(m) + passive_force(m.fiber_length)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_muscle_zero_activation() {
        let m = new_muscle(500.0);
        assert_eq!(m.activation, 0.0);
        assert!((m.fiber_length - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_force_zero_when_inactive() {
        let m = new_muscle(500.0);
        assert!(muscle_force(&m).abs() < 1e-5);
    }

    #[test]
    fn test_force_max_at_full_activation_optimal_length() {
        let mut m = new_muscle(500.0);
        muscle_set_activation(&mut m, 1.0);
        let f = muscle_force(&m);
        assert!((f - 500.0).abs() < 1.0); /* ~f_max at optimal */
    }

    #[test]
    fn test_activation_clamped() {
        let mut m = new_muscle(100.0);
        muscle_set_activation(&mut m, 2.0);
        assert!(m.activation <= 1.0);
        muscle_set_activation(&mut m, -1.0);
        assert!(m.activation >= 0.0);
    }

    #[test]
    fn test_force_length_at_optimal() {
        assert!((force_length(1.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_force_length_decays_away_from_optimal() {
        assert!(force_length(0.5) < force_length(1.0));
        assert!(force_length(1.5) < force_length(1.0));
    }

    #[test]
    fn test_force_velocity_at_zero() {
        /* no velocity -> value near 1.0 */
        let fv = force_velocity(0.0);
        assert!((fv - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_passive_force_zero_below_optimal() {
        assert_eq!(passive_force(0.9), 0.0);
        assert_eq!(passive_force(1.0), 0.0);
    }

    #[test]
    fn test_muscle_activate_converges() {
        let mut m = new_muscle(100.0);
        for _ in 0..200 {
            muscle_activate(&mut m, 1.0, 0.01, 0.05);
        }
        assert!(m.activation > 0.99);
    }

    #[test]
    fn test_total_force_gt_active_when_stretched() {
        let mut m = new_muscle(100.0);
        muscle_set_activation(&mut m, 0.5);
        m.fiber_length = 1.2;
        assert!(total_muscle_force(&m) > muscle_force(&m));
    }
}
