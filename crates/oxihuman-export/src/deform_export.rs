// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]

/// Deformation data export (lattice, cage, etc.).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DeformExport {
    pub deform_type: DeformType,
    pub original_positions: Vec<[f32; 3]>,
    pub deformed_positions: Vec<[f32; 3]>,
}

/// Type of deformation.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum DeformType {
    Lattice,
    Cage,
    Morph,
    Twist,
}

#[allow(dead_code)]
impl DeformExport {
    pub fn new(deform_type: DeformType, original: Vec<[f32; 3]>, deformed: Vec<[f32; 3]>) -> Self {
        Self { deform_type, original_positions: original, deformed_positions: deformed }
    }

    pub fn vertex_count(&self) -> usize { self.original_positions.len() }

    /// Compute max displacement.
    pub fn max_displacement(&self) -> f32 {
        self.original_positions.iter().zip(self.deformed_positions.iter())
            .map(|(o, d)| {
                let dx = o[0]-d[0]; let dy = o[1]-d[1]; let dz = o[2]-d[2];
                (dx*dx+dy*dy+dz*dz).sqrt()
            })
            .fold(0.0f32, f32::max)
    }

    /// Compute average displacement.
    pub fn average_displacement(&self) -> f32 {
        if self.original_positions.is_empty() { return 0.0; }
        let total: f32 = self.original_positions.iter().zip(self.deformed_positions.iter())
            .map(|(o, d)| {
                let dx = o[0]-d[0]; let dy = o[1]-d[1]; let dz = o[2]-d[2];
                (dx*dx+dy*dy+dz*dz).sqrt()
            }).sum();
        total / self.original_positions.len() as f32
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self.vertex_count() as u32).to_le_bytes());
        for p in &self.deformed_positions {
            for &f in p { bytes.extend_from_slice(&f.to_le_bytes()); }
        }
        bytes
    }

    pub fn to_json(&self) -> String {
        format!(
            "{{\"type\":\"{:?}\",\"vertices\":{},\"max_disp\":{},\"avg_disp\":{}}}",
            self.deform_type, self.vertex_count(), self.max_displacement(), self.average_displacement()
        )
    }
}

/// Compute deltas (deformed - original).
#[allow(dead_code)]
pub fn compute_deltas(de: &DeformExport) -> Vec<[f32; 3]> {
    de.original_positions.iter().zip(de.deformed_positions.iter())
        .map(|(o, d)| [d[0]-o[0], d[1]-o[1], d[2]-o[2]])
        .collect()
}

/// Validate deform export.
#[allow(dead_code)]
pub fn validate_deform(de: &DeformExport) -> bool {
    de.original_positions.len() == de.deformed_positions.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> DeformExport {
        DeformExport::new(
            DeformType::Lattice,
            vec![[0.0,0.0,0.0],[1.0,0.0,0.0]],
            vec![[0.0,0.1,0.0],[1.0,0.1,0.0]],
        )
    }

    #[test]
    fn test_vertex_count() { assert_eq!(sample().vertex_count(), 2); }

    #[test]
    fn test_max_displacement() { assert!(sample().max_displacement() > 0.0); }

    #[test]
    fn test_avg_displacement() { assert!(sample().average_displacement() > 0.0); }

    #[test]
    fn test_to_bytes() { assert!(!sample().to_bytes().is_empty()); }

    #[test]
    fn test_to_json() { assert!(sample().to_json().contains("Lattice")); }

    #[test]
    fn test_validate() { assert!(validate_deform(&sample())); }

    #[test]
    fn test_deltas() {
        let d = compute_deltas(&sample());
        assert!((d[0][1] - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_invalid() {
        let de = DeformExport::new(DeformType::Morph, vec![[0.0;3]], vec![]);
        assert!(!validate_deform(&de));
    }

    #[test]
    fn test_no_displacement() {
        let de = DeformExport::new(DeformType::Cage, vec![[0.0;3]], vec![[0.0;3]]);
        assert!((de.max_displacement()).abs() < 1e-6);
    }

    #[test]
    fn test_deform_type() {
        assert_eq!(sample().deform_type, DeformType::Lattice);
    }
}
