// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Anticipation: brief opposite motion before main action.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnticipationMorph {
    pub target: f32,
    pub current: f32,
    pub anticipation_amt: f32,
    pub phase: u8,
}

#[allow(dead_code)]
pub fn new_anticipation_morph(anticipation_amt: f32) -> AnticipationMorph {
    AnticipationMorph { target: 0.0, current: 0.0, anticipation_amt, phase: 2 /* done */ }
}

#[allow(dead_code)]
pub fn am_trigger(m: &mut AnticipationMorph, target: f32) {
    m.target = target;
    m.phase = 0;
}

#[allow(dead_code)]
pub fn am_step(m: &mut AnticipationMorph, dt: f32) {
    match m.phase {
        0 => {
            /* phase 0: move opposite to target direction */
            let dir = if m.target > m.current { -1.0 } else { 1.0 };
            m.current += dir * m.anticipation_amt * dt;
            m.phase = 1;
        }
        1 => {
            /* phase 1: move toward target */
            let diff = m.target - m.current;
            m.current += diff * (5.0 * dt).min(1.0);
            if (m.current - m.target).abs() < 1e-4 {
                m.current = m.target;
                m.phase = 2;
            }
        }
        _ => { /* done */ }
    }
}

#[allow(dead_code)]
pub fn am_value(m: &AnticipationMorph) -> f32 {
    m.current
}

#[allow(dead_code)]
pub fn am_phase(m: &AnticipationMorph) -> u8 {
    m.phase
}

#[allow(dead_code)]
pub fn am_is_done(m: &AnticipationMorph) -> bool {
    m.phase >= 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_sets_target() {
        let mut m = new_anticipation_morph(0.2);
        am_trigger(&mut m, 1.0);
        assert!((m.target - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_trigger_sets_phase_zero() {
        let mut m = new_anticipation_morph(0.2);
        am_trigger(&mut m, 1.0);
        assert_eq!(am_phase(&m), 0);
    }

    #[test]
    fn test_phase_transitions_to_one() {
        let mut m = new_anticipation_morph(0.1);
        am_trigger(&mut m, 1.0);
        am_step(&mut m, 0.1);
        assert_eq!(am_phase(&m), 1);
    }

    #[test]
    fn test_phase_one_moves_toward_target() {
        let mut m = new_anticipation_morph(0.0);
        am_trigger(&mut m, 1.0);
        am_step(&mut m, 0.0); /* phase 0: no anticipation */
        am_step(&mut m, 0.5); /* phase 1: move toward target */
        assert!(m.current > 0.0);
    }

    #[test]
    fn test_is_done_initially() {
        let m = new_anticipation_morph(0.2);
        assert!(am_is_done(&m));
    }

    #[test]
    fn test_is_not_done_after_trigger() {
        let mut m = new_anticipation_morph(0.2);
        am_trigger(&mut m, 1.0);
        assert!(!am_is_done(&m));
    }

    #[test]
    fn test_anticipation_amt_stored() {
        let m = new_anticipation_morph(0.35);
        assert!((m.anticipation_amt - 0.35).abs() < 1e-6);
    }

    #[test]
    fn test_value_finite_after_steps() {
        let mut m = new_anticipation_morph(0.1);
        am_trigger(&mut m, 0.5);
        for _ in 0..20 {
            am_step(&mut m, 0.1);
        }
        assert!(am_value(&m).is_finite());
    }
}
