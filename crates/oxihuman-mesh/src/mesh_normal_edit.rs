#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Normal editing: custom normal overrides per vertex.

#[allow(dead_code)]
pub struct NormalEdit {
    pub vert_idx: u32,
    pub custom_normal: [f32; 3],
}

#[allow(dead_code)]
pub struct NormalEditLayer {
    pub edits: Vec<NormalEdit>,
}

#[allow(dead_code)]
pub fn new_normal_edit_layer() -> NormalEditLayer {
    NormalEditLayer { edits: vec![] }
}

#[allow(dead_code)]
pub fn set_custom_normal(layer: &mut NormalEditLayer, vert: u32, n: [f32; 3]) {
    for edit in &mut layer.edits {
        if edit.vert_idx == vert {
            edit.custom_normal = n;
            return;
        }
    }
    layer.edits.push(NormalEdit {
        vert_idx: vert,
        custom_normal: n,
    });
}

#[allow(dead_code)]
pub fn get_custom_normal(layer: &NormalEditLayer, vert: u32) -> Option<[f32; 3]> {
    layer
        .edits
        .iter()
        .find(|e| e.vert_idx == vert)
        .map(|e| e.custom_normal)
}

#[allow(dead_code)]
pub fn apply_normal_edits(base_normals: &[[f32; 3]], layer: &NormalEditLayer) -> Vec<[f32; 3]> {
    let mut out = base_normals.to_vec();
    for edit in &layer.edits {
        let vi = edit.vert_idx as usize;
        if vi < out.len() {
            out[vi] = edit.custom_normal;
        }
    }
    out
}

#[allow(dead_code)]
pub fn edit_count(layer: &NormalEditLayer) -> usize {
    layer.edits.len()
}

// ── New required API ──────────────────────────────────────────────────────────

pub struct NormalEditParams {
    pub mode: u8,
    pub target_normal: [f32; 3],
    pub factor: f32,
}

pub fn new_normal_edit_params(mode: u8) -> NormalEditParams {
    NormalEditParams {
        mode,
        target_normal: [0.0, 1.0, 0.0],
        factor: 1.0,
    }
}

pub fn normal_flip(n: [f32; 3]) -> [f32; 3] {
    [-n[0], -n[1], -n[2]]
}

pub fn normal_normalize(n: [f32; 3]) -> [f32; 3] {
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        [n[0] / len, n[1] / len, n[2] / len]
    }
}

pub fn normal_blend(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    normal_normalize([
        a[0] * (1.0 - t) + b[0] * t,
        a[1] * (1.0 - t) + b[1] * t,
        a[2] * (1.0 - t) + b[2] * t,
    ])
}

pub fn normal_is_valid(n: [f32; 3]) -> bool {
    let mag = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    (mag - 1.0).abs() < 0.01
}

pub fn normal_edit_apply(n: [f32; 3], params: &NormalEditParams) -> [f32; 3] {
    match params.mode {
        0 => normal_flip(n),
        1 => normal_normalize(params.target_normal),
        2 => normal_blend(n, params.target_normal, params.factor),
        _ => n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_layer_is_empty() {
        let layer = new_normal_edit_layer();
        assert_eq!(edit_count(&layer), 0);
    }

    #[test]
    fn set_and_get_normal() {
        let mut layer = new_normal_edit_layer();
        set_custom_normal(&mut layer, 2, [0.0, 1.0, 0.0]);
        let n = get_custom_normal(&layer, 2).unwrap();
        assert!((n[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn get_missing_returns_none() {
        let layer = new_normal_edit_layer();
        assert!(get_custom_normal(&layer, 99).is_none());
    }

    #[test]
    fn set_overrides_existing() {
        let mut layer = new_normal_edit_layer();
        set_custom_normal(&mut layer, 0, [1.0, 0.0, 0.0]);
        set_custom_normal(&mut layer, 0, [0.0, 0.0, 1.0]);
        assert_eq!(edit_count(&layer), 1);
        let n = get_custom_normal(&layer, 0).unwrap();
        assert!((n[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn apply_edits_replaces_base() {
        let base = vec![[0.0, 0.0, 1.0], [1.0, 0.0, 0.0]];
        let mut layer = new_normal_edit_layer();
        set_custom_normal(&mut layer, 0, [0.0, 1.0, 0.0]);
        let out = apply_normal_edits(&base, &layer);
        assert!((out[0][1] - 1.0).abs() < 1e-5);
        assert!((out[1][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn apply_edits_out_of_range_ignored() {
        let base = vec![[1.0, 0.0, 0.0]];
        let mut layer = new_normal_edit_layer();
        set_custom_normal(&mut layer, 100, [0.0, 1.0, 0.0]);
        let out = apply_normal_edits(&base, &layer);
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn edit_count_increments() {
        let mut layer = new_normal_edit_layer();
        set_custom_normal(&mut layer, 0, [1.0, 0.0, 0.0]);
        set_custom_normal(&mut layer, 1, [0.0, 1.0, 0.0]);
        assert_eq!(edit_count(&layer), 2);
    }

    #[test]
    fn apply_empty_layer_returns_base() {
        let base = vec![[0.5, 0.5, 0.0], [0.0, 0.5, 0.5]];
        let layer = new_normal_edit_layer();
        let out = apply_normal_edits(&base, &layer);
        assert_eq!(out.len(), 2);
        assert!((out[0][0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn multiple_edits_on_different_verts() {
        let mut layer = new_normal_edit_layer();
        for i in 0u32..5 {
            set_custom_normal(&mut layer, i, [i as f32, 0.0, 0.0]);
        }
        assert_eq!(edit_count(&layer), 5);
    }
}
