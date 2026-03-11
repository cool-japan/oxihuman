#![allow(dead_code)]
//! Vertex welding by threshold.

/// Result of vertex welding.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct WeldResult2 {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<[u32; 3]>,
    pub welded_count: usize,
}

/// Weld vertices closer than a distance threshold.
#[allow(dead_code)]
pub fn weld_vertices(positions: &[[f32; 3]], indices: &[[u32; 3]], threshold: f32) -> WeldResult2 {
    let map = weld_map(positions, threshold);
    let mut new_positions: Vec<[f32; 3]> = Vec::new();
    let mut remap = vec![0u32; positions.len()];
    let mut seen = vec![false; positions.len()];
    for i in 0..positions.len() {
        let target = map[i] as usize;
        if !seen[target] {
            remap[target] = new_positions.len() as u32;
            new_positions.push(positions[target]);
            seen[target] = true;
        }
        remap[i] = remap[target];
    }
    let new_indices: Vec<[u32; 3]> = indices
        .iter()
        .map(|tri| {
            [
                remap[tri[0] as usize],
                remap[tri[1] as usize],
                remap[tri[2] as usize],
            ]
        })
        .filter(|tri| tri[0] != tri[1] && tri[1] != tri[2] && tri[0] != tri[2])
        .collect();
    let welded_count = positions.len() - new_positions.len();
    WeldResult2 {
        positions: new_positions,
        indices: new_indices,
        welded_count,
    }
}

/// Weld by position only (same as weld_vertices).
#[allow(dead_code)]
pub fn weld_by_position(
    positions: &[[f32; 3]],
    indices: &[[u32; 3]],
    threshold: f32,
) -> WeldResult2 {
    weld_vertices(positions, indices, threshold)
}

/// Weld by normal similarity (dot product threshold).
#[allow(dead_code)]
pub fn weld_by_normal(normals: &[[f32; 3]], dot_threshold: f32) -> Vec<u32> {
    let n = normals.len();
    let mut map: Vec<u32> = (0..n as u32).collect();
    for i in 0..n {
        for j in 0..i {
            if map[j] == j as u32 {
                let dot = normals[i][0] * normals[j][0]
                    + normals[i][1] * normals[j][1]
                    + normals[i][2] * normals[j][2];
                if dot >= dot_threshold {
                    map[i] = j as u32;
                    break;
                }
            }
        }
    }
    map
}

/// Count how many vertices would be welded.
#[allow(dead_code)]
pub fn weld_count(positions: &[[f32; 3]], threshold: f32) -> usize {
    let map = weld_map(positions, threshold);
    let unique: std::collections::HashSet<u32> = map.into_iter().collect();
    positions.len() - unique.len()
}

/// Return the weld threshold used.
#[allow(dead_code)]
pub fn weld_threshold_value(threshold: f32) -> f32 {
    threshold
}

/// Compute a weld map: for each vertex, the index of its representative.
#[allow(dead_code)]
pub fn weld_map(positions: &[[f32; 3]], threshold: f32) -> Vec<u32> {
    let n = positions.len();
    let mut map: Vec<u32> = (0..n as u32).collect();
    let t2 = threshold * threshold;
    for i in 0..n {
        for j in 0..i {
            if map[j] == j as u32 {
                let dx = positions[i][0] - positions[j][0];
                let dy = positions[i][1] - positions[j][1];
                let dz = positions[i][2] - positions[j][2];
                if dx * dx + dy * dy + dz * dz < t2 {
                    map[i] = j as u32;
                    break;
                }
            }
        }
    }
    map
}

/// Un-weld a vertex (duplicate it).
#[allow(dead_code)]
pub fn unweld_vertex(
    positions: &mut Vec<[f32; 3]>,
    indices: &mut [[u32; 3]],
    vertex: u32,
    face_idx: usize,
) -> u32 {
    if face_idx >= indices.len() {
        return vertex;
    }
    let new_idx = positions.len() as u32;
    positions.push(positions[vertex as usize]);
    let tri = &mut indices[face_idx];
    for v in tri.iter_mut() {
        if *v == vertex {
            *v = new_idx;
            break;
        }
    }
    new_idx
}

/// Get weld statistics.
#[allow(dead_code)]
pub fn weld_statistics(positions: &[[f32; 3]], threshold: f32) -> (usize, usize) {
    let map = weld_map(positions, threshold);
    let unique: std::collections::HashSet<u32> = map.into_iter().collect();
    (positions.len(), unique.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weld_vertices_no_weld() {
        let p = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let i = vec![[0u32, 1, 2]];
        let r = weld_vertices(&p, &i, 0.01);
        assert_eq!(r.welded_count, 0);
    }

    #[test]
    fn test_weld_vertices_with_weld() {
        let p = vec![
            [0.0, 0.0, 0.0],
            [0.001, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let i = vec![[0u32, 2, 3], [1, 2, 3]];
        let r = weld_vertices(&p, &i, 0.01);
        assert!(r.welded_count > 0);
    }

    #[test]
    fn test_weld_by_position() {
        let p = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let i = vec![[0u32, 1, 2]];
        let r = weld_by_position(&p, &i, 0.01);
        assert_eq!(r.welded_count, 0);
    }

    #[test]
    fn test_weld_by_normal() {
        let normals = vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]];
        let map = weld_by_normal(&normals, 0.99);
        assert_eq!(map[1], 0);
        assert_eq!(map[2], 2);
    }

    #[test]
    fn test_weld_count() {
        let p = vec![[0.0, 0.0, 0.0], [0.001, 0.0, 0.0], [1.0, 0.0, 0.0]];
        assert!(weld_count(&p, 0.01) > 0);
    }

    #[test]
    fn test_weld_threshold_value() {
        assert!((weld_threshold_value(0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_weld_map() {
        let p = vec![[0.0, 0.0, 0.0], [0.001, 0.0, 0.0]];
        let map = weld_map(&p, 0.01);
        assert_eq!(map[1], 0);
    }

    #[test]
    fn test_unweld_vertex() {
        let mut p = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let mut i = vec![[0u32, 1, 2]];
        let new_idx = unweld_vertex(&mut p, &mut i, 0, 0);
        assert_eq!(new_idx, 3);
        assert_eq!(p.len(), 4);
    }

    #[test]
    fn test_weld_statistics() {
        let p = vec![[0.0, 0.0, 0.0], [0.001, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let (total, unique) = weld_statistics(&p, 0.01);
        assert_eq!(total, 3);
        assert_eq!(unique, 2);
    }

    #[test]
    fn test_weld_empty() {
        let r = weld_vertices(&[], &[], 0.01);
        assert_eq!(r.welded_count, 0);
        assert!(r.positions.is_empty());
    }
}
