// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Symmetric 3x3 stress tensor field stored as `[s11,s22,s33,s12,s13,s23]`.
pub struct StressField {
    pub width: usize,
    pub height: usize,
    pub tensors: Vec<[f32; 6]>,
}

pub fn new_stress_field(w: usize, h: usize) -> StressField {
    StressField {
        width: w,
        height: h,
        tensors: vec![[0.0; 6]; w * h],
    }
}

pub fn stress_set(f: &mut StressField, x: usize, y: usize, t: [f32; 6]) {
    if x < f.width && y < f.height {
        f.tensors[y * f.width + x] = t;
    }
}

pub fn stress_get(f: &StressField, x: usize, y: usize) -> [f32; 6] {
    if x < f.width && y < f.height {
        f.tensors[y * f.width + x]
    } else {
        [0.0; 6]
    }
}

/// Von Mises stress = sqrt(0.5*((s11-s22)^2 + (s22-s33)^2 + (s33-s11)^2 + 6*(s12^2+s13^2+s23^2)))
pub fn stress_von_mises(t: [f32; 6]) -> f32 {
    let [s11, s22, s33, s12, s13, s23] = t;
    let vm = 0.5
        * ((s11 - s22) * (s11 - s22)
            + (s22 - s33) * (s22 - s33)
            + (s33 - s11) * (s33 - s11)
            + 6.0 * (s12 * s12 + s13 * s13 + s23 * s23));
    vm.max(0.0).sqrt()
}

/// Approximate max principal stress (mean of diagonal + shear magnitude).
pub fn stress_max_principal_approx(t: [f32; 6]) -> f32 {
    let [s11, s22, s33, s12, s13, s23] = t;
    let mean = (s11 + s22 + s33) / 3.0;
    let shear_mag = (s12 * s12 + s13 * s13 + s23 * s23).sqrt();
    mean + shear_mag
}

pub fn stress_to_bytes(f: &StressField) -> Vec<u8> {
    let mut out = Vec::with_capacity(f.tensors.len() * 24);
    for t in &f.tensors {
        for &v in t {
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stress_field_size() {
        let f = new_stress_field(4, 4);
        assert_eq!(f.tensors.len(), 16);
    }

    #[test]
    fn test_stress_set_get() {
        let mut f = new_stress_field(4, 4);
        stress_set(&mut f, 1, 2, [1.0, 2.0, 3.0, 0.5, 0.0, 0.0]);
        let t = stress_get(&f, 1, 2);
        assert!((t[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_stress_von_mises_uniaxial() {
        /* uniaxial: s11=1, rest=0 -> vm = 1 */
        let vm = stress_von_mises([1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert!((vm - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_stress_von_mises_zero() {
        let vm = stress_von_mises([0.0; 6]);
        assert!(vm.abs() < 1e-6);
    }

    #[test]
    fn test_stress_max_principal_approx() {
        let p = stress_max_principal_approx([3.0, 3.0, 3.0, 0.0, 0.0, 0.0]);
        assert!((p - 3.0).abs() < 1e-4);
    }

    #[test]
    fn test_stress_to_bytes_len() {
        let f = new_stress_field(3, 3);
        assert_eq!(stress_to_bytes(&f).len(), 9 * 24);
    }

    #[test]
    fn test_stress_get_oob() {
        let f = new_stress_field(4, 4);
        assert_eq!(stress_get(&f, 10, 10), [0.0; 6]);
    }
}
