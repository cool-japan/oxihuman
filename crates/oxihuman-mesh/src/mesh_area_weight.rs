// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]

/// Area-weighted vertex attribute for mesh processing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AreaWeight {
    /// Per-face areas.
    pub face_areas: Vec<f32>,
    /// Per-vertex accumulated area weights.
    pub vertex_weights: Vec<f32>,
}

#[allow(dead_code)]
impl AreaWeight {
    /// Compute area weights from positions and triangle indices.
    pub fn compute(positions: &[[f32; 3]], indices: &[u32]) -> Self {
        let mut face_areas = Vec::new();
        let mut vertex_weights = vec![0.0f32; positions.len()];
        for tri in indices.chunks_exact(3) {
            let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
            let area = triangle_area(&positions[a], &positions[b], &positions[c]);
            face_areas.push(area);
            let third = area / 3.0;
            vertex_weights[a] += third;
            vertex_weights[b] += third;
            vertex_weights[c] += third;
        }
        Self { face_areas, vertex_weights }
    }

    /// Total surface area.
    pub fn total_area(&self) -> f32 {
        self.face_areas.iter().sum()
    }

    /// Number of faces.
    pub fn face_count(&self) -> usize {
        self.face_areas.len()
    }

    /// Normalized vertex weights (sum = 1).
    pub fn normalized_vertex_weights(&self) -> Vec<f32> {
        let total: f32 = self.vertex_weights.iter().sum();
        if total <= 0.0 {
            return vec![0.0; self.vertex_weights.len()];
        }
        self.vertex_weights.iter().map(|w| w / total).collect()
    }

    /// Max face area.
    pub fn max_face_area(&self) -> f32 {
        self.face_areas.iter().cloned().fold(0.0f32, f32::max)
    }

    /// Min face area (among non-zero).
    pub fn min_face_area(&self) -> f32 {
        self.face_areas
            .iter()
            .cloned()
            .filter(|a| *a > 0.0)
            .fold(f32::MAX, f32::min)
    }
}

/// Compute triangle area from 3 vertices.
#[allow(dead_code)]
pub fn triangle_area(a: &[f32; 3], b: &[f32; 3], c: &[f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt()
}

/// Compute area-weighted centroid.
#[allow(dead_code)]
pub fn area_weighted_centroid(positions: &[[f32; 3]], indices: &[u32]) -> [f32; 3] {
    let mut centroid = [0.0f32; 3];
    let mut total_area = 0.0f32;
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let area = triangle_area(&positions[a], &positions[b], &positions[c]);
        let cx = (positions[a][0] + positions[b][0] + positions[c][0]) / 3.0;
        let cy = (positions[a][1] + positions[b][1] + positions[c][1]) / 3.0;
        let cz = (positions[a][2] + positions[b][2] + positions[c][2]) / 3.0;
        centroid[0] += cx * area;
        centroid[1] += cy * area;
        centroid[2] += cz * area;
        total_area += area;
    }
    if total_area > 0.0 {
        centroid[0] /= total_area;
        centroid[1] /= total_area;
        centroid[2] /= total_area;
    }
    centroid
}

/// Compute area weights and return as flat Vec.
#[allow(dead_code)]
pub fn compute_area_weights(positions: &[[f32; 3]], indices: &[u32]) -> Vec<f32> {
    AreaWeight::compute(positions, indices).vertex_weights
}

/// Serialize area weights to JSON string.
#[allow(dead_code)]
pub fn area_weights_to_json(aw: &AreaWeight) -> String {
    format!(
        "{{\"total_area\":{},\"face_count\":{},\"max_face_area\":{},\"min_face_area\":{}}}",
        aw.total_area(),
        aw.face_count(),
        aw.max_face_area(),
        aw.min_face_area()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_tri() -> ([[f32; 3]; 3], Vec<u32>) {
        ([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]], vec![0, 1, 2])
    }

    #[test]
    fn test_triangle_area() {
        let (pos, _) = unit_tri();
        let area = triangle_area(&pos[0], &pos[1], &pos[2]);
        assert!((area - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_area_weight_compute() {
        let (pos, idx) = unit_tri();
        let aw = AreaWeight::compute(&pos, &idx);
        assert_eq!(aw.face_count(), 1);
        assert!((aw.total_area() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_vertex_weights() {
        let (pos, idx) = unit_tri();
        let aw = AreaWeight::compute(&pos, &idx);
        for w in &aw.vertex_weights {
            assert!((*w - 0.5 / 3.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_normalized() {
        let (pos, idx) = unit_tri();
        let aw = AreaWeight::compute(&pos, &idx);
        let norm = aw.normalized_vertex_weights();
        let sum: f32 = norm.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_centroid() {
        let (pos, idx) = unit_tri();
        let c = area_weighted_centroid(&pos, &idx);
        assert!((c[0] - 1.0 / 3.0).abs() < 1e-5);
        assert!((c[1] - 1.0 / 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_max_min_area() {
        let (pos, idx) = unit_tri();
        let aw = AreaWeight::compute(&pos, &idx);
        assert!((aw.max_face_area() - 0.5).abs() < 1e-6);
        assert!((aw.min_face_area() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_empty() {
        let aw = AreaWeight::compute(&[], &[]);
        assert_eq!(aw.total_area(), 0.0);
        assert_eq!(aw.face_count(), 0);
    }

    #[test]
    fn test_compute_area_weights_fn() {
        let (pos, idx) = unit_tri();
        let weights = compute_area_weights(&pos, &idx);
        assert_eq!(weights.len(), 3);
    }

    #[test]
    fn test_to_json() {
        let (pos, idx) = unit_tri();
        let aw = AreaWeight::compute(&pos, &idx);
        let json = area_weights_to_json(&aw);
        assert!(json.contains("total_area"));
    }

    #[test]
    fn test_degenerate_triangle() {
        let pos = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let idx = vec![0, 1, 2];
        let aw = AreaWeight::compute(&pos, &idx);
        assert!(aw.total_area() < 1e-6);
    }
}
