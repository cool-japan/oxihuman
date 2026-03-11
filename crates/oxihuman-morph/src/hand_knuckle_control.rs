// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hand knuckle prominence and definition control.

/// Finger index (0 = index, 1 = middle, 2 = ring, 3 = pinky).
pub const FINGER_COUNT: usize = 4;

/// State per hand (one hand, use two for bilateral).
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct KnuckleState {
    /// Knuckle prominence for each finger [index, middle, ring, pinky] (0..1).
    pub prominence: [f32; FINGER_COUNT],
    /// Knuckle definition (sharpness) per finger (0..1).
    pub definition: [f32; FINGER_COUNT],
}

/// Config.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct KnuckleConfig {
    pub max_prominence: f32,
}

impl Default for KnuckleConfig {
    fn default() -> Self {
        Self {
            max_prominence: 1.0,
        }
    }
}
impl Default for KnuckleState {
    fn default() -> Self {
        Self {
            prominence: [0.0; FINGER_COUNT],
            definition: [0.5; FINGER_COUNT],
        }
    }
}

#[allow(dead_code)]
pub fn new_knuckle_state() -> KnuckleState {
    KnuckleState::default()
}

#[allow(dead_code)]
pub fn default_knuckle_config() -> KnuckleConfig {
    KnuckleConfig::default()
}

#[allow(dead_code)]
pub fn kk_set_prominence(state: &mut KnuckleState, cfg: &KnuckleConfig, finger: usize, v: f32) {
    if finger < FINGER_COUNT {
        state.prominence[finger] = v.clamp(0.0, cfg.max_prominence);
    }
}

#[allow(dead_code)]
pub fn kk_set_definition(state: &mut KnuckleState, finger: usize, v: f32) {
    if finger < FINGER_COUNT {
        state.definition[finger] = v.clamp(0.0, 1.0);
    }
}

#[allow(dead_code)]
pub fn kk_set_all_prominence(state: &mut KnuckleState, cfg: &KnuckleConfig, v: f32) {
    let v = v.clamp(0.0, cfg.max_prominence);
    #[allow(clippy::needless_range_loop)]
    for i in 0..FINGER_COUNT {
        state.prominence[i] = v;
    }
}

#[allow(dead_code)]
pub fn kk_reset(state: &mut KnuckleState) {
    *state = KnuckleState::default();
}

#[allow(dead_code)]
pub fn kk_average_prominence(state: &KnuckleState) -> f32 {
    state.prominence.iter().sum::<f32>() / FINGER_COUNT as f32
}

#[allow(dead_code)]
pub fn kk_blend(a: &KnuckleState, b: &KnuckleState, t: f32) -> KnuckleState {
    let t = t.clamp(0.0, 1.0);
    let mut out = KnuckleState::default();
    #[allow(clippy::needless_range_loop)]
    for i in 0..FINGER_COUNT {
        out.prominence[i] = a.prominence[i] + (b.prominence[i] - a.prominence[i]) * t;
        out.definition[i] = a.definition[i] + (b.definition[i] - a.definition[i]) * t;
    }
    out
}

#[allow(dead_code)]
pub fn kk_is_neutral(state: &KnuckleState) -> bool {
    state.prominence.iter().all(|&v| v < 1e-4)
}

#[allow(dead_code)]
pub fn kk_to_weights(state: &KnuckleState) -> Vec<f32> {
    let mut w = Vec::with_capacity(FINGER_COUNT * 2);
    for &p in &state.prominence {
        w.push(p);
    }
    for &d in &state.definition {
        w.push(d);
    }
    w
}

#[allow(dead_code)]
pub fn kk_to_json(state: &KnuckleState) -> String {
    let p: Vec<String> = state
        .prominence
        .iter()
        .map(|v| format!("{:.4}", v))
        .collect();
    let d: Vec<String> = state
        .definition
        .iter()
        .map(|v| format!("{:.4}", v))
        .collect();
    format!(
        "{{\"prominence\":[{}],\"definition\":[{}]}}",
        p.join(","),
        d.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_neutral() {
        assert!(kk_is_neutral(&new_knuckle_state()));
    }

    #[test]
    fn set_prominence_clamps() {
        let mut s = new_knuckle_state();
        let cfg = default_knuckle_config();
        kk_set_prominence(&mut s, &cfg, 0, 5.0);
        assert!(s.prominence[0] <= cfg.max_prominence);
    }

    #[test]
    fn out_of_range_finger_ignored() {
        let mut s = new_knuckle_state();
        let cfg = default_knuckle_config();
        kk_set_prominence(&mut s, &cfg, 99, 1.0);
        assert!(kk_is_neutral(&s));
    }

    #[test]
    fn set_all_prominence() {
        let mut s = new_knuckle_state();
        let cfg = default_knuckle_config();
        kk_set_all_prominence(&mut s, &cfg, 0.7);
        assert!(s.prominence.iter().all(|&v| (v - 0.7).abs() < 1e-5));
    }

    #[test]
    fn reset_neutral() {
        let mut s = new_knuckle_state();
        let cfg = default_knuckle_config();
        kk_set_all_prominence(&mut s, &cfg, 1.0);
        kk_reset(&mut s);
        assert!(kk_is_neutral(&s));
    }

    #[test]
    fn average_prominence_zero() {
        assert!((kk_average_prominence(&new_knuckle_state())).abs() < 1e-5);
    }

    #[test]
    fn blend_midpoint() {
        let cfg = default_knuckle_config();
        let mut a = new_knuckle_state();
        let mut b = new_knuckle_state();
        kk_set_prominence(&mut a, &cfg, 0, 0.0);
        kk_set_prominence(&mut b, &cfg, 0, 1.0);
        let m = kk_blend(&a, &b, 0.5);
        assert!((m.prominence[0] - 0.5).abs() < 1e-4);
    }

    #[test]
    fn weights_len() {
        assert_eq!(kk_to_weights(&new_knuckle_state()).len(), FINGER_COUNT * 2);
    }

    #[test]
    fn json_has_prominence() {
        assert!(kk_to_json(&new_knuckle_state()).contains("prominence"));
    }

    #[test]
    fn definition_clamped() {
        let mut s = new_knuckle_state();
        kk_set_definition(&mut s, 1, 5.0);
        assert!(s.definition[1] <= 1.0);
    }
}
