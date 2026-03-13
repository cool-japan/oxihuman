// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export channel maps (roughness, metallic, AO) for PBR materials.

/// Channel map type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelType {
    Roughness,
    Metallic,
    AmbientOcclusion,
    Height,
    Opacity,
}

/// A single channel map buffer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChannelMap {
    pub channel_type: ChannelType,
    pub width: u32,
    pub height: u32,
    pub data: Vec<f32>,
}

#[allow(dead_code)]
pub fn new_channel_map(ch: ChannelType, w: u32, h: u32) -> ChannelMap {
    ChannelMap { channel_type: ch, width: w, height: h, data: vec![0.0; (w * h) as usize] }
}

#[allow(dead_code)]
pub fn channel_set_pixel(map: &mut ChannelMap, x: u32, y: u32, value: f32) {
    let idx = (y * map.width + x) as usize;
    if idx < map.data.len() {
        map.data[idx] = value.clamp(0.0, 1.0);
    }
}

#[allow(dead_code)]
pub fn channel_get_pixel(map: &ChannelMap, x: u32, y: u32) -> f32 {
    let idx = (y * map.width + x) as usize;
    if idx < map.data.len() { map.data[idx] } else { 0.0 }
}

#[allow(dead_code)]
pub fn channel_pixel_count(map: &ChannelMap) -> usize {
    map.data.len()
}

#[allow(dead_code)]
pub fn channel_average(map: &ChannelMap) -> f32 {
    if map.data.is_empty() { return 0.0; }
    map.data.iter().sum::<f32>() / map.data.len() as f32
}

#[allow(dead_code)]
pub fn channel_fill(map: &mut ChannelMap, value: f32) {
    let v = value.clamp(0.0, 1.0);
    for d in map.data.iter_mut() { *d = v; }
}

#[allow(dead_code)]
pub fn channel_type_name(ct: ChannelType) -> &'static str {
    match ct {
        ChannelType::Roughness => "roughness",
        ChannelType::Metallic => "metallic",
        ChannelType::AmbientOcclusion => "ao",
        ChannelType::Height => "height",
        ChannelType::Opacity => "opacity",
    }
}

#[allow(dead_code)]
pub fn channel_to_bytes(map: &ChannelMap) -> Vec<u8> {
    map.data.iter().map(|&v| (v.clamp(0.0, 1.0) * 255.0) as u8).collect()
}

#[allow(dead_code)]
pub fn channel_map_to_json(map: &ChannelMap) -> String {
    format!(
        r#"{{"type":"{}","width":{},"height":{},"pixels":{}}}"#,
        channel_type_name(map.channel_type), map.width, map.height, map.data.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_map() {
        let m = new_channel_map(ChannelType::Roughness, 4, 4);
        assert_eq!(channel_pixel_count(&m), 16);
    }

    #[test]
    fn test_set_get_pixel() {
        let mut m = new_channel_map(ChannelType::Metallic, 2, 2);
        channel_set_pixel(&mut m, 1, 0, 0.75);
        assert!((channel_get_pixel(&m, 1, 0) - 0.75).abs() < 1e-5);
    }

    #[test]
    fn test_clamp() {
        let mut m = new_channel_map(ChannelType::Height, 1, 1);
        channel_set_pixel(&mut m, 0, 0, 2.0);
        assert!((channel_get_pixel(&m, 0, 0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_average() {
        let mut m = new_channel_map(ChannelType::AmbientOcclusion, 2, 1);
        channel_set_pixel(&mut m, 0, 0, 0.5);
        channel_set_pixel(&mut m, 1, 0, 1.0);
        assert!((channel_average(&m) - 0.75).abs() < 1e-5);
    }

    #[test]
    fn test_fill() {
        let mut m = new_channel_map(ChannelType::Roughness, 3, 3);
        channel_fill(&mut m, 0.5);
        assert!((channel_average(&m) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_type_name() {
        assert_eq!(channel_type_name(ChannelType::Opacity), "opacity");
    }

    #[test]
    fn test_to_bytes() {
        let mut m = new_channel_map(ChannelType::Roughness, 1, 1);
        channel_set_pixel(&mut m, 0, 0, 1.0);
        let bytes = channel_to_bytes(&m);
        assert_eq!(bytes[0], 255);
    }

    #[test]
    fn test_to_json() {
        let m = new_channel_map(ChannelType::Metallic, 2, 2);
        let json = channel_map_to_json(&m);
        assert!(json.contains("metallic"));
    }

    #[test]
    fn test_out_of_bounds() {
        let m = new_channel_map(ChannelType::Height, 2, 2);
        assert!((channel_get_pixel(&m, 99, 99) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_empty_average() {
        let m = new_channel_map(ChannelType::Roughness, 0, 0);
        assert!((channel_average(&m) - 0.0).abs() < 1e-5);
    }

}
