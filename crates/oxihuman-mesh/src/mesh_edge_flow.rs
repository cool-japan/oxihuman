#![allow(dead_code)]
//! Edge flow computation and analysis.

/// Edge flow data for a mesh.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeFlow {
    pub edges: Vec<[u32; 2]>,
    pub directions: Vec<[f32; 3]>,
    pub magnitudes: Vec<f32>,
}

/// Compute edge flow from positions and edges.
#[allow(dead_code)]
pub fn compute_edge_flow(positions: &[[f32; 3]], edges: &[[u32; 2]]) -> EdgeFlow {
    let mut directions = Vec::with_capacity(edges.len());
    let mut magnitudes = Vec::with_capacity(edges.len());
    for e in edges {
        let a = positions[e[0] as usize];
        let b = positions[e[1] as usize];
        let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let mag = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        if mag > 1e-12 {
            directions.push([d[0] / mag, d[1] / mag, d[2] / mag]);
        } else {
            directions.push([0.0, 0.0, 0.0]);
        }
        magnitudes.push(mag);
    }
    EdgeFlow {
        edges: edges.to_vec(),
        directions,
        magnitudes,
    }
}

/// Get direction for edge at index.
#[allow(dead_code)]
pub fn flow_direction(flow: &EdgeFlow, index: usize) -> [f32; 3] {
    if index < flow.directions.len() {
        flow.directions[index]
    } else {
        [0.0, 0.0, 0.0]
    }
}

/// Get magnitude for edge at index.
#[allow(dead_code)]
pub fn flow_magnitude_ef(flow: &EdgeFlow, index: usize) -> f32 {
    if index < flow.magnitudes.len() {
        flow.magnitudes[index]
    } else {
        0.0
    }
}

/// Compute a per-vertex flow field by averaging incident edge directions.
#[allow(dead_code)]
pub fn flow_vertex_field(positions: &[[f32; 3]], flow: &EdgeFlow) -> Vec<[f32; 3]> {
    let mut field = vec![[0.0_f32; 3]; positions.len()];
    let mut counts = vec![0u32; positions.len()];
    for (i, e) in flow.edges.iter().enumerate() {
        let d = flow.directions[i];
        for &vi in &[e[0] as usize, e[1] as usize] {
            field[vi][0] += d[0];
            field[vi][1] += d[1];
            field[vi][2] += d[2];
            counts[vi] += 1;
        }
    }
    for (i, c) in counts.iter().enumerate() {
        if *c > 0 {
            let inv = 1.0 / *c as f32;
            field[i][0] *= inv;
            field[i][1] *= inv;
            field[i][2] *= inv;
        }
    }
    field
}

/// Smooth edge flow by iterative Laplacian averaging of edge directions and magnitudes.
///
/// For each iteration every edge's direction and magnitude are averaged with
/// those of adjacent edges (edges that share a vertex).  A Laplacian factor of
/// 0.5 is used: `new = 0.5 * self + 0.5 * neighbour_avg`.
/// If there are no edges, or `iterations` is 0, the original flow is returned.
#[allow(dead_code)]
pub fn smooth_edge_flow(flow: &EdgeFlow, iterations: u32) -> EdgeFlow {
    if iterations == 0 || flow.edges.is_empty() {
        return flow.clone();
    }

    // Determine the number of vertices referenced by the edges.
    let max_v = flow
        .edges
        .iter()
        .flat_map(|e| [e[0], e[1]])
        .max()
        .map(|m| m as usize + 1)
        .unwrap_or(0);

    // Build vertex → list of edge indices adjacency.
    let mut v_to_edges: Vec<Vec<usize>> = vec![Vec::new(); max_v];
    for (ei, e) in flow.edges.iter().enumerate() {
        let a = e[0] as usize;
        let b = e[1] as usize;
        if a < max_v {
            v_to_edges[a].push(ei);
        }
        if b < max_v {
            v_to_edges[b].push(ei);
        }
    }

    // For each edge, its "neighbourhood" is the union of edges adjacent to
    // either endpoint (excluding itself).
    let ne = flow.edges.len();
    let mut edge_neighbors: Vec<Vec<usize>> = Vec::with_capacity(ne);
    for (ei, e) in flow.edges.iter().enumerate() {
        let mut nbrs: Vec<usize> = Vec::new();
        for &vi in &[e[0] as usize, e[1] as usize] {
            if vi < max_v {
                for &nei in &v_to_edges[vi] {
                    if nei != ei && !nbrs.contains(&nei) {
                        nbrs.push(nei);
                    }
                }
            }
        }
        edge_neighbors.push(nbrs);
    }

    let mut directions = flow.directions.clone();
    let mut magnitudes = flow.magnitudes.clone();
    const LAMBDA: f32 = 0.5;

    for _ in 0..iterations {
        let prev_dirs = directions.clone();
        let prev_mags = magnitudes.clone();

        for ei in 0..ne {
            let nbrs = &edge_neighbors[ei];
            if nbrs.is_empty() {
                continue;
            }
            let count = nbrs.len() as f32;

            // Average neighbouring directions.
            let mut avg_dir = [0.0f32; 3];
            let mut avg_mag = 0.0f32;
            for &ni in nbrs {
                avg_dir[0] += prev_dirs[ni][0];
                avg_dir[1] += prev_dirs[ni][1];
                avg_dir[2] += prev_dirs[ni][2];
                avg_mag += prev_mags[ni];
            }
            avg_dir[0] /= count;
            avg_dir[1] /= count;
            avg_dir[2] /= count;
            avg_mag /= count;

            // Laplacian blend.
            let new_dir = [
                prev_dirs[ei][0] * (1.0 - LAMBDA) + avg_dir[0] * LAMBDA,
                prev_dirs[ei][1] * (1.0 - LAMBDA) + avg_dir[1] * LAMBDA,
                prev_dirs[ei][2] * (1.0 - LAMBDA) + avg_dir[2] * LAMBDA,
            ];
            // Renormalise direction if non-degenerate.
            let len = (new_dir[0] * new_dir[0]
                + new_dir[1] * new_dir[1]
                + new_dir[2] * new_dir[2])
                .sqrt();
            directions[ei] = if len > 1e-12 {
                [new_dir[0] / len, new_dir[1] / len, new_dir[2] / len]
            } else {
                new_dir
            };
            magnitudes[ei] = prev_mags[ei] * (1.0 - LAMBDA) + avg_mag * LAMBDA;
        }
    }

    EdgeFlow {
        edges: flow.edges.clone(),
        directions,
        magnitudes,
    }
}

/// Serialize flow to JSON string.
#[allow(dead_code)]
pub fn flow_to_json(flow: &EdgeFlow) -> String {
    format!(
        "{{\"edge_count\":{},\"avg_magnitude\":{:.4}}}",
        flow.edges.len(),
        if flow.magnitudes.is_empty() {
            0.0
        } else {
            flow.magnitudes.iter().sum::<f32>() / flow.magnitudes.len() as f32
        }
    )
}

/// Compute divergence at each vertex.
#[allow(dead_code)]
pub fn flow_divergence(flow: &EdgeFlow, vertex_count: usize) -> Vec<f32> {
    let mut div = vec![0.0_f32; vertex_count];
    for (i, e) in flow.edges.iter().enumerate() {
        let m = flow.magnitudes[i];
        let a = e[0] as usize;
        let b = e[1] as usize;
        if a < vertex_count {
            div[a] += m;
        }
        if b < vertex_count {
            div[b] -= m;
        }
    }
    div
}

/// Compute curl-like measure at each vertex.
#[allow(dead_code)]
pub fn flow_curl(flow: &EdgeFlow, vertex_count: usize) -> Vec<f32> {
    let mut curl = vec![0.0_f32; vertex_count];
    for (i, e) in flow.edges.iter().enumerate() {
        let d = flow.directions[i];
        let cross_mag = (d[0] * d[0] + d[1] * d[1]).sqrt();
        let a = e[0] as usize;
        if a < vertex_count {
            curl[a] += cross_mag;
        }
    }
    curl
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_edge_flow() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let edges = vec![[0, 1]];
        let ef = compute_edge_flow(&pos, &edges);
        assert_eq!(ef.directions.len(), 1);
        assert!((ef.magnitudes[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_flow_direction() {
        let ef = EdgeFlow {
            edges: vec![[0, 1]],
            directions: vec![[1.0, 0.0, 0.0]],
            magnitudes: vec![1.0],
        };
        assert_eq!(flow_direction(&ef, 0), [1.0, 0.0, 0.0]);
        assert_eq!(flow_direction(&ef, 5), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_flow_magnitude() {
        let ef = EdgeFlow {
            edges: vec![[0, 1]],
            directions: vec![[1.0, 0.0, 0.0]],
            magnitudes: vec![2.5],
        };
        assert!((flow_magnitude_ef(&ef, 0) - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_flow_vertex_field() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let ef = compute_edge_flow(&pos, &[[0, 1]]);
        let field = flow_vertex_field(&pos, &ef);
        assert_eq!(field.len(), 2);
    }

    #[test]
    fn test_smooth_edge_flow() {
        let ef = EdgeFlow {
            edges: vec![],
            directions: vec![],
            magnitudes: vec![],
        };
        let s = smooth_edge_flow(&ef, 1);
        assert_eq!(s.edges.len(), 0);
    }

    #[test]
    fn test_flow_to_json() {
        let ef = EdgeFlow {
            edges: vec![],
            directions: vec![],
            magnitudes: vec![],
        };
        let j = flow_to_json(&ef);
        assert!(j.contains("edge_count"));
    }

    #[test]
    fn test_flow_divergence() {
        let ef = EdgeFlow {
            edges: vec![[0, 1]],
            directions: vec![[1.0, 0.0, 0.0]],
            magnitudes: vec![1.0],
        };
        let d = flow_divergence(&ef, 2);
        assert!((d[0] - 1.0).abs() < 1e-6);
        assert!((d[1] + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_flow_curl() {
        let ef = EdgeFlow {
            edges: vec![[0, 1]],
            directions: vec![[1.0, 0.0, 0.0]],
            magnitudes: vec![1.0],
        };
        let c = flow_curl(&ef, 2);
        assert_eq!(c.len(), 2);
    }

    #[test]
    fn test_compute_edge_flow_zero_length() {
        let pos = vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let ef = compute_edge_flow(&pos, &[[0, 1]]);
        assert_eq!(ef.directions[0], [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_flow_magnitude_out_of_bounds() {
        let ef = EdgeFlow {
            edges: vec![],
            directions: vec![],
            magnitudes: vec![],
        };
        assert!((flow_magnitude_ef(&ef, 0)).abs() < 1e-6);
    }
}
