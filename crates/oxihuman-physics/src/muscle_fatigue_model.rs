// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Three-compartment muscle fatigue model (Liu 2002).
//! ma (active) + mr (resting) + mf (fatigued) = 1.0

pub struct MuscleFatigue {
    pub ma: f32,
    pub mr: f32,
    pub mf: f32,
    pub f_load: f32,
}

pub fn new_muscle_fatigue() -> MuscleFatigue {
    MuscleFatigue {
        ma: 1.0,
        mr: 0.0,
        mf: 0.0,
        f_load: 0.0,
    }
}

pub fn fatigue_step(m: &mut MuscleFatigue, load: f32, dt: f32) {
    /* fatigue rate proportional to load; recovery from mf to mr */
    let f_rate = 0.1 * load;
    let r_rate = 0.02;
    let delta_fa = f_rate * m.ma * dt;
    let delta_rf = r_rate * m.mf * dt;
    let delta_fa = delta_fa.min(m.ma);
    let delta_rf = delta_rf.min(m.mf);
    m.ma -= delta_fa;
    m.mf += delta_fa - delta_rf;
    m.mr += delta_rf;
    m.f_load = load;
    /* clamp */
    m.ma = m.ma.max(0.0);
    m.mr = m.mr.max(0.0);
    m.mf = m.mf.max(0.0);
}

pub fn fatigue_percent(m: &MuscleFatigue) -> f32 {
    m.mf * 100.0
}

pub fn fatigue_can_exert(m: &MuscleFatigue, required: f32) -> bool {
    m.ma >= required
}

pub fn fatigue_recovery_step(m: &mut MuscleFatigue, dt: f32) {
    /* recovery: mf → mr → ma */
    let r_rate = 0.05;
    let delta = r_rate * m.mf * dt;
    let delta = delta.min(m.mf);
    m.mf -= delta;
    m.mr += delta;
}

pub fn fatigue_reset(m: &mut MuscleFatigue) {
    m.ma = 1.0;
    m.mr = 0.0;
    m.mf = 0.0;
    m.f_load = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_muscle_fatigue() {
        /* initial state: fully active, no fatigue */
        let m = new_muscle_fatigue();
        assert!((m.ma - 1.0).abs() < 1e-5);
        assert!((m.mf).abs() < 1e-5);
    }

    #[test]
    fn test_fatigue_step_increases_mf() {
        /* fatigue increases after step under load */
        let mut m = new_muscle_fatigue();
        fatigue_step(&mut m, 0.5, 1.0);
        assert!(m.mf > 0.0);
    }

    #[test]
    fn test_fatigue_step_decreases_ma() {
        /* active fraction decreases under load */
        let mut m = new_muscle_fatigue();
        fatigue_step(&mut m, 0.5, 1.0);
        assert!(m.ma < 1.0);
    }

    #[test]
    fn test_fatigue_percent() {
        /* fatigue_percent returns mf as percentage */
        let mut m = new_muscle_fatigue();
        fatigue_step(&mut m, 1.0, 5.0);
        assert!(fatigue_percent(&m) > 0.0);
    }

    #[test]
    fn test_fatigue_can_exert() {
        /* can exert low load when fresh */
        let m = new_muscle_fatigue();
        assert!(fatigue_can_exert(&m, 0.5));
        assert!(!fatigue_can_exert(&m, 1.5));
    }

    #[test]
    fn test_fatigue_recovery() {
        /* recovery step reduces fatigue */
        let mut m = new_muscle_fatigue();
        fatigue_step(&mut m, 1.0, 5.0);
        let mf_before = m.mf;
        fatigue_recovery_step(&mut m, 10.0);
        assert!(m.mf < mf_before);
    }

    #[test]
    fn test_fatigue_reset() {
        /* reset restores initial state */
        let mut m = new_muscle_fatigue();
        fatigue_step(&mut m, 1.0, 10.0);
        fatigue_reset(&mut m);
        assert!((m.ma - 1.0).abs() < 1e-5);
        assert!((m.mf).abs() < 1e-5);
    }
}
