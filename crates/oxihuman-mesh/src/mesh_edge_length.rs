#![allow(dead_code)]
//! Edge length statistics.

/// Edge length statistics result.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeLengthStats {
    pub min: f32,
    pub max: f32,
    pub avg: f32,
    pub count: usize,
}

/// Compute the length of a single edge.
#[allow(dead_code)]
pub fn edge_length(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Compute all edge lengths from triangle indices (unique edges).
#[allow(dead_code)]
pub fn all_edge_lengths(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> Vec<f32> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut lengths = Vec::new();
    for tri in indices {
        for i in 0..3 {
            let a = tri[i];
            let b = tri[(i + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            if seen.insert(key) {
                lengths.push(edge_length(positions[a as usize], positions[b as usize]));
            }
        }
    }
    lengths
}

/// Find the minimum edge length.
#[allow(dead_code)]
pub fn min_edge_length(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> f32 {
    all_edge_lengths(positions, indices)
        .into_iter()
        .fold(f32::MAX, f32::min)
}

/// Find the maximum edge length.
#[allow(dead_code)]
pub fn max_edge_length(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> f32 {
    all_edge_lengths(positions, indices)
        .into_iter()
        .fold(0.0f32, f32::max)
}

/// Compute the average edge length.
#[allow(dead_code)]
pub fn avg_edge_length(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> f32 {
    let lengths = all_edge_lengths(positions, indices);
    if lengths.is_empty() {
        return 0.0;
    }
    let total: f32 = lengths.iter().sum();
    total / lengths.len() as f32
}

/// Compute the variance of edge lengths.
#[allow(dead_code)]
pub fn edge_length_variance(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> f32 {
    let lengths = all_edge_lengths(positions, indices);
    if lengths.is_empty() {
        return 0.0;
    }
    let avg = lengths.iter().sum::<f32>() / lengths.len() as f32;
    let var: f32 = lengths.iter().map(|&l| (l - avg) * (l - avg)).sum::<f32>();
    var / lengths.len() as f32
}

/// Return edges shorter than a threshold.
#[allow(dead_code)]
pub fn short_edges(positions: &[[f32; 3]], indices: &[[u32; 3]], threshold: f32) -> Vec<(u32, u32)> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for tri in indices {
        for i in 0..3 {
            let a = tri[i];
            let b = tri[(i + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            if seen.insert(key) {
                let len = edge_length(positions[a as usize], positions[b as usize]);
                if len < threshold {
                    result.push(key);
                }
            }
        }
    }
    result
}

/// Return edges longer than a threshold.
#[allow(dead_code)]
pub fn long_edges(positions: &[[f32; 3]], indices: &[[u32; 3]], threshold: f32) -> Vec<(u32, u32)> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for tri in indices {
        for i in 0..3 {
            let a = tri[i];
            let b = tri[(i + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            if seen.insert(key) {
                let len = edge_length(positions[a as usize], positions[b as usize]);
                if len > threshold {
                    result.push(key);
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let p = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let i = vec![[0u32, 1, 2]];
        (p, i)
    }

    #[test]
    fn test_edge_length_basic() {
        let l = edge_length([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]);
        assert!((l - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_all_edge_lengths() {
        let (p, i) = tri();
        let lengths = all_edge_lengths(&p, &i);
        assert_eq!(lengths.len(), 3);
    }

    #[test]
    fn test_min_edge_length() {
        let (p, i) = tri();
        let m = min_edge_length(&p, &i);
        assert!((m - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_max_edge_length() {
        let (p, i) = tri();
        let m = max_edge_length(&p, &i);
        assert!((m - 2.0f32.sqrt()).abs() < 1e-5);
    }

    #[test]
    fn test_avg_edge_length() {
        let (p, i) = tri();
        let a = avg_edge_length(&p, &i);
        assert!(a > 0.0);
    }

    #[test]
    fn test_avg_edge_length_empty() {
        assert!((avg_edge_length(&[], &[])).abs() < 1e-6);
    }

    #[test]
    fn test_edge_length_variance() {
        let (p, i) = tri();
        let v = edge_length_variance(&p, &i);
        assert!(v >= 0.0);
    }

    #[test]
    fn test_short_edges() {
        let (p, i) = tri();
        let s = short_edges(&p, &i, 1.1);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_long_edges() {
        let (p, i) = tri();
        let l = long_edges(&p, &i, 1.3);
        assert!(!l.is_empty());
    }

    #[test]
    fn test_edge_length_zero() {
        let l = edge_length([1.0, 2.0, 3.0], [1.0, 2.0, 3.0]);
        assert!(l.abs() < 1e-6);
    }
}
