// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct SpinalSegment {
    pub vertebra_mass: f32,
    pub disc_stiffness: f32,
    pub angle_rad: f32,
    pub angular_vel: f32,
}

pub fn new_spinal_segment(mass: f32, stiffness: f32) -> SpinalSegment {
    SpinalSegment {
        vertebra_mass: mass,
        disc_stiffness: stiffness,
        angle_rad: 0.0,
        angular_vel: 0.0,
    }
}

pub fn spinal_segment_step(s: &mut SpinalSegment, torque: f32, dt: f32) {
    let inertia = s.vertebra_mass * 0.01; /* rough estimate */
    let restoring = -s.disc_stiffness * s.angle_rad;
    let alpha = (torque + restoring) / inertia.max(1e-9);
    s.angular_vel += alpha * dt;
    s.angle_rad += s.angular_vel * dt;
}

pub fn spinal_column_range_of_motion(segments: &[SpinalSegment]) -> f32 {
    segments.iter().map(|s| s.angle_rad.abs()).sum()
}

pub fn spinal_segment_is_overloaded(s: &SpinalSegment, max_angle_rad: f32) -> bool {
    s.angle_rad.abs() > max_angle_rad
}

pub fn spinal_total_stiffness(segments: &[SpinalSegment]) -> f32 {
    if segments.is_empty() {
        return 0.0;
    }
    /* springs in series: 1/k_total = sum(1/k_i) */
    let sum_recip: f32 = segments
        .iter()
        .map(|s| 1.0 / s.disc_stiffness.max(1e-9))
        .sum();
    1.0 / sum_recip
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_angle_zero() {
        /* new segment starts at zero angle */
        let s = new_spinal_segment(1.0, 100.0);
        assert_eq!(s.angle_rad, 0.0);
    }

    #[test]
    fn step_changes_angle() {
        /* applying torque changes angle */
        let mut s = new_spinal_segment(1.0, 100.0);
        spinal_segment_step(&mut s, 1.0, 0.01);
        assert_ne!(s.angle_rad, 0.0);
    }

    #[test]
    fn range_of_motion_sum() {
        /* ROM is sum of absolute angles */
        let mut s1 = new_spinal_segment(1.0, 100.0);
        let mut s2 = new_spinal_segment(1.0, 100.0);
        s1.angle_rad = 0.1;
        s2.angle_rad = 0.05;
        assert!((spinal_column_range_of_motion(&[s1, s2]) - 0.15).abs() < 1e-6);
    }

    #[test]
    fn overloaded_when_angle_exceeds_limit() {
        /* overloaded when angle exceeds max */
        let mut s = new_spinal_segment(1.0, 100.0);
        s.angle_rad = 0.5;
        assert!(spinal_segment_is_overloaded(&s, 0.3));
    }

    #[test]
    fn not_overloaded_within_limit() {
        /* within limits is not overloaded */
        let s = new_spinal_segment(1.0, 100.0);
        assert!(!spinal_segment_is_overloaded(&s, 0.3));
    }

    #[test]
    fn total_stiffness_positive() {
        /* total stiffness is positive for valid segments */
        let segments = vec![
            new_spinal_segment(1.0, 100.0),
            new_spinal_segment(1.0, 200.0),
        ];
        assert!(spinal_total_stiffness(&segments) > 0.0);
    }

    #[test]
    fn empty_stiffness_zero() {
        /* empty segment list gives zero stiffness */
        assert_eq!(spinal_total_stiffness(&[]), 0.0);
    }
}
