#![allow(dead_code)]

use std::collections::HashMap;

/// A boundary loop as a sequence of vertex indices.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoundaryLoop2 {
    pub vertices: Vec<usize>,
    pub is_closed: bool,
}

/// Information about all boundary loops in a mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoundaryInfo2 {
    pub loops: Vec<BoundaryLoop2>,
    pub total_boundary_edges: usize,
}

/// Find all boundary loops from a list of edges and faces.
/// Boundary edges appear in only one face.
#[allow(dead_code)]
pub fn find_boundary_loops2(edges: &[[usize; 2]], face_edges: &[Vec<usize>]) -> BoundaryInfo2 {
    let mut edge_face_count: HashMap<usize, usize> = HashMap::new();
    for face in face_edges {
        for &ei in face {
            *edge_face_count.entry(ei).or_insert(0) += 1;
        }
    }
    let boundary_edge_indices: Vec<usize> = edge_face_count
        .iter()
        .filter(|(_, &count)| count == 1)
        .map(|(&ei, _)| ei)
        .collect();

    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for &ei in &boundary_edge_indices {
        if ei < edges.len() {
            let e = edges[ei];
            adj.entry(e[0]).or_default().push(e[1]);
            adj.entry(e[1]).or_default().push(e[0]);
        }
    }

    let mut visited: HashMap<usize, bool> = HashMap::new();
    let mut loops = Vec::new();
    for &start in adj.keys() {
        if visited.get(&start).copied().unwrap_or(false) {
            continue;
        }
        let mut loop_verts = Vec::new();
        let mut current = start;
        let mut is_closed = false;
        loop {
            if visited.get(&current).copied().unwrap_or(false) {
                if current == start && loop_verts.len() > 2 {
                    is_closed = true;
                }
                break;
            }
            visited.insert(current, true);
            loop_verts.push(current);
            if let Some(neighbors) = adj.get(&current) {
                if let Some(&next) = neighbors.iter().find(|n| !visited.get(n).copied().unwrap_or(false)) {
                    current = next;
                } else {
                    if neighbors.contains(&start) && loop_verts.len() > 2 {
                        is_closed = true;
                    }
                    break;
                }
            } else {
                break;
            }
        }
        if !loop_verts.is_empty() {
            loops.push(BoundaryLoop2 {
                vertices: loop_verts,
                is_closed,
            });
        }
    }

    BoundaryInfo2 {
        total_boundary_edges: boundary_edge_indices.len(),
        loops,
    }
}

/// Count the number of boundary loops.
#[allow(dead_code)]
pub fn boundary_loop_count2(info: &BoundaryInfo2) -> usize {
    info.loops.len()
}

/// Count total boundary edges.
#[allow(dead_code)]
pub fn boundary_edge_count2(info: &BoundaryInfo2) -> usize {
    info.total_boundary_edges
}

/// Check if a vertex is on the boundary.
#[allow(dead_code)]
pub fn is_boundary_vertex2(info: &BoundaryInfo2, vertex: usize) -> bool {
    info.loops.iter().any(|l| l.vertices.contains(&vertex))
}

/// Compute the total length of a boundary loop.
#[allow(dead_code)]
pub fn boundary_length2(vertices: &[[f32; 3]], loop_data: &BoundaryLoop2) -> f32 {
    if loop_data.vertices.len() < 2 {
        return 0.0;
    }
    let mut len = 0.0f32;
    for i in 0..loop_data.vertices.len() - 1 {
        let a = vertices[loop_data.vertices[i]];
        let b = vertices[loop_data.vertices[i + 1]];
        let d = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt();
        len += d;
    }
    if loop_data.is_closed && loop_data.vertices.len() > 2 {
        let a = vertices[loop_data.vertices[loop_data.vertices.len() - 1]];
        let b = vertices[loop_data.vertices[0]];
        len += ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt();
    }
    len
}

/// Compute the centroid of a boundary loop.
#[allow(dead_code)]
pub fn boundary_centroid2(vertices: &[[f32; 3]], loop_data: &BoundaryLoop2) -> [f32; 3] {
    if loop_data.vertices.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let mut sum = [0.0f32; 3];
    for &vi in &loop_data.vertices {
        sum[0] += vertices[vi][0];
        sum[1] += vertices[vi][1];
        sum[2] += vertices[vi][2];
    }
    let n = loop_data.vertices.len() as f32;
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

/// Close a boundary loop by connecting last vertex to first.
#[allow(dead_code)]
pub fn close_boundary2(loop_data: &mut BoundaryLoop2) {
    loop_data.is_closed = true;
}

/// Get all vertices in a boundary loop.
#[allow(dead_code)]
pub fn boundary_vertices2(loop_data: &BoundaryLoop2) -> Vec<usize> {
    loop_data.vertices.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_boundary_info() -> BoundaryInfo2 {
        BoundaryInfo2 {
            loops: vec![BoundaryLoop2 {
                vertices: vec![0, 1, 2],
                is_closed: true,
            }],
            total_boundary_edges: 3,
        }
    }

    #[test]
    fn test_boundary_loop_count() {
        let info = sample_boundary_info();
        assert_eq!(boundary_loop_count2(&info), 1);
    }

    #[test]
    fn test_boundary_edge_count() {
        let info = sample_boundary_info();
        assert_eq!(boundary_edge_count2(&info), 3);
    }

    #[test]
    fn test_is_boundary_vertex() {
        let info = sample_boundary_info();
        assert!(is_boundary_vertex2(&info, 0));
        assert!(!is_boundary_vertex2(&info, 5));
    }

    #[test]
    fn test_boundary_length() {
        let verts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let bl = BoundaryLoop2 { vertices: vec![0, 1, 2], is_closed: false };
        let len = boundary_length2(&verts, &bl);
        assert!((len - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_boundary_length_closed() {
        let verts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let bl = BoundaryLoop2 { vertices: vec![0, 1, 2], is_closed: true };
        let len = boundary_length2(&verts, &bl);
        assert!(len > 2.0);
    }

    #[test]
    fn test_boundary_centroid() {
        let verts = vec![
            [0.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [0.0, 3.0, 0.0],
        ];
        let bl = BoundaryLoop2 { vertices: vec![0, 1, 2], is_closed: true };
        let c = boundary_centroid2(&verts, &bl);
        assert!((c[0] - 1.0).abs() < 1e-6);
        assert!((c[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_close_boundary() {
        let mut bl = BoundaryLoop2 { vertices: vec![0, 1, 2], is_closed: false };
        close_boundary2(&mut bl);
        assert!(bl.is_closed);
    }

    #[test]
    fn test_boundary_vertices() {
        let bl = BoundaryLoop2 { vertices: vec![5, 6, 7], is_closed: false };
        assert_eq!(boundary_vertices2(&bl), vec![5, 6, 7]);
    }

    #[test]
    fn test_find_boundary_loops_simple() {
        let edges = vec![[0, 1], [1, 2], [2, 0], [0, 3], [1, 3], [2, 3]];
        // Triangle 0-1-2 uses edges 0,1,2; Triangle 0-1-3 uses edges 0,3,4; etc.
        // Edge 0 ([0,1]) shared by two faces => not boundary
        let face_edges = vec![
            vec![0, 1, 2],
            vec![0, 3, 4],
        ];
        let info = find_boundary_loops2(&edges, &face_edges);
        assert!(info.total_boundary_edges > 0);
    }

    #[test]
    fn test_empty_boundary() {
        let info = BoundaryInfo2 { loops: vec![], total_boundary_edges: 0 };
        assert_eq!(boundary_loop_count2(&info), 0);
        assert!(!is_boundary_vertex2(&info, 0));
    }
}
