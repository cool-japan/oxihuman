// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export a set of keyframes for multiple channels.

/// A single keyframe value.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct KeyframeValue {
    pub time: f32,
    pub value: f32,
}

/// A named channel with keyframes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct KeyframeChannel {
    pub name: String,
    pub keys: Vec<KeyframeValue>,
}

/// A keyframe set export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct KeyframeSetExport {
    pub channels: Vec<KeyframeChannel>,
}

/// Create a new keyframe set.
#[allow(dead_code)]
pub fn new_keyframe_set_export() -> KeyframeSetExport {
    KeyframeSetExport {
        channels: Vec::new(),
    }
}

/// Add a channel.
#[allow(dead_code)]
pub fn add_channel(export: &mut KeyframeSetExport, channel: KeyframeChannel) {
    export.channels.push(channel);
}

/// Count channels.
#[allow(dead_code)]
pub fn channel_count_ks(export: &KeyframeSetExport) -> usize {
    export.channels.len()
}

/// Total keyframes across all channels.
#[allow(dead_code)]
pub fn total_keyframe_count(export: &KeyframeSetExport) -> usize {
    export.channels.iter().map(|c| c.keys.len()).sum()
}

/// Find channel by name.
#[allow(dead_code)]
pub fn find_channel_ks<'a>(
    export: &'a KeyframeSetExport,
    name: &str,
) -> Option<&'a KeyframeChannel> {
    export.channels.iter().find(|c| c.name == name)
}

/// Duration = max time across all channels.
#[allow(dead_code)]
pub fn keyframe_set_duration(export: &KeyframeSetExport) -> f32 {
    export
        .channels
        .iter()
        .flat_map(|c| c.keys.iter().map(|k| k.time))
        .fold(0.0f32, f32::max)
}

/// Sample a channel at time t (linear interpolation).
#[allow(dead_code)]
pub fn sample_channel_ks(channel: &KeyframeChannel, t: f32) -> f32 {
    if channel.keys.is_empty() {
        return 0.0;
    }
    if channel.keys.len() == 1 {
        return channel.keys[0].value;
    }
    let idx = channel
        .keys
        .partition_point(|k| k.time <= t)
        .saturating_sub(1);
    let i0 = idx.min(channel.keys.len() - 1);
    let i1 = (idx + 1).min(channel.keys.len() - 1);
    if i0 == i1 {
        return channel.keys[i0].value;
    }
    let k0 = channel.keys[i0];
    let k1 = channel.keys[i1];
    let dt = k1.time - k0.time;
    if dt <= 0.0 {
        return k0.value;
    }
    let f = ((t - k0.time) / dt).clamp(0.0, 1.0);
    k0.value + f * (k1.value - k0.value)
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn keyframe_set_to_json(export: &KeyframeSetExport) -> String {
    format!(
        "{{\"channels\":{},\"total_keys\":{}}}",
        export.channels.len(),
        total_keyframe_count(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn linear_channel() -> KeyframeChannel {
        KeyframeChannel {
            name: "tx".to_string(),
            keys: vec![
                KeyframeValue {
                    time: 0.0,
                    value: 0.0,
                },
                KeyframeValue {
                    time: 1.0,
                    value: 1.0,
                },
            ],
        }
    }

    #[test]
    fn test_add_and_count() {
        let mut e = new_keyframe_set_export();
        add_channel(&mut e, linear_channel());
        assert_eq!(channel_count_ks(&e), 1);
    }

    #[test]
    fn test_total_keyframe_count() {
        let mut e = new_keyframe_set_export();
        add_channel(&mut e, linear_channel());
        assert_eq!(total_keyframe_count(&e), 2);
    }

    #[test]
    fn test_find_channel() {
        let mut e = new_keyframe_set_export();
        add_channel(&mut e, linear_channel());
        assert!(find_channel_ks(&e, "tx").is_some());
    }

    #[test]
    fn test_duration() {
        let mut e = new_keyframe_set_export();
        add_channel(&mut e, linear_channel());
        assert!((keyframe_set_duration(&e) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_sample_at_start() {
        let c = linear_channel();
        assert!(sample_channel_ks(&c, 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_sample_at_end() {
        let c = linear_channel();
        assert!((sample_channel_ks(&c, 1.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_sample_at_mid() {
        let c = linear_channel();
        assert!((sample_channel_ks(&c, 0.5) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_keyframe_set_to_json() {
        let e = new_keyframe_set_export();
        let j = keyframe_set_to_json(&e);
        assert!(j.contains("channels"));
    }

    #[test]
    fn test_sample_empty_channel() {
        let c = KeyframeChannel {
            name: "x".to_string(),
            keys: vec![],
        };
        assert!(sample_channel_ks(&c, 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_empty_duration_zero() {
        let e = new_keyframe_set_export();
        assert!(keyframe_set_duration(&e).abs() < 1e-6);
    }
}
