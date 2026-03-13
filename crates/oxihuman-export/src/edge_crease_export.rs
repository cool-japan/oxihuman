// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]

/// Edge crease export for subdivision surface data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeCreaseExport {
    pub edges: Vec<(u32, u32)>,
    pub sharpness: Vec<f32>,
}

#[allow(dead_code)]
impl EdgeCreaseExport {
    /// Create empty.
    pub fn new() -> Self { Self { edges: Vec::new(), sharpness: Vec::new() } }

    /// Add a crease edge.
    pub fn add(&mut self, v0: u32, v1: u32, sharp: f32) {
        let e = if v0 < v1 { (v0, v1) } else { (v1, v0) };
        self.edges.push(e);
        self.sharpness.push(sharp.clamp(0.0, 10.0));
    }

    /// Edge count.
    pub fn count(&self) -> usize { self.edges.len() }

    /// Get sharpness for an edge.
    pub fn get_sharpness(&self, v0: u32, v1: u32) -> Option<f32> {
        let e = if v0 < v1 { (v0, v1) } else { (v1, v0) };
        self.edges.iter().position(|x| *x == e).map(|i| self.sharpness[i])
    }

    /// Export to binary.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self.edges.len() as u32).to_le_bytes());
        for (&(v0, v1), &s) in self.edges.iter().zip(self.sharpness.iter()) {
            bytes.extend_from_slice(&v0.to_le_bytes());
            bytes.extend_from_slice(&v1.to_le_bytes());
            bytes.extend_from_slice(&s.to_le_bytes());
        }
        bytes
    }

    /// Byte size.
    pub fn byte_size(&self) -> usize { 4 + self.edges.len() * 12 }

    /// Max sharpness.
    pub fn max_sharpness(&self) -> f32 {
        self.sharpness.iter().cloned().fold(0.0f32, f32::max)
    }

    /// Average sharpness.
    pub fn average_sharpness(&self) -> f32 {
        if self.sharpness.is_empty() { return 0.0; }
        self.sharpness.iter().sum::<f32>() / self.sharpness.len() as f32
    }
}

impl Default for EdgeCreaseExport {
    fn default() -> Self { Self::new() }
}

/// Export to JSON.
#[allow(dead_code)]
pub fn edge_crease_to_json(ec: &EdgeCreaseExport) -> String {
    format!(
        "{{\"count\":{},\"max_sharpness\":{},\"avg_sharpness\":{}}}",
        ec.count(), ec.max_sharpness(), ec.average_sharpness()
    )
}

/// Validate crease data.
#[allow(dead_code)]
pub fn validate_edge_crease(ec: &EdgeCreaseExport) -> bool {
    ec.edges.len() == ec.sharpness.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() { assert_eq!(EdgeCreaseExport::new().count(), 0); }

    #[test]
    fn test_add() {
        let mut ec = EdgeCreaseExport::new();
        ec.add(0, 1, 2.5);
        assert_eq!(ec.count(), 1);
    }

    #[test]
    fn test_get_sharpness() {
        let mut ec = EdgeCreaseExport::new();
        ec.add(0, 1, 3.0);
        assert!((ec.get_sharpness(0, 1).expect("should succeed") - 3.0).abs() < 1e-5);
        assert!((ec.get_sharpness(1, 0).expect("should succeed") - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_missing() {
        let ec = EdgeCreaseExport::new();
        assert!(ec.get_sharpness(0, 1).is_none());
    }

    #[test]
    fn test_to_bytes() {
        let mut ec = EdgeCreaseExport::new();
        ec.add(0, 1, 1.0);
        assert_eq!(ec.to_bytes().len(), ec.byte_size());
    }

    #[test]
    fn test_max_sharpness() {
        let mut ec = EdgeCreaseExport::new();
        ec.add(0, 1, 2.0);
        ec.add(1, 2, 5.0);
        assert!((ec.max_sharpness() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_clamp() {
        let mut ec = EdgeCreaseExport::new();
        ec.add(0, 1, 20.0);
        assert!((ec.sharpness[0] - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_validate() {
        let mut ec = EdgeCreaseExport::new();
        ec.add(0, 1, 1.0);
        assert!(validate_edge_crease(&ec));
    }

    #[test]
    fn test_to_json() {
        let mut ec = EdgeCreaseExport::new();
        ec.add(0, 1, 1.0);
        assert!(edge_crease_to_json(&ec).contains("count"));
    }

    #[test]
    fn test_average() {
        let mut ec = EdgeCreaseExport::new();
        ec.add(0, 1, 2.0);
        ec.add(1, 2, 4.0);
        assert!((ec.average_sharpness() - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_default() { assert_eq!(EdgeCreaseExport::default().count(), 0); }
}
