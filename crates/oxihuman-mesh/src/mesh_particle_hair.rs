// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Particle hair strand mesh generation.

/// A single hair strand: ordered list of control points.
#[derive(Debug, Clone)]
pub struct HairStrand {
    pub root: usize,
    pub points: Vec<[f32; 3]>,
}

impl HairStrand {
    /// Create a straight strand from root position in direction.
    pub fn straight(
        root_idx: usize,
        root_pos: [f32; 3],
        direction: [f32; 3],
        length: f32,
        segments: usize,
    ) -> Self {
        let n = segments + 1;
        let mut points = Vec::with_capacity(n);
        for i in 0..n {
            let t = if segments == 0 {
                0.0
            } else {
                i as f32 / segments as f32
            };
            points.push([
                root_pos[0] + direction[0] * length * t,
                root_pos[1] + direction[1] * length * t,
                root_pos[2] + direction[2] * length * t,
            ]);
        }
        Self {
            root: root_idx,
            points,
        }
    }

    /// Return strand length (sum of segment lengths).
    pub fn strand_length(&self) -> f32 {
        let mut len = 0.0_f32;
        for i in 1..self.points.len() {
            let dx = self.points[i][0] - self.points[i - 1][0];
            let dy = self.points[i][1] - self.points[i - 1][1];
            let dz = self.points[i][2] - self.points[i - 1][2];
            len += (dx * dx + dy * dy + dz * dz).sqrt();
        }
        len
    }

    /// Return segment count.
    pub fn segment_count(&self) -> usize {
        self.points.len().saturating_sub(1)
    }
}

/// Particle hair mesh: collection of strands.
#[derive(Debug, Clone)]
pub struct ParticleHairMesh {
    pub strands: Vec<HairStrand>,
}

impl ParticleHairMesh {
    /// Create an empty particle hair mesh.
    pub fn new() -> Self {
        Self {
            strands: Vec::new(),
        }
    }

    /// Add a strand.
    pub fn add_strand(&mut self, strand: HairStrand) {
        self.strands.push(strand);
    }

    /// Return number of strands.
    pub fn strand_count(&self) -> usize {
        self.strands.len()
    }

    /// Total number of points across all strands.
    pub fn total_points(&self) -> usize {
        self.strands.iter().map(|s| s.points.len()).sum()
    }
}

impl Default for ParticleHairMesh {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute average strand length.
pub fn average_strand_length(mesh: &ParticleHairMesh) -> f32 {
    if mesh.strands.is_empty() {
        return 0.0;
    }
    let sum: f32 = mesh.strands.iter().map(|s| s.strand_length()).sum();
    sum / mesh.strands.len() as f32
}

/// Return the longest strand length.
pub fn max_strand_length(mesh: &ParticleHairMesh) -> f32 {
    mesh.strands
        .iter()
        .map(|s| s.strand_length())
        .fold(0.0_f32, f32::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn straight_strand() -> HairStrand {
        HairStrand::straight(0, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 10.0, 5)
    }

    #[test]
    fn test_strand_point_count() {
        /* straight strand has segments+1 points */
        let s = straight_strand();
        assert_eq!(s.points.len(), 6);
    }

    #[test]
    fn test_strand_length() {
        /* strand length matches requested length */
        let s = straight_strand();
        assert!((s.strand_length() - 10.0).abs() < 1e-4);
    }

    #[test]
    fn test_segment_count() {
        /* segment count is point count - 1 */
        let s = straight_strand();
        assert_eq!(s.segment_count(), 5);
    }

    #[test]
    fn test_add_strand() {
        /* adding strands increments count */
        let mut mesh = ParticleHairMesh::new();
        mesh.add_strand(straight_strand());
        mesh.add_strand(straight_strand());
        assert_eq!(mesh.strand_count(), 2);
    }

    #[test]
    fn test_total_points() {
        /* total points sums across strands */
        let mut mesh = ParticleHairMesh::new();
        mesh.add_strand(straight_strand());
        mesh.add_strand(straight_strand());
        assert_eq!(mesh.total_points(), 12);
    }

    #[test]
    fn test_average_strand_length() {
        /* average length is computed correctly */
        let mut mesh = ParticleHairMesh::new();
        mesh.add_strand(straight_strand());
        let avg = average_strand_length(&mesh);
        assert!((avg - 10.0).abs() < 1e-4);
    }

    #[test]
    fn test_max_strand_length_empty() {
        /* max length on empty mesh is 0 */
        let mesh = ParticleHairMesh::new();
        assert_eq!(max_strand_length(&mesh), 0.0);
    }

    #[test]
    fn test_max_strand_length() {
        /* max length picks the longest */
        let mut mesh = ParticleHairMesh::new();
        mesh.add_strand(HairStrand::straight(0, [0.0; 3], [1.0, 0.0, 0.0], 5.0, 2));
        mesh.add_strand(HairStrand::straight(1, [0.0; 3], [1.0, 0.0, 0.0], 20.0, 2));
        assert!((max_strand_length(&mesh) - 20.0).abs() < 1e-4);
    }
}
