// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::TAU;

const REST_RADIUS: f32 = 0.012; // 12 mm (small intestine)

/// Peristaltic wave segment.
pub struct PeristalticSegment {
    pub position: f32,
    pub radius: f32,
    pub contraction_phase: f32,
    pub wave_speed: f32,
    pub frequency: f32,
}

impl PeristalticSegment {
    pub fn new(position: f32) -> Self {
        PeristalticSegment {
            position,
            radius: REST_RADIUS,
            contraction_phase: 0.0,
            wave_speed: 0.02, // m/s
            frequency: 0.2,   // Hz (≈12 cycles/min)
        }
    }
}

pub fn new_peristaltic_segment(position: f32) -> PeristalticSegment {
    PeristalticSegment::new(position)
}

pub fn peristalsis_step(s: &mut PeristalticSegment, dt: f32) {
    s.contraction_phase += TAU * s.frequency * dt;
    s.radius = peristalsis_radius(s);
}

/// r(t) = rest * (1 - 0.3 * sin(phase))
pub fn peristalsis_radius(s: &PeristalticSegment) -> f32 {
    REST_RADIUS * (1.0 - 0.3 * s.contraction_phase.sin())
}

pub fn peristalsis_is_contracted(s: &PeristalticSegment) -> bool {
    s.radius < REST_RADIUS
}

pub fn peristalsis_bolus_velocity(s: &PeristalticSegment) -> f32 {
    s.wave_speed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* new segment starts at rest radius */
        let s = new_peristaltic_segment(0.0);
        assert!((s.contraction_phase - 0.0).abs() < 1e-7);
    }

    #[test]
    fn test_radius_bounds() {
        /* radius stays within [0.7, 1.3] * rest */
        let mut s = new_peristaltic_segment(0.0);
        for _ in 0..100 {
            peristalsis_step(&mut s, 0.1);
            assert!(s.radius >= REST_RADIUS * 0.5);
            assert!(s.radius <= REST_RADIUS * 1.5);
        }
    }

    #[test]
    fn test_step_advances_phase() {
        /* stepping advances contraction phase */
        let mut s = new_peristaltic_segment(0.0);
        peristalsis_step(&mut s, 0.5);
        assert!(s.contraction_phase > 0.0);
    }

    #[test]
    fn test_bolus_velocity() {
        /* bolus velocity equals wave speed */
        let s = new_peristaltic_segment(0.0);
        assert!((peristalsis_bolus_velocity(&s) - s.wave_speed).abs() < 1e-7);
    }

    #[test]
    fn test_is_contracted() {
        /* at sin=1 (max contraction), radius < rest */
        let mut s = new_peristaltic_segment(0.0);
        s.contraction_phase = std::f32::consts::FRAC_PI_2; // sin = 1
        s.radius = peristalsis_radius(&s);
        assert!(peristalsis_is_contracted(&s));
    }

    #[test]
    fn test_is_not_contracted_at_rest() {
        /* at phase=0, sin=0, radius = rest */
        let s = new_peristaltic_segment(0.0);
        /* radius at phase=0: rest*(1-0.3*0) = rest */
        assert!(!peristalsis_is_contracted(&s));
    }

    #[test]
    fn test_position_set() {
        /* position field is stored correctly */
        let s = new_peristaltic_segment(0.15);
        assert!((s.position - 0.15).abs() < 1e-7);
    }
}
