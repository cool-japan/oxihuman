// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct RoughnessMetalnessMap {
    pub width: u32,
    pub height: u32,
    pub roughness: Vec<f32>,
    pub metalness: Vec<f32>,
}

pub fn new_roughness_metalness_map(w: u32, h: u32) -> RoughnessMetalnessMap {
    let n = (w * h) as usize;
    RoughnessMetalnessMap {
        width: w,
        height: h,
        roughness: vec![0.5; n],
        metalness: vec![0.0; n],
    }
}

pub fn rm_set(m: &mut RoughnessMetalnessMap, x: u32, y: u32, r: f32, met: f32) {
    if x < m.width && y < m.height {
        let idx = (y * m.width + x) as usize;
        m.roughness[idx] = r.clamp(0.0, 1.0);
        m.metalness[idx] = met.clamp(0.0, 1.0);
    }
}

pub fn rm_get(m: &RoughnessMetalnessMap, x: u32, y: u32) -> (f32, f32) {
    if x < m.width && y < m.height {
        let idx = (y * m.width + x) as usize;
        (m.roughness[idx], m.metalness[idx])
    } else {
        (0.5, 0.0)
    }
}

pub fn rm_mean_roughness(m: &RoughnessMetalnessMap) -> f32 {
    if m.roughness.is_empty() {
        return 0.0;
    }
    m.roughness.iter().sum::<f32>() / m.roughness.len() as f32
}

pub fn rm_to_bytes(m: &RoughnessMetalnessMap) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&m.width.to_le_bytes());
    b.extend_from_slice(&m.height.to_le_bytes());
    for (&r, &met) in m.roughness.iter().zip(m.metalness.iter()) {
        b.push((r * 255.0) as u8);
        b.push((met * 255.0) as u8);
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_roughness_metalness_map() {
        /* dimensions stored */
        let m = new_roughness_metalness_map(4, 4);
        assert_eq!(m.width, 4);
        assert_eq!(m.roughness.len(), 16);
    }

    #[test]
    fn test_rm_set_get() {
        /* set and get roundtrip */
        let mut m = new_roughness_metalness_map(4, 4);
        rm_set(&mut m, 1, 1, 0.8, 0.3);
        let (r, met) = rm_get(&m, 1, 1);
        assert!((r - 0.8).abs() < 1e-5);
        assert!((met - 0.3).abs() < 1e-5);
    }

    #[test]
    fn test_rm_mean_roughness() {
        /* mean roughness of all-0.5 map */
        let m = new_roughness_metalness_map(2, 2);
        assert!((rm_mean_roughness(&m) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_rm_to_bytes() {
        /* bytes non-empty */
        let m = new_roughness_metalness_map(2, 2);
        let b = rm_to_bytes(&m);
        assert!(!b.is_empty());
    }

    #[test]
    fn test_rm_get_oob() {
        /* out-of-bounds returns defaults */
        let m = new_roughness_metalness_map(2, 2);
        let (r, _) = rm_get(&m, 99, 99);
        assert!((r - 0.5).abs() < 1e-5);
    }
}
