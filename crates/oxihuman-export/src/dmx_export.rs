// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! DMX512 frame export (512-channel byte array).

pub const DMX_CHANNEL_COUNT: usize = 512;

/// A single DMX512 frame.
#[allow(dead_code)]
pub struct DmxFrame {
    pub channels: [u8; DMX_CHANNEL_COUNT],
    pub universe: u8,
}

impl DmxFrame {
    #[allow(dead_code)]
    pub fn new(universe: u8) -> Self {
        Self {
            channels: [0u8; DMX_CHANNEL_COUNT],
            universe,
        }
    }
}

/// Set a single channel value (1-indexed per DMX spec).
#[allow(dead_code)]
pub fn set_channel(frame: &mut DmxFrame, channel: usize, value: u8) {
    if (1..=DMX_CHANNEL_COUNT).contains(&channel) {
        frame.channels[channel - 1] = value;
    }
}

/// Get a channel value.
#[allow(dead_code)]
pub fn get_channel(frame: &DmxFrame, channel: usize) -> u8 {
    if (1..=DMX_CHANNEL_COUNT).contains(&channel) {
        frame.channels[channel - 1]
    } else {
        0
    }
}

/// Set RGB on three consecutive channels starting at `start_channel`.
#[allow(dead_code)]
pub fn set_rgb(frame: &mut DmxFrame, start_channel: usize, r: u8, g: u8, b: u8) {
    set_channel(frame, start_channel, r);
    set_channel(frame, start_channel + 1, g);
    set_channel(frame, start_channel + 2, b);
}

/// Set all channels to a single value.
#[allow(dead_code)]
pub fn fill_frame(frame: &mut DmxFrame, value: u8) {
    frame.channels.fill(value);
}

/// Serialize frame to bytes (512 bytes).
#[allow(dead_code)]
pub fn serialize_dmx_frame(frame: &DmxFrame) -> Vec<u8> {
    frame.channels.to_vec()
}

/// Number of non-zero channels.
#[allow(dead_code)]
pub fn active_channel_count(frame: &DmxFrame) -> usize {
    frame.channels.iter().filter(|&&v| v != 0).count()
}

/// Blend two frames (A over B with alpha).
#[allow(dead_code)]
pub fn blend_frames(a: &DmxFrame, b: &DmxFrame, alpha: f32) -> DmxFrame {
    let mut out = DmxFrame::new(a.universe);
    for i in 0..DMX_CHANNEL_COUNT {
        let va = a.channels[i] as f32;
        let vb = b.channels[i] as f32;
        out.channels[i] = (va * alpha + vb * (1.0 - alpha)) as u8;
    }
    out
}

/// Peak value across all channels.
#[allow(dead_code)]
pub fn peak_channel_value(frame: &DmxFrame) -> u8 {
    *frame.channels.iter().max().unwrap_or(&0)
}

/// Clear the frame (all zeros).
#[allow(dead_code)]
pub fn clear_frame(frame: &mut DmxFrame) {
    frame.channels.fill(0);
}

/// Copy frame data from another frame.
#[allow(dead_code)]
pub fn copy_frame(src: &DmxFrame, dst: &mut DmxFrame) {
    dst.channels.copy_from_slice(&src.channels);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_has_512_channels() {
        let f = DmxFrame::new(0);
        assert_eq!(f.channels.len(), DMX_CHANNEL_COUNT);
    }

    #[test]
    fn set_get_channel_roundtrip() {
        let mut f = DmxFrame::new(0);
        set_channel(&mut f, 1, 200);
        assert_eq!(get_channel(&f, 1), 200);
    }

    #[test]
    fn channel_out_of_range_ignored() {
        let mut f = DmxFrame::new(0);
        set_channel(&mut f, 0, 100);
        set_channel(&mut f, 513, 100);
        assert_eq!(active_channel_count(&f), 0);
    }

    #[test]
    fn set_rgb_sets_three_channels() {
        let mut f = DmxFrame::new(0);
        set_rgb(&mut f, 1, 255, 128, 64);
        assert_eq!(get_channel(&f, 1), 255);
        assert_eq!(get_channel(&f, 2), 128);
        assert_eq!(get_channel(&f, 3), 64);
    }

    #[test]
    fn fill_frame_all_same() {
        let mut f = DmxFrame::new(0);
        fill_frame(&mut f, 42);
        assert!(f.channels.iter().all(|&v| v == 42));
    }

    #[test]
    fn serialize_dmx_frame_512_bytes() {
        let f = DmxFrame::new(0);
        let bytes = serialize_dmx_frame(&f);
        assert_eq!(bytes.len(), 512);
    }

    #[test]
    fn active_channel_count_correct() {
        let mut f = DmxFrame::new(0);
        set_channel(&mut f, 1, 100);
        set_channel(&mut f, 2, 200);
        assert_eq!(active_channel_count(&f), 2);
    }

    #[test]
    fn blend_frames_midpoint() {
        let mut a = DmxFrame::new(0);
        let mut b = DmxFrame::new(0);
        set_channel(&mut a, 1, 200);
        set_channel(&mut b, 1, 100);
        let blended = blend_frames(&a, &b, 0.5);
        let v = get_channel(&blended, 1);
        assert!((140..=160).contains(&v), "blend not near midpoint: {v}");
    }

    #[test]
    fn peak_channel_value_correct() {
        let mut f = DmxFrame::new(0);
        set_channel(&mut f, 10, 123);
        assert_eq!(peak_channel_value(&f), 123);
    }

    #[test]
    fn clear_frame_all_zero() {
        let mut f = DmxFrame::new(0);
        fill_frame(&mut f, 50);
        clear_frame(&mut f);
        assert_eq!(active_channel_count(&f), 0);
    }
}
