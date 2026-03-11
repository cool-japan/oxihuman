// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cinespace .csp LUT export.

/// A 1D pre-shaper curve entry.
#[derive(Debug, Clone)]
pub struct CspShaper {
    pub channel: usize, /* 0=R 1=G 2=B */
    pub entries: Vec<f32>,
}

/// A Cinespace .csp LUT.
#[derive(Debug, Clone)]
pub struct CspLut {
    pub title: String,
    pub size_3d: usize,
    /// Optional 1D pre-shaper per channel.
    pub shapers: Vec<CspShaper>,
    /// Flattened 3D LUT data (RGB triplets).
    pub data_3d: Vec<[f32; 3]>,
    pub domain_min: [f32; 3],
    pub domain_max: [f32; 3],
}

/// Create a new identity CSP LUT.
pub fn new_csp_lut(size: usize) -> CspLut {
    let n = size * size * size;
    let mut data = Vec::with_capacity(n);
    for i in 0..n {
        let b = i / (size * size);
        let g = (i / size) % size;
        let r = i % size;
        let sv = if size > 1 { (size - 1) as f32 } else { 1.0 };
        data.push([r as f32 / sv, g as f32 / sv, b as f32 / sv]);
    }
    CspLut {
        title: "Identity".to_string(),
        size_3d: size,
        shapers: Vec::new(),
        data_3d: data,
        domain_min: [0.0, 0.0, 0.0],
        domain_max: [1.0, 1.0, 1.0],
    }
}

/// Entry count.
pub fn csp_entry_count(lut: &CspLut) -> usize {
    lut.data_3d.len()
}

/// Validate the LUT.
pub fn validate_csp_lut(lut: &CspLut) -> bool {
    let expected = lut.size_3d * lut.size_3d * lut.size_3d;
    lut.size_3d >= 2 && lut.data_3d.len() == expected
}

/// Serialize to a .csp string.
pub fn csp_to_string(lut: &CspLut) -> String {
    let mut out = format!(
        "CSPLUTV100\n3D\n\nBEGIN METADATA\n{}\nEND METADATA\n\n",
        lut.title
    );
    /* Pre-shaper sections */
    for ch in 0..3 {
        let label = ["Red", "Green", "Blue"][ch];
        out.push_str(&format!("{}Shaper 2\n", label));
        out.push_str(&format!(
            "{:.6} {:.6}\n",
            lut.domain_min[ch], lut.domain_max[ch]
        ));
        out.push_str("0.000000 1.000000\n");
    }
    out.push_str(&format!(
        "\n{} {} {}\n\n",
        lut.size_3d, lut.size_3d, lut.size_3d
    ));
    for &[r, g, b] in &lut.data_3d {
        out.push_str(&format!("{:.6} {:.6} {:.6}\n", r, g, b));
    }
    out
}

/// Estimate the .csp file size.
pub fn csp_size_bytes(lut: &CspLut) -> usize {
    csp_to_string(lut).len()
}

/// Add a 1D pre-shaper.
pub fn csp_add_shaper(lut: &mut CspLut, channel: usize, entries: Vec<f32>) {
    lut.shapers.push(CspShaper { channel, entries });
}

/// Sample the LUT (nearest-neighbor stub).
pub fn csp_sample(lut: &CspLut, r: f32, g: f32, b: f32) -> [f32; 3] {
    let s = (lut.size_3d - 1) as f32;
    let ri = (r * s).round() as usize;
    let gi = (g * s).round() as usize;
    let bi = (b * s).round() as usize;
    let ri = ri.min(lut.size_3d - 1);
    let gi = gi.min(lut.size_3d - 1);
    let bi = bi.min(lut.size_3d - 1);
    let idx = bi * lut.size_3d * lut.size_3d + gi * lut.size_3d + ri;
    lut.data_3d.get(idx).copied().unwrap_or([r, g, b])
}

/// Compute the average output brightness.
pub fn csp_average_brightness(lut: &CspLut) -> f32 {
    if lut.data_3d.is_empty() {
        return 0.0;
    }
    let total: f32 = lut
        .data_3d
        .iter()
        .map(|&[r, g, b]| 0.2126 * r + 0.7152 * g + 0.0722 * b)
        .sum();
    total / lut.data_3d.len() as f32
}

/// Shaper count.
pub fn csp_shaper_count(lut: &CspLut) -> usize {
    lut.shapers.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_count() {
        let lut = new_csp_lut(4);
        assert_eq!(csp_entry_count(&lut), 64);
    }

    #[test]
    fn test_validate_identity() {
        let lut = new_csp_lut(4);
        assert!(validate_csp_lut(&lut));
    }

    #[test]
    fn test_to_string_contains_header() {
        let lut = new_csp_lut(4);
        assert!(csp_to_string(&lut).contains("CSPLUTV100"));
    }

    #[test]
    fn test_size_bytes_positive() {
        assert!(csp_size_bytes(&new_csp_lut(4)) > 0);
    }

    #[test]
    fn test_sample_black() {
        let lut = new_csp_lut(4);
        let out = csp_sample(&lut, 0.0, 0.0, 0.0);
        assert!(out[0] < 0.01);
    }

    #[test]
    fn test_sample_white() {
        let lut = new_csp_lut(4);
        let out = csp_sample(&lut, 1.0, 1.0, 1.0);
        assert!(out[0] > 0.99);
    }

    #[test]
    fn test_add_shaper() {
        let mut lut = new_csp_lut(4);
        csp_add_shaper(&mut lut, 0, vec![0.0, 0.5, 1.0]);
        assert_eq!(csp_shaper_count(&lut), 1);
    }

    #[test]
    fn test_average_brightness_identity() {
        let lut = new_csp_lut(4);
        let avg = csp_average_brightness(&lut);
        /* For identity LUT of size 4, average should be ~0.5 */
        assert!(avg > 0.3 && avg < 0.7);
    }

    #[test]
    fn test_validate_size_one_fails() {
        let lut = CspLut {
            title: "X".to_string(),
            size_3d: 1,
            shapers: vec![],
            data_3d: vec![[0.5, 0.5, 0.5]],
            domain_min: [0.0, 0.0, 0.0],
            domain_max: [1.0, 1.0, 1.0],
        };
        assert!(!validate_csp_lut(&lut));
    }
}
