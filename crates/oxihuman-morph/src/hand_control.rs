// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Hand and finger morphology controls for character customization.

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FingerIdx {
    Thumb,
    Index,
    Middle,
    Ring,
    Pinky,
}

impl FingerIdx {
    fn as_usize(self) -> usize {
        match self {
            FingerIdx::Thumb => 0,
            FingerIdx::Index => 1,
            FingerIdx::Middle => 2,
            FingerIdx::Ring => 3,
            FingerIdx::Pinky => 4,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct FingerConfig {
    pub length: f32,
    pub thickness: f32,
    pub knuckle_size: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HandConfig {
    pub finger_configs: [FingerConfig; 5],
    pub hand_width: f32,
    pub palm_length: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HandState {
    pub finger_lengths: [f32; 5],
    pub finger_thickness: [f32; 5],
    pub hand_width: f32,
    pub palm_length: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HandMorphWeights {
    pub long_fingers: f32,
    pub short_fingers: f32,
    pub wide_hand: f32,
    pub narrow_hand: f32,
    pub thick_fingers: f32,
}

#[allow(dead_code)]
pub fn default_finger_config() -> FingerConfig {
    FingerConfig {
        length: 0.5,
        thickness: 0.5,
        knuckle_size: 0.5,
    }
}

#[allow(dead_code)]
pub fn default_hand_config() -> HandConfig {
    let fc = default_finger_config();
    HandConfig {
        finger_configs: [fc; 5],
        hand_width: 0.5,
        palm_length: 0.5,
    }
}

#[allow(dead_code)]
pub fn new_hand_state() -> HandState {
    HandState {
        finger_lengths: [0.5; 5],
        finger_thickness: [0.5; 5],
        hand_width: 0.5,
        palm_length: 0.5,
    }
}

#[allow(dead_code)]
pub fn set_finger_length(state: &mut HandState, finger: FingerIdx, length: f32) {
    state.finger_lengths[finger.as_usize()] = length.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_hand_width(state: &mut HandState, width: f32) {
    state.hand_width = width.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_hand_weights(state: &HandState, _cfg: &HandConfig) -> HandMorphWeights {
    let avg_len = avg_finger_length(state);
    let long_fingers = (avg_len - 0.5).max(0.0) * 2.0;
    let short_fingers = (0.5 - avg_len).max(0.0) * 2.0;
    let wide_hand = (state.hand_width - 0.5).max(0.0) * 2.0;
    let narrow_hand = (0.5 - state.hand_width).max(0.0) * 2.0;
    let avg_thickness = state.finger_thickness.iter().sum::<f32>() / 5.0;
    let thick_fingers = avg_thickness;
    HandMorphWeights {
        long_fingers,
        short_fingers,
        wide_hand,
        narrow_hand,
        thick_fingers,
    }
}

#[allow(dead_code)]
pub fn blend_hands(a: &HandState, b: &HandState, t: f32) -> HandState {
    let t = t.clamp(0.0, 1.0);
    let u = 1.0 - t;
    let mut finger_lengths = [0.0f32; 5];
    let mut finger_thickness = [0.0f32; 5];
    for i in 0..5 {
        finger_lengths[i] = a.finger_lengths[i] * u + b.finger_lengths[i] * t;
        finger_thickness[i] = a.finger_thickness[i] * u + b.finger_thickness[i] * t;
    }
    HandState {
        finger_lengths,
        finger_thickness,
        hand_width: a.hand_width * u + b.hand_width * t,
        palm_length: a.palm_length * u + b.palm_length * t,
    }
}

#[allow(dead_code)]
pub fn reset_hand(state: &mut HandState) {
    *state = new_hand_state();
}

#[allow(dead_code)]
pub fn hand_state_to_json(state: &HandState) -> String {
    let fl: Vec<String> = state.finger_lengths.iter().map(|v| format!("{:.4}", v)).collect();
    let ft: Vec<String> = state.finger_thickness.iter().map(|v| format!("{:.4}", v)).collect();
    format!(
        r#"{{"finger_lengths":[{}],"finger_thickness":[{}],"hand_width":{:.4},"palm_length":{:.4}}}"#,
        fl.join(","),
        ft.join(","),
        state.hand_width,
        state.palm_length
    )
}

#[allow(dead_code)]
pub fn finger_length(state: &HandState, finger: FingerIdx) -> f32 {
    state.finger_lengths[finger.as_usize()]
}

#[allow(dead_code)]
pub fn avg_finger_length(state: &HandState) -> f32 {
    state.finger_lengths.iter().sum::<f32>() / 5.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_finger_config() {
        let fc = default_finger_config();
        assert!((fc.length - 0.5).abs() < 1e-5);
        assert!((fc.thickness - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_new_hand_state() {
        let s = new_hand_state();
        assert_eq!(s.finger_lengths, [0.5; 5]);
        assert!((s.hand_width - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_set_finger_length_clamp() {
        let mut s = new_hand_state();
        set_finger_length(&mut s, FingerIdx::Index, 2.0);
        assert!((s.finger_lengths[1] - 1.0).abs() < 1e-5);
        set_finger_length(&mut s, FingerIdx::Index, -1.0);
        assert!(s.finger_lengths[1].abs() < 1e-5);
    }

    #[test]
    fn test_set_hand_width() {
        let mut s = new_hand_state();
        set_hand_width(&mut s, 0.8);
        assert!((s.hand_width - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_compute_hand_weights_wide() {
        let cfg = default_hand_config();
        let mut s = new_hand_state();
        set_hand_width(&mut s, 1.0);
        let w = compute_hand_weights(&s, &cfg);
        assert!((w.wide_hand - 1.0).abs() < 1e-5);
        assert!(w.narrow_hand.abs() < 1e-5);
    }

    #[test]
    fn test_blend_hands() {
        let a = new_hand_state();
        let mut b = new_hand_state();
        set_finger_length(&mut b, FingerIdx::Thumb, 1.0);
        let blended = blend_hands(&a, &b, 0.5);
        assert!((blended.finger_lengths[0] - 0.75).abs() < 1e-4);
    }

    #[test]
    fn test_reset_hand() {
        let mut s = new_hand_state();
        set_hand_width(&mut s, 0.9);
        reset_hand(&mut s);
        assert!((s.hand_width - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_hand_state_to_json() {
        let s = new_hand_state();
        let json = hand_state_to_json(&s);
        assert!(json.contains("finger_lengths"));
        assert!(json.contains("hand_width"));
    }

    #[test]
    fn test_finger_length_accessor() {
        let mut s = new_hand_state();
        set_finger_length(&mut s, FingerIdx::Middle, 0.8);
        assert!((finger_length(&s, FingerIdx::Middle) - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_avg_finger_length() {
        let s = new_hand_state();
        assert!((avg_finger_length(&s) - 0.5).abs() < 1e-5);
    }
}
