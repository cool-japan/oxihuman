// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Custom split normals per face-corner.

/// A face-corner split normal: face index, corner index (0..3), normal vector.
#[derive(Clone, Copy)]
pub struct SplitNormal {
    pub face: u32,
    pub corner: u8,
    pub normal: [f32; 3],
}

/// Layer storing custom split normals.
pub struct SplitNormalLayer {
    pub entries: Vec<SplitNormal>,
}

/// Create a new empty split normal layer.
pub fn new_split_normal_layer() -> SplitNormalLayer {
    SplitNormalLayer {
        entries: Vec::new(),
    }
}

fn safe_normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-9 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

/// Set a custom split normal for a face-corner; normalises input automatically.
pub fn set_split_normal(layer: &mut SplitNormalLayer, face: u32, corner: u8, normal: [f32; 3]) {
    let n = safe_normalize(normal);
    if let Some(e) = layer
        .entries
        .iter_mut()
        .find(|e| e.face == face && e.corner == corner)
    {
        e.normal = n;
    } else {
        layer.entries.push(SplitNormal {
            face,
            corner,
            normal: n,
        });
    }
}

/// Get the custom split normal for a face-corner; None if not set.
pub fn get_split_normal(layer: &SplitNormalLayer, face: u32, corner: u8) -> Option<[f32; 3]> {
    layer
        .entries
        .iter()
        .find(|e| e.face == face && e.corner == corner)
        .map(|e| e.normal)
}

/// Total number of custom split normals stored.
pub fn split_normal_count(layer: &SplitNormalLayer) -> usize {
    layer.entries.len()
}

/// Clear all custom split normals.
pub fn clear_split_normals(layer: &mut SplitNormalLayer) {
    layer.entries.clear();
}

/// Validate that all normals are unit length (within tolerance).
pub fn validate_split_normals(layer: &SplitNormalLayer) -> bool {
    layer.entries.iter().all(|e| {
        let len_sq =
            e.normal[0] * e.normal[0] + e.normal[1] * e.normal[1] + e.normal[2] * e.normal[2];
        (len_sq - 1.0).abs() < 1e-4
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_layer_is_empty() {
        let layer = new_split_normal_layer();
        assert_eq!(split_normal_count(&layer), 0 /* empty */);
    }

    #[test]
    fn set_and_get_split_normal() {
        let mut layer = new_split_normal_layer();
        set_split_normal(&mut layer, 0, 0, [0.0, 1.0, 0.0]);
        let n = get_split_normal(&layer, 0, 0);
        assert!(n.is_some() /* found */);
        assert!((n.unwrap()[1] - 1.0).abs() < 1e-6 /* Y up */);
    }

    #[test]
    fn get_missing_returns_none() {
        let layer = new_split_normal_layer();
        assert!(get_split_normal(&layer, 5, 2).is_none() /* missing */);
    }

    #[test]
    fn set_normalises_input() {
        let mut layer = new_split_normal_layer();
        set_split_normal(&mut layer, 0, 0, [3.0, 0.0, 0.0]);
        let n = get_split_normal(&layer, 0, 0).unwrap();
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-6 /* normalised */);
    }

    #[test]
    fn overwrite_updates_existing() {
        let mut layer = new_split_normal_layer();
        set_split_normal(&mut layer, 1, 2, [1.0, 0.0, 0.0]);
        set_split_normal(&mut layer, 1, 2, [0.0, 0.0, 1.0]);
        assert_eq!(split_normal_count(&layer), 1 /* not duplicated */);
        let n = get_split_normal(&layer, 1, 2).unwrap();
        assert!((n[2] - 1.0).abs() < 1e-6 /* updated to Z */);
    }

    #[test]
    fn clear_removes_all() {
        let mut layer = new_split_normal_layer();
        set_split_normal(&mut layer, 0, 0, [0.0, 1.0, 0.0]);
        clear_split_normals(&mut layer);
        assert_eq!(split_normal_count(&layer), 0 /* cleared */);
    }

    #[test]
    fn validate_passes_unit_normals() {
        let mut layer = new_split_normal_layer();
        set_split_normal(&mut layer, 0, 0, [1.0, 0.0, 0.0]);
        set_split_normal(&mut layer, 0, 1, [0.0, 1.0, 0.0]);
        assert!(validate_split_normals(&layer) /* valid */);
    }

    #[test]
    fn multiple_face_corners_independent() {
        let mut layer = new_split_normal_layer();
        set_split_normal(&mut layer, 0, 0, [1.0, 0.0, 0.0]);
        set_split_normal(&mut layer, 0, 1, [0.0, 1.0, 0.0]);
        set_split_normal(&mut layer, 1, 0, [0.0, 0.0, 1.0]);
        assert_eq!(split_normal_count(&layer), 3 /* three entries */);
    }

    #[test]
    fn safe_normalize_zero_vector() {
        let n = safe_normalize([0.0, 0.0, 0.0]);
        assert!((n[2] - 1.0).abs() < 1e-6 /* fallback Z */);
    }
}
