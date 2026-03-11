// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! UV scale-to-bounds operation — fits UV islands into a target rectangle.

/// Target bounds for UV scale operation.
pub struct UvTargetBounds {
    pub min_u: f32,
    pub min_v: f32,
    pub max_u: f32,
    pub max_v: f32,
}

impl Default for UvTargetBounds {
    fn default() -> Self {
        Self {
            min_u: 0.0,
            min_v: 0.0,
            max_u: 1.0,
            max_v: 1.0,
        }
    }
}

/// Scale and translate UVs to fit within target bounds, preserving aspect ratio.
pub fn scale_uvs_to_bounds(uvs: &mut [[f32; 2]], target: &UvTargetBounds) -> bool {
    if uvs.is_empty() {
        return false;
    }
    let mut min_u = uvs[0][0];
    let mut min_v = uvs[0][1];
    let mut max_u = uvs[0][0];
    let mut max_v = uvs[0][1];
    for uv in uvs.iter().skip(1) {
        min_u = min_u.min(uv[0]);
        min_v = min_v.min(uv[1]);
        max_u = max_u.max(uv[0]);
        max_v = max_v.max(uv[1]);
    }
    let src_w = max_u - min_u;
    let src_h = max_v - min_v;
    let tgt_w = target.max_u - target.min_u;
    let tgt_h = target.max_v - target.min_v;
    if src_w < 1e-9 && src_h < 1e-9 {
        return false;
    }
    let scale = if src_w < 1e-9 {
        tgt_h / src_h
    } else if src_h < 1e-9 {
        tgt_w / src_w
    } else {
        (tgt_w / src_w).min(tgt_h / src_h)
    };
    for uv in uvs.iter_mut() {
        uv[0] = target.min_u + (uv[0] - min_u) * scale;
        uv[1] = target.min_v + (uv[1] - min_v) * scale;
    }
    true
}

/// Scale UVs uniformly by a factor around the centroid.
pub fn scale_uvs_uniform(uvs: &mut [[f32; 2]], factor: f32) {
    if uvs.is_empty() {
        return;
    }
    let mut cu = 0.0f32;
    let mut cv = 0.0f32;
    for uv in uvs.iter() {
        cu += uv[0];
        cv += uv[1];
    }
    let n = uvs.len() as f32;
    cu /= n;
    cv /= n;
    for uv in uvs.iter_mut() {
        uv[0] = cu + (uv[0] - cu) * factor;
        uv[1] = cv + (uv[1] - cv) * factor;
    }
}

/// Compute UV extents [width, height].
pub fn uv_extents(uvs: &[[f32; 2]]) -> [f32; 2] {
    if uvs.is_empty() {
        return [0.0, 0.0];
    }
    let mut min_u = uvs[0][0];
    let mut min_v = uvs[0][1];
    let mut max_u = uvs[0][0];
    let mut max_v = uvs[0][1];
    for uv in uvs.iter().skip(1) {
        min_u = min_u.min(uv[0]);
        min_v = min_v.min(uv[1]);
        max_u = max_u.max(uv[0]);
        max_v = max_v.max(uv[1]);
    }
    [max_u - min_u, max_v - min_v]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_to_bounds_moves_to_unit_square() {
        let mut uvs = vec![[2.0f32, 3.0], [4.0, 3.0], [4.0, 5.0], [2.0, 5.0]];
        let target = UvTargetBounds::default();
        let ok = scale_uvs_to_bounds(&mut uvs, &target);
        assert!(ok /* successful */);
        let ext = uv_extents(&uvs);
        assert!(ext[0] <= 1.0 + 1e-5 /* fits in width */);
        assert!(ext[1] <= 1.0 + 1e-5 /* fits in height */);
    }

    #[test]
    fn scale_empty_returns_false() {
        let mut uvs: Vec<[f32; 2]> = vec![];
        let ok = scale_uvs_to_bounds(&mut uvs, &UvTargetBounds::default());
        assert!(!ok /* empty */);
    }

    #[test]
    fn scale_uniform_factor_1_unchanged() {
        let mut uvs = vec![[0.2f32, 0.3], [0.8, 0.7]];
        let original = uvs.clone();
        scale_uvs_uniform(&mut uvs, 1.0);
        for (a, b) in uvs.iter().zip(original.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-6 /* unchanged */);
        }
    }

    #[test]
    fn scale_uniform_factor_2_doubles_extents() {
        let mut uvs = vec![[0.0f32, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let ext_before = uv_extents(&uvs);
        scale_uvs_uniform(&mut uvs, 2.0);
        let ext_after = uv_extents(&uvs);
        assert!((ext_after[0] - ext_before[0] * 2.0).abs() < 1e-5 /* doubled width */);
    }

    #[test]
    fn uv_extents_empty() {
        let ext = uv_extents(&[]);
        assert_eq!(ext, [0.0, 0.0] /* zero */);
    }

    #[test]
    fn uv_extents_single() {
        let ext = uv_extents(&[[0.5, 0.5]]);
        assert_eq!(ext, [0.0, 0.0] /* zero extent */);
    }

    #[test]
    fn uv_extents_correct() {
        let uvs = vec![[0.1f32, 0.2], [0.9, 0.8]];
        let ext = uv_extents(&uvs);
        assert!((ext[0] - 0.8).abs() < 1e-6 /* width 0.8 */);
        assert!((ext[1] - 0.6).abs() < 1e-6 /* height 0.6 */);
    }

    #[test]
    fn default_bounds_is_unit_square() {
        let b = UvTargetBounds::default();
        assert!((b.max_u - 1.0).abs() < 1e-6 /* max U = 1 */);
        assert!((b.max_v - 1.0).abs() < 1e-6 /* max V = 1 */);
    }

    #[test]
    fn scale_uniform_empty_no_panic() {
        let mut uvs: Vec<[f32; 2]> = vec![];
        scale_uvs_uniform(&mut uvs, 2.0);
        assert_eq!(uvs.len(), 0 /* still empty */);
    }
}
