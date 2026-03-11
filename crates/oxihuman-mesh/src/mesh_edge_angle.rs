#![allow(dead_code)]
//! Edge angle computation.

use std::f32::consts::PI;

/// Edge angle data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeAngle {
    pub edges: Vec<[u32; 2]>,
    pub angles: Vec<f32>,
}

/// Compute the dihedral angle at an edge given two face normals.
#[allow(dead_code)]
pub fn compute_edge_angle(n1: [f32; 3], n2: [f32; 3]) -> f32 {
    let dot = n1[0] * n2[0] + n1[1] * n2[1] + n1[2] * n2[2];
    dot.clamp(-1.0, 1.0).acos()
}

/// Compute all edge angles from edge list and face normals.
#[allow(dead_code)]
pub fn all_edge_angles(
    edges: &[[u32; 2]],
    face_normals_a: &[[f32; 3]],
    face_normals_b: &[[f32; 3]],
) -> EdgeAngle {
    let len = edges.len().min(face_normals_a.len()).min(face_normals_b.len());
    let mut angles = Vec::with_capacity(len);
    for i in 0..len {
        angles.push(compute_edge_angle(face_normals_a[i], face_normals_b[i]));
    }
    EdgeAngle {
        edges: edges[..len].to_vec(),
        angles,
    }
}

/// Get edges with angle above threshold (sharp edges).
#[allow(dead_code)]
pub fn sharp_edges_by_angle(ea: &EdgeAngle, threshold: f32) -> Vec<[u32; 2]> {
    ea.edges
        .iter()
        .zip(ea.angles.iter())
        .filter(|(_, a)| **a > threshold)
        .map(|(e, _)| *e)
        .collect()
}

/// Get edges with angle below threshold (smooth edges).
#[allow(dead_code)]
pub fn smooth_edges_by_angle(ea: &EdgeAngle, threshold: f32) -> Vec<[u32; 2]> {
    ea.edges
        .iter()
        .zip(ea.angles.iter())
        .filter(|(_, a)| **a <= threshold)
        .map(|(e, _)| *e)
        .collect()
}

/// Get max edge angle.
#[allow(dead_code)]
pub fn max_edge_angle(ea: &EdgeAngle) -> f32 {
    ea.angles.iter().cloned().fold(0.0_f32, f32::max)
}

/// Get min edge angle.
#[allow(dead_code)]
pub fn min_edge_angle(ea: &EdgeAngle) -> f32 {
    ea.angles.iter().cloned().fold(PI, f32::min)
}

/// Get average edge angle.
#[allow(dead_code)]
pub fn avg_edge_angle(ea: &EdgeAngle) -> f32 {
    if ea.angles.is_empty() {
        return 0.0;
    }
    ea.angles.iter().sum::<f32>() / ea.angles.len() as f32
}

/// Compute angle histogram with given bin count.
#[allow(dead_code)]
pub fn angle_histogram(ea: &EdgeAngle, bins: usize) -> Vec<usize> {
    let bins = bins.max(1);
    let mut hist = vec![0usize; bins];
    let bin_size = PI / bins as f32;
    for &a in &ea.angles {
        let idx = ((a / bin_size) as usize).min(bins - 1);
        hist[idx] += 1;
    }
    hist
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_edge_angle_parallel() {
        let angle = compute_edge_angle([0.0, 1.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(angle.abs() < 1e-6);
    }

    #[test]
    fn test_compute_edge_angle_perpendicular() {
        let angle = compute_edge_angle([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((angle - PI / 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_compute_edge_angle_opposite() {
        let angle = compute_edge_angle([0.0, 1.0, 0.0], [0.0, -1.0, 0.0]);
        assert!((angle - PI).abs() < 1e-5);
    }

    #[test]
    fn test_all_edge_angles() {
        let edges = vec![[0, 1]];
        let na = vec![[0.0, 1.0, 0.0]];
        let nb = vec![[1.0, 0.0, 0.0]];
        let ea = all_edge_angles(&edges, &na, &nb);
        assert_eq!(ea.angles.len(), 1);
    }

    #[test]
    fn test_sharp_edges() {
        let ea = EdgeAngle {
            edges: vec![[0, 1], [1, 2]],
            angles: vec![0.5, 1.5],
        };
        let sharp = sharp_edges_by_angle(&ea, 1.0);
        assert_eq!(sharp.len(), 1);
    }

    #[test]
    fn test_smooth_edges() {
        let ea = EdgeAngle {
            edges: vec![[0, 1], [1, 2]],
            angles: vec![0.5, 1.5],
        };
        let smooth = smooth_edges_by_angle(&ea, 1.0);
        assert_eq!(smooth.len(), 1);
    }

    #[test]
    fn test_max_edge_angle() {
        let ea = EdgeAngle { edges: vec![], angles: vec![0.1, 0.5, 0.3] };
        assert!((max_edge_angle(&ea) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_min_edge_angle() {
        let ea = EdgeAngle { edges: vec![], angles: vec![0.1, 0.5, 0.3] };
        assert!((min_edge_angle(&ea) - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_avg_edge_angle() {
        let ea = EdgeAngle { edges: vec![], angles: vec![1.0, 2.0, 3.0] };
        assert!((avg_edge_angle(&ea) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_angle_histogram() {
        let ea = EdgeAngle { edges: vec![], angles: vec![0.1, 0.5, 2.0] };
        let hist = angle_histogram(&ea, 4);
        assert_eq!(hist.len(), 4);
    }
}
