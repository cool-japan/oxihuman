// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! 3D LUT .cube format export.

/// A 3D LUT in .cube format.
#[derive(Debug, Clone)]
pub struct CubeLut {
    pub title: String,
    pub size: usize, /* LUT_3D_SIZE (e.g., 33) */
    pub domain_min: [f32; 3],
    pub domain_max: [f32; 3],
    /// Flattened RGB values: size^3 entries, each [r, g, b].
    pub data: Vec<[f32; 3]>,
}

/// Create a new identity 3D LUT.
pub fn new_cube_lut(size: usize) -> CubeLut {
    let n = size * size * size;
    let mut data = Vec::with_capacity(n);
    for i in 0..n {
        let b = i / (size * size);
        let g = (i / size) % size;
        let r = i % size;
        let sv = if size > 1 { (size - 1) as f32 } else { 1.0 };
        data.push([r as f32 / sv, g as f32 / sv, b as f32 / sv]);
    }
    CubeLut {
        title: "Identity LUT".to_string(),
        size,
        domain_min: [0.0, 0.0, 0.0],
        domain_max: [1.0, 1.0, 1.0],
        data,
    }
}

/// Return the entry count.
pub fn cube_entry_count(lut: &CubeLut) -> usize {
    lut.data.len()
}

/// Expected entry count for a given size.
pub fn expected_entry_count(size: usize) -> usize {
    size * size * size
}

/// Validate the LUT.
pub fn validate_cube_lut(lut: &CubeLut) -> bool {
    lut.size >= 2
        && lut.data.len() == expected_entry_count(lut.size)
        && lut.data.iter().all(|&[r, g, b]| {
            (0.0..=1.0).contains(&r) && (0.0..=1.0).contains(&g) && (0.0..=1.0).contains(&b)
        })
}

/// Serialize the LUT to a .cube string.
pub fn cube_to_string(lut: &CubeLut) -> String {
    let mut out = format!(
        "TITLE \"{}\"\nLUT_3D_SIZE {}\nDOMAIN_MIN {} {} {}\nDOMAIN_MAX {} {} {}\n\n",
        lut.title,
        lut.size,
        lut.domain_min[0],
        lut.domain_min[1],
        lut.domain_min[2],
        lut.domain_max[0],
        lut.domain_max[1],
        lut.domain_max[2],
    );
    for &[r, g, b] in &lut.data {
        out.push_str(&format!("{:.6} {:.6} {:.6}\n", r, g, b));
    }
    out
}

/// Estimate the .cube file size.
pub fn cube_size_bytes(lut: &CubeLut) -> usize {
    cube_to_string(lut).len()
}

/// Sample the LUT at normalized coordinates (trilinear, stub: nearest).
pub fn cube_sample(lut: &CubeLut, r: f32, g: f32, b: f32) -> [f32; 3] {
    let s = (lut.size - 1) as f32;
    let ri = (r * s).round() as usize;
    let gi = (g * s).round() as usize;
    let bi = (b * s).round() as usize;
    let ri = ri.min(lut.size - 1);
    let gi = gi.min(lut.size - 1);
    let bi = bi.min(lut.size - 1);
    let idx = bi * lut.size * lut.size + gi * lut.size + ri;
    lut.data.get(idx).copied().unwrap_or([r, g, b])
}

/// Apply a CDL-like gain to all LUT entries.
pub fn cube_apply_gain(lut: &mut CubeLut, gain: [f32; 3]) {
    for entry in &mut lut.data {
        entry[0] = (entry[0] * gain[0]).clamp(0.0, 1.0);
        entry[1] = (entry[1] * gain[1]).clamp(0.0, 1.0);
        entry[2] = (entry[2] * gain[2]).clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_count() {
        let lut = new_cube_lut(4);
        assert_eq!(cube_entry_count(&lut), 64);
    }

    #[test]
    fn test_expected_entry_count() {
        assert_eq!(expected_entry_count(33), 35937);
    }

    #[test]
    fn test_validate_identity() {
        let lut = new_cube_lut(4);
        assert!(validate_cube_lut(&lut));
    }

    #[test]
    fn test_to_string_contains_title() {
        let lut = new_cube_lut(4);
        let s = cube_to_string(&lut);
        assert!(s.contains("TITLE"));
        assert!(s.contains("LUT_3D_SIZE 4"));
    }

    #[test]
    fn test_size_bytes_positive() {
        let lut = new_cube_lut(4);
        assert!(cube_size_bytes(&lut) > 0);
    }

    #[test]
    fn test_sample_identity() {
        let lut = new_cube_lut(4);
        let out = cube_sample(&lut, 0.0, 0.0, 0.0);
        assert!(out[0] < 0.01 && out[1] < 0.01 && out[2] < 0.01);
    }

    #[test]
    fn test_apply_gain() {
        let mut lut = new_cube_lut(4);
        cube_apply_gain(&mut lut, [0.5, 0.5, 0.5]);
        /* All entries should be <= 0.5 */
        assert!(lut.data.iter().all(|&[r, _, _]| r <= 0.5001));
    }

    #[test]
    fn test_validate_size_one_fails() {
        let lut = CubeLut {
            title: "X".to_string(),
            size: 1,
            domain_min: [0.0, 0.0, 0.0],
            domain_max: [1.0, 1.0, 1.0],
            data: vec![[0.5, 0.5, 0.5]],
        };
        assert!(!validate_cube_lut(&lut));
    }

    #[test]
    fn test_sample_high() {
        let lut = new_cube_lut(4);
        let out = cube_sample(&lut, 1.0, 1.0, 1.0);
        /* Identity LUT: output ≈ input */
        assert!(out[0] > 0.99 && out[1] > 0.99 && out[2] > 0.99);
    }
}
