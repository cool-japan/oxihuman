// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]

/// Per-face color assignment.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceColorMap {
    pub colors: Vec<[f32; 4]>,
}

#[allow(dead_code)]
impl FaceColorMap {
    /// Create with uniform color for all faces.
    pub fn uniform(face_count: usize, color: [f32; 4]) -> Self {
        Self { colors: vec![color; face_count] }
    }

    /// Create from a list of colors.
    pub fn from_colors(colors: Vec<[f32; 4]>) -> Self {
        Self { colors }
    }

    /// Number of faces.
    pub fn face_count(&self) -> usize {
        self.colors.len()
    }

    /// Get color of a face.
    pub fn get(&self, face: usize) -> [f32; 4] {
        self.colors[face]
    }

    /// Set color of a face.
    pub fn set(&mut self, face: usize, color: [f32; 4]) {
        self.colors[face] = color;
    }

    /// Convert face colors to vertex colors by averaging per-vertex.
    #[allow(clippy::needless_range_loop)]
    pub fn to_vertex_colors(&self, indices: &[u32], vertex_count: usize) -> Vec<[f32; 4]> {
        let mut accum = vec![[0.0f32; 4]; vertex_count];
        let mut count = vec![0u32; vertex_count];
        for (fi, tri) in indices.chunks_exact(3).enumerate() {
            if fi >= self.colors.len() { break; }
            let c = &self.colors[fi];
            for &v in tri {
                let vi = v as usize;
                for k in 0..4 { accum[vi][k] += c[k]; }
                count[vi] += 1;
            }
        }
        for i in 0..vertex_count {
            if count[i] > 0 {
                for k in 0..4 { accum[i][k] /= count[i] as f32; }
            }
        }
        accum
    }

    /// Average color across all faces.
    #[allow(clippy::needless_range_loop)]
    pub fn average_color(&self) -> [f32; 4] {
        if self.colors.is_empty() { return [0.0; 4]; }
        let mut avg = [0.0f32; 4];
        for c in &self.colors {
            for k in 0..4 { avg[k] += c[k]; }
        }
        let n = self.colors.len() as f32;
        for k in 0..4 { avg[k] /= n; }
        avg
    }

    /// Check if all faces have the same color.
    pub fn is_uniform(&self) -> bool {
        if self.colors.len() <= 1 { return true; }
        let first = &self.colors[0];
        self.colors.iter().all(|c| c == first)
    }
}

/// Create a gradient face color map.
#[allow(dead_code)]
pub fn gradient_face_colors(face_count: usize, start: [f32; 4], end: [f32; 4]) -> FaceColorMap {
    let mut colors = Vec::with_capacity(face_count);
    for i in 0..face_count {
        let t = if face_count > 1 { i as f32 / (face_count - 1) as f32 } else { 0.0 };
        let mut c = [0.0f32; 4];
        for k in 0..4 { c[k] = start[k] + (end[k] - start[k]) * t; }
        colors.push(c);
    }
    FaceColorMap { colors }
}

/// Serialize face color map to JSON.
#[allow(dead_code)]
pub fn face_color_to_json(map: &FaceColorMap) -> String {
    format!("{{\"face_count\":{},\"uniform\":{}}}", map.face_count(), map.is_uniform())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform() {
        let m = FaceColorMap::uniform(4, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(m.face_count(), 4);
        assert!(m.is_uniform());
    }

    #[test]
    fn test_get_set() {
        let mut m = FaceColorMap::uniform(2, [1.0, 1.0, 1.0, 1.0]);
        m.set(0, [0.0, 0.0, 0.0, 1.0]);
        assert!((m.get(0)[0]).abs() < 1e-6);
    }

    #[test]
    fn test_average_color() {
        let m = FaceColorMap::from_colors(vec![[1.0,0.0,0.0,1.0],[0.0,1.0,0.0,1.0]]);
        let avg = m.average_color();
        assert!((avg[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_not_uniform() {
        let m = FaceColorMap::from_colors(vec![[1.0,0.0,0.0,1.0],[0.0,1.0,0.0,1.0]]);
        assert!(!m.is_uniform());
    }

    #[test]
    fn test_to_vertex_colors() {
        let m = FaceColorMap::uniform(1, [1.0, 0.0, 0.0, 1.0]);
        let vc = m.to_vertex_colors(&[0, 1, 2], 3);
        assert_eq!(vc.len(), 3);
        assert!((vc[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_gradient() {
        let m = gradient_face_colors(3, [0.0,0.0,0.0,1.0], [1.0,1.0,1.0,1.0]);
        assert_eq!(m.face_count(), 3);
        assert!((m.get(2)[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_empty() {
        let m = FaceColorMap::from_colors(vec![]);
        assert_eq!(m.face_count(), 0);
        assert!(m.is_uniform());
    }

    #[test]
    fn test_to_json() {
        let m = FaceColorMap::uniform(2, [1.0, 0.0, 0.0, 1.0]);
        let json = face_color_to_json(&m);
        assert!(json.contains("face_count"));
    }

    #[test]
    fn test_single_face_gradient() {
        let m = gradient_face_colors(1, [0.0,0.0,0.0,1.0], [1.0,1.0,1.0,1.0]);
        assert!((m.get(0)[0]).abs() < 1e-5);
    }

    #[test]
    fn test_average_empty() {
        let m = FaceColorMap::from_colors(vec![]);
        let avg = m.average_color();
        assert_eq!(avg, [0.0, 0.0, 0.0, 0.0]);
    }
}
