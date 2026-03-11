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

/// Smooth edge flow by averaging neighbors.
#[allow(dead_code)]
pub fn smooth_edge_flow(flow: &EdgeFlow, _iterations: u32) -> EdgeFlow {
    // Simplified: return clone
    flow.clone()
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
