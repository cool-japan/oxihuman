// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]

/// Fur/hair strand export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FurExport {
    pub strands: Vec<FurStrand>,
}

/// A single fur strand.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FurStrand {
    pub points: Vec<[f32; 3]>,
    pub thickness: f32,
}

#[allow(dead_code)]
impl FurExport {
    pub fn new() -> Self { Self { strands: Vec::new() } }

    pub fn add_strand(&mut self, points: Vec<[f32; 3]>, thickness: f32) {
        self.strands.push(FurStrand { points, thickness });
    }

    pub fn strand_count(&self) -> usize { self.strands.len() }

    pub fn total_points(&self) -> usize {
        self.strands.iter().map(|s| s.points.len()).sum()
    }

    pub fn average_length(&self) -> f32 {
        if self.strands.is_empty() { return 0.0; }
        let total: f32 = self.strands.iter().map(|s| strand_length(&s.points)).sum();
        total / self.strands.len() as f32
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self.strands.len() as u32).to_le_bytes());
        for s in &self.strands {
            bytes.extend_from_slice(&(s.points.len() as u32).to_le_bytes());
            bytes.extend_from_slice(&s.thickness.to_le_bytes());
            for p in &s.points {
                for &f in p { bytes.extend_from_slice(&f.to_le_bytes()); }
            }
        }
        bytes
    }

    pub fn to_json(&self) -> String {
        format!(
            "{{\"strands\":{},\"total_points\":{},\"avg_length\":{}}}",
            self.strand_count(), self.total_points(), self.average_length()
        )
    }
}

impl Default for FurExport {
    fn default() -> Self { Self::new() }
}

#[allow(dead_code)]
fn strand_length(points: &[[f32; 3]]) -> f32 {
    let mut len = 0.0f32;
    for pair in points.windows(2) {
        let dx = pair[1][0]-pair[0][0];
        let dy = pair[1][1]-pair[0][1];
        let dz = pair[1][2]-pair[0][2];
        len += (dx*dx+dy*dy+dz*dz).sqrt();
    }
    len
}

/// Validate fur export.
#[allow(dead_code)]
pub fn validate_fur(fur: &FurExport) -> bool {
    fur.strands.iter().all(|s| s.points.len() >= 2 && s.thickness > 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_fur() -> FurExport {
        let mut f = FurExport::new();
        f.add_strand(vec![[0.0,0.0,0.0],[0.0,1.0,0.0]], 0.01);
        f.add_strand(vec![[1.0,0.0,0.0],[1.0,2.0,0.0]], 0.02);
        f
    }

    #[test]
    fn test_strand_count() { assert_eq!(sample_fur().strand_count(), 2); }

    #[test]
    fn test_total_points() { assert_eq!(sample_fur().total_points(), 4); }

    #[test]
    fn test_average_length() {
        let avg = sample_fur().average_length();
        assert!(avg > 0.0);
    }

    #[test]
    fn test_to_bytes() { assert!(!sample_fur().to_bytes().is_empty()); }

    #[test]
    fn test_to_json() { assert!(sample_fur().to_json().contains("strands")); }

    #[test]
    fn test_validate() { assert!(validate_fur(&sample_fur())); }

    #[test]
    fn test_invalid_single_point() {
        let mut f = FurExport::new();
        f.add_strand(vec![[0.0,0.0,0.0]], 0.01);
        assert!(!validate_fur(&f));
    }

    #[test]
    fn test_empty() { assert_eq!(FurExport::new().strand_count(), 0); }

    #[test]
    fn test_default() { assert_eq!(FurExport::default().strand_count(), 0); }

    #[test]
    fn test_strand_length() {
        let len = strand_length(&[[0.0,0.0,0.0],[0.0,3.0,0.0],[0.0,3.0,4.0]]);
        assert!((len - 7.0).abs() < 1e-5);
    }
}
