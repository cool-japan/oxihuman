// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export color lookup tables (LUT) for color grading.

/// LUT export data (3D LUT as a flat array).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LutExport {
    pub size: u32,
    pub data: Vec<[f32; 3]>,
}

#[allow(dead_code)]
pub fn new_identity_lut(size: u32) -> LutExport {
    let n = (size * size * size) as usize;
    let mut data = Vec::with_capacity(n);
    let s = size as f32;
    for b in 0..size { for g in 0..size { for r in 0..size {
        data.push([r as f32 / (s - 1.0), g as f32 / (s - 1.0), b as f32 / (s - 1.0)]);
    }}}
    LutExport { size, data }
}

#[allow(dead_code)]
pub fn lut_entry_count(lut: &LutExport) -> usize { lut.data.len() }

#[allow(dead_code)]
pub fn lut_sample(lut: &LutExport, r: f32, g: f32, b: f32) -> [f32; 3] {
    let s = lut.size as f32 - 1.0;
    let ri = (r.clamp(0.0, 1.0) * s) as usize;
    let gi = (g.clamp(0.0, 1.0) * s) as usize;
    let bi = (b.clamp(0.0, 1.0) * s) as usize;
    let idx = bi * (lut.size as usize * lut.size as usize) + gi * lut.size as usize + ri;
    if idx < lut.data.len() { lut.data[idx] } else { [r, g, b] }
}

#[allow(dead_code)]
pub fn lut_apply_contrast(lut: &mut LutExport, contrast: f32) {
    for c in lut.data.iter_mut() {
        c[0] = ((c[0] - 0.5) * contrast + 0.5).clamp(0.0, 1.0);
        c[1] = ((c[1] - 0.5) * contrast + 0.5).clamp(0.0, 1.0);
        c[2] = ((c[2] - 0.5) * contrast + 0.5).clamp(0.0, 1.0);
    }
}

#[allow(dead_code)]
pub fn lut_to_cube_string(lut: &LutExport) -> String {
    let mut s = format!("LUT_3D_SIZE {}
", lut.size);
    for c in &lut.data {
        s.push_str(&format!("{:.6} {:.6} {:.6}
", c[0], c[1], c[2]));
    }
    s
}

#[allow(dead_code)]
pub fn lut_data_size_bytes(lut: &LutExport) -> usize { lut.data.len() * 12 }

#[allow(dead_code)]
pub fn lut_to_json(lut: &LutExport) -> String {
    format!(r#"{{"size":{},"entries":{},"bytes":{}}}"#, lut.size, lut_entry_count(lut), lut_data_size_bytes(lut))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_lut() {
        let lut = new_identity_lut(4);
        assert_eq!(lut_entry_count(&lut), 64);
    }

    #[test]
    fn test_sample_identity() {
        let lut = new_identity_lut(8);
        let c = lut_sample(&lut, 0.0, 0.0, 0.0);
        assert!(c[0].abs() < 0.01);
    }

    #[test]
    fn test_sample_white() {
        let lut = new_identity_lut(8);
        let c = lut_sample(&lut, 1.0, 1.0, 1.0);
        assert!((c[0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_contrast() {
        let mut lut = new_identity_lut(4);
        lut_apply_contrast(&mut lut, 2.0);
        // Midpoint should stay roughly 0.5
        let c = lut_sample(&lut, 0.5, 0.5, 0.5);
        assert!((c[0] - 0.5).abs() < 0.2);
    }

    #[test]
    fn test_cube_string() {
        let lut = new_identity_lut(2);
        let s = lut_to_cube_string(&lut);
        assert!(s.starts_with("LUT_3D_SIZE"));
    }

    #[test]
    fn test_data_size() {
        let lut = new_identity_lut(4);
        assert_eq!(lut_data_size_bytes(&lut), 64 * 12);
    }

    #[test]
    fn test_to_json() {
        let lut = new_identity_lut(2);
        let json = lut_to_json(&lut);
        assert!(json.contains("size"));
    }

    #[test]
    fn test_lut_size_1() {
        let lut = new_identity_lut(1);
        assert_eq!(lut_entry_count(&lut), 1);
    }

    #[test]
    fn test_sample_clamp() {
        let lut = new_identity_lut(4);
        let c = lut_sample(&lut, -1.0, 2.0, 0.5);
        assert!((0.0..=1.0).contains(&c[0]));
    }

    #[test]
    fn test_contrast_clamp() {
        let mut lut = new_identity_lut(4);
        lut_apply_contrast(&mut lut, 10.0);
        for c in &lut.data {
            assert!((0.0..=1.0).contains(&c[0]));
        }
    }

}
