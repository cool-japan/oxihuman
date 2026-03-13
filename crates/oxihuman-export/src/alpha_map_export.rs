// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Alpha map export: per-pixel alpha channel image buffer.

/// Alpha map buffer (single-channel f32).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AlphaMapExport {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<f32>,
}

/// Create a new alpha map buffer, all zeros.
#[allow(dead_code)]
pub fn new_alpha_map(width: usize, height: usize) -> AlphaMapExport {
    AlphaMapExport {
        width,
        height,
        pixels: vec![0.0; width * height],
    }
}

/// Set alpha at (x, y).
#[allow(dead_code)]
pub fn set_alpha(am: &mut AlphaMapExport, x: usize, y: usize, value: f32) {
    if let Some(px) = am.pixels.get_mut(y * am.width + x) {
        *px = value.clamp(0.0, 1.0);
    }
}

/// Get alpha at (x, y).
#[allow(dead_code)]
pub fn get_alpha(am: &AlphaMapExport, x: usize, y: usize) -> Option<f32> {
    am.pixels.get(y * am.width + x).copied()
}

/// Total pixel count.
#[allow(dead_code)]
pub fn alpha_pixel_count(am: &AlphaMapExport) -> usize {
    am.pixels.len()
}

/// Average alpha value.
#[allow(dead_code)]
pub fn average_alpha(am: &AlphaMapExport) -> f32 {
    if am.pixels.is_empty() {
        return 0.0;
    }
    am.pixels.iter().sum::<f32>() / am.pixels.len() as f32
}

/// Fill entire buffer with a value.
#[allow(dead_code)]
pub fn fill_alpha(am: &mut AlphaMapExport, value: f32) {
    let v = value.clamp(0.0, 1.0);
    for px in &mut am.pixels {
        *px = v;
    }
}

/// Invert alpha (1.0 - value).
#[allow(dead_code)]
pub fn invert_alpha(am: &mut AlphaMapExport) {
    for px in &mut am.pixels {
        *px = 1.0 - *px;
    }
}

/// Validate: all pixels in [0, 1].
#[allow(dead_code)]
pub fn validate_alpha_map(am: &AlphaMapExport) -> bool {
    am.pixels.iter().all(|&p| (0.0..=1.0).contains(&p))
}

/// Encode to PGM bytes (8-bit grayscale).
#[allow(dead_code)]
pub fn encode_pgm(am: &AlphaMapExport) -> Vec<u8> {
    let header = format!("P5\n{} {}\n255\n", am.width, am.height);
    let mut out: Vec<u8> = header.into_bytes();
    for &p in &am.pixels {
        out.push((p.clamp(0.0, 1.0) * 255.0) as u8);
    }
    out
}

/// Export to JSON summary.
#[allow(dead_code)]
pub fn alpha_map_to_json(am: &AlphaMapExport) -> String {
    format!(
        "{{\"width\":{},\"height\":{},\"average_alpha\":{:.6}}}",
        am.width,
        am.height,
        average_alpha(am)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_alpha_map() {
        let am = new_alpha_map(4, 4);
        assert_eq!(alpha_pixel_count(&am), 16);
    }

    #[test]
    fn test_set_get_alpha() {
        let mut am = new_alpha_map(4, 4);
        set_alpha(&mut am, 2, 1, 0.7);
        let v = get_alpha(&am, 2, 1).expect("should succeed");
        assert!((v - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_get_alpha_oob() {
        let am = new_alpha_map(2, 2);
        assert!(get_alpha(&am, 10, 10).is_none());
    }

    #[test]
    fn test_average_alpha_zero() {
        let am = new_alpha_map(2, 2);
        assert!((average_alpha(&am)).abs() < 1e-9);
    }

    #[test]
    fn test_fill_alpha() {
        let mut am = new_alpha_map(2, 2);
        fill_alpha(&mut am, 0.5);
        assert!((average_alpha(&am) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_invert_alpha() {
        let mut am = new_alpha_map(2, 2);
        fill_alpha(&mut am, 0.3);
        invert_alpha(&mut am);
        assert!((average_alpha(&am) - 0.7).abs() < 1e-5);
    }

    #[test]
    fn test_validate_alpha_map() {
        let am = new_alpha_map(2, 2);
        assert!(validate_alpha_map(&am));
    }

    #[test]
    fn test_encode_pgm_header() {
        let am = new_alpha_map(2, 2);
        let pgm = encode_pgm(&am);
        assert!(!pgm.is_empty());
        assert!(pgm.starts_with(b"P5"));
    }

    #[test]
    fn test_alpha_map_to_json() {
        let am = new_alpha_map(3, 3);
        let j = alpha_map_to_json(&am);
        assert!(j.contains("\"width\":3"));
    }

    #[test]
    fn test_alpha_value_in_range() {
        let mut am = new_alpha_map(1, 1);
        set_alpha(&mut am, 0, 0, 0.5);
        let v = get_alpha(&am, 0, 0).expect("should succeed");
        assert!((0.0..=1.0).contains(&v));
    }
}
