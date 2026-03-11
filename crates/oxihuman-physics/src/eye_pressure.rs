// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Intraocular pressure (IOP) model stub.

/// An intraocular pressure model (fluid compartment).
#[derive(Debug, Clone)]
pub struct EyePressure {
    /// Aqueous humor production rate (m³/s).
    pub production_rate: f32,
    /// Outflow facility (m³/s/Pa).
    pub outflow_facility: f32,
    /// Episcleral venous pressure (Pa).
    pub episcleral_pressure: f32,
    /// Current IOP (Pa; normal ~2 kPa = ~15 mmHg).
    pub iop: f32,
    /// Eye volume (m³, ~6.5 mL).
    pub volume: f32,
    /// Ocular rigidity (Pa/m³).
    pub rigidity: f32,
}

impl EyePressure {
    pub fn new() -> Self {
        EyePressure {
            production_rate: 3.33e-10,   /* ~2 µL/min */
            outflow_facility: 3.0e-13,   /* 0.3 µL/min/mmHg */
            episcleral_pressure: 1200.0, /* ~9 mmHg */
            iop: 2000.0,                 /* ~15 mmHg */
            volume: 6.5e-6,
            rigidity: 2.6e9,
        }
    }
}

impl Default for EyePressure {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new IOP model.
pub fn new_eye_pressure() -> EyePressure {
    EyePressure::new()
}

/// Advance IOP model by `dt` seconds (Goldmann equation).
pub fn iop_step(eye: &mut EyePressure, dt: f32) {
    /* Goldmann: F = C * (IOP - Pvein) */
    let outflow = eye.outflow_facility * (eye.iop - eye.episcleral_pressure).max(0.0);
    let net_flow = eye.production_rate - outflow;
    /* volume change -> pressure change via rigidity */
    let dp = eye.rigidity * net_flow * dt;
    eye.iop += dp;
    eye.iop = eye.iop.max(0.0);
}

/// Return IOP in mmHg.
pub fn iop_mmhg(eye: &EyePressure) -> f32 {
    eye.iop * 0.0075006
}

/// Return `true` if IOP is in the normal range (10–21 mmHg).
pub fn iop_is_normal(eye: &EyePressure) -> bool {
    let mmhg = iop_mmhg(eye);
    (10.0..=21.0).contains(&mmhg)
}

/// Simulate medication: increase outflow facility by factor.
pub fn iop_apply_medication(eye: &mut EyePressure, factor: f32) {
    eye.outflow_facility *= factor.max(1.0);
}

/// Return steady-state IOP given current production and outflow.
pub fn iop_steady_state(eye: &EyePressure) -> f32 {
    eye.production_rate / eye.outflow_facility.max(1e-20) + eye.episcleral_pressure
}

/// Set production rate (m³/s).
pub fn iop_set_production(eye: &mut EyePressure, rate: f32) {
    eye.production_rate = rate.max(0.0);
}

/// Return `true` if IOP is in glaucoma risk zone (>21 mmHg).
pub fn iop_is_elevated(eye: &EyePressure) -> bool {
    iop_mmhg(eye) > 21.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_iop_normal() {
        let eye = new_eye_pressure();
        assert!(iop_is_normal(&eye));
    }

    #[test]
    fn test_step_does_not_crash() {
        let mut eye = new_eye_pressure();
        for _ in 0..100 {
            iop_step(&mut eye, 0.01);
        }
        assert!(eye.iop >= 0.0);
    }

    #[test]
    fn test_iop_mmhg_conversion() {
        let eye = new_eye_pressure();
        let mmhg = iop_mmhg(&eye);
        assert!(mmhg > 0.0);
    }

    #[test]
    fn test_elevated_when_high() {
        let mut eye = new_eye_pressure();
        eye.iop = 4000.0; /* ~30 mmHg */
        assert!(iop_is_elevated(&eye));
    }

    #[test]
    fn test_not_elevated_when_normal() {
        let eye = new_eye_pressure();
        assert!(!iop_is_elevated(&eye));
    }

    #[test]
    fn test_medication_reduces_iop() {
        let mut eye = new_eye_pressure();
        let iop0 = eye.iop;
        iop_apply_medication(&mut eye, 2.0);
        for _ in 0..1000 {
            iop_step(&mut eye, 0.01);
        }
        assert!(eye.iop < iop0);
    }

    #[test]
    fn test_steady_state_positive() {
        let eye = new_eye_pressure();
        assert!(iop_steady_state(&eye) > 0.0);
    }

    #[test]
    fn test_set_production_zero_reduces_iop() {
        let mut eye = new_eye_pressure();
        iop_set_production(&mut eye, 0.0);
        for _ in 0..1000 {
            iop_step(&mut eye, 0.01);
        }
        /* time constant ~1282s; after 10s IOP still near initial but strictly less */
        assert!(eye.iop < 2000.0);
    }

    #[test]
    fn test_iop_nonnegative() {
        let mut eye = new_eye_pressure();
        eye.iop = 0.0;
        iop_step(&mut eye, 1.0);
        assert!(eye.iop >= 0.0);
    }
}
