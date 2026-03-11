// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]

use std::f32::consts::PI;

/// Edge crease data for subdivision surface control.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CreaseEdge {
    pub edge: (u32, u32),
    pub sharpness: f32,
}

/// Collection of crease edges.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct CreaseEdgeSet {
    pub edges: Vec<CreaseEdge>,
}

#[allow(dead_code)]
impl CreaseEdgeSet {
    /// Create a new empty crease edge set.
    pub fn new() -> Self {
        Self { edges: Vec::new() }
    }

    /// Add a crease edge with given sharpness.
    pub fn add(&mut self, v0: u32, v1: u32, sharpness: f32) {
        let edge = if v0 < v1 { (v0, v1) } else { (v1, v0) };
        let sharpness = sharpness.clamp(0.0, 10.0);
        self.edges.push(CreaseEdge { edge, sharpness });
    }

    /// Find crease sharpness for a given edge.
    pub fn sharpness_of(&self, v0: u32, v1: u32) -> f32 {
        let edge = if v0 < v1 { (v0, v1) } else { (v1, v0) };
        self.edges
            .iter()
            .find(|e| e.edge == edge)
            .map(|e| e.sharpness)
            .unwrap_or(0.0)
    }

    /// Check if an edge is creased.
    pub fn is_creased(&self, v0: u32, v1: u32) -> bool {
        self.sharpness_of(v0, v1) > 0.0
    }

    /// Total number of crease edges.
    pub fn count(&self) -> usize {
        self.edges.len()
    }

    /// Average sharpness.
    pub fn average_sharpness(&self) -> f32 {
        if self.edges.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.edges.iter().map(|e| e.sharpness).sum();
        sum / self.edges.len() as f32
    }

    /// Maximum sharpness.
    pub fn max_sharpness(&self) -> f32 {
        self.edges.iter().map(|e| e.sharpness).fold(0.0f32, f32::max)
    }

    /// Remove edges with zero sharpness.
    pub fn remove_zero(&mut self) {
        self.edges.retain(|e| e.sharpness > 0.0);
    }
}

/// Detect crease edges from face normals using angle threshold.
#[allow(dead_code)]
pub fn detect_crease_edges(
    positions: &[[f32; 3]],
    indices: &[u32],
    angle_threshold_deg: f32,
) -> CreaseEdgeSet {
    let threshold_rad = angle_threshold_deg * PI / 180.0;
    let mut normals = Vec::new();
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let ab = [positions[b][0] - positions[a][0], positions[b][1] - positions[a][1], positions[b][2] - positions[a][2]];
        let ac = [positions[c][0] - positions[a][0], positions[c][1] - positions[a][1], positions[c][2] - positions[a][2]];
        let n = [ab[1]*ac[2]-ab[2]*ac[1], ab[2]*ac[0]-ab[0]*ac[2], ab[0]*ac[1]-ab[1]*ac[0]];
        let len = (n[0]*n[0]+n[1]*n[1]+n[2]*n[2]).sqrt();
        if len > 1e-10 {
            normals.push([n[0]/len, n[1]/len, n[2]/len]);
        } else {
            normals.push([0.0, 1.0, 0.0]);
        }
    }
    let mut set = CreaseEdgeSet::new();
    let tri_count = indices.len() / 3;
    for i in 0..tri_count {
        for j in (i+1)..tri_count {
            for ei in 0..3 {
                let e0 = indices[i*3+ei];
                let e1 = indices[i*3+(ei+1)%3];
                for ej in 0..3 {
                    let f0 = indices[j*3+ej];
                    let f1 = indices[j*3+(ej+1)%3];
                    if (e0 == f1 && e1 == f0) || (e0 == f0 && e1 == f1) {
                        let dot = normals[i][0]*normals[j][0]+normals[i][1]*normals[j][1]+normals[i][2]*normals[j][2];
                        let angle = dot.clamp(-1.0, 1.0).acos();
                        if angle > threshold_rad {
                            set.add(e0, e1, angle / PI * 10.0);
                        }
                    }
                }
            }
        }
    }
    set
}

/// Serialize crease set to JSON.
#[allow(dead_code)]
pub fn crease_set_to_json(set: &CreaseEdgeSet) -> String {
    format!(
        "{{\"count\":{},\"avg_sharpness\":{},\"max_sharpness\":{}}}",
        set.count(),
        set.average_sharpness(),
        set.max_sharpness()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_crease() {
        let mut set = CreaseEdgeSet::new();
        set.add(0, 1, 2.0);
        assert_eq!(set.count(), 1);
    }

    #[test]
    fn test_sharpness_lookup() {
        let mut set = CreaseEdgeSet::new();
        set.add(0, 1, 3.5);
        assert!((set.sharpness_of(0, 1) - 3.5).abs() < 1e-6);
        assert!((set.sharpness_of(1, 0) - 3.5).abs() < 1e-6);
    }

    #[test]
    fn test_is_creased() {
        let mut set = CreaseEdgeSet::new();
        set.add(0, 1, 1.0);
        assert!(set.is_creased(0, 1));
        assert!(!set.is_creased(2, 3));
    }

    #[test]
    fn test_average_sharpness() {
        let mut set = CreaseEdgeSet::new();
        set.add(0, 1, 2.0);
        set.add(1, 2, 4.0);
        assert!((set.average_sharpness() - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_max_sharpness() {
        let mut set = CreaseEdgeSet::new();
        set.add(0, 1, 2.0);
        set.add(1, 2, 5.0);
        assert!((set.max_sharpness() - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_remove_zero() {
        let mut set = CreaseEdgeSet::new();
        set.add(0, 1, 0.0);
        set.add(1, 2, 1.0);
        set.remove_zero();
        assert_eq!(set.count(), 1);
    }

    #[test]
    fn test_clamp_sharpness() {
        let mut set = CreaseEdgeSet::new();
        set.add(0, 1, 15.0);
        assert!((set.edges[0].sharpness - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_empty_set() {
        let set = CreaseEdgeSet::new();
        assert_eq!(set.count(), 0);
        assert_eq!(set.average_sharpness(), 0.0);
    }

    #[test]
    fn test_detect_flat() {
        let pos = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0],[1.5,1.0,0.0]];
        let idx = vec![0,1,2, 1,3,2];
        let set = detect_crease_edges(&pos, &idx, 10.0);
        assert_eq!(set.count(), 0);
    }

    #[test]
    fn test_to_json() {
        let mut set = CreaseEdgeSet::new();
        set.add(0, 1, 2.0);
        let json = crease_set_to_json(&set);
        assert!(json.contains("count"));
    }
}
