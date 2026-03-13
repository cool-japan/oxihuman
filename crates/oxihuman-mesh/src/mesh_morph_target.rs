// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Morph target (blend shape) storage for mesh.

#[allow(dead_code)]
pub struct MorphTarget {
    pub name: String,
    pub deltas: Vec<[f32; 3]>,
}

#[allow(dead_code)]
pub struct MorphTargetSet {
    pub targets: Vec<MorphTarget>,
    pub base_vertex_count: usize,
}

#[allow(dead_code)]
pub fn new_morph_target_set(base_vertex_count: usize) -> MorphTargetSet {
    MorphTargetSet { targets: Vec::new(), base_vertex_count }
}

#[allow(dead_code)]
pub fn mts_add_target(
    s: &mut MorphTargetSet,
    name: &str,
    deltas: Vec<[f32; 3]>,
) -> Option<usize> {
    if deltas.len() != s.base_vertex_count {
        return None;
    }
    let idx = s.targets.len();
    s.targets.push(MorphTarget { name: name.to_string(), deltas });
    Some(idx)
}

#[allow(dead_code)]
pub fn mts_target_count(s: &MorphTargetSet) -> usize {
    s.targets.len()
}

#[allow(dead_code)]
pub fn mts_apply(
    s: &MorphTargetSet,
    weights: &[f32],
    base_positions: &[[f32; 3]],
) -> Vec<[f32; 3]> {
    let n = base_positions.len();
    let mut result: Vec<[f32; 3]> = base_positions.to_vec();
    for (ti, target) in s.targets.iter().enumerate() {
        let w = if ti < weights.len() { weights[ti] } else { 0.0 };
        if w == 0.0 { continue; }
        let count = n.min(target.deltas.len());
        for (i, res) in result.iter_mut().enumerate().take(count) {
            res[0] += target.deltas[i][0] * w;
            res[1] += target.deltas[i][1] * w;
            res[2] += target.deltas[i][2] * w;
        }
    }
    result
}

#[allow(dead_code)]
pub fn mts_get_target<'a>(s: &'a MorphTargetSet, name: &str) -> Option<&'a MorphTarget> {
    s.targets.iter().find(|t| t.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_set() -> MorphTargetSet {
        new_morph_target_set(3)
    }

    #[test]
    fn test_new_empty() {
        let s = make_set();
        assert_eq!(mts_target_count(&s), 0);
    }

    #[test]
    fn test_add_target() {
        let mut s = make_set();
        let deltas = vec![[0.0, 1.0, 0.0]; 3];
        let idx = mts_add_target(&mut s, "smile", deltas);
        assert_eq!(idx, Some(0));
        assert_eq!(mts_target_count(&s), 1);
    }

    #[test]
    fn test_mismatch_returns_none() {
        let mut s = make_set();
        let deltas = vec![[0.0, 1.0, 0.0]; 2];
        let idx = mts_add_target(&mut s, "bad", deltas);
        assert!(idx.is_none());
    }

    #[test]
    fn test_apply_zero_weights_equals_base() {
        let mut s = make_set();
        let deltas = vec![[0.0, 1.0, 0.0]; 3];
        mts_add_target(&mut s, "smile", deltas);
        let base = vec![[0.0f32, 0.0, 0.0]; 3];
        let result = mts_apply(&s, &[0.0], &base);
        assert_eq!(result, base);
    }

    #[test]
    fn test_apply_full_weight() {
        let mut s = make_set();
        let deltas = vec![[0.0, 1.0, 0.0]; 3];
        mts_add_target(&mut s, "smile", deltas);
        let base = vec![[0.0f32, 0.0, 0.0]; 3];
        let result = mts_apply(&s, &[1.0], &base);
        assert!((result[0][1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_get_target_found() {
        let mut s = make_set();
        let deltas = vec![[0.0f32; 3]; 3];
        mts_add_target(&mut s, "blink", deltas);
        let t = mts_get_target(&s, "blink");
        assert!(t.is_some());
    }

    #[test]
    fn test_get_target_not_found() {
        let s = make_set();
        let t = mts_get_target(&s, "nonexistent");
        assert!(t.is_none());
    }

    #[test]
    fn test_count_multiple() {
        let mut s = make_set();
        for i in 0..4 {
            let deltas = vec![[i as f32, 0.0, 0.0]; 3];
            mts_add_target(&mut s, &format!("t{}", i), deltas);
        }
        assert_eq!(mts_target_count(&s), 4);
    }
}
