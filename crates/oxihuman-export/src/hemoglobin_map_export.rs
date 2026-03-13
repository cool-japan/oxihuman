// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct HemoglobinMap {
    pub width: u32,
    pub height: u32,
    pub oxy: Vec<f32>,
    pub deoxy: Vec<f32>,
}

pub fn new_hemoglobin_map(w: u32, h: u32) -> HemoglobinMap {
    let n = (w * h) as usize;
    HemoglobinMap {
        width: w,
        height: h,
        oxy: vec![0.0; n],
        deoxy: vec![0.0; n],
    }
}

fn idx(m: &HemoglobinMap, x: u32, y: u32) -> usize {
    (y * m.width + x) as usize
}

pub fn hemo_map_set(m: &mut HemoglobinMap, x: u32, y: u32, oxy: f32, deoxy: f32) {
    let i = idx(m, x, y);
    m.oxy[i] = oxy;
    m.deoxy[i] = deoxy;
}

pub fn hemo_map_get(m: &HemoglobinMap, x: u32, y: u32) -> (f32, f32) {
    let i = idx(m, x, y);
    (m.oxy[i], m.deoxy[i])
}

pub fn hemo_map_oxygen_saturation(m: &HemoglobinMap, x: u32, y: u32) -> f32 {
    let (oxy, deoxy) = hemo_map_get(m, x, y);
    let total = oxy + deoxy;
    if total < 1e-8 {
        0.0
    } else {
        oxy / total
    }
}

pub fn hemo_map_to_bytes(m: &HemoglobinMap) -> Vec<u8> {
    m.oxy
        .iter()
        .zip(m.deoxy.iter())
        .flat_map(|(&oxy, &deoxy)| {
            let o = (oxy.clamp(0.0, 1.0) * 255.0) as u8;
            let d = (deoxy.clamp(0.0, 1.0) * 255.0) as u8;
            [o, d, 0u8, 255u8]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_hemoglobin_map() {
        /* correct size */
        let m = new_hemoglobin_map(4, 4);
        assert_eq!(m.oxy.len(), 16);
    }

    #[test]
    fn test_set_get() {
        /* round-trip */
        let mut m = new_hemoglobin_map(2, 2);
        hemo_map_set(&mut m, 0, 0, 0.9, 0.1);
        let (oxy, deoxy) = hemo_map_get(&m, 0, 0);
        assert!((oxy - 0.9).abs() < 1e-6);
        assert!((deoxy - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_oxygen_saturation_half() {
        /* equal oxy and deoxy => 0.5 */
        let mut m = new_hemoglobin_map(2, 2);
        hemo_map_set(&mut m, 0, 0, 0.5, 0.5);
        assert!((hemo_map_oxygen_saturation(&m, 0, 0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_oxygen_saturation_zero() {
        /* zero total => 0 */
        let m = new_hemoglobin_map(2, 2);
        assert!((hemo_map_oxygen_saturation(&m, 0, 0)).abs() < 1e-6);
    }

    #[test]
    fn test_to_bytes_length() {
        /* 4 bytes per pixel */
        let m = new_hemoglobin_map(3, 3);
        let bytes = hemo_map_to_bytes(&m);
        assert_eq!(bytes.len(), 3 * 3 * 4);
    }
}
