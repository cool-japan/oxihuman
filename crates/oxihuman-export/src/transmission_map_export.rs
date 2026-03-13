// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct TransmissionMap {
    pub width: u32,
    pub height: u32,
    pub data: Vec<f32>,
}

pub fn new_transmission_map(w: u32, h: u32) -> TransmissionMap {
    TransmissionMap {
        width: w,
        height: h,
        data: vec![0.0; (w * h) as usize],
    }
}

pub fn trans_set(m: &mut TransmissionMap, x: u32, y: u32, v: f32) {
    if x < m.width && y < m.height {
        m.data[(y * m.width + x) as usize] = v.clamp(0.0, 1.0);
    }
}

pub fn trans_get(m: &TransmissionMap, x: u32, y: u32) -> f32 {
    if x < m.width && y < m.height {
        m.data[(y * m.width + x) as usize]
    } else {
        0.0
    }
}

pub fn trans_mean(m: &TransmissionMap) -> f32 {
    if m.data.is_empty() {
        return 0.0;
    }
    m.data.iter().sum::<f32>() / m.data.len() as f32
}

pub fn trans_threshold_mask(m: &TransmissionMap, thr: f32) -> Vec<bool> {
    m.data.iter().map(|&v| v >= thr).collect()
}

pub fn trans_to_bytes(m: &TransmissionMap) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&m.width.to_le_bytes());
    b.extend_from_slice(&m.height.to_le_bytes());
    for &v in &m.data {
        b.push((v * 255.0) as u8);
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_transmission_map() {
        /* initialized to zero */
        let m = new_transmission_map(4, 4);
        assert_eq!(m.data[0], 0.0);
    }

    #[test]
    fn test_trans_set_get() {
        /* set and get roundtrip */
        let mut m = new_transmission_map(4, 4);
        trans_set(&mut m, 1, 1, 0.7);
        assert!((trans_get(&m, 1, 1) - 0.7).abs() < 1e-5);
    }

    #[test]
    fn test_trans_mean_zero() {
        /* all-zero mean */
        let m = new_transmission_map(3, 3);
        assert!(trans_mean(&m).abs() < 1e-6);
    }

    #[test]
    fn test_trans_threshold_mask() {
        /* threshold at 0 on zero map gives all-false */
        let mut m = new_transmission_map(2, 2);
        trans_set(&mut m, 0, 0, 0.8);
        let mask = trans_threshold_mask(&m, 0.5);
        assert!(mask[0]);
        assert!(!mask[1]);
    }

    #[test]
    fn test_trans_to_bytes() {
        /* bytes correct size */
        let m = new_transmission_map(2, 2);
        let b = trans_to_bytes(&m);
        /* 8 bytes header + 4 pixels */
        assert_eq!(b.len(), 12);
    }

    #[test]
    fn test_trans_get_oob() {
        /* out-of-bounds returns 0 */
        let m = new_transmission_map(2, 2);
        assert_eq!(trans_get(&m, 99, 99), 0.0);
    }
}
