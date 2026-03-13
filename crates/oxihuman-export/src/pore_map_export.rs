// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct PoreMap {
    pub width: u32,
    pub height: u32,
    pub pore_mask: Vec<bool>,
    pub pore_sizes: Vec<f32>,
}

pub fn new_pore_map(w: u32, h: u32) -> PoreMap {
    let n = (w * h) as usize;
    PoreMap {
        width: w,
        height: h,
        pore_mask: vec![false; n],
        pore_sizes: vec![0.0; n],
    }
}

fn idx(m: &PoreMap, x: u32, y: u32) -> usize {
    (y * m.width + x) as usize
}

pub fn pore_set(m: &mut PoreMap, x: u32, y: u32, is_pore: bool, size: f32) {
    let i = idx(m, x, y);
    m.pore_mask[i] = is_pore;
    m.pore_sizes[i] = size;
}

pub fn pore_get(m: &PoreMap, x: u32, y: u32) -> (bool, f32) {
    let i = idx(m, x, y);
    (m.pore_mask[i], m.pore_sizes[i])
}

pub fn pore_count(m: &PoreMap) -> usize {
    m.pore_mask.iter().filter(|&&b| b).count()
}

pub fn pore_mean_size(m: &PoreMap) -> f32 {
    let pores: Vec<f32> = m
        .pore_mask
        .iter()
        .zip(m.pore_sizes.iter())
        .filter(|(&is_p, _)| is_p)
        .map(|(_, &s)| s)
        .collect();
    if pores.is_empty() {
        return 0.0;
    }
    pores.iter().sum::<f32>() / pores.len() as f32
}

pub fn pore_density(m: &PoreMap) -> f32 {
    let total = (m.width * m.height) as usize;
    if total == 0 {
        return 0.0;
    }
    pore_count(m) as f32 / total as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pore_map() {
        /* correct size */
        let m = new_pore_map(4, 4);
        assert_eq!(m.pore_mask.len(), 16);
    }

    #[test]
    fn test_set_get() {
        /* round-trip */
        let mut m = new_pore_map(2, 2);
        pore_set(&mut m, 1, 0, true, 0.5);
        let (is_p, sz) = pore_get(&m, 1, 0);
        assert!(is_p);
        assert!((sz - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_pore_count() {
        /* count pores */
        let mut m = new_pore_map(2, 1);
        pore_set(&mut m, 0, 0, true, 1.0);
        assert_eq!(pore_count(&m), 1);
    }

    #[test]
    fn test_mean_size() {
        /* mean of two pores */
        let mut m = new_pore_map(3, 1);
        pore_set(&mut m, 0, 0, true, 1.0);
        pore_set(&mut m, 1, 0, true, 3.0);
        assert!((pore_mean_size(&m) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_pore_density() {
        /* half pores */
        let mut m = new_pore_map(2, 1);
        pore_set(&mut m, 0, 0, true, 1.0);
        assert!((pore_density(&m) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_no_pores() {
        /* no pores => density 0 */
        let m = new_pore_map(4, 4);
        assert!((pore_density(&m)).abs() < 1e-6);
    }
}
