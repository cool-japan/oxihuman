// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Palm splay morph control — independent per-finger splay angles.

/// Per-finger splay weights for one hand.
#[derive(Debug, Clone, Default)]
pub struct PalmSplayState {
    pub index: f32,
    pub middle: f32,
    pub ring: f32,
    pub pinky: f32,
    pub thumb_ab: f32,
}

/// Set all fingers to the same splay amount.
pub fn ps_set_all(state: &mut PalmSplayState, amount: f32) {
    let a = amount.clamp(0.0, 1.0);
    state.index = a;
    state.middle = a;
    state.ring = a;
    state.pinky = a;
    state.thumb_ab = a;
}

/// Set individual finger splay by index (0=index,1=middle,2=ring,3=pinky,4=thumb).
pub fn ps_set_finger(state: &mut PalmSplayState, finger: usize, amount: f32) {
    let a = amount.clamp(0.0, 1.0);
    match finger {
        0 => state.index = a,
        1 => state.middle = a,
        2 => state.ring = a,
        3 => state.pinky = a,
        4 => state.thumb_ab = a,
        _ => {}
    }
}

/// Return the mean splay across all fingers.
pub fn ps_mean(state: &PalmSplayState) -> f32 {
    (state.index + state.middle + state.ring + state.pinky + state.thumb_ab) / 5.0
}

/// Return the splay for a given finger index.
pub fn ps_get_finger(state: &PalmSplayState, finger: usize) -> f32 {
    match finger {
        0 => state.index,
        1 => state.middle,
        2 => state.ring,
        3 => state.pinky,
        4 => state.thumb_ab,
        _ => 0.0,
    }
}

/// Reset all finger splays to zero.
pub fn ps_reset(state: &mut PalmSplayState) {
    *state = PalmSplayState::default();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_zero() {
        /* default should be zero for all fingers */
        let s = PalmSplayState::default();
        assert_eq!(ps_mean(&s), 0.0);
    }

    #[test]
    fn test_set_all() {
        /* set_all should apply to every finger */
        let mut s = PalmSplayState::default();
        ps_set_all(&mut s, 0.5);
        assert!((s.index - 0.5).abs() < 1e-6);
        assert!((s.pinky - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_finger_index() {
        /* set_finger(0) should set index only */
        let mut s = PalmSplayState::default();
        ps_set_finger(&mut s, 0, 0.8);
        assert!((s.index - 0.8).abs() < 1e-6);
        assert_eq!(s.middle, 0.0);
    }

    #[test]
    fn test_set_finger_thumb() {
        /* set_finger(4) should set thumb abduction */
        let mut s = PalmSplayState::default();
        ps_set_finger(&mut s, 4, 0.6);
        assert!((s.thumb_ab - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_mean() {
        /* mean should average all five fingers */
        let mut s = PalmSplayState::default();
        ps_set_all(&mut s, 0.6);
        assert!((ps_mean(&s) - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_get_finger() {
        /* get_finger should return correct value */
        let mut s = PalmSplayState::default();
        ps_set_finger(&mut s, 2, 0.9);
        assert!((ps_get_finger(&s, 2) - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_invalid_finger_get() {
        /* invalid finger index should return 0.0 */
        let s = PalmSplayState::default();
        assert_eq!(ps_get_finger(&s, 99), 0.0);
    }

    #[test]
    fn test_reset() {
        /* reset should zero all fingers */
        let mut s = PalmSplayState::default();
        ps_set_all(&mut s, 1.0);
        ps_reset(&mut s);
        assert_eq!(ps_mean(&s), 0.0);
    }

    #[test]
    fn test_clamp() {
        /* values above 1 should be clamped */
        let mut s = PalmSplayState::default();
        ps_set_all(&mut s, 5.0);
        assert!((s.index - 1.0).abs() < 1e-6);
    }
}
