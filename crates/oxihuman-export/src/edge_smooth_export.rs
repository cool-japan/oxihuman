// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export per-edge smooth/hard marking data.

/// Edge smoothness descriptor.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeSmooth {
    Smooth,
    Hard,
}

/// Per-edge smooth flag storage.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct EdgeSmoothExport {
    pub edges: Vec<(u32, u32)>,
    pub smooth_flags: Vec<EdgeSmooth>,
}

/// Create a new export.
#[allow(dead_code)]
pub fn new_edge_smooth_export() -> EdgeSmoothExport {
    EdgeSmoothExport::default()
}

/// Add an edge with its smooth flag.
#[allow(dead_code)]
pub fn add_edge(export: &mut EdgeSmoothExport, a: u32, b: u32, smooth: EdgeSmooth) {
    let key = if a <= b { (a, b) } else { (b, a) };
    export.edges.push(key);
    export.smooth_flags.push(smooth);
}

/// Count smooth edges.
#[allow(dead_code)]
pub fn smooth_count(export: &EdgeSmoothExport) -> usize {
    export
        .smooth_flags
        .iter()
        .filter(|&&s| s == EdgeSmooth::Smooth)
        .count()
}

/// Count hard edges.
#[allow(dead_code)]
pub fn hard_count(export: &EdgeSmoothExport) -> usize {
    export
        .smooth_flags
        .iter()
        .filter(|&&s| s == EdgeSmooth::Hard)
        .count()
}

/// Find the smooth flag for an edge (a, b).
#[allow(dead_code)]
pub fn edge_flag(export: &EdgeSmoothExport, a: u32, b: u32) -> Option<EdgeSmooth> {
    let key = if a <= b { (a, b) } else { (b, a) };
    export
        .edges
        .iter()
        .zip(export.smooth_flags.iter())
        .find(|(&e, _)| e == key)
        .map(|(_, &f)| f)
}

/// Build from triangle indices, marking all edges smooth.
#[allow(dead_code)]
pub fn from_triangles_all_smooth(indices: &[u32]) -> EdgeSmoothExport {
    let mut export = new_edge_smooth_export();
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0], tri[1], tri[2]);
        for &(u, v) in &[(a, b), (b, c), (a, c)] {
            if edge_flag(&export, u, v).is_none() {
                add_edge(&mut export, u, v, EdgeSmooth::Smooth);
            }
        }
    }
    export
}

/// Serialise to byte buffer: 1 = smooth, 0 = hard.
#[allow(dead_code)]
pub fn serialise_edge_smooth(export: &EdgeSmoothExport) -> Vec<u8> {
    export
        .smooth_flags
        .iter()
        .map(|&s| if s == EdgeSmooth::Smooth { 1u8 } else { 0u8 })
        .collect()
}

/// Fraction of edges that are smooth.
#[allow(dead_code)]
pub fn smooth_fraction(export: &EdgeSmoothExport) -> f32 {
    if export.smooth_flags.is_empty() {
        return 0.0;
    }
    smooth_count(export) as f32 / export.smooth_flags.len() as f32
}

/// Mark all edges as hard.
#[allow(dead_code)]
pub fn mark_all_hard(export: &mut EdgeSmoothExport) {
    for flag in &mut export.smooth_flags {
        *flag = EdgeSmooth::Hard;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_edge() {
        let mut e = new_edge_smooth_export();
        add_edge(&mut e, 0, 1, EdgeSmooth::Smooth);
        assert_eq!(smooth_count(&e), 1);
    }

    #[test]
    fn test_hard_count() {
        let mut e = new_edge_smooth_export();
        add_edge(&mut e, 0, 1, EdgeSmooth::Hard);
        assert_eq!(hard_count(&e), 1);
        assert_eq!(smooth_count(&e), 0);
    }

    #[test]
    fn test_edge_flag_found() {
        let mut e = new_edge_smooth_export();
        add_edge(&mut e, 2, 5, EdgeSmooth::Hard);
        assert_eq!(edge_flag(&e, 5, 2), Some(EdgeSmooth::Hard));
    }

    #[test]
    fn test_edge_flag_not_found() {
        let e = new_edge_smooth_export();
        assert!(edge_flag(&e, 0, 1).is_none());
    }

    #[test]
    fn test_from_triangles_all_smooth() {
        let idx = vec![0u32, 1, 2];
        let e = from_triangles_all_smooth(&idx);
        assert_eq!(smooth_count(&e), 3);
    }

    #[test]
    fn test_serialise_edge_smooth() {
        let mut e = new_edge_smooth_export();
        add_edge(&mut e, 0, 1, EdgeSmooth::Smooth);
        add_edge(&mut e, 1, 2, EdgeSmooth::Hard);
        let b = serialise_edge_smooth(&e);
        assert_eq!(b, vec![1, 0]);
    }

    #[test]
    fn test_smooth_fraction() {
        let mut e = new_edge_smooth_export();
        add_edge(&mut e, 0, 1, EdgeSmooth::Smooth);
        add_edge(&mut e, 1, 2, EdgeSmooth::Hard);
        assert!((smooth_fraction(&e) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_mark_all_hard() {
        let mut e = from_triangles_all_smooth(&[0u32, 1, 2]);
        mark_all_hard(&mut e);
        assert_eq!(hard_count(&e), e.smooth_flags.len());
    }

    #[test]
    fn test_empty_smooth_fraction() {
        let e = new_edge_smooth_export();
        assert_eq!(smooth_fraction(&e), 0.0);
    }

    #[test]
    fn test_canonical_order() {
        let mut e = new_edge_smooth_export();
        add_edge(&mut e, 5, 2, EdgeSmooth::Smooth);
        assert_eq!(e.edges[0], (2, 5));
    }
}
