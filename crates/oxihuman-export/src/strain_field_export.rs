// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Scalar von Mises strain per vertex.
pub struct StrainField {
    pub vertex_count: usize,
    pub strains: Vec<f32>,
}

pub fn new_strain_field(n: usize) -> StrainField {
    StrainField {
        vertex_count: n,
        strains: vec![0.0; n],
    }
}

pub fn strain_set(f: &mut StrainField, i: usize, s: f32) {
    if i < f.vertex_count {
        f.strains[i] = s;
    }
}

pub fn strain_get(f: &StrainField, i: usize) -> f32 {
    if i < f.vertex_count {
        f.strains[i]
    } else {
        0.0
    }
}

pub fn strain_max(f: &StrainField) -> f32 {
    f.strains.iter().cloned().fold(0.0f32, f32::max)
}

pub fn strain_mean(f: &StrainField) -> f32 {
    let n = f.strains.len();
    if n == 0 {
        return 0.0;
    }
    f.strains.iter().sum::<f32>() / n as f32
}

pub fn strain_to_bytes(f: &StrainField) -> Vec<u8> {
    let mut out = Vec::with_capacity(f.strains.len() * 4);
    for &s in &f.strains {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

pub fn strain_exceeds_threshold(f: &StrainField, thr: f32) -> Vec<usize> {
    f.strains
        .iter()
        .enumerate()
        .filter_map(|(i, &s)| if s > thr { Some(i) } else { None })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_strain_field() {
        let f = new_strain_field(10);
        assert_eq!(f.vertex_count, 10);
    }

    #[test]
    fn test_strain_set_get() {
        let mut f = new_strain_field(5);
        strain_set(&mut f, 2, 0.05);
        assert!((strain_get(&f, 2) - 0.05).abs() < 1e-6);
    }

    #[test]
    fn test_strain_get_oob() {
        let f = new_strain_field(5);
        assert!((strain_get(&f, 10) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_strain_max() {
        let mut f = new_strain_field(4);
        strain_set(&mut f, 0, 0.01);
        strain_set(&mut f, 1, 0.05);
        strain_set(&mut f, 2, 0.03);
        assert!((strain_max(&f) - 0.05).abs() < 1e-6);
    }

    #[test]
    fn test_strain_mean() {
        let mut f = new_strain_field(4);
        for i in 0..4 {
            strain_set(&mut f, i, i as f32);
        }
        /* mean of 0+1+2+3 = 1.5 */
        assert!((strain_mean(&f) - 1.5).abs() < 1e-5);
    }

    #[test]
    fn test_strain_to_bytes_len() {
        let f = new_strain_field(8);
        assert_eq!(strain_to_bytes(&f).len(), 32);
    }

    #[test]
    fn test_strain_exceeds_threshold() {
        let mut f = new_strain_field(5);
        strain_set(&mut f, 1, 0.1);
        strain_set(&mut f, 3, 0.2);
        let exceed = strain_exceeds_threshold(&f, 0.05);
        assert!(exceed.contains(&1) && exceed.contains(&3));
    }
}
