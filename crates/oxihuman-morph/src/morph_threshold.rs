// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Morph threshold gating.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphThreshold {
    pub threshold: f32,
    pub on_value: f32,
    pub off_value: f32,
    pub hysteresis: f32,
}

#[allow(dead_code)]
pub fn new_morph_threshold(threshold: f32) -> MorphThreshold {
    MorphThreshold { threshold, on_value: 1.0, off_value: 0.0, hysteresis: 0.0 }
}

#[allow(dead_code)]
pub fn mt_evaluate(thresh: &MorphThreshold, input: f32, current_state: bool) -> (bool, f32) {
    let new_state = if current_state {
        /* hysteresis: must drop below threshold - hysteresis to turn off */
        input >= thresh.threshold - thresh.hysteresis
    } else {
        /* must rise above threshold to turn on */
        input >= thresh.threshold
    };
    let value = if new_state { thresh.on_value } else { thresh.off_value };
    (new_state, value)
}

#[allow(dead_code)]
pub fn mt_threshold(thresh: &MorphThreshold) -> f32 {
    thresh.threshold
}

#[allow(dead_code)]
pub fn mt_set_hysteresis(thresh: &mut MorphThreshold, h: f32) {
    thresh.hysteresis = h;
}

#[allow(dead_code)]
pub fn mt_set_values(thresh: &mut MorphThreshold, on: f32, off: f32) {
    thresh.on_value = on;
    thresh.off_value = off;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_below_threshold_stays_off() {
        let t = new_morph_threshold(0.5);
        let (state, val) = mt_evaluate(&t, 0.3, false);
        assert!(!state);
        assert!((val - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_above_threshold_turns_on() {
        let t = new_morph_threshold(0.5);
        let (state, val) = mt_evaluate(&t, 0.8, false);
        assert!(state);
        assert!((val - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_hysteresis_keeps_on_in_zone() {
        let mut t = new_morph_threshold(0.5);
        mt_set_hysteresis(&mut t, 0.2);
        /* currently on, input at 0.35 which is in hysteresis zone (0.3 to 0.5) */
        let (state, _) = mt_evaluate(&t, 0.35, true);
        assert!(state); /* stays on */
    }

    #[test]
    fn test_hysteresis_turns_off_below_zone() {
        let mut t = new_morph_threshold(0.5);
        mt_set_hysteresis(&mut t, 0.2);
        /* below threshold - hysteresis = 0.3 */
        let (state, _) = mt_evaluate(&t, 0.2, true);
        assert!(!state);
    }

    #[test]
    fn test_set_values() {
        let mut t = new_morph_threshold(0.5);
        mt_set_values(&mut t, 0.8, 0.1);
        let (_, val_on) = mt_evaluate(&t, 0.9, false);
        let (_, val_off) = mt_evaluate(&t, 0.1, false);
        assert!((val_on - 0.8).abs() < 1e-6);
        assert!((val_off - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_threshold_getter() {
        let t = new_morph_threshold(0.7);
        assert!((mt_threshold(&t) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_at_exact_threshold_turns_on() {
        let t = new_morph_threshold(0.5);
        let (state, _) = mt_evaluate(&t, 0.5, false);
        assert!(state);
    }

    #[test]
    fn test_default_on_off_values() {
        let t = new_morph_threshold(0.5);
        assert!((t.on_value - 1.0).abs() < 1e-6);
        assert!((t.off_value - 0.0).abs() < 1e-6);
    }
}
