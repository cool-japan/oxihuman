// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hair guide curve mesh generation.

/// A single hair guide curve, defined by a sequence of control points.
#[derive(Debug, Clone)]
pub struct HairGuideCurve {
    pub points: Vec<[f32; 3]>,
    pub width: f32,
    pub stiffness: f32,
}

impl HairGuideCurve {
    /// Create a straight guide from root to tip.
    pub fn straight(root: [f32; 3], tip: [f32; 3], segments: usize) -> Self {
        let n = (segments + 1).max(2);
        let points = (0..n)
            .map(|i| {
                let t = i as f32 / (n - 1) as f32;
                [
                    root[0] + t * (tip[0] - root[0]),
                    root[1] + t * (tip[1] - root[1]),
                    root[2] + t * (tip[2] - root[2]),
                ]
            })
            .collect();
        Self {
            points,
            width: 0.002,
            stiffness: 0.8,
        }
    }

    /// Length of the guide curve.
    pub fn arc_length(&self) -> f32 {
        self.points
            .windows(2)
            .map(|w| {
                let d = [w[1][0] - w[0][0], w[1][1] - w[0][1], w[1][2] - w[0][2]];
                (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
            })
            .sum()
    }

    /// Number of segments in the guide.
    pub fn segment_count(&self) -> usize {
        self.points.len().saturating_sub(1)
    }
}

/// A collection of hair guide curves.
#[derive(Debug, Clone, Default)]
pub struct HairGuideMesh {
    pub guides: Vec<HairGuideCurve>,
}

impl HairGuideMesh {
    /// Add a guide curve.
    pub fn add_guide(&mut self, g: HairGuideCurve) {
        self.guides.push(g);
    }

    /// Total number of guide points.
    pub fn total_points(&self) -> usize {
        self.guides.iter().map(|g| g.points.len()).sum()
    }
}

/// Generate a uniform grid of straight guides on a flat patch.
pub fn uniform_guide_grid(
    origin: [f32; 3],
    rows: usize,
    cols: usize,
    spacing: f32,
    length: f32,
    segments: usize,
) -> HairGuideMesh {
    let mut mesh = HairGuideMesh::default();
    for r in 0..rows {
        for c in 0..cols {
            let root = [
                origin[0] + c as f32 * spacing,
                origin[1],
                origin[2] + r as f32 * spacing,
            ];
            let tip = [root[0], root[1] + length, root[2]];
            mesh.add_guide(HairGuideCurve::straight(root, tip, segments));
        }
    }
    mesh
}

/// Validate that all guides have at least 2 points.
pub fn validate_guides(mesh: &HairGuideMesh) -> bool {
    mesh.guides.iter().all(|g| g.points.len() >= 2)
}

/// Average arc length across all guides.
pub fn average_guide_length(mesh: &HairGuideMesh) -> f32 {
    if mesh.guides.is_empty() {
        return 0.0;
    }
    let total: f32 = mesh.guides.iter().map(|g| g.arc_length()).sum();
    total / mesh.guides.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_guide_point_count() {
        /* 4 segments → 5 points */
        let g = HairGuideCurve::straight([0.0; 3], [0.0, 1.0, 0.0], 4);
        assert_eq!(g.points.len(), 5);
    }

    #[test]
    fn arc_length_correct() {
        /* straight guide of length 1 */
        let g = HairGuideCurve::straight([0.0; 3], [0.0, 1.0, 0.0], 4);
        assert!((g.arc_length() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn segment_count() {
        /* 5 points → 4 segments */
        let g = HairGuideCurve::straight([0.0; 3], [1.0, 0.0, 0.0], 4);
        assert_eq!(g.segment_count(), 4);
    }

    #[test]
    fn uniform_grid_guide_count() {
        /* 2×3 grid → 6 guides */
        let m = uniform_guide_grid([0.0; 3], 2, 3, 0.1, 0.5, 4);
        assert_eq!(m.guides.len(), 6);
    }

    #[test]
    fn total_points_count() {
        /* 6 guides × 5 points each */
        let m = uniform_guide_grid([0.0; 3], 2, 3, 0.1, 0.5, 4);
        assert_eq!(m.total_points(), 30);
    }

    #[test]
    fn validate_guides_ok() {
        /* generated guides should be valid */
        let m = uniform_guide_grid([0.0; 3], 2, 2, 0.1, 0.3, 2);
        assert!(validate_guides(&m));
    }

    #[test]
    fn validate_guides_fails_empty() {
        /* empty point list is invalid */
        let mut m = HairGuideMesh::default();
        m.guides.push(HairGuideCurve {
            points: vec![],
            width: 0.002,
            stiffness: 0.8,
        });
        assert!(!validate_guides(&m));
    }

    #[test]
    fn average_length_empty_mesh() {
        /* empty mesh → 0.0 */
        let m = HairGuideMesh::default();
        assert_eq!(average_guide_length(&m), 0.0);
    }

    #[test]
    fn average_length_nonzero() {
        /* average should be > 0 */
        let m = uniform_guide_grid([0.0; 3], 2, 2, 0.1, 1.0, 4);
        assert!(average_guide_length(&m) > 0.0);
    }

    #[test]
    fn default_width() {
        /* default width is 0.002 */
        let g = HairGuideCurve::straight([0.0; 3], [1.0, 0.0, 0.0], 2);
        assert!((g.width - 0.002).abs() < 1e-7);
    }
}
