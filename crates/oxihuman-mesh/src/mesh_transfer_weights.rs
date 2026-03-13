// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct WeightMap {
    pub weights: Vec<f32>,
    pub vertex_count: usize,
}

pub fn new_weight_map(n: usize) -> WeightMap {
    WeightMap {
        weights: vec![0.0; n],
        vertex_count: n,
    }
}

pub fn weight_set(m: &mut WeightMap, i: usize, w: f32) {
    if i < m.vertex_count {
        m.weights[i] = w.clamp(0.0, 1.0);
    }
}

pub fn weight_get(m: &WeightMap, i: usize) -> f32 {
    if i < m.vertex_count {
        m.weights[i]
    } else {
        0.0
    }
}

pub fn weight_normalize(m: &mut WeightMap) {
    let sum: f32 = m.weights.iter().sum();
    if sum > 1e-10 {
        for w in &mut m.weights {
            *w /= sum;
        }
    }
}

pub fn weight_transfer_nearest(
    source: &WeightMap,
    source_pos: &[[f32; 3]],
    target_pos: &[[f32; 3]],
) -> WeightMap {
    let n = target_pos.len();
    let mut result = new_weight_map(n);
    for (ti, tp) in target_pos.iter().enumerate() {
        let mut best_dist = f32::MAX;
        let mut best_idx = 0usize;
        for (si, sp) in source_pos.iter().enumerate() {
            let d = (tp[0] - sp[0]).powi(2) + (tp[1] - sp[1]).powi(2) + (tp[2] - sp[2]).powi(2);
            if d < best_dist {
                best_dist = d;
                best_idx = si;
            }
        }
        result.weights[ti] = weight_get(source, best_idx);
    }
    result
}

pub fn weight_blend(a: &WeightMap, b: &WeightMap, t: f32) -> WeightMap {
    let n = a.vertex_count.min(b.vertex_count);
    let mut result = new_weight_map(n);
    let t = t.clamp(0.0, 1.0);
    for i in 0..n {
        result.weights[i] = a.weights[i] * (1.0 - t) + b.weights[i] * t;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_weight_map() {
        /* all weights start at zero */
        let m = new_weight_map(4);
        assert_eq!(m.vertex_count, 4);
        assert!(m.weights.iter().all(|&w| w == 0.0));
    }

    #[test]
    fn test_weight_set_get() {
        /* set and get roundtrip */
        let mut m = new_weight_map(3);
        weight_set(&mut m, 1, 0.7);
        assert!((weight_get(&m, 1) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_weight_normalize() {
        /* after normalize, sum approaches 1 */
        let mut m = new_weight_map(3);
        weight_set(&mut m, 0, 1.0);
        weight_set(&mut m, 1, 1.0);
        weight_set(&mut m, 2, 2.0);
        weight_normalize(&mut m);
        let sum: f32 = m.weights.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_weight_transfer_nearest() {
        /* nearest transfer returns source weight */
        let mut src = new_weight_map(2);
        weight_set(&mut src, 0, 0.8);
        weight_set(&mut src, 1, 0.2);
        let src_pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let tgt_pos = vec![[0.1f32, 0.0, 0.0]];
        let result = weight_transfer_nearest(&src, &src_pos, &tgt_pos);
        assert!((result.weights[0] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_weight_blend_midpoint() {
        /* t=0.5 gives average */
        let mut a = new_weight_map(2);
        weight_set(&mut a, 0, 0.0);
        weight_set(&mut a, 1, 1.0);
        let mut b = new_weight_map(2);
        weight_set(&mut b, 0, 1.0);
        weight_set(&mut b, 1, 0.0);
        let c = weight_blend(&a, &b, 0.5);
        assert!((c.weights[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_weight_blend_factor_zero() {
        /* t=0 returns a */
        let mut a = new_weight_map(2);
        weight_set(&mut a, 0, 0.3);
        let b = new_weight_map(2);
        let c = weight_blend(&a, &b, 0.0);
        assert!((c.weights[0] - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_weight_get_out_of_bounds() {
        /* out-of-bounds returns 0 */
        let m = new_weight_map(2);
        assert_eq!(weight_get(&m, 99), 0.0);
    }
}
