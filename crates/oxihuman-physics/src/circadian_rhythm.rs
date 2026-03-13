// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Simplified circadian oscillator model.

use std::f32::consts::TAU;

pub struct CircadianOscillator {
    pub phase: f32,
    pub period_hours: f32,
    pub amplitude: f32,
}

pub fn new_circadian_oscillator() -> CircadianOscillator {
    CircadianOscillator {
        phase: 0.0,
        period_hours: 24.0,
        amplitude: 1.0,
    }
}

pub fn circadian_step(o: &mut CircadianOscillator, dt_hours: f32) {
    o.phase = (o.phase + TAU * dt_hours / o.period_hours) % TAU;
}

pub fn circadian_alertness(o: &CircadianOscillator) -> f32 {
    /* cosine of phase: peak alertness at phase=0, min at phase=PI */
    ((o.phase.cos() + 1.0) * 0.5 * o.amplitude).clamp(0.0, 1.0)
}

pub fn circadian_is_nighttime(o: &CircadianOscillator) -> bool {
    /* nighttime when alertness < 0.3 */
    circadian_alertness(o) < 0.3
}

pub fn circadian_phase_shift(o: &mut CircadianOscillator, delta_hours: f32) {
    o.phase = (o.phase + TAU * delta_hours / o.period_hours).rem_euclid(TAU);
}

pub fn circadian_time_to_peak(o: &CircadianOscillator) -> f32 {
    /* time to return to phase=0 (peak alertness) */
    if o.phase <= 1e-6 {
        return 0.0;
    }
    (TAU - o.phase) / TAU * o.period_hours
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_circadian_oscillator() {
        /* new oscillator starts at phase 0 */
        let o = new_circadian_oscillator();
        assert!((o.phase).abs() < 1e-5);
        assert!((o.period_hours - 24.0).abs() < 1e-5);
    }

    #[test]
    fn test_circadian_step_advances_phase() {
        /* phase advances with each step */
        let mut o = new_circadian_oscillator();
        let phase_before = o.phase;
        circadian_step(&mut o, 1.0);
        assert!(o.phase > phase_before);
    }

    #[test]
    fn test_circadian_alertness_at_phase_zero() {
        /* alertness is maximum at phase 0 */
        let o = new_circadian_oscillator();
        let alertness = circadian_alertness(&o);
        assert!((alertness - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_circadian_alertness_range() {
        /* alertness is always between 0 and 1 */
        let mut o = new_circadian_oscillator();
        for _ in 0..24 {
            circadian_step(&mut o, 1.0);
            let a = circadian_alertness(&o);
            assert!((0.0..=1.0).contains(&a));
        }
    }

    #[test]
    fn test_circadian_is_nighttime() {
        /* nighttime detection based on low alertness */
        let mut o = new_circadian_oscillator();
        o.phase = std::f32::consts::PI; /* opposite peak = max sleep */
        assert!(circadian_is_nighttime(&o));
    }

    #[test]
    fn test_circadian_phase_shift() {
        /* phase shift changes phase */
        let mut o = new_circadian_oscillator();
        let before = o.phase;
        circadian_phase_shift(&mut o, 6.0);
        assert!(o.phase != before);
    }

    #[test]
    fn test_circadian_time_to_peak() {
        /* time to peak is within [0, period_hours] */
        let mut o = new_circadian_oscillator();
        circadian_step(&mut o, 6.0);
        let ttp = circadian_time_to_peak(&o);
        assert!((0.0..=24.0).contains(&ttp));
    }
}
