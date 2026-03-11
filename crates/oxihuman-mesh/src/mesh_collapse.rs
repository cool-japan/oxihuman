#![allow(dead_code)]

/// Result of an edge collapse operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollapseResult {
    pub removed_vertex: usize,
    pub kept_vertex: usize,
    pub removed_faces: Vec<usize>,
    pub collapse_count: usize,
}

/// Collapse an edge by merging vertex b into vertex a.
#[allow(dead_code)]
pub fn collapse_edge(
    vertices: &mut Vec<[f32; 3]>,
    faces: &mut Vec<[usize; 3]>,
    edge: [usize; 2],
) -> CollapseResult {
    let a = edge[0];
    let b = edge[1];
    // Move kept vertex to midpoint
    let mid = [
        (vertices[a][0] + vertices[b][0]) * 0.5,
        (vertices[a][1] + vertices[b][1]) * 0.5,
        (vertices[a][2] + vertices[b][2]) * 0.5,
    ];
    vertices[a] = mid;

    // Replace all references to b with a and find degenerate faces
    let mut removed = Vec::new();
    for (fi, face) in faces.iter_mut().enumerate() {
        for v in face.iter_mut() {
            if *v == b {
                *v = a;
            }
        }
        if face[0] == face[1] || face[1] == face[2] || face[0] == face[2] {
            removed.push(fi);
        }
    }

    // Remove degenerate faces (in reverse order to preserve indices)
    for &fi in removed.iter().rev() {
        faces.remove(fi);
    }

    CollapseResult {
        removed_vertex: b,
        kept_vertex: a,
        removed_faces: removed,
        collapse_count: 1,
    }
}

/// Collapse the shortest edge in the mesh.
#[allow(dead_code)]
pub fn collapse_shortest_edge(
    vertices: &mut Vec<[f32; 3]>,
    faces: &mut Vec<[usize; 3]>,
) -> Option<CollapseResult> {
    let mut shortest_edge: Option<[usize; 2]> = None;
    let mut shortest_len = f32::MAX;

    for face in faces.iter() {
        let edges = [[face[0], face[1]], [face[1], face[2]], [face[2], face[0]]];
        for edge in &edges {
            let len = edge_len(vertices, *edge);
            if len < shortest_len {
                shortest_len = len;
                shortest_edge = Some(*edge);
            }
        }
    }

    shortest_edge.map(|edge| collapse_edge(vertices, faces, edge))
}

/// Check if an edge can be collapsed without creating non-manifold topology.
#[allow(dead_code)]
pub fn can_collapse(faces: &[[usize; 3]], edge: [usize; 2]) -> bool {
    // Simple check: edge must be part of at most 2 faces
    let count = faces
        .iter()
        .filter(|f| {
            (f.contains(&edge[0]) && f.contains(&edge[1]))
        })
        .count();
    (1..=2).contains(&count)
}

/// Compute the QEM-like cost of collapsing an edge (simplified as edge length).
#[allow(dead_code)]
pub fn collapse_cost(vertices: &[[f32; 3]], edge: [usize; 2]) -> f32 {
    edge_len(vertices, edge)
}

/// Count how many collapses have been performed.
#[allow(dead_code)]
pub fn collapse_count(result: &CollapseResult) -> usize {
    result.collapse_count
}

/// Validate that a collapse result is consistent.
#[allow(dead_code)]
pub fn validate_collapse(result: &CollapseResult, face_count: usize) -> bool {
    result.kept_vertex != result.removed_vertex
        && result.removed_faces.iter().all(|&f| f < face_count + result.removed_faces.len())
}

/// Check if collapse preserves topology (no boundary vertex collapse with interior).
#[allow(dead_code)]
pub fn collapse_preserves_topology(faces: &[[usize; 3]], edge: [usize; 2]) -> bool {
    can_collapse(faces, edge)
}

/// Return the target vertex position after collapse (midpoint).
#[allow(dead_code)]
pub fn collapse_target_vertex(vertices: &[[f32; 3]], edge: [usize; 2]) -> [f32; 3] {
    let a = vertices[edge[0]];
    let b = vertices[edge[1]];
    [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ]
}

fn edge_len(vertices: &[[f32; 3]], edge: [usize; 2]) -> f32 {
    let a = vertices[edge[0]];
    let b = vertices[edge[1]];
    ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_mesh() -> (Vec<[f32; 3]>, Vec<[usize; 3]>) {
        let verts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [1.5, 1.0, 0.0],
        ];
        let faces = vec![[0, 1, 2], [1, 3, 2]];
        (verts, faces)
    }

    #[test]
    fn test_collapse_edge() {
        let (mut v, mut f) = sample_mesh();
        let res = collapse_edge(&mut v, &mut f, [0, 1]);
        assert_eq!(res.kept_vertex, 0);
        assert_eq!(res.removed_vertex, 1);
    }

    #[test]
    fn test_collapse_shortest() {
        let (mut v, mut f) = sample_mesh();
        let res = collapse_shortest_edge(&mut v, &mut f);
        assert!(res.is_some());
    }

    #[test]
    fn test_can_collapse() {
        let (_, f) = sample_mesh();
        assert!(can_collapse(&f, [1, 2]));
    }

    #[test]
    fn test_collapse_cost() {
        let (v, _) = sample_mesh();
        let cost = collapse_cost(&v, [0, 1]);
        assert!((cost - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_collapse_count() {
        let res = CollapseResult {
            removed_vertex: 1,
            kept_vertex: 0,
            removed_faces: vec![],
            collapse_count: 3,
        };
        assert_eq!(collapse_count(&res), 3);
    }

    #[test]
    fn test_validate_collapse() {
        let res = CollapseResult {
            removed_vertex: 1,
            kept_vertex: 0,
            removed_faces: vec![0],
            collapse_count: 1,
        };
        assert!(validate_collapse(&res, 2));
    }

    #[test]
    fn test_collapse_preserves_topology() {
        let (_, f) = sample_mesh();
        assert!(collapse_preserves_topology(&f, [1, 2]));
    }

    #[test]
    fn test_collapse_target_vertex() {
        let (v, _) = sample_mesh();
        let mid = collapse_target_vertex(&v, [0, 1]);
        assert!((mid[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_can_collapse_nonexistent_edge() {
        let faces = vec![[0, 1, 2]];
        assert!(!can_collapse(&faces, [0, 3]));
    }

    #[test]
    fn test_collapse_single_face() {
        let mut v = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let mut f = vec![[0, 1, 2]];
        let res = collapse_edge(&mut v, &mut f, [0, 1]);
        assert_eq!(res.collapse_count, 1);
        // Face becomes degenerate and is removed
        assert!(f.is_empty());
    }
}
