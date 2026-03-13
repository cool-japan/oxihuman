// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Mirror and cut a mesh along an axis-aligned plane.

#![allow(dead_code)]

/// Axis along which to mirror or cut.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirrorAxis {
    /// Mirror across X = 0.
    X,
    /// Mirror across Y = 0.
    Y,
    /// Mirror across Z = 0.
    Z,
}

impl MirrorAxis {
    /// Returns the axis index (0=X, 1=Y, 2=Z).
    #[allow(dead_code)]
    pub fn index(self) -> usize {
        match self {
            Self::X => 0,
            Self::Y => 1,
            Self::Z => 2,
        }
    }

    /// Returns the axis name string.
    #[allow(dead_code)]
    pub fn name(self) -> &'static str {
        match self {
            Self::X => "X",
            Self::Y => "Y",
            Self::Z => "Z",
        }
    }
}

/// Result of a mirror-cut operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MirrorCutResult {
    /// Positive-side vertex positions.
    pub positions: Vec<[f32; 3]>,
    /// Triangle indices into `positions`.
    pub indices: Vec<u32>,
    /// Number of vertices kept from the positive side.
    pub kept_count: usize,
    /// Number of vertices discarded from the negative side.
    pub discarded_count: usize,
}

/// Keep only triangles entirely on the positive side of the plane, then
/// append their mirror images (reflected across the plane).
#[allow(dead_code)]
pub fn mirror_cut(
    positions: &[[f32; 3]],
    indices: &[u32],
    axis: MirrorAxis,
    threshold: f32,
) -> MirrorCutResult {
    let ax = axis.index();
    // Collect positive-side triangles
    let mut new_pos: Vec<[f32; 3]> = Vec::new();
    let mut new_idx: Vec<u32> = Vec::new();
    let mut kept = 0usize;
    let mut discarded = 0usize;

    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let ia = indices[t * 3] as usize;
        let ib = indices[t * 3 + 1] as usize;
        let ic = indices[t * 3 + 2] as usize;
        if ia >= positions.len() || ib >= positions.len() || ic >= positions.len() {
            continue;
        }
        let pa = positions[ia];
        let pb = positions[ib];
        let pc = positions[ic];
        if pa[ax] >= threshold && pb[ax] >= threshold && pc[ax] >= threshold {
            // Keep positive side
            let base = new_pos.len() as u32;
            new_pos.push(pa);
            new_pos.push(pb);
            new_pos.push(pc);
            new_idx.extend_from_slice(&[base, base + 1, base + 2]);
            kept += 3;
        } else {
            discarded += 3;
        }
    }

    // Append mirrored (reflected) copies
    let original_count = new_pos.len();
    for i in 0..original_count {
        let mut mirrored = new_pos[i];
        mirrored[ax] = -mirrored[ax];
        new_pos.push(mirrored);
    }
    // Mirror index winding (reverse to maintain outward normals)
    let orig_tri_count = new_idx.len() / 3;
    for t in 0..orig_tri_count {
        let a = new_idx[t * 3] + original_count as u32;
        let b = new_idx[t * 3 + 1] + original_count as u32;
        let c = new_idx[t * 3 + 2] + original_count as u32;
        new_idx.extend_from_slice(&[a, c, b]);
    }

    MirrorCutResult {
        positions: new_pos,
        indices: new_idx,
        kept_count: kept,
        discarded_count: discarded,
    }
}

/// Return only the positive-side vertices (without mirroring).
#[allow(dead_code)]
pub fn cut_positive_side(
    positions: &[[f32; 3]],
    axis: MirrorAxis,
    threshold: f32,
) -> Vec<[f32; 3]> {
    let ax = axis.index();
    positions
        .iter()
        .copied()
        .filter(|p| p[ax] >= threshold)
        .collect()
}

/// Return only the negative-side vertices.
#[allow(dead_code)]
pub fn cut_negative_side(
    positions: &[[f32; 3]],
    axis: MirrorAxis,
    threshold: f32,
) -> Vec<[f32; 3]> {
    let ax = axis.index();
    positions
        .iter()
        .copied()
        .filter(|p| p[ax] < threshold)
        .collect()
}

/// Count vertices strictly on the positive side of the cut plane.
#[allow(dead_code)]
pub fn count_positive_vertices(positions: &[[f32; 3]], axis: MirrorAxis, threshold: f32) -> usize {
    let ax = axis.index();
    positions.iter().filter(|p| p[ax] >= threshold).count()
}

/// Compute the bounding box extents on the given axis.
#[allow(dead_code)]
pub fn axis_extent(positions: &[[f32; 3]], axis: MirrorAxis) -> (f32, f32) {
    let ax = axis.index();
    if positions.is_empty() {
        return (0.0, 0.0);
    }
    let min = positions.iter().map(|p| p[ax]).fold(f32::MAX, f32::min);
    let max = positions.iter().map(|p| p[ax]).fold(f32::MIN, f32::max);
    (min, max)
}

/// Serialise the result as a minimal JSON string.
#[allow(dead_code)]
pub fn mirror_cut_to_json(result: &MirrorCutResult) -> String {
    format!(
        "{{\"vertices\":{},\"indices\":{},\"kept\":{},\"discarded\":{}}}",
        result.positions.len(),
        result.indices.len(),
        result.kept_count,
        result.discarded_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_positions() -> Vec<[f32; 3]> {
        vec![
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 0.0, 1.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 1.0, 0.0],
            [-1.0, 0.0, 1.0],
        ]
    }

    fn sample_indices() -> Vec<u32> {
        vec![0, 1, 2, 3, 4, 5]
    }

    #[test]
    fn test_mirror_axis_index() {
        assert_eq!(MirrorAxis::X.index(), 0);
        assert_eq!(MirrorAxis::Y.index(), 1);
        assert_eq!(MirrorAxis::Z.index(), 2);
    }

    #[test]
    fn test_mirror_axis_name() {
        assert_eq!(MirrorAxis::X.name(), "X");
    }

    #[test]
    fn test_cut_keeps_positive() {
        let pos = sample_positions();
        let idx = sample_indices();
        let result = mirror_cut(&pos, &idx, MirrorAxis::X, 0.0);
        // Only positive x triangle kept before mirroring
        assert!(result.kept_count > 0);
    }

    #[test]
    fn test_cut_discards_negative() {
        let pos = sample_positions();
        let idx = sample_indices();
        let result = mirror_cut(&pos, &idx, MirrorAxis::X, 0.0);
        assert!(result.discarded_count > 0);
    }

    #[test]
    fn test_mirror_doubles_triangles() {
        let pos = sample_positions();
        let idx = sample_indices();
        let result = mirror_cut(&pos, &idx, MirrorAxis::X, 0.0);
        // 1 positive tri -> 2 tris = 6 indices
        assert_eq!(result.indices.len(), 6);
    }

    #[test]
    fn test_cut_positive_side() {
        let pos = sample_positions();
        let kept = cut_positive_side(&pos, MirrorAxis::X, 0.0);
        assert_eq!(kept.len(), 3);
    }

    #[test]
    fn test_cut_negative_side() {
        let pos = sample_positions();
        let neg = cut_negative_side(&pos, MirrorAxis::X, 0.0);
        assert_eq!(neg.len(), 3);
    }

    #[test]
    fn test_count_positive() {
        let pos = sample_positions();
        assert_eq!(count_positive_vertices(&pos, MirrorAxis::X, 0.0), 3);
    }

    #[test]
    fn test_axis_extent() {
        let pos = sample_positions();
        let (mn, mx) = axis_extent(&pos, MirrorAxis::X);
        assert!((mn - (-1.0)).abs() < 1e-5);
        assert!((mx - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let pos = sample_positions();
        let idx = sample_indices();
        let result = mirror_cut(&pos, &idx, MirrorAxis::X, 0.0);
        let json = mirror_cut_to_json(&result);
        assert!(json.contains("vertices"));
    }
}
