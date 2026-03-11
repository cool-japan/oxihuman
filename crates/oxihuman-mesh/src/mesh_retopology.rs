// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Retopology hint generation and quad-dominant mesh analysis.

// ── Structs ───────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RetopologyConfig {
    pub target_edge_length: f32,
    pub quad_bias: f32,
    pub smooth_iterations: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RetopologyHint {
    pub vertex_idx: u32,
    pub suggested_position: [f32; 3],
    pub flow_direction: [f32; 3],
    pub valence: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RetopologyResult {
    pub hints: Vec<RetopologyHint>,
    pub quad_count: usize,
    pub triangle_count: usize,
    pub avg_edge_length: f32,
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_retopology_config() -> RetopologyConfig {
    RetopologyConfig {
        target_edge_length: 0.1,
        quad_bias: 0.8,
        smooth_iterations: 3,
    }
}

#[allow(dead_code)]
pub fn vertex_valence_retopo(vertex: u32, triangles: &[[u32; 3]]) -> u32 {
    triangles
        .iter()
        .filter(|tri| tri.contains(&vertex))
        .count() as u32
}

#[allow(dead_code)]
pub fn is_irregular_vertex(valence: u32, is_boundary: bool) -> bool {
    if is_boundary {
        valence != 3
    } else {
        valence != 4
    }
}

#[allow(dead_code)]
pub fn suggest_edge_flow(positions: &[[f32; 3]], normals: &[[f32; 3]], idx: u32) -> [f32; 3] {
    let i = idx as usize;
    if i >= positions.len() || i >= normals.len() {
        return [1.0, 0.0, 0.0];
    }
    let n = normals[i];
    // tangent perpendicular to normal in XZ plane
    let len = (n[0] * n[0] + n[2] * n[2]).sqrt();
    if len < 1e-8 {
        return [1.0, 0.0, 0.0];
    }
    [-n[2] / len, 0.0, n[0] / len]
}

#[allow(dead_code)]
pub fn retopology_hint_to_json(h: &RetopologyHint) -> String {
    format!(
        "{{\"vertex_idx\":{},\"suggested_position\":[{},{},{}],\
         \"flow_direction\":[{},{},{}],\"valence\":{}}}",
        h.vertex_idx,
        h.suggested_position[0],
        h.suggested_position[1],
        h.suggested_position[2],
        h.flow_direction[0],
        h.flow_direction[1],
        h.flow_direction[2],
        h.valence
    )
}

#[allow(dead_code)]
pub fn retopology_result_to_json(r: &RetopologyResult) -> String {
    let hints_json: Vec<String> = r.hints.iter().map(retopology_hint_to_json).collect();
    format!(
        "{{\"quad_count\":{},\"triangle_count\":{},\"avg_edge_length\":{},\"hints\":[{}]}}",
        r.quad_count,
        r.triangle_count,
        r.avg_edge_length,
        hints_json.join(",")
    )
}

#[allow(dead_code)]
pub fn avg_valence(positions: &[[f32; 3]], triangles: &[[u32; 3]]) -> f32 {
    let n = positions.len();
    if n == 0 {
        return 0.0;
    }
    let total: u32 = (0..n as u32)
        .map(|v| vertex_valence_retopo(v, triangles))
        .sum();
    total as f32 / n as f32
}

#[allow(dead_code)]
pub fn irregular_vertex_count(positions: &[[f32; 3]], triangles: &[[u32; 3]]) -> usize {
    (0..positions.len() as u32)
        .filter(|&v| {
            let val = vertex_valence_retopo(v, triangles);
            is_irregular_vertex(val, false)
        })
        .count()
}

#[allow(dead_code)]
pub fn edge_length_histogram(
    positions: &[[f32; 3]],
    triangles: &[[u32; 3]],
    bins: usize,
) -> Vec<usize> {
    if bins == 0 || positions.is_empty() || triangles.is_empty() {
        return vec![0; bins.max(1)];
    }
    // collect all edge lengths
    let mut lengths: Vec<f32> = Vec::new();
    for tri in triangles {
        for &(a, b) in &[(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            let ai = a as usize;
            let bi = b as usize;
            if ai < positions.len() && bi < positions.len() {
                let dx = positions[ai][0] - positions[bi][0];
                let dy = positions[ai][1] - positions[bi][1];
                let dz = positions[ai][2] - positions[bi][2];
                lengths.push((dx * dx + dy * dy + dz * dz).sqrt());
            }
        }
    }
    if lengths.is_empty() {
        return vec![0; bins];
    }
    let min_l = lengths.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_l = lengths.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let range = (max_l - min_l).max(1e-8);
    let mut hist = vec![0usize; bins];
    for &l in &lengths {
        let bin = ((l - min_l) / range * bins as f32)
            .floor()
            .clamp(0.0, (bins - 1) as f32) as usize;
        hist[bin] += 1;
    }
    hist
}

fn compute_avg_edge_length(positions: &[[f32; 3]], triangles: &[[u32; 3]]) -> f32 {
    let mut total = 0.0f32;
    let mut count = 0usize;
    for tri in triangles {
        for &(a, b) in &[(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            let ai = a as usize;
            let bi = b as usize;
            if ai < positions.len() && bi < positions.len() {
                let dx = positions[ai][0] - positions[bi][0];
                let dy = positions[ai][1] - positions[bi][1];
                let dz = positions[ai][2] - positions[bi][2];
                total += (dx * dx + dy * dy + dz * dz).sqrt();
                count += 1;
            }
        }
    }
    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
}

#[allow(dead_code)]
pub fn analyze_topology(
    positions: &[[f32; 3]],
    triangles: &[[u32; 3]],
    _cfg: &RetopologyConfig,
) -> RetopologyResult {
    let avg_edge_length = compute_avg_edge_length(positions, triangles);
    let hints: Vec<RetopologyHint> = (0..positions.len() as u32)
        .map(|v| {
            let val = vertex_valence_retopo(v, triangles);
            let pos = if (v as usize) < positions.len() {
                positions[v as usize]
            } else {
                [0.0; 3]
            };
            let flow = suggest_edge_flow(positions, &[], v);
            RetopologyHint {
                vertex_idx: v,
                suggested_position: pos,
                flow_direction: flow,
                valence: val,
            }
        })
        .collect();
    RetopologyResult {
        hints,
        quad_count: 0,
        triangle_count: triangles.len(),
        avg_edge_length,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ]
    }

    fn simple_triangles() -> Vec<[u32; 3]> {
        vec![[0, 1, 2], [1, 3, 2]]
    }

    #[test]
    fn test_default_config() {
        let cfg = default_retopology_config();
        assert!(cfg.target_edge_length > 0.0);
        assert!(cfg.quad_bias > 0.0);
    }

    #[test]
    fn test_vertex_valence_retopo() {
        let tris = simple_triangles();
        let v0 = vertex_valence_retopo(0, &tris);
        assert_eq!(v0, 1); // appears in 1 triangle
        let v1 = vertex_valence_retopo(1, &tris);
        assert_eq!(v1, 2); // appears in 2 triangles
    }

    #[test]
    fn test_is_irregular_vertex() {
        assert!(!is_irregular_vertex(4, false)); // regular interior
        assert!(is_irregular_vertex(3, false));  // irregular interior
        assert!(!is_irregular_vertex(3, true));  // regular boundary
        assert!(is_irregular_vertex(4, true));   // irregular boundary
    }

    #[test]
    fn test_avg_valence() {
        let pos = simple_positions();
        let tris = simple_triangles();
        let av = avg_valence(&pos, &tris);
        assert!(av > 0.0);
    }

    #[test]
    fn test_edge_length_histogram() {
        let pos = simple_positions();
        let tris = simple_triangles();
        let hist = edge_length_histogram(&pos, &tris, 4);
        assert_eq!(hist.len(), 4);
        let total: usize = hist.iter().sum();
        assert!(total > 0);
    }

    #[test]
    fn test_analyze_topology_triangle_count() {
        let pos = simple_positions();
        let tris = simple_triangles();
        let cfg = default_retopology_config();
        let result = analyze_topology(&pos, &tris, &cfg);
        assert_eq!(result.triangle_count, 2);
        assert_eq!(result.hints.len(), 4);
    }

    #[test]
    fn test_retopology_hint_to_json() {
        let h = RetopologyHint {
            vertex_idx: 0,
            suggested_position: [1.0, 2.0, 3.0],
            flow_direction: [0.0, 1.0, 0.0],
            valence: 4,
        };
        let json = retopology_hint_to_json(&h);
        assert!(json.contains("\"vertex_idx\":0"));
        assert!(json.contains("\"valence\":4"));
    }

    #[test]
    fn test_retopology_result_to_json() {
        let pos = simple_positions();
        let tris = simple_triangles();
        let cfg = default_retopology_config();
        let result = analyze_topology(&pos, &tris, &cfg);
        let json = retopology_result_to_json(&result);
        assert!(json.contains("\"triangle_count\":2"));
    }

    #[test]
    fn test_suggest_edge_flow_fallback() {
        let flow = suggest_edge_flow(&[], &[], 99);
        assert!((flow[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_irregular_vertex_count() {
        let pos = simple_positions();
        let tris = simple_triangles();
        // All vertices have valence 1 or 2, which are != 4, so all are irregular interior
        let count = irregular_vertex_count(&pos, &tris);
        assert_eq!(count, 4);
    }
}
