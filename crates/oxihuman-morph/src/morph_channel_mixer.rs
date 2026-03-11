#![allow(dead_code)]
//! Channel mixer for blending multiple morph channels with individual weights.

use std::collections::HashMap;

/// A mixer that combines multiple named morph channels.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChannelMixer {
    channels: Vec<(String, f32)>,
}

/// Create a new empty [`ChannelMixer`].
#[allow(dead_code)]
pub fn new_channel_mixer() -> ChannelMixer {
    ChannelMixer {
        channels: Vec::new(),
    }
}

/// Add a named channel with the given weight.
#[allow(dead_code)]
pub fn add_channel_cm(mixer: &mut ChannelMixer, name: &str, weight: f32) {
    mixer.channels.push((name.to_string(), weight));
}

/// Mix all channels, returning a map of channel name to weighted value.
#[allow(dead_code)]
pub fn mix_channels(mixer: &ChannelMixer) -> HashMap<String, f32> {
    let total: f32 = mixer.channels.iter().map(|(_, w)| w.abs()).sum();
    let mut result = HashMap::new();
    if total < 1e-9 {
        return result;
    }
    for (name, weight) in &mixer.channels {
        let entry = result.entry(name.clone()).or_insert(0.0);
        *entry += weight / total;
    }
    result
}

/// Return the number of channels.
#[allow(dead_code)]
pub fn channel_count_cm(mixer: &ChannelMixer) -> usize {
    mixer.channels.len()
}

/// Return the weight of the channel at `index`, or 0.0 if out of range.
#[allow(dead_code)]
pub fn channel_weight_cm(mixer: &ChannelMixer, index: usize) -> f32 {
    mixer.channels.get(index).map_or(0.0, |(_, w)| *w)
}

/// Set the weight of the channel at `index`. No-op if out of range.
#[allow(dead_code)]
pub fn set_channel_weight_cm(mixer: &mut ChannelMixer, index: usize, weight: f32) {
    if let Some(entry) = mixer.channels.get_mut(index) {
        entry.1 = weight;
    }
}

/// Serialize the mixer state to a JSON-like string.
#[allow(dead_code)]
pub fn mixer_to_json(mixer: &ChannelMixer) -> String {
    let entries: Vec<String> = mixer
        .channels
        .iter()
        .map(|(n, w)| format!("{{\"name\":\"{n}\",\"weight\":{w}}}"))
        .collect();
    format!("{{\"channels\":[{}]}}", entries.join(","))
}

/// Remove all channels from the mixer.
#[allow(dead_code)]
pub fn mixer_clear(mixer: &mut ChannelMixer) {
    mixer.channels.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_channel_mixer() {
        let m = new_channel_mixer();
        assert_eq!(channel_count_cm(&m), 0);
    }

    #[test]
    fn test_add_channel() {
        let mut m = new_channel_mixer();
        add_channel_cm(&mut m, "smile", 0.5);
        assert_eq!(channel_count_cm(&m), 1);
    }

    #[test]
    fn test_channel_weight() {
        let mut m = new_channel_mixer();
        add_channel_cm(&mut m, "smile", 0.8);
        assert!((channel_weight_cm(&m, 0) - 0.8).abs() < 1e-6);
        assert!((channel_weight_cm(&m, 99) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_channel_weight() {
        let mut m = new_channel_mixer();
        add_channel_cm(&mut m, "frown", 0.3);
        set_channel_weight_cm(&mut m, 0, 0.9);
        assert!((channel_weight_cm(&m, 0) - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_mix_channels_empty() {
        let m = new_channel_mixer();
        let result = mix_channels(&m);
        assert!(result.is_empty());
    }

    #[test]
    fn test_mix_channels_single() {
        let mut m = new_channel_mixer();
        add_channel_cm(&mut m, "jaw", 1.0);
        let result = mix_channels(&m);
        assert!((result["jaw"] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_mixer_to_json() {
        let mut m = new_channel_mixer();
        add_channel_cm(&mut m, "a", 1.0);
        let json = mixer_to_json(&m);
        assert!(json.contains("\"name\":\"a\""));
    }

    #[test]
    fn test_mixer_clear() {
        let mut m = new_channel_mixer();
        add_channel_cm(&mut m, "x", 0.5);
        mixer_clear(&mut m);
        assert_eq!(channel_count_cm(&m), 0);
    }

    #[test]
    fn test_mix_channels_multiple() {
        let mut m = new_channel_mixer();
        add_channel_cm(&mut m, "a", 0.5);
        add_channel_cm(&mut m, "b", 0.5);
        let result = mix_channels(&m);
        assert!((result["a"] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_weight_out_of_range() {
        let mut m = new_channel_mixer();
        set_channel_weight_cm(&mut m, 10, 1.0);
        assert_eq!(channel_count_cm(&m), 0);
    }
}
