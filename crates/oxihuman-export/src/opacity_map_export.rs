// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct OpacityMap {
    pub width: u32,
    pub height: u32,
    pub data: Vec<f32>,
}

pub fn new_opacity_map(w: u32, h: u32) -> OpacityMap {
    OpacityMap {
        width: w,
        height: h,
        data: vec![1.0; (w * h) as usize],
    }
}

pub fn opacity_set(m: &mut OpacityMap, x: u32, y: u32, v: f32) {
    if x < m.width && y < m.height {
        m.data[(y * m.width + x) as usize] = v.clamp(0.0, 1.0);
    }
}

pub fn opacity_get(m: &OpacityMap, x: u32, y: u32) -> f32 {
    if x < m.width && y < m.height {
        m.data[(y * m.width + x) as usize]
    } else {
        0.0
    }
}

pub fn opacity_to_u8(m: &OpacityMap) -> Vec<u8> {
    m.data.iter().map(|&v| (v * 255.0) as u8).collect()
}

pub fn opacity_mean(m: &OpacityMap) -> f32 {
    if m.data.is_empty() {
        return 0.0;
    }
    m.data.iter().sum::<f32>() / m.data.len() as f32
}

pub fn opacity_threshold_mask(m: &OpacityMap, thr: f32) -> Vec<bool> {
    m.data.iter().map(|&v| v >= thr).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_opacity_map() {
        /* dimensions and default data */
        let m = new_opacity_map(4, 4);
        assert_eq!(m.data.len(), 16);
        assert!((m.data[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_opacity_set_get() {
        /* set and get roundtrip */
        let mut m = new_opacity_map(4, 4);
        opacity_set(&mut m, 2, 2, 0.3);
        assert!((opacity_get(&m, 2, 2) - 0.3).abs() < 1e-5);
    }

    #[test]
    fn test_opacity_to_u8() {
        /* converts to u8 range */
        let m = new_opacity_map(2, 2);
        let b = opacity_to_u8(&m);
        assert_eq!(b.len(), 4);
        assert_eq!(b[0], 255);
    }

    #[test]
    fn test_opacity_mean() {
        /* mean of all-1 map is 1 */
        let m = new_opacity_map(3, 3);
        assert!((opacity_mean(&m) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_opacity_threshold_mask() {
        /* threshold at 0.5 on all-1 map */
        let m = new_opacity_map(2, 2);
        let mask = opacity_threshold_mask(&m, 0.5);
        assert!(mask.iter().all(|&b| b));
    }

    #[test]
    fn test_opacity_get_oob() {
        /* out-of-bounds returns 0 */
        let m = new_opacity_map(2, 2);
        assert_eq!(opacity_get(&m, 99, 99), 0.0);
    }
}
