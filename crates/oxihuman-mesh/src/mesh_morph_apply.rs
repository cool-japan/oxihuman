#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Apply morph target (shape key) to base mesh.

#[allow(dead_code)]
pub struct MorphApplyResult {
    pub verts: Vec<[f32; 3]>,
}

#[allow(dead_code)]
pub fn apply_morph(
    base: &[[f32; 3]],
    offsets: &[[f32; 3]],
    weight: f32,
) -> MorphApplyResult {
    let n = base.len().min(offsets.len());
    let mut verts = base.to_vec();
    for i in 0..n {
        verts[i][0] += weight * offsets[i][0];
        verts[i][1] += weight * offsets[i][1];
        verts[i][2] += weight * offsets[i][2];
    }
    MorphApplyResult { verts }
}

#[allow(dead_code)]
pub fn apply_multiple_morphs(
    base: &[[f32; 3]],
    morphs: &[(&[[f32; 3]], f32)],
) -> MorphApplyResult {
    let mut verts = base.to_vec();
    for (offsets, weight) in morphs {
        let n = verts.len().min(offsets.len());
        for i in 0..n {
            verts[i][0] += weight * offsets[i][0];
            verts[i][1] += weight * offsets[i][1];
            verts[i][2] += weight * offsets[i][2];
        }
    }
    MorphApplyResult { verts }
}

#[allow(dead_code)]
pub fn morph_magnitude(offsets: &[[f32; 3]]) -> f32 {
    if offsets.is_empty() {
        return 0.0;
    }
    let mut sum = 0.0f32;
    for o in offsets {
        sum += (o[0] * o[0] + o[1] * o[1] + o[2] * o[2]).sqrt();
    }
    sum / offsets.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
    }

    fn offsets() -> Vec<[f32; 3]> {
        vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
    }

    #[test]
    fn apply_morph_weight_one() {
        let result = apply_morph(&base(), &offsets(), 1.0);
        assert!((result.verts[0][0] - 1.0).abs() < 1e-6);
        assert!((result.verts[1][1] - 1.0).abs() < 1e-6);
        assert!((result.verts[2][2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn apply_morph_weight_zero_unchanged() {
        let result = apply_morph(&base(), &offsets(), 0.0);
        assert!((result.verts[0][0]).abs() < 1e-6);
    }

    #[test]
    fn apply_morph_half_weight() {
        let result = apply_morph(&base(), &offsets(), 0.5);
        assert!((result.verts[0][0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn apply_morph_preserves_count() {
        let result = apply_morph(&base(), &offsets(), 1.0);
        assert_eq!(result.verts.len(), base().len());
    }

    #[test]
    fn apply_multiple_morphs_additive() {
        let off1: Vec<[f32; 3]> = vec![[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let off2: Vec<[f32; 3]> = vec![[0.0, 1.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let result = apply_multiple_morphs(&base(), &[(&off1, 1.0), (&off2, 1.0)]);
        assert!((result.verts[0][0] - 1.0).abs() < 1e-6);
        assert!((result.verts[0][1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn apply_multiple_morphs_empty_list() {
        let result = apply_multiple_morphs(&base(), &[]);
        assert_eq!(result.verts.len(), base().len());
        assert!((result.verts[0][0]).abs() < 1e-6);
    }

    #[test]
    fn morph_magnitude_zero_offsets() {
        let offs = vec![[0.0f32; 3]; 3];
        assert!((morph_magnitude(&offs)).abs() < 1e-6);
    }

    #[test]
    fn morph_magnitude_unit_offsets() {
        let offs = vec![[1.0f32, 0.0, 0.0]; 3];
        assert!((morph_magnitude(&offs) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn morph_magnitude_empty() {
        assert!((morph_magnitude(&[])).abs() < 1e-6);
    }
}
