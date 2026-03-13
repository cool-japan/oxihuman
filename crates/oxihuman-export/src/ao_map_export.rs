// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Ambient occlusion map export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AoMapConfig {
    pub width: usize,
    pub height: usize,
    pub samples: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AoMapExport {
    pub pixels: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

#[allow(dead_code)]
pub fn default_ao_map_config() -> AoMapConfig {
    AoMapConfig {
        width: 256,
        height: 256,
        samples: 16,
    }
}
#[allow(dead_code)]
pub fn new_ao_map(config: &AoMapConfig) -> AoMapExport {
    AoMapExport {
        pixels: vec![1.0; config.width * config.height],
        width: config.width,
        height: config.height,
    }
}
#[allow(dead_code)]
pub fn ao_set_pixel(map: &mut AoMapExport, x: usize, y: usize, val: f32) {
    if x < map.width && y < map.height {
        map.pixels[y * map.width + x] = val.clamp(0.0, 1.0);
    }
}
#[allow(dead_code)]
pub fn ao_get_pixel(map: &AoMapExport, x: usize, y: usize) -> f32 {
    if x < map.width && y < map.height {
        map.pixels[y * map.width + x]
    } else {
        0.0
    }
}
#[allow(dead_code)]
pub fn ao_pixel_count(map: &AoMapExport) -> usize {
    map.pixels.len()
}
#[allow(dead_code)]
pub fn ao_average(map: &AoMapExport) -> f32 {
    if map.pixels.is_empty() {
        0.0
    } else {
        map.pixels.iter().sum::<f32>() / map.pixels.len() as f32
    }
}
#[allow(dead_code)]
pub fn ao_to_bytes(map: &AoMapExport) -> Vec<u8> {
    map.pixels.iter().map(|&v| (v * 255.0) as u8).collect()
}
#[allow(dead_code)]
pub fn ao_to_json(map: &AoMapExport) -> String {
    format!(
        "{{\"width\":{},\"height\":{},\"pixels\":{}}}",
        map.width,
        map.height,
        map.pixels.len()
    )
}
#[allow(dead_code)]
pub fn ao_validate(map: &AoMapExport) -> bool {
    map.pixels.len() == map.width * map.height && map.pixels.iter().all(|v| (0.0..=1.0).contains(v))
}

// ── New required API ──────────────────────────────────────────────────────────

pub struct AoMap {
    pub width: u32,
    pub height: u32,
    pub data: Vec<f32>,
}

pub fn new_ao_map_req(w: u32, h: u32) -> AoMap {
    AoMap {
        width: w,
        height: h,
        data: vec![1.0; (w * h) as usize],
    }
}

pub fn ao_set(m: &mut AoMap, x: u32, y: u32, v: f32) {
    if x < m.width && y < m.height {
        m.data[(y * m.width + x) as usize] = v.clamp(0.0, 1.0);
    }
}

pub fn ao_get(m: &AoMap, x: u32, y: u32) -> f32 {
    if x < m.width && y < m.height {
        m.data[(y * m.width + x) as usize]
    } else {
        0.0
    }
}

pub fn ao_to_u8(m: &AoMap) -> Vec<u8> {
    m.data.iter().map(|&v| (v * 255.0) as u8).collect()
}

pub fn ao_mean(m: &AoMap) -> f32 {
    if m.data.is_empty() {
        return 0.0;
    }
    m.data.iter().sum::<f32>() / m.data.len() as f32
}

pub fn ao_to_bytes_req(m: &AoMap) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&m.width.to_le_bytes());
    b.extend_from_slice(&m.height.to_le_bytes());
    for &v in &m.data {
        b.extend_from_slice(&v.to_le_bytes());
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_default() {
        let c = default_ao_map_config();
        assert_eq!(c.width, 256);
    }
    #[test]
    fn test_new() {
        let m = new_ao_map(&default_ao_map_config());
        assert_eq!(m.pixels.len(), 256 * 256);
    }
    #[test]
    fn test_set_get() {
        let mut m = new_ao_map(&AoMapConfig {
            width: 4,
            height: 4,
            samples: 1,
        });
        ao_set_pixel(&mut m, 1, 1, 0.5);
        assert!((ao_get_pixel(&m, 1, 1) - 0.5).abs() < 1e-6);
    }
    #[test]
    fn test_pixel_count() {
        let m = new_ao_map(&AoMapConfig {
            width: 4,
            height: 4,
            samples: 1,
        });
        assert_eq!(ao_pixel_count(&m), 16);
    }
    #[test]
    fn test_average() {
        let m = new_ao_map(&AoMapConfig {
            width: 2,
            height: 2,
            samples: 1,
        });
        assert!((ao_average(&m) - 1.0).abs() < 1e-6);
    }
    #[test]
    fn test_to_bytes() {
        let m = new_ao_map(&AoMapConfig {
            width: 2,
            height: 2,
            samples: 1,
        });
        let b = ao_to_bytes(&m);
        assert_eq!(b.len(), 4);
    }
    #[test]
    fn test_to_json() {
        let m = new_ao_map(&AoMapConfig {
            width: 2,
            height: 2,
            samples: 1,
        });
        assert!(ao_to_json(&m).contains("width"));
    }
    #[test]
    fn test_validate() {
        let m = new_ao_map(&AoMapConfig {
            width: 2,
            height: 2,
            samples: 1,
        });
        assert!(ao_validate(&m));
    }
    #[test]
    fn test_clamp() {
        let mut m = new_ao_map(&AoMapConfig {
            width: 2,
            height: 2,
            samples: 1,
        });
        ao_set_pixel(&mut m, 0, 0, 2.0);
        assert!((ao_get_pixel(&m, 0, 0) - 1.0).abs() < 1e-6);
    }
    #[test]
    fn test_oob() {
        let m = new_ao_map(&AoMapConfig {
            width: 2,
            height: 2,
            samples: 1,
        });
        assert!((ao_get_pixel(&m, 99, 99)).abs() < 1e-6);
    }
}
