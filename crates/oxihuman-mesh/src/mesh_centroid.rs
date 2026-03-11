#![allow(dead_code)]
//! Compute per-face and per-mesh centroids.

/// Per-face centroid result.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct FaceCentroidResult {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Compute the centroid of a single triangle face.
#[allow(dead_code)]
pub fn mc_face_centroid(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    [
        (a[0] + b[0] + c[0]) / 3.0,
        (a[1] + b[1] + c[1]) / 3.0,
        (a[2] + b[2] + c[2]) / 3.0,
    ]
}

/// Compute the centroid of an entire mesh (average of all vertex positions).
#[allow(dead_code)]
pub fn mc_mesh_centroid(positions: &[[f32; 3]]) -> [f32; 3] {
    if positions.is_empty() {
        return [0.0; 3];
    }
    let n = positions.len() as f32;
    let mut sum = [0.0f32; 3];
    for p in positions {
        sum[0] += p[0];
        sum[1] += p[1];
        sum[2] += p[2];
    }
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

/// Compute a weighted centroid given positions and weights.
#[allow(dead_code)]
pub fn weighted_centroid(positions: &[[f32; 3]], weights: &[f32]) -> [f32; 3] {
    if positions.is_empty() || weights.is_empty() {
        return [0.0; 3];
    }
    let len = positions.len().min(weights.len());
    let mut sum = [0.0f32; 3];
    let mut w_sum = 0.0f32;
    for i in 0..len {
        sum[0] += positions[i][0] * weights[i];
        sum[1] += positions[i][1] * weights[i];
        sum[2] += positions[i][2] * weights[i];
        w_sum += weights[i];
    }
    if w_sum.abs() < 1e-12 {
        return [0.0; 3];
    }
    [sum[0] / w_sum, sum[1] / w_sum, sum[2] / w_sum]
}

/// Compute centroid of selected vertices by index.
#[allow(dead_code)]
pub fn centroid_of_selection(positions: &[[f32; 3]], indices: &[usize]) -> [f32; 3] {
    if indices.is_empty() {
        return [0.0; 3];
    }
    let mut sum = [0.0f32; 3];
    let mut count = 0u32;
    for &i in indices {
        if i < positions.len() {
            sum[0] += positions[i][0];
            sum[1] += positions[i][1];
            sum[2] += positions[i][2];
            count += 1;
        }
    }
    if count == 0 {
        return [0.0; 3];
    }
    let n = count as f32;
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

/// Euclidean distance between two centroids.
#[allow(dead_code)]
pub fn centroid_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Convert a centroid to a flat array (identity, for API consistency).
#[allow(dead_code)]
pub fn centroid_to_array(c: [f32; 3]) -> [f32; 3] {
    c
}

/// Compute a cloud of face centroids for all faces.
#[allow(dead_code)]
pub fn centroid_cloud(
    positions: &[[f32; 3]],
    indices: &[[u32; 3]],
) -> Vec<[f32; 3]> {
    indices
        .iter()
        .map(|tri| {
            let a = positions[tri[0] as usize];
            let b = positions[tri[1] as usize];
            let c = positions[tri[2] as usize];
            mc_face_centroid(a, b, c)
        })
        .collect()
}

/// Count the number of centroids that would be produced.
#[allow(dead_code)]
pub fn centroid_count(indices: &[[u32; 3]]) -> usize {
    indices.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_face_centroid_basic() {
        let c = mc_face_centroid([0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]);
        assert!((c[0] - 1.0).abs() < 1e-6);
        assert!((c[1] - 1.0).abs() < 1e-6);
        assert!((c[2]).abs() < 1e-6);
    }

    #[test]
    fn test_mesh_centroid_basic() {
        let positions = vec![[1.0, 2.0, 3.0], [3.0, 4.0, 5.0]];
        let c = mc_mesh_centroid(&positions);
        assert!((c[0] - 2.0).abs() < 1e-6);
        assert!((c[1] - 3.0).abs() < 1e-6);
        assert!((c[2] - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_mesh_centroid_empty() {
        let c = mc_mesh_centroid(&[]);
        assert_eq!(c, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_weighted_centroid() {
        let positions = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let weights = vec![1.0, 3.0];
        let c = weighted_centroid(&positions, &weights);
        assert!((c[0] - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_weighted_centroid_zero_weight() {
        let c = weighted_centroid(&[[1.0, 2.0, 3.0]], &[0.0]);
        assert_eq!(c, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_centroid_of_selection() {
        let positions = vec![[0.0, 0.0, 0.0], [2.0, 4.0, 6.0], [4.0, 8.0, 12.0]];
        let c = centroid_of_selection(&positions, &[0, 2]);
        assert!((c[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_centroid_distance() {
        let d = centroid_distance([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]);
        assert!((d - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_centroid_to_array() {
        assert_eq!(centroid_to_array([1.0, 2.0, 3.0]), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_centroid_cloud() {
        let positions = vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]];
        let indices = vec![[0u32, 1, 2]];
        let cloud = centroid_cloud(&positions, &indices);
        assert_eq!(cloud.len(), 1);
        assert!((cloud[0][0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_centroid_count() {
        let indices = vec![[0u32, 1, 2], [1, 2, 3]];
        assert_eq!(centroid_count(&indices), 2);
    }
}
