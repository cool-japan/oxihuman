// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Catmull-Clark subdivision (one level, quad mesh input) — alternate implementation.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CatmullClarkConfig {
    pub iterations: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CatmullClarkResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub iterations_done: usize,
}

#[allow(dead_code)]
pub fn default_catmull_clark_config() -> CatmullClarkConfig {
    CatmullClarkConfig { iterations: 1 }
}

/// Stub Catmull-Clark subdivision: for each quad, generate a face point
/// at the centroid. Produces 4 sub-quads per original quad.
#[allow(dead_code)]
pub fn catmull_clark_subdivide(
    positions: &[[f32; 3]],
    quad_indices: &[u32],
    config: &CatmullClarkConfig,
) -> CatmullClarkResult {
    let mut pos = positions.to_vec();
    let mut idx = quad_indices.to_vec();
    for _ in 0..config.iterations {
        let (new_pos, new_idx) = subdivide_once_cc(&pos, &idx);
        pos = new_pos;
        idx = new_idx;
    }
    CatmullClarkResult { positions: pos, indices: idx, iterations_done: config.iterations }
}

fn subdivide_once_cc(positions: &[[f32; 3]], quad_indices: &[u32]) -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut new_pos = positions.to_vec();
    let mut new_idx: Vec<u32> = Vec::new();
    for quad in quad_indices.chunks_exact(4) {
        let a = quad[0] as usize;
        let b = quad[1] as usize;
        let c = quad[2] as usize;
        let d = quad[3] as usize;
        // face point
        let fp = centroid4(positions[a], positions[b], positions[c], positions[d]);
        let fp_idx = new_pos.len() as u32;
        new_pos.push(fp);
        // edge midpoints
        let e0 = midpoint(positions[a], positions[b]);
        let e0_idx = new_pos.len() as u32;
        new_pos.push(e0);
        let e1 = midpoint(positions[b], positions[c]);
        let e1_idx = new_pos.len() as u32;
        new_pos.push(e1);
        let e2 = midpoint(positions[c], positions[d]);
        let e2_idx = new_pos.len() as u32;
        new_pos.push(e2);
        let e3 = midpoint(positions[d], positions[a]);
        let e3_idx = new_pos.len() as u32;
        new_pos.push(e3);
        // 4 sub-quads
        new_idx.extend_from_slice(&[quad[0], e0_idx, fp_idx, e3_idx]);
        new_idx.extend_from_slice(&[e0_idx, quad[1], e1_idx, fp_idx]);
        new_idx.extend_from_slice(&[fp_idx, e1_idx, quad[2], e2_idx]);
        new_idx.extend_from_slice(&[e3_idx, fp_idx, e2_idx, quad[3]]);
    }
    (new_pos, new_idx)
}

fn centroid4(a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3]) -> [f32; 3] {
    [
        (a[0] + b[0] + c[0] + d[0]) * 0.25,
        (a[1] + b[1] + c[1] + d[1]) * 0.25,
        (a[2] + b[2] + c[2] + d[2]) * 0.25,
    ]
}

fn midpoint(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5, (a[2] + b[2]) * 0.5]
}

#[allow(dead_code)]
pub fn cc_vertex_count(result: &CatmullClarkResult) -> usize {
    result.positions.len()
}

#[allow(dead_code)]
pub fn cc_face_count(result: &CatmullClarkResult) -> usize {
    result.indices.len() / 4
}

#[allow(dead_code)]
pub fn cc_validate_config(config: &CatmullClarkConfig) -> bool {
    config.iterations > 0
}

#[allow(dead_code)]
pub fn cc_to_json(result: &CatmullClarkResult) -> String {
    format!(
        r#"{{"vertices":{},"quads":{},"iterations":{}}}"#,
        result.positions.len(),
        cc_face_count(result),
        result.iterations_done
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_quad() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let idx = vec![0u32, 1, 2, 3];
        (pos, idx)
    }

    #[test]
    fn one_iteration_produces_four_quads() {
        let (pos, idx) = unit_quad();
        let cfg = default_catmull_clark_config();
        let res = catmull_clark_subdivide(&pos, &idx, &cfg);
        assert_eq!(cc_face_count(&res), 4);
    }

    #[test]
    fn vertex_count_increases() {
        let (pos, idx) = unit_quad();
        let cfg = default_catmull_clark_config();
        let res = catmull_clark_subdivide(&pos, &idx, &cfg);
        assert!(cc_vertex_count(&res) > pos.len());
    }

    #[test]
    fn two_iterations_produces_sixteen_quads() {
        let (pos, idx) = unit_quad();
        let cfg = CatmullClarkConfig { iterations: 2 };
        let res = catmull_clark_subdivide(&pos, &idx, &cfg);
        assert_eq!(cc_face_count(&res), 16);
    }

    #[test]
    fn iterations_done_matches() {
        let (pos, idx) = unit_quad();
        let cfg = CatmullClarkConfig { iterations: 3 };
        let res = catmull_clark_subdivide(&pos, &idx, &cfg);
        assert_eq!(res.iterations_done, 3);
    }

    #[test]
    fn validate_zero_fails() {
        let cfg = CatmullClarkConfig { iterations: 0 };
        assert!(!cc_validate_config(&cfg));
    }

    #[test]
    fn validate_positive_ok() {
        let cfg = default_catmull_clark_config();
        assert!(cc_validate_config(&cfg));
    }

    #[test]
    fn to_json_has_fields() {
        let (pos, idx) = unit_quad();
        let cfg = default_catmull_clark_config();
        let res = catmull_clark_subdivide(&pos, &idx, &cfg);
        let json = cc_to_json(&res);
        assert!(json.contains("vertices"));
        assert!(json.contains("quads"));
        assert!(json.contains("iterations"));
    }

    #[test]
    fn empty_input_gives_empty_output() {
        let cfg = default_catmull_clark_config();
        let res = catmull_clark_subdivide(&[], &[], &cfg);
        assert_eq!(cc_vertex_count(&res), 0);
        assert_eq!(cc_face_count(&res), 0);
    }
}
