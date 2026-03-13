// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct MelaninMap {
    pub width: u32,
    pub height: u32,
    pub eumelanin: Vec<f32>,
    pub pheomelanin: Vec<f32>,
}

pub fn new_melanin_map(w: u32, h: u32) -> MelaninMap {
    let n = (w * h) as usize;
    MelaninMap {
        width: w,
        height: h,
        eumelanin: vec![0.0; n],
        pheomelanin: vec![0.0; n],
    }
}

fn idx(m: &MelaninMap, x: u32, y: u32) -> usize {
    (y * m.width + x) as usize
}

pub fn melanin_map_set(m: &mut MelaninMap, x: u32, y: u32, eu: f32, ph: f32) {
    let i = idx(m, x, y);
    m.eumelanin[i] = eu;
    m.pheomelanin[i] = ph;
}

pub fn melanin_map_get(m: &MelaninMap, x: u32, y: u32) -> (f32, f32) {
    let i = idx(m, x, y);
    (m.eumelanin[i], m.pheomelanin[i])
}

pub fn melanin_map_total(m: &MelaninMap, x: u32, y: u32) -> f32 {
    let (eu, ph) = melanin_map_get(m, x, y);
    eu + ph
}

pub fn melanin_map_to_bytes(m: &MelaninMap) -> Vec<u8> {
    m.eumelanin
        .iter()
        .zip(m.pheomelanin.iter())
        .flat_map(|(&eu, &ph)| {
            let eu_b = (eu.clamp(0.0, 1.0) * 255.0) as u8;
            let ph_b = (ph.clamp(0.0, 1.0) * 255.0) as u8;
            [eu_b, ph_b, 0u8, 255u8]
        })
        .collect()
}

pub fn melanin_map_mean_eu(m: &MelaninMap) -> f32 {
    if m.eumelanin.is_empty() {
        return 0.0;
    }
    m.eumelanin.iter().sum::<f32>() / m.eumelanin.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_melanin_map() {
        /* correct size */
        let m = new_melanin_map(4, 4);
        assert_eq!(m.width, 4);
        assert_eq!(m.eumelanin.len(), 16);
    }

    #[test]
    fn test_set_get() {
        /* round-trip */
        let mut m = new_melanin_map(2, 2);
        melanin_map_set(&mut m, 1, 0, 0.8, 0.2);
        let (eu, ph) = melanin_map_get(&m, 1, 0);
        assert!((eu - 0.8).abs() < 1e-6);
        assert!((ph - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_total() {
        /* eu + ph */
        let mut m = new_melanin_map(2, 2);
        melanin_map_set(&mut m, 0, 0, 0.3, 0.4);
        assert!((melanin_map_total(&m, 0, 0) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_to_bytes_length() {
        /* 4 bytes per pixel */
        let m = new_melanin_map(3, 3);
        let bytes = melanin_map_to_bytes(&m);
        assert_eq!(bytes.len(), 3 * 3 * 4);
    }

    #[test]
    fn test_mean_eu() {
        /* mean */
        let mut m = new_melanin_map(2, 1);
        melanin_map_set(&mut m, 0, 0, 0.0, 0.0);
        melanin_map_set(&mut m, 1, 0, 1.0, 0.0);
        assert!((melanin_map_mean_eu(&m) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_bytes_values() {
        /* max eu => 255 in byte 0 */
        let mut m = new_melanin_map(1, 1);
        melanin_map_set(&mut m, 0, 0, 1.0, 0.0);
        let bytes = melanin_map_to_bytes(&m);
        assert_eq!(bytes[0], 255);
    }
}
