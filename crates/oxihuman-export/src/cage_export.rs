// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]

/// Cage mesh export for normal map baking.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CageExport {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub offset: f32,
}

#[allow(dead_code)]
impl CageExport {
    /// Create cage from mesh with uniform offset along normals.
    pub fn from_mesh(positions: &[[f32; 3]], normals: &[[f32; 3]], offset: f32) -> Self {
        let cage_pos: Vec<[f32; 3]> = positions.iter().zip(normals.iter()).map(|(p, n)| {
            [p[0]+n[0]*offset, p[1]+n[1]*offset, p[2]+n[2]*offset]
        }).collect();
        Self { positions: cage_pos, normals: normals.to_vec(), offset }
    }

    pub fn vertex_count(&self) -> usize { self.positions.len() }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self.positions.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.offset.to_le_bytes());
        for p in &self.positions {
            for &f in p { bytes.extend_from_slice(&f.to_le_bytes()); }
        }
        bytes
    }

    pub fn byte_size(&self) -> usize { 8 + self.positions.len() * 12 }

    pub fn to_json(&self) -> String {
        format!("{{\"vertices\":{},\"offset\":{}}}", self.vertex_count(), self.offset)
    }

    /// Compute AABB of cage.
    pub fn aabb(&self) -> ([f32; 3], [f32; 3]) {
        if self.positions.is_empty() { return ([0.0;3],[0.0;3]); }
        let mut min = self.positions[0];
        let mut max = self.positions[0];
        for p in &self.positions {
            for i in 0..3 {
                if p[i] < min[i] { min[i] = p[i]; }
                if p[i] > max[i] { max[i] = p[i]; }
            }
        }
        (min, max)
    }
}

/// Validate cage export.
#[allow(dead_code)]
pub fn validate_cage(cage: &CageExport) -> bool {
    cage.positions.len() == cage.normals.len() && cage.offset > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> CageExport {
        CageExport::from_mesh(
            &[[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],
            &[[0.0,0.0,1.0],[0.0,0.0,1.0],[0.0,0.0,1.0]],
            0.1,
        )
    }

    #[test]
    fn test_vertex_count() { assert_eq!(sample().vertex_count(), 3); }

    #[test]
    fn test_offset_applied() {
        let c = sample();
        assert!((c.positions[0][2] - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_to_bytes() { assert_eq!(sample().to_bytes().len(), sample().byte_size()); }

    #[test]
    fn test_to_json() { assert!(sample().to_json().contains("offset")); }

    #[test]
    fn test_validate() { assert!(validate_cage(&sample())); }

    #[test]
    fn test_aabb() {
        let (min, max) = sample().aabb();
        assert!(max[0] > min[0]);
    }

    #[test]
    fn test_zero_offset_invalid() {
        let c = CageExport::from_mesh(&[[0.0;3]], &[[0.0,0.0,1.0]], 0.0);
        assert!(!validate_cage(&c));
    }

    #[test]
    fn test_empty() {
        let c = CageExport::from_mesh(&[], &[], 1.0);
        assert_eq!(c.vertex_count(), 0);
    }

    #[test]
    fn test_byte_size() { assert!(sample().byte_size() > 0); }

    #[test]
    fn test_large_offset() {
        let c = CageExport::from_mesh(&[[0.0;3]], &[[0.0,1.0,0.0]], 10.0);
        assert!((c.positions[0][1] - 10.0).abs() < 1e-5);
    }
}
