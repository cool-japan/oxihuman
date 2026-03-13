// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export camera shake parameters and keyframe data.

use std::f32::consts::TAU;

/// Camera shake channel type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShakeChannel {
    TranslateX,
    TranslateY,
    TranslateZ,
    RotateX,
    RotateY,
    RotateZ,
}

/// A single shake keyframe.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShakeKeyframe {
    pub time: f32,
    pub amplitude: f32,
    pub frequency: f32,
    pub channel: ShakeChannel,
}

/// Camera shake export data.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct CameraShakeExport {
    pub keyframes: Vec<ShakeKeyframe>,
    pub duration: f32,
    pub decay: f32,
}

/// Create a new camera shake export.
#[allow(dead_code)]
pub fn new_camera_shake(duration: f32, decay: f32) -> CameraShakeExport {
    CameraShakeExport {
        keyframes: vec![],
        duration,
        decay,
    }
}

/// Add a keyframe.
#[allow(dead_code)]
pub fn add_shake_keyframe(
    export: &mut CameraShakeExport,
    time: f32,
    amplitude: f32,
    frequency: f32,
    channel: ShakeChannel,
) {
    export.keyframes.push(ShakeKeyframe {
        time,
        amplitude,
        frequency,
        channel,
    });
}

/// Evaluate the shake displacement at a given time using sine oscillation.
#[allow(dead_code)]
pub fn evaluate_shake(export: &CameraShakeExport, time: f32, channel: ShakeChannel) -> f32 {
    export
        .keyframes
        .iter()
        .filter(|k| k.channel == channel)
        .map(|k| {
            let envelope = (-export.decay * (time - k.time).max(0.0)).exp();
            k.amplitude * (TAU * k.frequency * time).sin() * envelope
        })
        .sum()
}

/// Count keyframes on a specific channel.
#[allow(dead_code)]
pub fn channel_keyframe_count(export: &CameraShakeExport, channel: ShakeChannel) -> usize {
    export
        .keyframes
        .iter()
        .filter(|k| k.channel == channel)
        .count()
}

/// Peak amplitude across all keyframes.
#[allow(dead_code)]
pub fn peak_amplitude(export: &CameraShakeExport) -> f32 {
    export
        .keyframes
        .iter()
        .map(|k| k.amplitude.abs())
        .fold(0.0_f32, f32::max)
}

/// Serialise shake export to a flat buffer.
#[allow(dead_code)]
pub fn serialise_shake(export: &CameraShakeExport) -> Vec<f32> {
    let mut buf = vec![export.duration, export.decay];
    for k in &export.keyframes {
        buf.push(k.time);
        buf.push(k.amplitude);
        buf.push(k.frequency);
    }
    buf
}

/// Check if shake has decayed below threshold at `time`.
#[allow(dead_code)]
pub fn is_decayed(export: &CameraShakeExport, time: f32, threshold: f32) -> bool {
    if export.keyframes.is_empty() {
        return true;
    }
    let max_env = (-export.decay * time).exp();
    max_env * peak_amplitude(export) < threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basic_shake() -> CameraShakeExport {
        let mut s = new_camera_shake(2.0, 1.0);
        add_shake_keyframe(&mut s, 0.0, 0.5, 10.0, ShakeChannel::TranslateX);
        s
    }

    #[test]
    fn test_new_camera_shake() {
        let s = new_camera_shake(1.5, 0.5);
        assert!((s.duration - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_add_shake_keyframe() {
        let s = basic_shake();
        assert_eq!(s.keyframes.len(), 1);
    }

    #[test]
    fn test_channel_count() {
        let s = basic_shake();
        assert_eq!(channel_keyframe_count(&s, ShakeChannel::TranslateX), 1);
        assert_eq!(channel_keyframe_count(&s, ShakeChannel::TranslateY), 0);
    }

    #[test]
    fn test_peak_amplitude() {
        let s = basic_shake();
        assert!((peak_amplitude(&s) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_zero_time() {
        let s = basic_shake();
        let v = evaluate_shake(&s, 0.0, ShakeChannel::TranslateX);
        assert!(v.abs() < 1e-5); // sin(0) = 0
    }

    #[test]
    fn test_is_decayed_far_future() {
        let s = basic_shake();
        assert!(is_decayed(&s, 100.0, 0.01));
    }

    #[test]
    fn test_is_not_decayed_at_start() {
        let s = basic_shake();
        assert!(!is_decayed(&s, 0.0, 0.1));
    }

    #[test]
    fn test_serialise_length() {
        let s = basic_shake();
        let buf = serialise_shake(&s);
        assert_eq!(buf.len(), 2 + 3); // duration, decay + 3 per keyframe
    }

    #[test]
    fn test_serialise_empty() {
        let s = new_camera_shake(1.0, 1.0);
        assert_eq!(serialise_shake(&s).len(), 2);
    }
}
