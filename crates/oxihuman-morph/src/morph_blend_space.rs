// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! 1D blend space for morphs.

#[allow(dead_code)]
pub struct BlendSpacePoint {
    pub param: f32,
    pub weights: Vec<f32>,
}

#[allow(dead_code)]
pub struct BlendSpace1D {
    pub points: Vec<BlendSpacePoint>,
    pub morph_count: usize,
}

#[allow(dead_code)]
pub fn new_blend_space_1d(morph_count: usize) -> BlendSpace1D {
    BlendSpace1D { points: Vec::new(), morph_count }
}

#[allow(dead_code)]
pub fn bs1_add_point(s: &mut BlendSpace1D, param: f32, weights: Vec<f32>) -> bool {
    if weights.len() != s.morph_count {
        return false;
    }
    let pos = s.points.partition_point(|p| p.param <= param);
    s.points.insert(pos, BlendSpacePoint { param, weights });
    true
}

#[allow(dead_code)]
pub fn bs1_evaluate(s: &BlendSpace1D, param: f32) -> Vec<f32> {
    if s.points.is_empty() {
        return vec![0.0; s.morph_count];
    }
    if param <= s.points[0].param {
        return s.points[0].weights.clone();
    }
    let last = &s.points[s.points.len() - 1];
    if param >= last.param {
        return last.weights.clone();
    }
    let idx = s.points.partition_point(|p| p.param <= param);
    let a = &s.points[idx - 1];
    let b = &s.points[idx];
    let span = b.param - a.param;
    let t = if span > 1e-7 { (param - a.param) / span } else { 0.0 };
    a.weights.iter().zip(b.weights.iter()).map(|(wa, wb)| wa + (wb - wa) * t).collect()
}

#[allow(dead_code)]
pub fn bs1_point_count(s: &BlendSpace1D) -> usize {
    s.points.len()
}

#[allow(dead_code)]
pub fn bs1_param_range(s: &BlendSpace1D) -> (f32, f32) {
    if s.points.is_empty() {
        return (0.0, 0.0);
    }
    (s.points[0].param, s.points[s.points.len() - 1].param)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_point() {
        let mut s = new_blend_space_1d(2);
        assert!(bs1_add_point(&mut s, 0.0, vec![0.0, 1.0]));
    }

    #[test]
    fn test_add_point_mismatch() {
        let mut s = new_blend_space_1d(2);
        assert!(!bs1_add_point(&mut s, 0.0, vec![0.0]));
    }

    #[test]
    fn test_evaluate_at_existing() {
        let mut s = new_blend_space_1d(1);
        bs1_add_point(&mut s, 0.0, vec![0.0]);
        bs1_add_point(&mut s, 1.0, vec![1.0]);
        let v = bs1_evaluate(&s, 0.0);
        assert!((v[0] - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_between() {
        let mut s = new_blend_space_1d(1);
        bs1_add_point(&mut s, 0.0, vec![0.0]);
        bs1_add_point(&mut s, 1.0, vec![1.0]);
        let v = bs1_evaluate(&s, 0.5);
        assert!((v[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_point_count() {
        let mut s = new_blend_space_1d(1);
        bs1_add_point(&mut s, 0.0, vec![0.0]);
        bs1_add_point(&mut s, 1.0, vec![1.0]);
        assert_eq!(bs1_point_count(&s), 2);
    }

    #[test]
    fn test_param_range() {
        let mut s = new_blend_space_1d(1);
        bs1_add_point(&mut s, -1.0, vec![0.0]);
        bs1_add_point(&mut s, 2.0, vec![1.0]);
        let (lo, hi) = bs1_param_range(&s);
        assert!((lo - (-1.0)).abs() < 1e-5);
        assert!((hi - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_empty_evaluate() {
        let s = new_blend_space_1d(2);
        let v = bs1_evaluate(&s, 0.5);
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn test_empty_param_range() {
        let s = new_blend_space_1d(1);
        let r = bs1_param_range(&s);
        assert_eq!(r, (0.0, 0.0));
    }
}
