// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]

/// Displacement map export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DisplacementMapExport {
    pub width: u32,
    pub height: u32,
    pub data: Vec<f32>,
    pub scale: f32,
}

#[allow(dead_code)]
impl DisplacementMapExport {
    /// Create a flat displacement map.
    pub fn flat(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0.0; (width * height) as usize],
            scale: 1.0,
        }
    }

    /// Create from height data.
    pub fn from_data(width: u32, height: u32, data: Vec<f32>, scale: f32) -> Self {
        Self {
            width,
            height,
            data,
            scale,
        }
    }

    /// Pixel count.
    pub fn pixel_count(&self) -> usize {
        (self.width * self.height) as usize
    }

    /// Get displacement at (x, y).
    pub fn get(&self, x: u32, y: u32) -> f32 {
        self.data[(y * self.width + x) as usize] * self.scale
    }

    /// Set displacement at (x, y).
    pub fn set(&mut self, x: u32, y: u32, value: f32) {
        self.data[(y * self.width + x) as usize] = value;
    }

    /// Min/max displacement.
    pub fn range(&self) -> (f32, f32) {
        let min = self.data.iter().cloned().fold(f32::MAX, f32::min);
        let max = self.data.iter().cloned().fold(f32::MIN, f32::max);
        (min * self.scale, max * self.scale)
    }

    /// Export to 16-bit integer bytes.
    pub fn to_u16_bytes(&self) -> Vec<u8> {
        let (min, max) = self.range();
        let range = (max - min).max(1e-10);
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.width.to_le_bytes());
        bytes.extend_from_slice(&self.height.to_le_bytes());
        for &v in &self.data {
            let norm = ((v * self.scale - min) / range * 65535.0) as u16;
            bytes.extend_from_slice(&norm.to_le_bytes());
        }
        bytes
    }

    /// Byte size of u16 export.
    pub fn u16_byte_size(&self) -> usize {
        8 + self.pixel_count() * 2
    }
}

/// Export to JSON.
#[allow(dead_code)]
pub fn displacement_map_to_json(map: &DisplacementMapExport) -> String {
    let (min, max) = map.range();
    format!(
        "{{\"width\":{},\"height\":{},\"scale\":{},\"min\":{},\"max\":{}}}",
        map.width, map.height, map.scale, min, max
    )
}

/// Validate map.
#[allow(dead_code)]
pub fn validate_displacement_map(map: &DisplacementMapExport) -> bool {
    map.data.len() == map.pixel_count() && map.data.iter().all(|v| v.is_finite())
}

// ── New required API ──────────────────────────────────────────────────────────

pub struct DisplacementMap {
    pub width: u32,
    pub height: u32,
    pub data: Vec<f32>,
    pub midlevel: f32,
    pub strength: f32,
}

pub fn new_displacement_map(w: u32, h: u32) -> DisplacementMap {
    DisplacementMap {
        width: w,
        height: h,
        data: vec![0.0; (w * h) as usize],
        midlevel: 0.5,
        strength: 1.0,
    }
}

pub fn disp_set(m: &mut DisplacementMap, x: u32, y: u32, v: f32) {
    if x < m.width && y < m.height {
        m.data[(y * m.width + x) as usize] = v;
    }
}

pub fn disp_get(m: &DisplacementMap, x: u32, y: u32) -> f32 {
    if x < m.width && y < m.height {
        m.data[(y * m.width + x) as usize]
    } else {
        0.0
    }
}

pub fn disp_to_u16(m: &DisplacementMap) -> Vec<u16> {
    m.data
        .iter()
        .map(|&v| {
            let norm = ((v - m.midlevel) * m.strength * 0.5 + 0.5).clamp(0.0, 1.0);
            (norm * 65535.0) as u16
        })
        .collect()
}

pub fn disp_max_height(m: &DisplacementMap) -> f32 {
    m.data.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
}

pub fn disp_to_bytes(m: &DisplacementMap) -> Vec<u8> {
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
    fn test_flat() {
        let m = DisplacementMapExport::flat(4, 4);
        assert_eq!(m.pixel_count(), 16);
    }

    #[test]
    fn test_get_set() {
        let mut m = DisplacementMapExport::flat(2, 2);
        m.set(0, 0, 0.5);
        assert!((m.get(0, 0) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_range() {
        let m = DisplacementMapExport::from_data(2, 1, vec![0.0, 1.0], 2.0);
        let (min, max) = m.range();
        assert!((min).abs() < 1e-5);
        assert!((max - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_u16_bytes() {
        let m = DisplacementMapExport::flat(2, 2);
        let bytes = m.to_u16_bytes();
        assert_eq!(bytes.len(), m.u16_byte_size());
    }

    #[test]
    fn test_validate() {
        let m = DisplacementMapExport::flat(2, 2);
        assert!(validate_displacement_map(&m));
    }

    #[test]
    fn test_to_json() {
        let m = DisplacementMapExport::flat(2, 2);
        let json = displacement_map_to_json(&m);
        assert!(json.contains("width"));
    }

    #[test]
    fn test_custom_scale() {
        let m = DisplacementMapExport::from_data(1, 1, vec![1.0], 3.0);
        assert!((m.get(0, 0) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_invalid() {
        let m = DisplacementMapExport {
            width: 2,
            height: 2,
            data: vec![0.0],
            scale: 1.0,
        };
        assert!(!validate_displacement_map(&m));
    }

    #[test]
    fn test_pixel_count() {
        let m = DisplacementMapExport::flat(3, 5);
        assert_eq!(m.pixel_count(), 15);
    }

    #[test]
    fn test_u16_byte_size() {
        let m = DisplacementMapExport::flat(4, 4);
        assert_eq!(m.u16_byte_size(), 8 + 32);
    }
}
