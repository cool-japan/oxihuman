#![allow(dead_code)]

/// Result of an edge split operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeSplitResult {
    pub new_vertex_index: usize,
    pub new_edge_a: [usize; 2],
    pub new_edge_b: [usize; 2],
    pub split_count: usize,
}

/// Split an edge at a given parameter t (0..1) between its two vertices.
#[allow(dead_code)]
pub fn split_edge(vertices: &[[f32; 3]], edge: [usize; 2], t: f32) -> ([f32; 3], EdgeSplitResult) {
    let a = vertices[edge[0]];
    let b = vertices[edge[1]];
    let new_pos = [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ];
    let new_idx = vertices.len();
    let result = EdgeSplitResult {
        new_vertex_index: new_idx,
        new_edge_a: [edge[0], new_idx],
        new_edge_b: [new_idx, edge[1]],
        split_count: 1,
    };
    (new_pos, result)
}

/// Split an edge at its midpoint.
#[allow(dead_code)]
pub fn split_edge_midpoint(vertices: &[[f32; 3]], edge: [usize; 2]) -> ([f32; 3], EdgeSplitResult) {
    split_edge(vertices, edge, 0.5)
}

/// Split all edges longer than the given threshold, returning the number of splits.
#[allow(dead_code)]
pub fn split_all_long_edges(vertices: &[[f32; 3]], edges: &[[usize; 2]], threshold: f32) -> usize {
    let mut count = 0usize;
    for edge in edges {
        let len = edge_length(vertices, *edge);
        if len > threshold {
            count += 1;
        }
    }
    count
}

fn edge_length(vertices: &[[f32; 3]], edge: [usize; 2]) -> f32 {
    let a = vertices[edge[0]];
    let b = vertices[edge[1]];
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Split an edge at parameter t.
#[allow(dead_code)]
pub fn split_edge_at_t(vertices: &[[f32; 3]], edge: [usize; 2], t: f32) -> [f32; 3] {
    let a = vertices[edge[0]];
    let b = vertices[edge[1]];
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Count how many edges would be split given a length threshold.
#[allow(dead_code)]
pub fn edge_split_count(vertices: &[[f32; 3]], edges: &[[usize; 2]], threshold: f32) -> usize {
    edges
        .iter()
        .filter(|e| edge_length(vertices, **e) > threshold)
        .count()
}

/// Validate that a split result references valid indices.
#[allow(dead_code)]
pub fn validate_split(result: &EdgeSplitResult, vertex_count: usize) -> bool {
    result.new_vertex_index <= vertex_count
        && result.new_edge_a[0] < vertex_count + 1
        && result.new_edge_a[1] < vertex_count + 1
        && result.new_edge_b[0] < vertex_count + 1
        && result.new_edge_b[1] < vertex_count + 1
}

/// Check whether splitting an edge would create triangles (always true for manifold meshes).
#[allow(dead_code)]
pub fn split_creates_triangles(face_vertex_count: usize) -> bool {
    face_vertex_count >= 3
}

/// Return a default split threshold based on average edge length.
#[allow(dead_code)]
pub fn split_threshold(vertices: &[[f32; 3]], edges: &[[usize; 2]], factor: f32) -> f32 {
    if edges.is_empty() {
        return 0.0;
    }
    let total: f32 = edges.iter().map(|e| edge_length(vertices, *e)).sum();
    (total / edges.len() as f32) * factor
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_verts() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]]
    }

    #[test]
    fn test_split_edge_midpoint() {
        let v = sample_verts();
        let (pos, res) = split_edge_midpoint(&v, [0, 1]);
        assert!((pos[0] - 1.0).abs() < 1e-6);
        assert_eq!(res.split_count, 1);
    }

    #[test]
    fn test_split_edge_at_t() {
        let v = sample_verts();
        let pos = split_edge_at_t(&v, [0, 1], 0.25);
        assert!((pos[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_split_edge_result_indices() {
        let v = sample_verts();
        let (_, res) = split_edge(&v, [0, 1], 0.5);
        assert_eq!(res.new_vertex_index, 3);
        assert_eq!(res.new_edge_a, [0, 3]);
        assert_eq!(res.new_edge_b, [3, 1]);
    }

    #[test]
    fn test_split_all_long_edges() {
        let v = sample_verts();
        let edges = vec![[0, 1], [1, 2], [2, 0]];
        let count = split_all_long_edges(&v, &edges, 1.5);
        assert!(count >= 1);
    }

    #[test]
    fn test_edge_split_count() {
        let v = sample_verts();
        let edges = vec![[0, 1], [1, 2], [2, 0]];
        let count = edge_split_count(&v, &edges, 100.0);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_validate_split() {
        let res = EdgeSplitResult {
            new_vertex_index: 3,
            new_edge_a: [0, 3],
            new_edge_b: [3, 1],
            split_count: 1,
        };
        assert!(validate_split(&res, 3));
    }

    #[test]
    fn test_split_creates_triangles() {
        assert!(split_creates_triangles(3));
        assert!(split_creates_triangles(4));
        assert!(!split_creates_triangles(2));
    }

    #[test]
    fn test_split_threshold() {
        let v = sample_verts();
        let edges = vec![[0, 1]];
        let t = split_threshold(&v, &edges, 1.0);
        assert!((t - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_split_threshold_empty() {
        let v = sample_verts();
        let edges: Vec<[usize; 2]> = vec![];
        let t = split_threshold(&v, &edges, 1.0);
        assert!((t - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_split_edge_endpoints() {
        let v = sample_verts();
        let (pos0, _) = split_edge(&v, [0, 1], 0.0);
        assert!((pos0[0] - 0.0).abs() < 1e-6);
        let (pos1, _) = split_edge(&v, [0, 1], 1.0);
        assert!((pos1[0] - 2.0).abs() < 1e-6);
    }
}
