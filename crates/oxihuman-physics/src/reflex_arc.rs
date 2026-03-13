// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Monosynaptic stretch reflex arc model.

pub struct ReflexArc {
    pub latency_ms: f32,
    pub gain: f32,
    pub threshold_strain: f32,
    pub fatigue: f32,
}

pub fn new_reflex_arc(latency_ms: f32) -> ReflexArc {
    ReflexArc {
        latency_ms: latency_ms.max(0.0),
        gain: 1.0,
        threshold_strain: 0.02,
        fatigue: 0.0,
    }
}

pub fn reflex_response(r: &ReflexArc, strain: f32, time_ms: f32) -> f32 {
    /* delayed response based on latency */
    if time_ms < r.latency_ms {
        return 0.0;
    }
    if strain < r.threshold_strain {
        return 0.0;
    }
    r.gain * (strain - r.threshold_strain) * (1.0 - r.fatigue).max(0.0)
}

pub fn reflex_is_active(r: &ReflexArc, strain: f32) -> bool {
    strain >= r.threshold_strain && r.fatigue < 1.0
}

pub fn reflex_peak_force(r: &ReflexArc, strain: f32) -> f32 {
    if strain < r.threshold_strain {
        return 0.0;
    }
    r.gain * (strain - r.threshold_strain) * (1.0 - r.fatigue).max(0.0)
}

pub fn reflex_apply_fatigue(r: &mut ReflexArc, amount: f32) {
    r.fatigue = (r.fatigue + amount).min(1.0);
}

pub fn reflex_reset_fatigue(r: &mut ReflexArc) {
    r.fatigue = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_reflex_arc() {
        /* new arc has correct latency */
        let r = new_reflex_arc(30.0);
        assert!((r.latency_ms - 30.0).abs() < 1e-5);
    }

    #[test]
    fn test_reflex_response_before_latency() {
        /* no response before latency */
        let r = new_reflex_arc(30.0);
        assert_eq!(reflex_response(&r, 0.1, 10.0), 0.0);
    }

    #[test]
    fn test_reflex_response_after_latency() {
        /* response occurs after latency period */
        let r = new_reflex_arc(30.0);
        assert!(reflex_response(&r, 0.1, 50.0) > 0.0);
    }

    #[test]
    fn test_reflex_is_active_above_threshold() {
        /* active when strain exceeds threshold */
        let r = new_reflex_arc(30.0);
        assert!(reflex_is_active(&r, 0.05));
    }

    #[test]
    fn test_reflex_is_not_active_below_threshold() {
        /* not active when strain below threshold */
        let r = new_reflex_arc(30.0);
        assert!(!reflex_is_active(&r, 0.001));
    }

    #[test]
    fn test_reflex_apply_fatigue() {
        /* fatigue accumulates */
        let mut r = new_reflex_arc(30.0);
        reflex_apply_fatigue(&mut r, 0.5);
        assert!((r.fatigue - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_reflex_reset_fatigue() {
        /* reset clears fatigue */
        let mut r = new_reflex_arc(30.0);
        reflex_apply_fatigue(&mut r, 0.8);
        reflex_reset_fatigue(&mut r);
        assert_eq!(r.fatigue, 0.0);
    }
}
