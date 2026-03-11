// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct ScarMap {
    pub width: u32,
    pub height: u32,
    pub elevation: Vec<f32>,
    pub maturity: Vec<f32>,
}

pub fn new_scar_map(w: u32, h: u32) -> ScarMap {
    let n = (w * h) as usize;
    ScarMap {
        width: w,
        height: h,
        elevation: vec![0.0; n],
        maturity: vec![0.0; n],
    }
}

fn idx(m: &ScarMap, x: u32, y: u32) -> usize {
    (y * m.width + x) as usize
}

pub fn scar_set(m: &mut ScarMap, x: u32, y: u32, elev: f32, maturity: f32) {
    let i = idx(m, x, y);
    m.elevation[i] = elev;
    m.maturity[i] = maturity;
}

pub fn scar_get(m: &ScarMap, x: u32, y: u32) -> (f32, f32) {
    let i = idx(m, x, y);
    (m.elevation[i], m.maturity[i])
}

pub fn scar_coverage(m: &ScarMap) -> f32 {
    let total = m.elevation.len();
    if total == 0 {
        return 0.0;
    }
    let scar = m.elevation.iter().filter(|&&v| v > 0.0).count();
    scar as f32 / total as f32
}

pub fn scar_mean_elevation(m: &ScarMap) -> f32 {
    if m.elevation.is_empty() {
        return 0.0;
    }
    m.elevation.iter().sum::<f32>() / m.elevation.len() as f32
}

pub fn scar_to_bytes(m: &ScarMap) -> Vec<u8> {
    m.elevation
        .iter()
        .zip(m.maturity.iter())
        .flat_map(|(&e, &mt)| {
            let eb = (e.clamp(0.0, 1.0) * 255.0) as u8;
            let mb = (mt.clamp(0.0, 1.0) * 255.0) as u8;
            [eb, mb, 0u8, 255u8]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_scar_map() {
        /* zeroed */
        let m = new_scar_map(4, 4);
        assert_eq!(m.elevation.len(), 16);
    }

    #[test]
    fn test_set_get() {
        /* round-trip */
        let mut m = new_scar_map(3, 3);
        scar_set(&mut m, 1, 1, 0.5, 0.8);
        let (e, mt) = scar_get(&m, 1, 1);
        assert!((e - 0.5).abs() < 1e-6);
        assert!((mt - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_coverage_zero() {
        /* no scars => 0 */
        let m = new_scar_map(4, 4);
        assert!((scar_coverage(&m)).abs() < 1e-6);
    }

    #[test]
    fn test_coverage_half() {
        /* half pixels elevated */
        let mut m = new_scar_map(2, 1);
        scar_set(&mut m, 0, 0, 0.5, 0.0);
        assert!((scar_coverage(&m) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_mean_elevation() {
        /* mean */
        let mut m = new_scar_map(2, 1);
        scar_set(&mut m, 0, 0, 0.0, 0.0);
        scar_set(&mut m, 1, 0, 1.0, 0.0);
        assert!((scar_mean_elevation(&m) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_bytes_length() {
        /* 4 bytes per pixel */
        let m = new_scar_map(3, 3);
        let bytes = scar_to_bytes(&m);
        assert_eq!(bytes.len(), 3 * 3 * 4);
    }
}
