#![allow(dead_code)]
//! Vertex snapping utilities.

/// Vertex snap result.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct VertexSnap {
    pub positions: Vec<[f32; 3]>,
    pub snapped_indices: Vec<usize>,
}

/// Snap all vertices to a grid of the given cell size.
#[allow(dead_code)]
pub fn snap_to_grid(positions: &[[f32; 3]], cell_size: f32) -> Vec<[f32; 3]> {
    if cell_size <= 0.0 {
        return positions.to_vec();
    }
    positions
        .iter()
        .map(|p| {
            [
                (p[0] / cell_size).round() * cell_size,
                (p[1] / cell_size).round() * cell_size,
                (p[2] / cell_size).round() * cell_size,
            ]
        })
        .collect()
}

/// Snap vertices that are within threshold distance of each other.
#[allow(dead_code)]
pub fn snap_threshold_vs(positions: &[[f32; 3]], threshold: f32) -> VertexSnap {
    let mut result = positions.to_vec();
    let mut snapped = Vec::new();
    let t2 = threshold * threshold;
    for i in 0..result.len() {
        for j in (i + 1)..positions.len() {
            let dx = result[i][0] - result[j][0];
            let dy = result[i][1] - result[j][1];
            let dz = result[i][2] - result[j][2];
            if dx * dx + dy * dy + dz * dz < t2 {
                result[j] = result[i];
                snapped.push(j);
            }
        }
    }
    VertexSnap {
        positions: result,
        snapped_indices: snapped,
    }
}

/// Snap a single vertex to the nearest vertex in a set.
#[allow(dead_code)]
pub fn snap_vertex_to_nearest(vertex: [f32; 3], targets: &[[f32; 3]]) -> [f32; 3] {
    if targets.is_empty() {
        return vertex;
    }
    let mut best = targets[0];
    let mut best_d = f32::MAX;
    for t in targets {
        let dx = vertex[0] - t[0];
        let dy = vertex[1] - t[1];
        let dz = vertex[2] - t[2];
        let d = dx * dx + dy * dy + dz * dz;
        if d < best_d {
            best_d = d;
            best = *t;
        }
    }
    best
}

/// Count how many vertices were snapped.
#[allow(dead_code)]
pub fn snap_count(snap: &VertexSnap) -> usize {
    snap.snapped_indices.len()
}

/// Return snapped positions.
#[allow(dead_code)]
pub fn snapped_positions(snap: &VertexSnap) -> &[[f32; 3]] {
    &snap.positions
}

/// Return the effective snap precision (smallest cell size).
#[allow(dead_code)]
pub fn snap_precision(cell_size: f32) -> f32 {
    cell_size.abs()
}

/// Snap vertices to a plane defined by normal and distance.
#[allow(dead_code)]
pub fn snap_to_plane(positions: &[[f32; 3]], normal: [f32; 3], dist: f32) -> Vec<[f32; 3]> {
    let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if len < 1e-12 {
        return positions.to_vec();
    }
    let n = [normal[0] / len, normal[1] / len, normal[2] / len];
    positions
        .iter()
        .map(|p| {
            let d = p[0] * n[0] + p[1] * n[1] + p[2] * n[2] - dist;
            [p[0] - d * n[0], p[1] - d * n[1], p[2] - d * n[2]]
        })
        .collect()
}

/// Undo snap by restoring original positions.
#[allow(dead_code)]
pub fn snap_undo(original: &[[f32; 3]], _snap: &VertexSnap) -> Vec<[f32; 3]> {
    original.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snap_to_grid() {
        let pts = vec![[0.1, 0.9, 0.5]];
        let r = snap_to_grid(&pts, 1.0);
        assert_eq!(r, vec![[0.0, 1.0, 1.0]]);
    }

    #[test]
    fn test_snap_to_grid_zero() {
        let pts = vec![[1.0, 2.0, 3.0]];
        let r = snap_to_grid(&pts, 0.0);
        assert_eq!(r, pts);
    }

    #[test]
    fn test_snap_threshold() {
        let pts = vec![[0.0, 0.0, 0.0], [0.01, 0.0, 0.0]];
        let s = snap_threshold_vs(&pts, 0.1);
        assert_eq!(s.positions[0], s.positions[1]);
    }

    #[test]
    fn test_snap_vertex_to_nearest() {
        let targets = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let r = snap_vertex_to_nearest([0.9, 0.0, 0.0], &targets);
        assert_eq!(r, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_snap_vertex_to_nearest_empty() {
        let r = snap_vertex_to_nearest([1.0, 2.0, 3.0], &[]);
        assert_eq!(r, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_snap_count() {
        let s = VertexSnap {
            positions: vec![],
            snapped_indices: vec![1, 3],
        };
        assert_eq!(snap_count(&s), 2);
    }

    #[test]
    fn test_snapped_positions() {
        let s = VertexSnap {
            positions: vec![[1.0, 2.0, 3.0]],
            snapped_indices: vec![],
        };
        assert_eq!(snapped_positions(&s).len(), 1);
    }

    #[test]
    fn test_snap_precision() {
        assert!((snap_precision(0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_snap_to_plane() {
        let pts = vec![[1.0, 2.0, 3.0]];
        let r = snap_to_plane(&pts, [0.0, 1.0, 0.0], 0.0);
        assert!((r[0][1]).abs() < 1e-6);
    }

    #[test]
    fn test_snap_undo() {
        let orig = vec![[1.0, 2.0, 3.0]];
        let s = VertexSnap {
            positions: vec![[0.0, 0.0, 0.0]],
            snapped_indices: vec![],
        };
        let r = snap_undo(&orig, &s);
        assert_eq!(r, orig);
    }
}
