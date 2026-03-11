// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Path cut: cut a mesh along a vertex path.

/// A path of vertex indices describing a cut.
#[derive(Debug, Clone, Default)]
pub struct PathCut {
    pub vertex_path: Vec<usize>,
}

/// Create a new empty `PathCut`.
pub fn new_path_cut() -> PathCut {
    PathCut::default()
}

/// Total path length (sum of edge lengths along path).
pub fn path_cut_length(positions: &[[f32; 3]], cut: &PathCut) -> f32 {
    if cut.vertex_path.len() < 2 {
        return 0.0;
    }
    cut.vertex_path
        .windows(2)
        .map(|w| {
            let a = positions[w[0]];
            let b = positions[w[1]];
            let dx = b[0] - a[0];
            let dy = b[1] - a[1];
            let dz = b[2] - a[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .sum()
}

/// True if the path forms a closed loop (first == last vertex).
pub fn path_cut_is_closed(cut: &PathCut) -> bool {
    cut.vertex_path.len() >= 2 && cut.vertex_path.first() == cut.vertex_path.last()
}

/// Number of vertices in the path.
pub fn path_cut_vertex_count(cut: &PathCut) -> usize {
    cut.vertex_path.len()
}

/// List of edges (pairs of vertex indices) in the path.
pub fn path_cut_edges(cut: &PathCut) -> Vec<[usize; 2]> {
    if cut.vertex_path.len() < 2 {
        return vec![];
    }
    cut.vertex_path.windows(2).map(|w| [w[0], w[1]]).collect()
}

/// Stub shortest-path: returns a direct two-vertex path.
pub fn path_shortest(from: usize, to: usize) -> PathCut {
    PathCut {
        vertex_path: vec![from, to],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
        ]
    }

    #[test]
    fn test_path_cut_length() {
        let pos = line_positions();
        let cut = PathCut {
            vertex_path: vec![0, 1, 2, 3],
        };
        let l = path_cut_length(&pos, &cut);
        assert!((l - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_path_cut_is_closed_false() {
        let cut = PathCut {
            vertex_path: vec![0, 1, 2],
        };
        assert!(!path_cut_is_closed(&cut));
    }

    #[test]
    fn test_path_cut_is_closed_true() {
        let cut = PathCut {
            vertex_path: vec![0, 1, 2, 0],
        };
        assert!(path_cut_is_closed(&cut));
    }

    #[test]
    fn test_path_cut_vertex_count() {
        let cut = PathCut {
            vertex_path: vec![0, 1, 2],
        };
        assert_eq!(path_cut_vertex_count(&cut), 3);
    }

    #[test]
    fn test_path_cut_edges() {
        let cut = PathCut {
            vertex_path: vec![0, 1, 2],
        };
        let edges = path_cut_edges(&cut);
        assert_eq!(edges.len(), 2);
        assert_eq!(edges[0], [0, 1]);
    }

    #[test]
    fn test_path_shortest() {
        let cut = path_shortest(0, 5);
        assert_eq!(cut.vertex_path.len(), 2);
        assert_eq!(cut.vertex_path[1], 5);
    }
}
