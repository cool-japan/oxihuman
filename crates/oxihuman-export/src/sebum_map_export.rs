// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct SebumMap {
    pub width: u32,
    pub height: u32,
    pub data: Vec<f32>,
}

pub fn new_sebum_map(w: u32, h: u32) -> SebumMap {
    let n = (w * h) as usize;
    SebumMap {
        width: w,
        height: h,
        data: vec![0.0; n],
    }
}

fn idx(m: &SebumMap, x: u32, y: u32) -> usize {
    (y * m.width + x) as usize
}

pub fn sebum_set(m: &mut SebumMap, x: u32, y: u32, v: f32) {
    let i = idx(m, x, y);
    m.data[i] = v;
}

pub fn sebum_get(m: &SebumMap, x: u32, y: u32) -> f32 {
    m.data[idx(m, x, y)]
}

pub fn sebum_mean(m: &SebumMap) -> f32 {
    if m.data.is_empty() {
        return 0.0;
    }
    m.data.iter().sum::<f32>() / m.data.len() as f32
}

pub fn sebum_to_bytes(m: &SebumMap) -> Vec<u8> {
    m.data
        .iter()
        .flat_map(|&v| {
            let b = (v.clamp(0.0, 1.0) * 255.0) as u8;
            [b, b, b, 255u8]
        })
        .collect()
}

/// T-zone detection: pixels above threshold.
pub fn sebum_zones(m: &SebumMap, threshold: f32) -> Vec<bool> {
    m.data.iter().map(|&v| v >= threshold).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sebum_map() {
        /* correct size */
        let m = new_sebum_map(4, 4);
        assert_eq!(m.data.len(), 16);
    }

    #[test]
    fn test_set_get() {
        /* round-trip */
        let mut m = new_sebum_map(2, 2);
        sebum_set(&mut m, 1, 1, 0.75);
        assert!((sebum_get(&m, 1, 1) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_mean() {
        /* mean */
        let mut m = new_sebum_map(2, 1);
        sebum_set(&mut m, 0, 0, 0.0);
        sebum_set(&mut m, 1, 0, 1.0);
        assert!((sebum_mean(&m) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_bytes_length() {
        /* 4 bytes per pixel */
        let m = new_sebum_map(3, 3);
        let bytes = sebum_to_bytes(&m);
        assert_eq!(bytes.len(), 3 * 3 * 4);
    }

    #[test]
    fn test_zones() {
        /* above threshold */
        let mut m = new_sebum_map(2, 1);
        sebum_set(&mut m, 0, 0, 0.3);
        sebum_set(&mut m, 1, 0, 0.8);
        let zones = sebum_zones(&m, 0.5);
        assert!(!zones[0]);
        assert!(zones[1]);
    }

    #[test]
    fn test_mean_empty() {
        /* empty => 0 */
        let m = SebumMap {
            width: 0,
            height: 0,
            data: vec![],
        };
        assert!((sebum_mean(&m)).abs() < 1e-6);
    }
}
