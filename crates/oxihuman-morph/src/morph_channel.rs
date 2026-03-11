#![allow(dead_code)]

/// A named morph channel with a blendable weight.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphChannel {
    name: String,
    weight: f32,
    active: bool,
}

/// Blend mode for combining channels.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelBlend {
    Additive,
    Override,
    Multiply,
}

/// Create a new morph channel with zero weight.
#[allow(dead_code)]
pub fn new_morph_channel(name: &str) -> MorphChannel {
    MorphChannel {
        name: name.to_string(),
        weight: 0.0,
        active: true,
    }
}

/// Set the weight of a channel (clamped to 0..=1).
#[allow(dead_code)]
pub fn channel_set_weight(ch: &mut MorphChannel, w: f32) {
    ch.weight = w.clamp(0.0, 1.0);
}

/// Get the current weight of a channel.
#[allow(dead_code)]
pub fn channel_get_weight(ch: &MorphChannel) -> f32 {
    ch.weight
}

/// Blend two channel weights using the given blend mode.
#[allow(dead_code)]
pub fn channel_blend(a: f32, b: f32, mode: ChannelBlend) -> f32 {
    match mode {
        ChannelBlend::Additive => (a + b).clamp(0.0, 1.0),
        ChannelBlend::Override => b,
        ChannelBlend::Multiply => a * b,
    }
}

/// Check whether the channel is active.
#[allow(dead_code)]
pub fn channel_is_active(ch: &MorphChannel) -> bool {
    ch.active
}

/// Get the name of the channel.
#[allow(dead_code)]
pub fn channel_name(ch: &MorphChannel) -> &str {
    &ch.name
}

/// Reset the channel to zero weight.
#[allow(dead_code)]
pub fn channel_reset(ch: &mut MorphChannel) {
    ch.weight = 0.0;
}

/// Return the number of channels in a slice.
#[allow(dead_code)]
pub fn channel_count(channels: &[MorphChannel]) -> usize {
    channels.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_morph_channel() {
        let ch = new_morph_channel("smile");
        assert_eq!(channel_name(&ch), "smile");
        assert_eq!(channel_get_weight(&ch), 0.0);
    }

    #[test]
    fn test_channel_set_weight() {
        let mut ch = new_morph_channel("a");
        channel_set_weight(&mut ch, 0.5);
        assert!((channel_get_weight(&ch) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_channel_set_weight_clamps() {
        let mut ch = new_morph_channel("a");
        channel_set_weight(&mut ch, 2.0);
        assert!((channel_get_weight(&ch) - 1.0).abs() < 1e-6);
        channel_set_weight(&mut ch, -1.0);
        assert_eq!(channel_get_weight(&ch), 0.0);
    }

    #[test]
    fn test_channel_blend_additive() {
        let r = channel_blend(0.3, 0.5, ChannelBlend::Additive);
        assert!((r - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_channel_blend_override() {
        let r = channel_blend(0.3, 0.7, ChannelBlend::Override);
        assert!((r - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_channel_blend_multiply() {
        let r = channel_blend(0.5, 0.4, ChannelBlend::Multiply);
        assert!((r - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_channel_is_active() {
        let ch = new_morph_channel("x");
        assert!(channel_is_active(&ch));
    }

    #[test]
    fn test_channel_reset() {
        let mut ch = new_morph_channel("x");
        channel_set_weight(&mut ch, 0.8);
        channel_reset(&mut ch);
        assert_eq!(channel_get_weight(&ch), 0.0);
    }

    #[test]
    fn test_channel_count() {
        let chs = vec![new_morph_channel("a"), new_morph_channel("b")];
        assert_eq!(channel_count(&chs), 2);
    }

    #[test]
    fn test_channel_blend_additive_clamps() {
        let r = channel_blend(0.8, 0.5, ChannelBlend::Additive);
        assert!((r - 1.0).abs() < 1e-6);
    }
}
