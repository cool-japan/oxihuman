// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct EmissionMap {
    pub width: u32,
    pub height: u32,
    pub data: Vec<[f32; 3]>,
    pub intensity: f32,
}

pub fn new_emission_map(w: u32, h: u32) -> EmissionMap {
    EmissionMap {
        width: w,
        height: h,
        data: vec![[0.0; 3]; (w * h) as usize],
        intensity: 1.0,
    }
}

pub fn emission_set(m: &mut EmissionMap, x: u32, y: u32, rgb: [f32; 3]) {
    if x < m.width && y < m.height {
        m.data[(y * m.width + x) as usize] = rgb;
    }
}

pub fn emission_get(m: &EmissionMap, x: u32, y: u32) -> [f32; 3] {
    if x < m.width && y < m.height {
        m.data[(y * m.width + x) as usize]
    } else {
        [0.0; 3]
    }
}

pub fn emission_to_bytes(m: &EmissionMap) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&m.width.to_le_bytes());
    b.extend_from_slice(&m.height.to_le_bytes());
    for px in &m.data {
        for &c in px {
            b.push((c * m.intensity * 255.0).clamp(0.0, 255.0) as u8);
        }
    }
    b
}

pub fn emission_total_power(m: &EmissionMap) -> f32 {
    m.data
        .iter()
        .map(|px| (px[0] + px[1] + px[2]) * m.intensity)
        .sum()
}

pub fn emission_mean(m: &EmissionMap) -> [f32; 3] {
    if m.data.is_empty() {
        return [0.0; 3];
    }
    let n = m.data.len() as f32;
    [
        m.data.iter().map(|px| px[0]).sum::<f32>() / n,
        m.data.iter().map(|px| px[1]).sum::<f32>() / n,
        m.data.iter().map(|px| px[2]).sum::<f32>() / n,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_emission_map() {
        /* data initialized to zero */
        let m = new_emission_map(4, 4);
        assert_eq!(m.data.len(), 16);
        assert_eq!(m.data[0], [0.0f32; 3]);
    }

    #[test]
    fn test_emission_set_get() {
        /* set and get roundtrip */
        let mut m = new_emission_map(4, 4);
        emission_set(&mut m, 1, 1, [1.0, 0.5, 0.0]);
        let px = emission_get(&m, 1, 1);
        assert!((px[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_emission_total_power_zero() {
        /* all-zero map has zero power */
        let m = new_emission_map(3, 3);
        assert!(emission_total_power(&m).abs() < 1e-6);
    }

    #[test]
    fn test_emission_mean_zero() {
        /* all-zero map has zero mean */
        let m = new_emission_map(2, 2);
        let mean = emission_mean(&m);
        assert_eq!(mean, [0.0f32; 3]);
    }

    #[test]
    fn test_emission_to_bytes() {
        /* bytes non-empty */
        let m = new_emission_map(2, 2);
        let b = emission_to_bytes(&m);
        assert!(!b.is_empty());
    }

    #[test]
    fn test_emission_get_oob() {
        /* out-of-bounds returns zero */
        let m = new_emission_map(2, 2);
        assert_eq!(emission_get(&m, 99, 99), [0.0f32; 3]);
    }
}
