// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Per-vertex deformation displacement field.
pub struct DeformationField {
    pub vertex_count: usize,
    pub displacements: Vec<[f32; 3]>,
}

pub fn new_deformation_field(n: usize) -> DeformationField {
    DeformationField {
        vertex_count: n,
        displacements: vec![[0.0; 3]; n],
    }
}

pub fn deform_set(f: &mut DeformationField, i: usize, d: [f32; 3]) {
    if i < f.vertex_count {
        f.displacements[i] = d;
    }
}

pub fn deform_get(f: &DeformationField, i: usize) -> [f32; 3] {
    if i < f.vertex_count {
        f.displacements[i]
    } else {
        [0.0; 3]
    }
}

pub fn deform_max_displacement(f: &DeformationField) -> f32 {
    f.displacements
        .iter()
        .map(|&d| (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt())
        .fold(0.0f32, f32::max)
}

pub fn deform_rms_displacement(f: &DeformationField) -> f32 {
    let n = f.displacements.len();
    if n == 0 {
        return 0.0;
    }
    let sum_sq: f32 = f
        .displacements
        .iter()
        .map(|&d| d[0] * d[0] + d[1] * d[1] + d[2] * d[2])
        .sum();
    (sum_sq / n as f32).sqrt()
}

pub fn deform_to_bytes(f: &DeformationField) -> Vec<u8> {
    let mut out = Vec::with_capacity(f.displacements.len() * 12);
    for &d in &f.displacements {
        out.extend_from_slice(&d[0].to_le_bytes());
        out.extend_from_slice(&d[1].to_le_bytes());
        out.extend_from_slice(&d[2].to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_deformation_field() {
        let f = new_deformation_field(10);
        assert_eq!(f.vertex_count, 10);
        assert_eq!(f.displacements.len(), 10);
    }

    #[test]
    fn test_deform_set_get() {
        let mut f = new_deformation_field(5);
        deform_set(&mut f, 2, [1.0, 2.0, 3.0]);
        let d = deform_get(&f, 2);
        assert!((d[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_deform_get_oob() {
        let f = new_deformation_field(5);
        assert_eq!(deform_get(&f, 10), [0.0; 3]);
    }

    #[test]
    fn test_deform_max_displacement() {
        let mut f = new_deformation_field(3);
        deform_set(&mut f, 0, [3.0, 4.0, 0.0]);
        deform_set(&mut f, 1, [1.0, 0.0, 0.0]);
        assert!((deform_max_displacement(&f) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_deform_rms_displacement_uniform() {
        let mut f = new_deformation_field(4);
        for i in 0..4 {
            deform_set(&mut f, i, [1.0, 0.0, 0.0]);
        }
        assert!((deform_rms_displacement(&f) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_deform_to_bytes_len() {
        let f = new_deformation_field(5);
        assert_eq!(deform_to_bytes(&f).len(), 60);
    }

    #[test]
    fn test_deform_rms_empty() {
        let f = new_deformation_field(0);
        assert!((deform_rms_displacement(&f) - 0.0).abs() < 1e-6);
    }
}
