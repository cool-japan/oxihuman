// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]

/// Channel-packed texture export (e.g., ORM = Occlusion, Roughness, Metallic).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChannelPackExport {
    pub width: u32,
    pub height: u32,
    pub channels: Vec<ChannelSource>,
    pub pixels: Vec<u8>,
}

/// Source for a single channel.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChannelSource {
    pub name: String,
    pub data: Vec<u8>,
}

#[allow(dead_code)]
impl ChannelPackExport {
    /// Create from separate channel data (up to 4 channels → RGBA).
    pub fn pack(width: u32, height: u32, channels: Vec<ChannelSource>) -> Self {
        let pixel_count = (width * height) as usize;
        let chan_count = channels.len().min(4);
        let mut pixels = vec![0u8; pixel_count * chan_count];
        #[allow(clippy::needless_range_loop)]
        for c in 0..chan_count {
            for i in 0..pixel_count {
                let val = if i < channels[c].data.len() { channels[c].data[i] } else { 0 };
                pixels[i * chan_count + c] = val;
            }
        }
        Self { width, height, channels, pixels }
    }

    /// Number of channels.
    pub fn channel_count(&self) -> usize {
        self.channels.len().min(4)
    }

    /// Pixel count.
    pub fn pixel_count(&self) -> usize {
        (self.width * self.height) as usize
    }

    /// Total byte size.
    pub fn byte_size(&self) -> usize {
        self.pixels.len()
    }

    /// Get pixel value at (x, y, channel).
    pub fn get_pixel(&self, x: u32, y: u32, channel: usize) -> u8 {
        let idx = (y * self.width + x) as usize;
        let cc = self.channel_count();
        self.pixels[idx * cc + channel]
    }
}

/// Export channel pack to raw bytes.
#[allow(dead_code)]
pub fn export_channel_pack_bytes(pack: &ChannelPackExport) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&pack.width.to_le_bytes());
    out.extend_from_slice(&pack.height.to_le_bytes());
    out.push(pack.channel_count() as u8);
    out.extend_from_slice(&pack.pixels);
    out
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn channel_pack_to_json(pack: &ChannelPackExport) -> String {
    let names: Vec<&str> = pack.channels.iter().map(|c| c.name.as_str()).collect();
    format!(
        "{{\"width\":{},\"height\":{},\"channels\":{},\"names\":{:?},\"bytes\":{}}}",
        pack.width, pack.height, pack.channel_count(), names, pack.byte_size()
    )
}

/// Create a solid channel.
#[allow(dead_code)]
pub fn solid_channel(name: &str, size: usize, value: u8) -> ChannelSource {
    ChannelSource { name: name.to_string(), data: vec![value; size] }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_channels() -> Vec<ChannelSource> {
        let size = 4;
        vec![
            solid_channel("R", size, 255),
            solid_channel("G", size, 128),
            solid_channel("B", size, 0),
        ]
    }

    #[test]
    fn test_pack() {
        let pack = ChannelPackExport::pack(2, 2, test_channels());
        assert_eq!(pack.channel_count(), 3);
        assert_eq!(pack.pixel_count(), 4);
    }

    #[test]
    fn test_get_pixel() {
        let pack = ChannelPackExport::pack(2, 2, test_channels());
        assert_eq!(pack.get_pixel(0, 0, 0), 255);
        assert_eq!(pack.get_pixel(0, 0, 1), 128);
        assert_eq!(pack.get_pixel(0, 0, 2), 0);
    }

    #[test]
    fn test_byte_size() {
        let pack = ChannelPackExport::pack(2, 2, test_channels());
        assert_eq!(pack.byte_size(), 12);
    }

    #[test]
    fn test_export_bytes() {
        let pack = ChannelPackExport::pack(2, 2, test_channels());
        let bytes = export_channel_pack_bytes(&pack);
        assert!(bytes.len() > 12);
    }

    #[test]
    fn test_to_json() {
        let pack = ChannelPackExport::pack(2, 2, test_channels());
        let json = channel_pack_to_json(&pack);
        assert!(json.contains("width"));
    }

    #[test]
    fn test_single_channel() {
        let pack = ChannelPackExport::pack(1, 1, vec![solid_channel("A", 1, 200)]);
        assert_eq!(pack.channel_count(), 1);
        assert_eq!(pack.get_pixel(0, 0, 0), 200);
    }

    #[test]
    fn test_empty_data() {
        let pack = ChannelPackExport::pack(0, 0, vec![]);
        assert_eq!(pack.pixel_count(), 0);
    }

    #[test]
    fn test_solid_channel() {
        let ch = solid_channel("test", 10, 42);
        assert_eq!(ch.data.len(), 10);
        assert_eq!(ch.data[0], 42);
    }

    #[test]
    fn test_four_channels() {
        let pack = ChannelPackExport::pack(1, 1, vec![
            solid_channel("R", 1, 10),
            solid_channel("G", 1, 20),
            solid_channel("B", 1, 30),
            solid_channel("A", 1, 40),
        ]);
        assert_eq!(pack.channel_count(), 4);
    }
}
