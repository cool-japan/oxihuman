// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Spring network mesh from point cloud with k-nearest edges.

/// A spring edge between two point indices.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct SpringEdge {
    pub a: usize,
    pub b: usize,
    pub rest_length: f32,
}

/// A spring network.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SpringNet {
    pub points: Vec<[f32; 3]>,
    pub edges: Vec<SpringEdge>,
}

/// Build a spring network by connecting each point to its k nearest neighbors.
#[allow(dead_code)]
pub fn build_spring_net(points: &[[f32; 3]], k: usize) -> SpringNet {
    let n = points.len();
    let mut edges: Vec<SpringEdge> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for i in 0..n {
        let mut dists: Vec<(f32, usize)> = (0..n)
            .filter(|&j| j != i)
            .map(|j| (dist3(points[i], points[j]), j))
            .collect();
        dists.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        for (d, j) in dists.iter().take(k) {
            let key = if i < *j { (i, *j) } else { (*j, i) };
            if seen.insert(key) {
                edges.push(SpringEdge {
                    a: i,
                    b: *j,
                    rest_length: *d,
                });
            }
        }
    }

    SpringNet {
        points: points.to_vec(),
        edges,
    }
}

/// Average rest length of all springs.
#[allow(dead_code)]
pub fn average_rest_length(net: &SpringNet) -> f32 {
    if net.edges.is_empty() {
        return 0.0;
    }
    net.edges.iter().map(|e| e.rest_length).sum::<f32>() / net.edges.len() as f32
}

/// Count edges.
#[allow(dead_code)]
pub fn spring_edge_count(net: &SpringNet) -> usize {
    net.edges.len()
}

/// Compute current spring stretch: |current_length - rest_length|.
#[allow(dead_code)]
pub fn spring_stretch(net: &SpringNet) -> Vec<f32> {
    net.edges
        .iter()
        .map(|e| {
            let cur = dist3(net.points[e.a], net.points[e.b]);
            (cur - e.rest_length).abs()
        })
        .collect()
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid_points() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ]
    }

    #[test]
    fn point_count_preserved() {
        let net = build_spring_net(&grid_points(), 2);
        assert_eq!(net.points.len(), 4);
    }

    #[test]
    fn edges_are_unique() {
        let net = build_spring_net(&grid_points(), 3);
        let mut seen = std::collections::HashSet::new();
        for e in &net.edges {
            let key = if e.a < e.b { (e.a, e.b) } else { (e.b, e.a) };
            assert!(seen.insert(key));
        }
    }

    #[test]
    fn rest_lengths_positive() {
        let net = build_spring_net(&grid_points(), 2);
        for e in &net.edges {
            assert!(e.rest_length > 0.0);
        }
    }

    #[test]
    fn average_rest_length_reasonable() {
        let net = build_spring_net(&grid_points(), 2);
        let avg = average_rest_length(&net);
        assert!(avg > 0.0 && avg < 3.0);
    }

    #[test]
    fn stretch_zero_at_rest() {
        let net = build_spring_net(&grid_points(), 2);
        let stretch = spring_stretch(&net);
        for s in stretch {
            assert!(s < 1e-5);
        }
    }

    #[test]
    fn empty_points() {
        let net = build_spring_net(&[], 2);
        assert!(net.edges.is_empty());
    }

    #[test]
    fn single_point_no_edges() {
        let net = build_spring_net(&[[0.0, 0.0, 0.0]], 2);
        assert!(net.edges.is_empty());
    }

    #[test]
    fn k_zero_no_edges() {
        let net = build_spring_net(&grid_points(), 0);
        assert!(net.edges.is_empty());
    }

    #[test]
    fn edge_count_helper() {
        let net = build_spring_net(&grid_points(), 2);
        assert_eq!(spring_edge_count(&net), net.edges.len());
    }

    #[test]
    fn edge_indices_valid() {
        let net = build_spring_net(&grid_points(), 2);
        for e in &net.edges {
            assert!(e.a < net.points.len());
            assert!(e.b < net.points.len());
        }
    }
}
