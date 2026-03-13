// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Crease edge weighting tool.

use std::collections::HashMap;

/// Canonical (ordered) edge key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeKey(pub u32, pub u32);

impl EdgeKey {
    pub fn new(a: u32, b: u32) -> Self {
        if a <= b {
            EdgeKey(a, b)
        } else {
            EdgeKey(b, a)
        }
    }
}

/// A crease weight map: edge -> sharpness (0.0 = smooth, 1.0 = fully sharp).
#[derive(Debug, Clone, Default)]
pub struct CreaseTool {
    pub creases: HashMap<EdgeKey, f32>,
}

/// Create a new crease tool.
pub fn new_crease_tool() -> CreaseTool {
    CreaseTool {
        creases: HashMap::new(),
    }
}

/// Set the crease sharpness for an edge.
pub fn set_crease(tool: &mut CreaseTool, a: u32, b: u32, sharpness: f32) {
    let key = EdgeKey::new(a, b);
    let s = sharpness.clamp(0.0, 1.0);
    if s < 1e-8 {
        tool.creases.remove(&key);
    } else {
        tool.creases.insert(key, s);
    }
}

/// Get the crease sharpness for an edge (0.0 if not set).
pub fn get_crease(tool: &CreaseTool, a: u32, b: u32) -> f32 {
    *tool.creases.get(&EdgeKey::new(a, b)).unwrap_or(&0.0)
}

/// Remove a crease.
pub fn remove_crease(tool: &mut CreaseTool, a: u32, b: u32) {
    tool.creases.remove(&EdgeKey::new(a, b));
}

/// Count total creased edges.
pub fn crease_count(tool: &CreaseTool) -> usize {
    tool.creases.len()
}

/// Max sharpness in the crease map.
pub fn max_sharpness(tool: &CreaseTool) -> f32 {
    tool.creases.values().cloned().fold(0.0f32, f32::max)
}

/// Average sharpness.
pub fn avg_sharpness(tool: &CreaseTool) -> f32 {
    if tool.creases.is_empty() {
        return 0.0;
    }
    tool.creases.values().sum::<f32>() / tool.creases.len() as f32
}

/// Auto-crease edges whose dihedral angle exceeds a threshold (degrees).
pub fn auto_crease_by_dihedral(
    tool: &mut CreaseTool,
    positions: &[[f32; 3]],
    indices: &[u32],
    threshold_deg: f32,
    sharpness: f32,
) {
    let threshold_rad = threshold_deg * std::f32::consts::PI / 180.0;
    let face_count = indices.len() / 3;
    /* build edge-to-face map */
    let mut edge_faces: HashMap<EdgeKey, Vec<usize>> = HashMap::new();
    for fi in 0..face_count {
        let base = fi * 3;
        let (a, b, c) = (indices[base], indices[base + 1], indices[base + 2]);
        for &ek in &[EdgeKey::new(a, b), EdgeKey::new(b, c), EdgeKey::new(c, a)] {
            edge_faces.entry(ek).or_default().push(fi);
        }
    }
    /* compute face normals */
    let face_normal = |fi: usize| -> [f32; 3] {
        let base = fi * 3;
        let (a, b, c) = (
            indices[base] as usize,
            indices[base + 1] as usize,
            indices[base + 2] as usize,
        );
        let pa = positions[a];
        let pb = positions[b];
        let pc = positions[c];
        let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let ac = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let n = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-10);
        [n[0] / len, n[1] / len, n[2] / len]
    };
    for (ek, faces) in &edge_faces {
        if faces.len() == 2 {
            let n0 = face_normal(faces[0]);
            let n1 = face_normal(faces[1]);
            let dot = (n0[0] * n1[0] + n0[1] * n1[1] + n0[2] * n1[2]).clamp(-1.0, 1.0);
            let angle = dot.acos();
            if angle > threshold_rad {
                tool.creases.insert(*ek, sharpness.clamp(0.0, 1.0));
            }
        }
    }
}

/// Clear all creases.
pub fn clear_creases(tool: &mut CreaseTool) {
    tool.creases.clear();
}

/// List all creased edge pairs.
pub fn list_creases(tool: &CreaseTool) -> Vec<(u32, u32, f32)> {
    tool.creases.iter().map(|(k, &s)| (k.0, k.1, s)).collect()
}

/// Scale all sharpness values by a factor.
pub fn scale_creases(tool: &mut CreaseTool, factor: f32) {
    for s in tool.creases.values_mut() {
        *s = (*s * factor).clamp(0.0, 1.0);
    }
    tool.creases.retain(|_, s| *s > 1e-8);
}

#[cfg(test)]
mod tests {
    use super::*;

    /* set and get */
    #[test]
    fn test_set_get_crease() {
        let mut t = new_crease_tool();
        set_crease(&mut t, 0, 1, 0.8);
        assert!((get_crease(&t, 0, 1) - 0.8).abs() < 1e-6);
    }

    /* symmetric edge key */
    #[test]
    fn test_edge_key_symmetric() {
        assert_eq!(EdgeKey::new(3, 1), EdgeKey::new(1, 3));
    }

    /* remove_crease */
    #[test]
    fn test_remove_crease() {
        let mut t = new_crease_tool();
        set_crease(&mut t, 0, 1, 0.5);
        remove_crease(&mut t, 0, 1);
        assert_eq!(crease_count(&t), 0);
    }

    /* max_sharpness */
    #[test]
    fn test_max_sharpness() {
        let mut t = new_crease_tool();
        set_crease(&mut t, 0, 1, 0.3);
        set_crease(&mut t, 1, 2, 0.9);
        assert!((max_sharpness(&t) - 0.9).abs() < 1e-6);
    }

    /* avg_sharpness */
    #[test]
    fn test_avg_sharpness() {
        let mut t = new_crease_tool();
        set_crease(&mut t, 0, 1, 0.4);
        set_crease(&mut t, 1, 2, 0.6);
        assert!((avg_sharpness(&t) - 0.5).abs() < 1e-5);
    }

    /* scale_creases */
    #[test]
    fn test_scale_creases() {
        let mut t = new_crease_tool();
        set_crease(&mut t, 0, 1, 0.8);
        scale_creases(&mut t, 0.5);
        assert!((get_crease(&t, 0, 1) - 0.4).abs() < 1e-5);
    }

    /* clear_creases */
    #[test]
    fn test_clear_creases() {
        let mut t = new_crease_tool();
        set_crease(&mut t, 0, 1, 1.0);
        clear_creases(&mut t);
        assert_eq!(crease_count(&t), 0);
    }

    /* auto_crease detects sharp edge */
    #[test]
    fn test_auto_crease_by_dihedral() {
        /* two triangles forming a 90-degree dihedral */
        let pts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, 0.0, 1.0],
        ];
        let idx = vec![0u32, 1, 2, 0, 2, 3];
        let mut t = new_crease_tool();
        auto_crease_by_dihedral(&mut t, &pts, &idx, 30.0, 1.0);
        assert!(crease_count(&t) > 0);
    }
}
