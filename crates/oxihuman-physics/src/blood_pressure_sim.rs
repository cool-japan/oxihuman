// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hemodynamic blood pressure model stub (Windkessel model).

/// A simple 2-element Windkessel model (compliance + resistance).
#[derive(Debug, Clone)]
pub struct BloodPressure {
    /// Arterial compliance (m³/Pa).
    pub compliance: f32,
    /// Peripheral resistance (Pa·s/m³).
    pub resistance: f32,
    /// Current arterial pressure (Pa = N/m²; 1 mmHg ≈ 133.3 Pa).
    pub pressure: f32,
    /// Heart rate (beats/min).
    pub heart_rate: f32,
    /// Stroke volume (m³ per beat).
    pub stroke_volume: f32,
}

impl BloodPressure {
    pub fn new() -> Self {
        BloodPressure {
            compliance: 1.0e-9, /* ~1 ml/mmHg */
            resistance: 1.0e9,  /* peripheral */
            pressure: 13_300.0, /* ~100 mmHg */
            heart_rate: 70.0,
            stroke_volume: 70e-6, /* 70 ml */
        }
    }
}

impl Default for BloodPressure {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new default blood pressure model.
pub fn new_blood_pressure() -> BloodPressure {
    BloodPressure::new()
}

/// Advance the Windkessel model by `dt` seconds.
pub fn bp_step(bp: &mut BloodPressure, dt: f32) {
    /* outflow = P / R */
    let outflow = bp.pressure / bp.resistance;
    /* no inflow between beats — only decay */
    bp.pressure -= outflow / bp.compliance * dt;
    bp.pressure = bp.pressure.max(0.0);
}

/// Inject a cardiac ejection (add stroke volume as pressure pulse).
pub fn bp_heartbeat(bp: &mut BloodPressure) {
    bp.pressure += bp.stroke_volume / bp.compliance;
}

/// Return pressure in mmHg (1 Pa ≈ 0.0075 mmHg).
pub fn bp_pressure_mmhg(bp: &BloodPressure) -> f32 {
    bp.pressure * 0.0075006
}

/// Return mean arterial pressure estimate over one cycle.
pub fn bp_mean_arterial_pressure(bp: &BloodPressure) -> f32 {
    /* MAP = DBP + 1/3 pulse pressure; crude estimate */
    bp.pressure * 0.9 /* simplified */
}

/// Return the pulse period (s).
pub fn bp_period(bp: &BloodPressure) -> f32 {
    60.0 / bp.heart_rate.max(1.0)
}

/// Simulate one full cardiac cycle; returns end-diastolic pressure.
pub fn bp_simulate_cycle(bp: &mut BloodPressure) -> f32 {
    let period = bp_period(bp);
    let steps = 100usize;
    let dt = period / steps as f32;
    bp_heartbeat(bp);
    for _ in 0..steps {
        bp_step(bp, dt);
    }
    bp.pressure
}

/// Return `true` if pressure is within normal range (80–120 mmHg).
pub fn bp_is_normal(bp: &BloodPressure) -> bool {
    let mmhg = bp_pressure_mmhg(bp);
    (80.0..=120.0).contains(&mmhg)
}

/// Set heart rate (clamped to 30–220 bpm).
pub fn bp_set_heart_rate(bp: &mut BloodPressure, bpm: f32) {
    bp.heart_rate = bpm.clamp(30.0, 220.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_pressure_nonzero() {
        let bp = new_blood_pressure();
        assert!(bp.pressure > 0.0);
    }

    #[test]
    fn test_step_reduces_pressure() {
        let mut bp = new_blood_pressure();
        let p0 = bp.pressure;
        bp_step(&mut bp, 0.01);
        assert!(bp.pressure < p0);
    }

    #[test]
    fn test_heartbeat_increases_pressure() {
        let mut bp = new_blood_pressure();
        let p0 = bp.pressure;
        bp_heartbeat(&mut bp);
        assert!(bp.pressure > p0);
    }

    #[test]
    fn test_pressure_mmhg_conversion() {
        let bp = new_blood_pressure();
        let mmhg = bp_pressure_mmhg(&bp);
        assert!(mmhg > 0.0);
    }

    #[test]
    fn test_period_at_70bpm() {
        let bp = new_blood_pressure();
        let t = bp_period(&bp);
        assert!((t - 60.0 / 70.0).abs() < 0.01);
    }

    #[test]
    fn test_set_heart_rate_clamped() {
        let mut bp = new_blood_pressure();
        bp_set_heart_rate(&mut bp, 300.0);
        assert!(bp.heart_rate <= 220.0);
        bp_set_heart_rate(&mut bp, 5.0);
        assert!(bp.heart_rate >= 30.0);
    }

    #[test]
    fn test_simulate_cycle_returns_pressure() {
        let mut bp = new_blood_pressure();
        let p = bp_simulate_cycle(&mut bp);
        assert!(p >= 0.0);
    }

    #[test]
    fn test_pressure_nonnegative_after_many_steps() {
        let mut bp = new_blood_pressure();
        for _ in 0..1000 {
            bp_step(&mut bp, 0.01);
        }
        assert!(bp.pressure >= 0.0);
    }

    #[test]
    fn test_mean_arterial_pressure_positive() {
        let bp = new_blood_pressure();
        assert!(bp_mean_arterial_pressure(&bp) > 0.0);
    }
}
