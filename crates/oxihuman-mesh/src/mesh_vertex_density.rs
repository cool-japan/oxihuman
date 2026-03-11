#![allow(dead_code)]
//! Vertex density computation for meshes.

/// Stores per-vertex density values.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexDensity {
    densities: Vec<f32>,
}

/// Compute vertex density based on surrounding edge lengths.
#[allow(dead_code)]
pub fn compute_vertex_density(positions: &[[f32; 3]], indices: &[u32]) -> VertexDensity {
    let n = positions.len();
    let mut edge_sum = vec![0.0f32; n];
    let mut edge_count = vec![0u32; n];
    for tri in indices.chunks(3) {
        if tri.len() < 3 { continue; }
        let a = tri[0] as usize;
        let b = tri[1] as usize;
        let c = tri[2] as usize;
        if a >= n || b >= n || c >= n { continue; }
        let pairs = [(a, b), (b, c), (c, a)];
        for &(i, j) in &pairs {
            let dx = positions[i][0] - positions[j][0];
            let dy = positions[i][1] - positions[j][1];
            let dz = positions[i][2] - positions[j][2];
            let len = (dx * dx + dy * dy + dz * dz).sqrt();
            edge_sum[i] += len;
            edge_count[i] += 1;
            edge_sum[j] += len;
            edge_count[j] += 1;
        }
    }
    let densities: Vec<f32> = (0..n)
        .map(|i| {
            if edge_count[i] == 0 {
                0.0
            } else {
                1.0 / (edge_sum[i] / edge_count[i] as f32)
            }
        })
        .collect();
    VertexDensity { densities }
}

/// Get density at a specific vertex index.
#[allow(dead_code)]
pub fn density_at(vd: &VertexDensity, idx: usize) -> f32 {
    vd.densities.get(idx).copied().unwrap_or(0.0)
}

/// Get minimum density.
#[allow(dead_code)]
pub fn min_density(vd: &VertexDensity) -> f32 {
    vd.densities.iter().copied().fold(f32::INFINITY, f32::min)
}

/// Get maximum density.
#[allow(dead_code)]
pub fn max_density(vd: &VertexDensity) -> f32 {
    vd.densities.iter().copied().fold(f32::NEG_INFINITY, f32::max)
}

/// Get average density.
#[allow(dead_code)]
pub fn avg_density(vd: &VertexDensity) -> f32 {
    if vd.densities.is_empty() {
        return 0.0;
    }
    let sum: f32 = vd.densities.iter().sum();
    sum / vd.densities.len() as f32
}

/// Build a histogram of density values with the given number of bins.
#[allow(dead_code)]
pub fn density_histogram(vd: &VertexDensity, bins: usize) -> Vec<u32> {
    if vd.densities.is_empty() || bins == 0 {
        return vec![0; bins.max(1)];
    }
    let lo = min_density(vd);
    let hi = max_density(vd);
    let range = hi - lo;
    if range <= 0.0 {
        let mut h = vec![0u32; bins];
        h[0] = vd.densities.len() as u32;
        return h;
    }
    let mut h = vec![0u32; bins];
    for &d in &vd.densities {
        let idx = ((d - lo) / range * (bins as f32 - 1.0)).round() as usize;
        let idx = idx.min(bins - 1);
        h[idx] += 1;
    }
    h
}

/// Convert density data to a JSON string.
#[allow(dead_code)]
pub fn density_to_json(vd: &VertexDensity) -> String {
    let vals: Vec<String> = vd.densities.iter().map(|d| format!("{:.6}", d)).collect();
    format!("{{\"densities\":[{}]}}", vals.join(","))
}

/// Return the number of density entries.
#[allow(dead_code)]
pub fn density_count(vd: &VertexDensity) -> usize {
    vd.densities.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
        ]
    }

    fn tri_indices() -> Vec<u32> {
        vec![0, 1, 2]
    }

    #[test]
    fn test_compute_vertex_density() {
        let vd = compute_vertex_density(&tri_positions(), &tri_indices());
        assert_eq!(density_count(&vd), 3);
    }

    #[test]
    fn test_density_at_valid() {
        let vd = compute_vertex_density(&tri_positions(), &tri_indices());
        let d = density_at(&vd, 0);
        assert!(d > 0.0);
    }

    #[test]
    fn test_density_at_out_of_range() {
        let vd = compute_vertex_density(&tri_positions(), &tri_indices());
        assert!((density_at(&vd, 999) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_min_density() {
        let vd = compute_vertex_density(&tri_positions(), &tri_indices());
        assert!(min_density(&vd) > 0.0);
    }

    #[test]
    fn test_max_density() {
        let vd = compute_vertex_density(&tri_positions(), &tri_indices());
        assert!(max_density(&vd) >= min_density(&vd));
    }

    #[test]
    fn test_avg_density() {
        let vd = compute_vertex_density(&tri_positions(), &tri_indices());
        let avg = avg_density(&vd);
        assert!(avg >= min_density(&vd));
        assert!(avg <= max_density(&vd));
    }

    #[test]
    fn test_density_histogram() {
        let vd = compute_vertex_density(&tri_positions(), &tri_indices());
        let h = density_histogram(&vd, 4);
        assert_eq!(h.len(), 4);
        let total: u32 = h.iter().sum();
        assert_eq!(total, 3);
    }

    #[test]
    fn test_density_to_json() {
        let vd = compute_vertex_density(&tri_positions(), &tri_indices());
        let j = density_to_json(&vd);
        assert!(j.contains("densities"));
    }

    #[test]
    fn test_density_count() {
        let vd = compute_vertex_density(&tri_positions(), &tri_indices());
        assert_eq!(density_count(&vd), 3);
    }

    #[test]
    fn test_empty_density() {
        let vd = compute_vertex_density(&[], &[]);
        assert_eq!(density_count(&vd), 0);
        assert!((avg_density(&vd) - 0.0).abs() < 1e-9);
    }
}
