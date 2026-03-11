// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct IorMap {
    pub width: u32,
    pub height: u32,
    pub data: Vec<f32>,
}

pub fn new_ior_map(w: u32, h: u32, default_ior: f32) -> IorMap {
    IorMap {
        width: w,
        height: h,
        data: vec![default_ior; (w * h) as usize],
    }
}

pub fn ior_set(m: &mut IorMap, x: u32, y: u32, v: f32) {
    if x < m.width && y < m.height {
        m.data[(y * m.width + x) as usize] = v;
    }
}

pub fn ior_get(m: &IorMap, x: u32, y: u32) -> f32 {
    if x < m.width && y < m.height {
        m.data[(y * m.width + x) as usize]
    } else {
        1.0
    }
}

pub fn ior_mean(m: &IorMap) -> f32 {
    if m.data.is_empty() {
        return 1.0;
    }
    m.data.iter().sum::<f32>() / m.data.len() as f32
}

pub fn ior_is_valid(ior: f32) -> bool {
    (1.0..=3.0).contains(&ior)
}

pub fn ior_to_bytes(m: &IorMap) -> Vec<u8> {
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
    fn test_new_ior_map() {
        /* default ior set */
        let m = new_ior_map(4, 4, 1.5);
        assert!((m.data[0] - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_ior_set_get() {
        /* set and get roundtrip */
        let mut m = new_ior_map(4, 4, 1.0);
        ior_set(&mut m, 1, 1, 1.8);
        assert!((ior_get(&m, 1, 1) - 1.8).abs() < 1e-5);
    }

    #[test]
    fn test_ior_mean() {
        /* mean of constant map */
        let m = new_ior_map(3, 3, 1.5);
        assert!((ior_mean(&m) - 1.5).abs() < 1e-5);
    }

    #[test]
    fn test_ior_is_valid() {
        /* valid range 1-3 */
        assert!(ior_is_valid(1.5));
        assert!(!ior_is_valid(0.5));
        assert!(!ior_is_valid(3.5));
    }

    #[test]
    fn test_ior_to_bytes() {
        /* bytes non-empty */
        let m = new_ior_map(2, 2, 1.5);
        let b = ior_to_bytes(&m);
        assert!(!b.is_empty());
    }

    #[test]
    fn test_ior_get_oob() {
        /* out-of-bounds returns 1.0 */
        let m = new_ior_map(2, 2, 1.5);
        assert!((ior_get(&m, 99, 99) - 1.0).abs() < 1e-6);
    }
}
