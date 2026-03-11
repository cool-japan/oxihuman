// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]

/// Convex hull export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConvexHullExport {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

#[allow(dead_code)]
impl ConvexHullExport {
    /// Create from vertices and indices.
    pub fn new(vertices: Vec<[f32; 3]>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    /// Vertex count.
    pub fn vertex_count(&self) -> usize { self.vertices.len() }

    /// Face count.
    pub fn face_count(&self) -> usize { self.indices.len() / 3 }

    /// Export to binary.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self.vertices.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&(self.indices.len() as u32).to_le_bytes());
        for v in &self.vertices {
            for &f in v { bytes.extend_from_slice(&f.to_le_bytes()); }
        }
        for &i in &self.indices { bytes.extend_from_slice(&i.to_le_bytes()); }
        bytes
    }

    /// Byte size.
    pub fn byte_size(&self) -> usize {
        8 + self.vertices.len() * 12 + self.indices.len() * 4
    }

    /// Compute AABB.
    pub fn aabb(&self) -> ([f32; 3], [f32; 3]) {
        if self.vertices.is_empty() { return ([0.0; 3], [0.0; 3]); }
        let mut min = self.vertices[0];
        let mut max = self.vertices[0];
        for v in &self.vertices {
            for i in 0..3 {
                if v[i] < min[i] { min[i] = v[i]; }
                if v[i] > max[i] { max[i] = v[i]; }
            }
        }
        (min, max)
    }

    /// Compute centroid.
    pub fn centroid(&self) -> [f32; 3] {
        if self.vertices.is_empty() { return [0.0; 3]; }
        let mut c = [0.0f32; 3];
        for v in &self.vertices { for i in 0..3 { c[i] += v[i]; } }
        let n = self.vertices.len() as f32;
        for c in &mut c { *c /= n; }
        c
    }
}

/// Export to JSON.
#[allow(dead_code)]
pub fn convex_hull_export_to_json(hull: &ConvexHullExport) -> String {
    format!(
        "{{\"vertices\":{},\"faces\":{},\"bytes\":{}}}",
        hull.vertex_count(), hull.face_count(), hull.byte_size()
    )
}

/// Validate hull data.
#[allow(dead_code)]
pub fn validate_hull_export(hull: &ConvexHullExport) -> bool {
    !hull.vertices.is_empty() && hull.indices.iter().all(|&i| (i as usize) < hull.vertices.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tetra() -> ConvexHullExport {
        ConvexHullExport::new(
            vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0],[0.5,0.5,1.0]],
            vec![0,1,2, 0,1,3, 1,2,3, 0,2,3],
        )
    }

    #[test]
    fn test_vertex_count() { assert_eq!(tetra().vertex_count(), 4); }

    #[test]
    fn test_face_count() { assert_eq!(tetra().face_count(), 4); }

    #[test]
    fn test_to_bytes() { assert_eq!(tetra().to_bytes().len(), tetra().byte_size()); }

    #[test]
    fn test_aabb() {
        let (min, max) = tetra().aabb();
        assert!((min[0]).abs() < 1e-5);
        assert!((max[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_centroid() {
        let c = tetra().centroid();
        assert!(c[0] > 0.0);
    }

    #[test]
    fn test_validate() { assert!(validate_hull_export(&tetra())); }

    #[test]
    fn test_to_json() { assert!(convex_hull_export_to_json(&tetra()).contains("vertices")); }

    #[test]
    fn test_empty() {
        let h = ConvexHullExport::new(vec![], vec![]);
        assert!(!validate_hull_export(&h));
    }

    #[test]
    fn test_byte_size() { assert!(tetra().byte_size() > 0); }

    #[test]
    fn test_single_vertex() {
        let h = ConvexHullExport::new(vec![[1.0,2.0,3.0]], vec![]);
        assert_eq!(h.centroid(), [1.0, 2.0, 3.0]);
    }
}
