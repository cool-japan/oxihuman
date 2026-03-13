// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct SkinDeformMap {
    pub vertex_count: usize,
    pub deformations: Vec<[f32; 3]>,
    pub stretch_factors: Vec<f32>,
}

pub fn new_skin_deform_map(n: usize) -> SkinDeformMap {
    SkinDeformMap {
        vertex_count: n,
        deformations: vec![[0.0; 3]; n],
        stretch_factors: vec![1.0; n],
    }
}

pub fn skin_deform_set(m: &mut SkinDeformMap, i: usize, d: [f32; 3], stretch: f32) {
    if i < m.vertex_count {
        m.deformations[i] = d;
        m.stretch_factors[i] = stretch;
    }
}

pub fn skin_deform_get(m: &SkinDeformMap, i: usize) -> ([f32; 3], f32) {
    if i < m.vertex_count {
        (m.deformations[i], m.stretch_factors[i])
    } else {
        ([0.0; 3], 1.0)
    }
}

pub fn skin_deform_max_stretch(m: &SkinDeformMap) -> f32 {
    m.stretch_factors.iter().cloned().fold(0.0f32, f32::max)
}

pub fn skin_deform_rms(m: &SkinDeformMap) -> f32 {
    if m.vertex_count == 0 {
        return 0.0;
    }
    let sum_sq: f32 = m
        .deformations
        .iter()
        .map(|d| d[0] * d[0] + d[1] * d[1] + d[2] * d[2])
        .sum();
    (sum_sq / m.vertex_count as f32).sqrt()
}

pub fn skin_deform_to_bytes(m: &SkinDeformMap) -> Vec<u8> {
    let mut b = Vec::new();
    let n = m.vertex_count as u32;
    b.extend_from_slice(&n.to_le_bytes());
    for d in &m.deformations {
        for &v in d {
            b.extend_from_slice(&v.to_le_bytes());
        }
    }
    for &s in &m.stretch_factors {
        b.extend_from_slice(&s.to_le_bytes());
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_skin_deform_map() {
        /* vertex count stored */
        let m = new_skin_deform_map(5);
        assert_eq!(m.vertex_count, 5);
    }

    #[test]
    fn test_skin_deform_set_get() {
        /* set and get roundtrip */
        let mut m = new_skin_deform_map(3);
        skin_deform_set(&mut m, 1, [1.0, 2.0, 3.0], 1.5);
        let (d, s) = skin_deform_get(&m, 1);
        assert!((d[0] - 1.0).abs() < 1e-6);
        assert!((s - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_skin_deform_max_stretch() {
        /* finds max stretch */
        let mut m = new_skin_deform_map(3);
        skin_deform_set(&mut m, 2, [0.0; 3], 3.0);
        assert!((skin_deform_max_stretch(&m) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_skin_deform_rms_zero() {
        /* no deformation = zero RMS */
        let m = new_skin_deform_map(4);
        assert!(skin_deform_rms(&m).abs() < 1e-6);
    }

    #[test]
    fn test_skin_deform_to_bytes() {
        /* bytes non-empty for populated map */
        let m = new_skin_deform_map(2);
        let b = skin_deform_to_bytes(&m);
        assert!(!b.is_empty());
    }

    #[test]
    fn test_skin_deform_get_oob() {
        /* out-of-bounds returns defaults */
        let m = new_skin_deform_map(2);
        let (d, s) = skin_deform_get(&m, 99);
        assert_eq!(d, [0.0f32; 3]);
        assert!((s - 1.0).abs() < 1e-6);
    }
}
