// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A single haptic feedback frame.
#[derive(Clone)]
pub struct HapticFrame {
    pub time_s: f32,
    pub force_n: [f32; 3],
    pub torque_nm: [f32; 3],
    pub vibration_hz: f32,
}

pub fn new_haptic_frame(t: f32, force: [f32; 3], vibration_hz: f32) -> HapticFrame {
    HapticFrame {
        time_s: t,
        force_n: force,
        torque_nm: [0.0; 3],
        vibration_hz,
    }
}

pub fn haptic_frame_to_bytes(frame: &HapticFrame) -> Vec<u8> {
    let mut out = Vec::with_capacity(7 * 4);
    out.extend_from_slice(&frame.time_s.to_le_bytes());
    for &v in &frame.force_n {
        out.extend_from_slice(&v.to_le_bytes());
    }
    for &v in &frame.torque_nm {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out.extend_from_slice(&frame.vibration_hz.to_le_bytes());
    out
}

pub fn haptic_sequence_to_bytes(frames: &[HapticFrame]) -> Vec<u8> {
    let mut out = Vec::with_capacity(frames.len() * 7 * 4);
    for frame in frames {
        out.extend(haptic_frame_to_bytes(frame));
    }
    out
}

pub fn haptic_max_force(frames: &[HapticFrame]) -> f32 {
    frames
        .iter()
        .map(|f| {
            let v = f.force_n;
            (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
        })
        .fold(0.0f32, f32::max)
}

pub fn haptic_frame_duration(frames: &[HapticFrame]) -> f32 {
    if frames.len() < 2 {
        return 0.0;
    }
    frames.last().map_or(0.0, |f| f.time_s) - frames.first().map_or(0.0, |f| f.time_s)
}

pub fn haptic_frame_count(frames: &[HapticFrame]) -> usize {
    frames.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_haptic_frame() {
        let f = new_haptic_frame(0.0, [1.0, 0.0, 0.0], 200.0);
        assert!((f.force_n[0] - 1.0).abs() < 1e-5);
        assert!((f.vibration_hz - 200.0).abs() < 1e-4);
    }

    #[test]
    fn test_haptic_frame_to_bytes_len() {
        /* time_s(4) + force_n(12) + torque_nm(12) + vibration_hz(4) = 32 */
        let f = new_haptic_frame(0.0, [0.0; 3], 0.0);
        assert_eq!(haptic_frame_to_bytes(&f).len(), 32);
    }

    #[test]
    fn test_haptic_sequence_to_bytes_len() {
        /* 2 frames * 32 bytes = 64 */
        let frames = vec![
            new_haptic_frame(0.0, [0.0; 3], 100.0),
            new_haptic_frame(1.0, [0.0; 3], 100.0),
        ];
        assert_eq!(haptic_sequence_to_bytes(&frames).len(), 64);
    }

    #[test]
    fn test_haptic_max_force() {
        let frames = vec![
            new_haptic_frame(0.0, [3.0, 4.0, 0.0], 100.0),
            new_haptic_frame(1.0, [1.0, 0.0, 0.0], 100.0),
        ];
        assert!((haptic_max_force(&frames) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_haptic_frame_duration() {
        let frames = vec![
            new_haptic_frame(0.1, [0.0; 3], 100.0),
            new_haptic_frame(2.1, [0.0; 3], 100.0),
        ];
        assert!((haptic_frame_duration(&frames) - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_haptic_frame_count() {
        let frames = vec![new_haptic_frame(0.0, [0.0; 3], 100.0); 5];
        assert_eq!(haptic_frame_count(&frames), 5);
    }

    #[test]
    fn test_haptic_frame_duration_single() {
        let frames = vec![new_haptic_frame(1.0, [0.0; 3], 100.0)];
        assert!((haptic_frame_duration(&frames) - 0.0).abs() < 1e-6);
    }
}
