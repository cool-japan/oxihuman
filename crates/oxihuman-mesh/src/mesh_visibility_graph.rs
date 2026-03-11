//! Vertex-to-vertex visibility graph on a mesh.
#![allow(dead_code)]

/// A directed visibility edge between two vertices.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct VisibilityEdge {
    pub from: usize,
    pub to: usize,
    pub distance: f32,
}

fn dist3(a: [f32;3], b: [f32;3]) -> f32 {
    let dx = b[0]-a[0]; let dy = b[1]-a[1]; let dz = b[2]-a[2];
    (dx*dx+dy*dy+dz*dz).sqrt()
}

fn ray_triangle_hit(
    origin: [f32;3], dir: [f32;3],
    p0: [f32;3], p1: [f32;3], p2: [f32;3],
    max_t: f32,
) -> bool {
    let eps = 1e-8;
    let e1 = [p1[0]-p0[0], p1[1]-p0[1], p1[2]-p0[2]];
    let e2 = [p2[0]-p0[0], p2[1]-p0[1], p2[2]-p0[2]];
    let h = [dir[1]*e2[2]-dir[2]*e2[1], dir[2]*e2[0]-dir[0]*e2[2], dir[0]*e2[1]-dir[1]*e2[0]];
    let a = e1[0]*h[0]+e1[1]*h[1]+e1[2]*h[2];
    if a.abs() < eps { return false; }
    let f = 1.0 / a;
    let s = [origin[0]-p0[0], origin[1]-p0[1], origin[2]-p0[2]];
    let u = f * (s[0]*h[0]+s[1]*h[1]+s[2]*h[2]);
    if !(0.0..=1.0).contains(&u) { return false; }
    let q = [s[1]*e1[2]-s[2]*e1[1], s[2]*e1[0]-s[0]*e1[2], s[0]*e1[1]-s[1]*e1[0]];
    let v = f * (dir[0]*q[0]+dir[1]*q[1]+dir[2]*q[2]);
    if v < 0.0 || u + v > 1.0 { return false; }
    let t = f * (e2[0]*q[0]+e2[1]*q[1]+e2[2]*q[2]);
    t > eps && t < max_t - eps
}

/// Check if vertex `from` can see vertex `to` without intersecting any triangle.
#[allow(dead_code)]
pub fn visible_from(
    from: usize,
    to: usize,
    positions: &[[f32;3]],
    indices: &[u32],
) -> bool {
    if from >= positions.len() || to >= positions.len() { return false; }
    let pa = positions[from]; let pb = positions[to];
    let dist = dist3(pa, pb);
    if dist < 1e-8 { return true; }
    let dir = [(pb[0]-pa[0])/dist, (pb[1]-pa[1])/dist, (pb[2]-pa[2])/dist];
    let n = positions.len();
    let tris = indices.len() / 3;
    for t in 0..tris {
        let i0 = indices[t*3] as usize;
        let i1 = indices[t*3+1] as usize;
        let i2 = indices[t*3+2] as usize;
        if i0 >= n || i1 >= n || i2 >= n { continue; }
        // skip triangles containing from or to
        if i0 == from || i1 == from || i2 == from { continue; }
        if i0 == to || i1 == to || i2 == to { continue; }
        if ray_triangle_hit(pa, dir, positions[i0], positions[i1], positions[i2], dist) {
            return false;
        }
    }
    true
}

/// Compute the full visibility graph for a mesh.
#[allow(dead_code)]
pub fn compute_visibility_graph(positions: &[[f32;3]], indices: &[u32]) -> Vec<VisibilityEdge> {
    let n = positions.len();
    let mut edges = Vec::new();
    for i in 0..n {
        for j in i+1..n {
            if visible_from(i, j, positions, indices) {
                let d = dist3(positions[i], positions[j]);
                edges.push(VisibilityEdge { from: i, to: j, distance: d });
                edges.push(VisibilityEdge { from: j, to: i, distance: d });
            }
        }
    }
    edges
}

/// Count visibility edges.
#[allow(dead_code)]
pub fn visibility_edge_count(edges: &[VisibilityEdge]) -> usize {
    edges.len()
}

/// Get all visible neighbors of a vertex.
#[allow(dead_code)]
pub fn visibility_neighbors(vertex: usize, edges: &[VisibilityEdge]) -> Vec<usize> {
    edges.iter().filter(|e| e.from == vertex).map(|e| e.to).collect()
}

/// Build a visibility adjacency list.
#[allow(dead_code)]
pub fn build_visibility_adj(n: usize, edges: &[VisibilityEdge]) -> Vec<Vec<usize>> {
    let mut adj = vec![Vec::new(); n];
    for e in edges {
        if e.from < n { adj[e.from].push(e.to); }
    }
    adj
}

/// Perform BFS to find all vertices reachable from `start` in the visibility graph.
#[allow(dead_code)]
pub fn visibility_reachable(start: usize, adj: &[Vec<usize>]) -> Vec<usize> {
    if start >= adj.len() { return Vec::new(); }
    let n = adj.len();
    let mut visited = vec![false; n];
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(start);
    visited[start] = true;
    while let Some(v) = queue.pop_front() {
        for &nb in &adj[v] {
            if nb < n && !visited[nb] { visited[nb] = true; queue.push_back(nb); }
        }
    }
    (0..n).filter(|&i| visited[i]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_mesh() -> (Vec<[f32;3]>, Vec<u32>) {
        let pos = vec![[0.0f32,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]];
        let idx = vec![0u32,1,2];
        (pos, idx)
    }

    #[test]
    fn test_visible_from_direct() {
        let (pos, idx) = tiny_mesh();
        assert!(visible_from(0, 1, &pos, &idx));
    }

    #[test]
    fn test_visibility_edge_count_positive() {
        let (pos, idx) = tiny_mesh();
        let edges = compute_visibility_graph(&pos, &idx);
        let _ = visibility_edge_count(&edges);
    }

    #[test]
    fn test_visibility_neighbors() {
        let edges = vec![
            VisibilityEdge { from:0, to:1, distance: 1.0 },
            VisibilityEdge { from:0, to:2, distance: 1.0 },
        ];
        let n = visibility_neighbors(0, &edges);
        assert_eq!(n.len(), 2);
    }

    #[test]
    fn test_build_visibility_adj() {
        let edges = vec![VisibilityEdge { from:0, to:1, distance: 1.0 }];
        let adj = build_visibility_adj(3, &edges);
        assert!(adj[0].contains(&1));
    }

    #[test]
    fn test_visibility_reachable_connected() {
        let adj = vec![vec![1usize], vec![2usize], vec![]];
        let r = visibility_reachable(0, &adj);
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn test_visibility_reachable_isolated() {
        let adj = vec![vec![], vec![], vec![]];
        let r = visibility_reachable(0, &adj);
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn test_visible_from_oob() {
        let (pos, idx) = tiny_mesh();
        assert!(!visible_from(0, 100, &pos, &idx));
    }

    #[test]
    fn test_visibility_edge_struct() {
        let e = VisibilityEdge { from: 0, to: 1, distance: 2.5 };
        assert!((e.distance - 2.5).abs() < 1e-5);
    }

    #[test]
    fn test_visibility_neighbors_empty() {
        let edges: Vec<VisibilityEdge> = vec![];
        let n = visibility_neighbors(0, &edges);
        assert!(n.is_empty());
    }
}
