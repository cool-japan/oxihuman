// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Symmetry axis: 0=X, 1=Y, 2=Z.
pub struct SymmetryMap {
    pub pairs: Vec<(usize, usize)>,
    pub mirror_axis: u8,
}

pub fn new_symmetry_map(axis: u8) -> SymmetryMap {
    SymmetryMap {
        pairs: vec![],
        mirror_axis: axis,
    }
}

pub fn sym_add_pair(m: &mut SymmetryMap, a: usize, b: usize) {
    m.pairs.push((a, b));
}

pub fn sym_find_mirror(m: &SymmetryMap, v: usize) -> Option<usize> {
    for &(a, b) in &m.pairs {
        if a == v {
            return Some(b);
        }
        if b == v {
            return Some(a);
        }
    }
    None
}

pub fn sym_pair_count(m: &SymmetryMap) -> usize {
    m.pairs.len()
}

pub fn sym_is_symmetric_position(a: [f32; 3], b: [f32; 3], axis: u8, eps: f32) -> bool {
    let axis = axis % 3;
    for i in 0..3 {
        if i == axis as usize {
            if (a[i] + b[i]).abs() > eps {
                return false;
            }
        } else if (a[i] - b[i]).abs() > eps {
            return false;
        }
    }
    true
}

pub fn sym_mirror_position(p: [f32; 3], axis: u8) -> [f32; 3] {
    let mut q = p;
    let ax = (axis % 3) as usize;
    q[ax] = -q[ax];
    q
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_symmetry_map() {
        /* empty on construction */
        let m = new_symmetry_map(0);
        assert_eq!(sym_pair_count(&m), 0);
        assert_eq!(m.mirror_axis, 0);
    }

    #[test]
    fn test_add_find_pair() {
        /* add and find mirror */
        let mut m = new_symmetry_map(0);
        sym_add_pair(&mut m, 3, 7);
        assert_eq!(sym_find_mirror(&m, 3), Some(7));
        assert_eq!(sym_find_mirror(&m, 7), Some(3));
    }

    #[test]
    fn test_find_mirror_not_found() {
        /* missing vertex */
        let m = new_symmetry_map(0);
        assert!(sym_find_mirror(&m, 99).is_none());
    }

    #[test]
    fn test_is_symmetric_x_axis() {
        /* X-axis symmetric */
        let a = [1.0, 0.0, 0.0_f32];
        let b = [-1.0, 0.0, 0.0_f32];
        assert!(sym_is_symmetric_position(a, b, 0, 1e-5));
    }

    #[test]
    fn test_mirror_position_x() {
        /* mirror flips X */
        let p = [2.0, 1.0, 3.0_f32];
        let m = sym_mirror_position(p, 0);
        assert!((m[0] + 2.0).abs() < 1e-6);
        assert!((m[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_pair_count() {
        /* count */
        let mut m = new_symmetry_map(1);
        sym_add_pair(&mut m, 0, 1);
        sym_add_pair(&mut m, 2, 3);
        assert_eq!(sym_pair_count(&m), 2);
    }

    #[test]
    fn test_not_symmetric() {
        /* mismatched positions */
        let a = [1.0, 0.0, 0.0_f32];
        let b = [1.0, 0.0, 0.0_f32];
        assert!(!sym_is_symmetric_position(a, b, 0, 1e-5));
    }
}
