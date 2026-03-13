// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Viscous linear damper (dashpot force model).

#![allow(dead_code)]

/// A viscous linear damper (dashpot).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DamperBody {
    /// Damping coefficient c (N·s/m).
    pub c: f32,
    /// Current velocity of the damper piston (m/s).
    pub velocity: f32,
    /// Current displacement (m).
    pub displacement: f32,
    /// Maximum stroke (±) in meters.
    pub max_stroke: f32,
    /// Accumulated force impulse.
    pub impulse: f32,
}

/// Create a new damper body.
#[allow(dead_code)]
pub fn new_damper_body(c: f32, max_stroke: f32) -> DamperBody {
    DamperBody {
        c: c.max(0.0),
        velocity: 0.0,
        displacement: 0.0,
        max_stroke: max_stroke.abs(),
        impulse: 0.0,
    }
}

/// Compute the damper force for a given relative velocity.
/// F = -c * v
#[allow(dead_code)]
pub fn damper_force(d: &DamperBody, rel_velocity: f32) -> f32 {
    -d.c * rel_velocity
}

/// Step the damper: apply relative velocity, update displacement.
#[allow(dead_code)]
pub fn damper_step(d: &mut DamperBody, rel_velocity: f32, dt: f32) {
    d.velocity = rel_velocity;
    let force = damper_force(d, rel_velocity);
    d.impulse += force.abs() * dt;
    d.displacement = (d.displacement + rel_velocity * dt).clamp(-d.max_stroke, d.max_stroke);
}

/// Check if the damper is at max stroke.
#[allow(dead_code)]
pub fn damper_at_limit(d: &DamperBody) -> bool {
    d.displacement.abs() >= d.max_stroke - 1e-6
}

/// Power dissipated by the damper: P = F * v = c * v^2.
#[allow(dead_code)]
pub fn damper_power(d: &DamperBody, rel_velocity: f32) -> f32 {
    d.c * rel_velocity * rel_velocity
}

/// Total energy dissipated (accumulated impulse × average velocity — simplified).
#[allow(dead_code)]
pub fn damper_total_impulse(d: &DamperBody) -> f32 {
    d.impulse
}

/// Reset the damper.
#[allow(dead_code)]
pub fn damper_reset(d: &mut DamperBody) {
    d.velocity = 0.0;
    d.displacement = 0.0;
    d.impulse = 0.0;
}

/// Set a new damping coefficient.
#[allow(dead_code)]
pub fn damper_set_c(d: &mut DamperBody, c: f32) {
    d.c = c.max(0.0);
}

/// Compression ratio: displacement / max_stroke.
#[allow(dead_code)]
pub fn damper_compression_ratio(d: &DamperBody) -> f32 {
    if d.max_stroke < 1e-10 {
        return 0.0;
    }
    d.displacement / d.max_stroke
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_damper() -> DamperBody {
        new_damper_body(100.0, 0.1)
    }

    #[test]
    fn test_initial_state() {
        let d = make_damper();
        assert_eq!(d.velocity, 0.0);
        assert_eq!(d.displacement, 0.0);
    }

    #[test]
    fn test_force_proportional_to_velocity() {
        let d = make_damper();
        let f1 = damper_force(&d, 1.0);
        let f2 = damper_force(&d, 2.0);
        assert!((f2 / f1 - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_force_negative_for_positive_velocity() {
        let d = make_damper();
        assert!(damper_force(&d, 1.0) < 0.0);
    }

    #[test]
    fn test_step_updates_displacement() {
        let mut d = make_damper();
        damper_step(&mut d, 0.5, 0.01);
        assert!((d.displacement - 0.005).abs() < 1e-6);
    }

    #[test]
    fn test_stroke_clamped() {
        let mut d = make_damper();
        for _ in 0..100 {
            damper_step(&mut d, 1.0, 0.1);
        }
        assert!(damper_at_limit(&d));
    }

    #[test]
    fn test_power_dissipation() {
        let d = make_damper();
        let p = damper_power(&d, 1.0);
        assert!((p - 100.0).abs() < 1e-5);
    }

    #[test]
    fn test_impulse_accumulates() {
        let mut d = make_damper();
        damper_step(&mut d, 1.0, 0.1);
        assert!(d.impulse > 0.0);
    }

    #[test]
    fn test_reset() {
        let mut d = make_damper();
        damper_step(&mut d, 1.0, 0.01);
        damper_reset(&mut d);
        assert_eq!(d.displacement, 0.0);
        assert_eq!(d.impulse, 0.0);
    }

    #[test]
    fn test_set_c() {
        let mut d = make_damper();
        damper_set_c(&mut d, 200.0);
        assert_eq!(d.c, 200.0);
    }

    #[test]
    fn test_compression_ratio() {
        let mut d = make_damper();
        damper_step(&mut d, 1.0, 0.05);
        let r = damper_compression_ratio(&d);
        assert!((0.0..=1.0).contains(&r));
    }
}
