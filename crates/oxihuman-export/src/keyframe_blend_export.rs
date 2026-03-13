// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Keyframe blending mode and weight export.

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyBlendMode {
    Linear,
    Ease,
    Constant,
    Bezier,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct KeyframeBlendEntry {
    pub time: f32,
    pub value: f32,
    pub blend_mode: KeyBlendMode,
    pub blend_weight: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct KeyframeBlendExport {
    pub channel_name: String,
    pub keys: Vec<KeyframeBlendEntry>,
}

#[allow(dead_code)]
pub fn new_keyframe_blend_export(channel: &str) -> KeyframeBlendExport {
    KeyframeBlendExport {
        channel_name: channel.to_string(),
        keys: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_blend_key(exp: &mut KeyframeBlendExport, time: f32, value: f32, mode: KeyBlendMode) {
    exp.keys.push(KeyframeBlendEntry {
        time,
        value,
        blend_mode: mode,
        blend_weight: 1.0,
    });
}

#[allow(dead_code)]
pub fn key_count_kbe(exp: &KeyframeBlendExport) -> usize {
    exp.keys.len()
}

#[allow(dead_code)]
pub fn channel_duration(exp: &KeyframeBlendExport) -> f32 {
    exp.keys.iter().map(|k| k.time).fold(0.0_f32, f32::max)
}

#[allow(dead_code)]
pub fn keys_of_mode(exp: &KeyframeBlendExport, mode: KeyBlendMode) -> usize {
    exp.keys.iter().filter(|k| k.blend_mode == mode).count()
}

#[allow(dead_code)]
pub fn sort_keys_by_time(exp: &mut KeyframeBlendExport) {
    exp.keys.sort_by(|a, b| {
        a.time
            .partial_cmp(&b.time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

#[allow(dead_code)]
pub fn sample_linear(exp: &KeyframeBlendExport, t: f32) -> f32 {
    if exp.keys.is_empty() {
        return 0.0;
    }
    let before: Vec<&KeyframeBlendEntry> = exp.keys.iter().filter(|k| k.time <= t).collect();
    let after: Vec<&KeyframeBlendEntry> = exp.keys.iter().filter(|k| k.time > t).collect();
    match (before.last(), after.first()) {
        (Some(a), Some(b)) => {
            let dt = b.time - a.time;
            if dt.abs() < 1e-6 {
                a.value
            } else {
                let alpha = (t - a.time) / dt;
                a.value + alpha * (b.value - a.value)
            }
        }
        (Some(a), None) => a.value,
        (None, Some(b)) => b.value,
        _ => 0.0,
    }
}

#[allow(dead_code)]
pub fn keyframe_blend_to_json(exp: &KeyframeBlendExport) -> String {
    format!(
        "{{\"channel\":\"{}\",\"key_count\":{}}}",
        exp.channel_name,
        key_count_kbe(exp)
    )
}

#[allow(dead_code)]
pub fn blend_mode_name(mode: KeyBlendMode) -> &'static str {
    match mode {
        KeyBlendMode::Linear => "linear",
        KeyBlendMode::Ease => "ease",
        KeyBlendMode::Constant => "constant",
        KeyBlendMode::Bezier => "bezier",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let exp = new_keyframe_blend_export("pos_x");
        assert_eq!(key_count_kbe(&exp), 0);
    }

    #[test]
    fn test_add_key() {
        let mut exp = new_keyframe_blend_export("pos_x");
        add_blend_key(&mut exp, 0.0, 0.0, KeyBlendMode::Linear);
        assert_eq!(key_count_kbe(&exp), 1);
    }

    #[test]
    fn test_channel_duration() {
        let mut exp = new_keyframe_blend_export("pos_x");
        add_blend_key(&mut exp, 0.0, 0.0, KeyBlendMode::Linear);
        add_blend_key(&mut exp, 2.0, 1.0, KeyBlendMode::Linear);
        assert!((channel_duration(&exp) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_keys_of_mode() {
        let mut exp = new_keyframe_blend_export("rot_y");
        add_blend_key(&mut exp, 0.0, 0.0, KeyBlendMode::Linear);
        add_blend_key(&mut exp, 1.0, 0.5, KeyBlendMode::Ease);
        assert_eq!(keys_of_mode(&exp, KeyBlendMode::Ease), 1);
    }

    #[test]
    fn test_sort_keys() {
        let mut exp = new_keyframe_blend_export("x");
        add_blend_key(&mut exp, 2.0, 1.0, KeyBlendMode::Linear);
        add_blend_key(&mut exp, 0.5, 0.5, KeyBlendMode::Linear);
        sort_keys_by_time(&mut exp);
        assert!((exp.keys[0].time - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_sample_linear_midpoint() {
        let mut exp = new_keyframe_blend_export("x");
        add_blend_key(&mut exp, 0.0, 0.0, KeyBlendMode::Linear);
        add_blend_key(&mut exp, 1.0, 1.0, KeyBlendMode::Linear);
        let v = sample_linear(&exp, 0.5);
        assert!((v - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_blend_mode_name() {
        assert_eq!(blend_mode_name(KeyBlendMode::Bezier), "bezier");
    }

    #[test]
    fn test_json_output() {
        let exp = new_keyframe_blend_export("scale");
        let j = keyframe_blend_to_json(&exp);
        assert!(j.contains("channel"));
    }

    #[test]
    fn test_sample_empty_zero() {
        let exp = new_keyframe_blend_export("x");
        assert!((sample_linear(&exp, 0.5)).abs() < 1e-6);
    }

    #[test]
    fn test_channel_name_stored() {
        let exp = new_keyframe_blend_export("my_channel");
        assert_eq!(exp.channel_name, "my_channel".to_string());
    }

    #[test]
    fn test_constant_mode_name() {
        assert_eq!(blend_mode_name(KeyBlendMode::Constant), "constant");
    }
}
