// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct BentNormalMap {
    pub normals: Vec<[f32; 3]>,
    pub ao: Vec<f32>,
}

pub fn new_bent_normal_map(n: usize) -> BentNormalMap {
    BentNormalMap {
        normals: vec![[0.0, 1.0, 0.0]; n],
        ao: vec![1.0; n],
    }
}

pub fn bent_normal_set(m: &mut BentNormalMap, i: usize, n: [f32; 3], ao: f32) {
    m.normals[i] = n;
    m.ao[i] = ao;
}

pub fn bent_normal_get(m: &BentNormalMap, i: usize) -> [f32; 3] {
    m.normals[i]
}

/// Remap normal from [-1,1] to `[0,1]` for color encoding.
pub fn bent_normal_to_color(n: [f32; 3]) -> [f32; 3] {
    [
        (n[0] * 0.5 + 0.5).clamp(0.0, 1.0),
        (n[1] * 0.5 + 0.5).clamp(0.0, 1.0),
        (n[2] * 0.5 + 0.5).clamp(0.0, 1.0),
    ]
}

pub fn bent_normal_ao(m: &BentNormalMap, i: usize) -> f32 {
    m.ao[i]
}

pub fn bent_normal_count(m: &BentNormalMap) -> usize {
    m.normals.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bent_normal_map() {
        /* default up normal */
        let m = new_bent_normal_map(3);
        assert_eq!(bent_normal_count(&m), 3);
        assert_eq!(bent_normal_get(&m, 0), [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_set_get() {
        /* round-trip */
        let mut m = new_bent_normal_map(4);
        bent_normal_set(&mut m, 2, [1.0, 0.0, 0.0], 0.8);
        assert_eq!(bent_normal_get(&m, 2), [1.0, 0.0, 0.0]);
        assert!((bent_normal_ao(&m, 2) - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_to_color_up() {
        /* up normal maps to [0.5, 1, 0.5] */
        let c = bent_normal_to_color([0.0, 1.0, 0.0]);
        assert!((c[0] - 0.5).abs() < 1e-6);
        assert!((c[1] - 1.0).abs() < 1e-6);
        assert!((c[2] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_color_neg() {
        /* -1 normal maps to 0 */
        let c = bent_normal_to_color([-1.0, -1.0, -1.0]);
        assert!(c[0] < 1e-6);
        assert!(c[1] < 1e-6);
        assert!(c[2] < 1e-6);
    }

    #[test]
    fn test_ao_default() {
        /* default ao is 1 */
        let m = new_bent_normal_map(2);
        assert!((bent_normal_ao(&m, 0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_count() {
        /* count matches size */
        let m = new_bent_normal_map(7);
        assert_eq!(bent_normal_count(&m), 7);
    }
}
