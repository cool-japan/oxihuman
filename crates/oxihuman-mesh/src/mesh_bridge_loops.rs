// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BridgeLoopsConfig {
    pub segments: usize,
    pub twist: i32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BridgeLoopsResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub new_faces: usize,
}

#[allow(dead_code)]
pub fn default_bridge_config() -> BridgeLoopsConfig {
    BridgeLoopsConfig { segments: 1, twist: 0 }
}

#[allow(dead_code)]
pub fn bridge_edge_loops(
    loop_a: &[[f32; 3]],
    loop_b: &[[f32; 3]],
    config: &BridgeLoopsConfig,
) -> BridgeLoopsResult {
    let n = loop_a.len().min(loop_b.len());
    if n == 0 {
        return BridgeLoopsResult {
            positions: Vec::new(),
            indices: Vec::new(),
            new_faces: 0,
        };
    }
    let twist = config.twist.rem_euclid(n as i32) as usize;
    let mut positions: Vec<[f32; 3]> = Vec::new();
    positions.extend_from_slice(loop_a);
    positions.extend_from_slice(loop_b);
    let mut indices: Vec<u32> = Vec::new();
    for i in 0..n {
        let next = (i + 1) % n;
        let b_i = (i + twist) % n;
        let b_next = (next + twist) % n;
        let a0 = i as u32;
        let a1 = next as u32;
        let b0 = (n + b_i) as u32;
        let b1 = (n + b_next) as u32;
        // two triangles per quad
        indices.extend_from_slice(&[a0, a1, b0]);
        indices.extend_from_slice(&[a1, b1, b0]);
    }
    let new_faces = n * 2;
    BridgeLoopsResult { positions, indices, new_faces }
}

#[allow(dead_code)]
pub fn bridge_face_count(result: &BridgeLoopsResult) -> usize {
    result.new_faces
}

#[allow(dead_code)]
pub fn bridge_vertex_count(result: &BridgeLoopsResult) -> usize {
    result.positions.len()
}

#[allow(dead_code)]
pub fn bridge_validate_loops(loop_a: &[[f32; 3]], loop_b: &[[f32; 3]]) -> bool {
    !loop_a.is_empty() && !loop_b.is_empty() && loop_a.len() == loop_b.len()
}

#[allow(dead_code)]
pub fn bridge_to_json(result: &BridgeLoopsResult) -> String {
    format!(
        r#"{{"vertex_count":{},"face_count":{}}}"#,
        result.positions.len(),
        result.new_faces
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square_loop(z: f32) -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, z],
            [1.0, 0.0, z],
            [1.0, 1.0, z],
            [0.0, 1.0, z],
        ]
    }

    #[test]
    fn test_default_config() {
        let cfg = default_bridge_config();
        assert_eq!(cfg.segments, 1);
        assert_eq!(cfg.twist, 0);
    }

    #[test]
    fn test_bridge_produces_vertices() {
        let la = square_loop(0.0);
        let lb = square_loop(1.0);
        let cfg = default_bridge_config();
        let res = bridge_edge_loops(&la, &lb, &cfg);
        assert_eq!(bridge_vertex_count(&res), 8);
    }

    #[test]
    fn test_bridge_face_count() {
        let la = square_loop(0.0);
        let lb = square_loop(1.0);
        let cfg = default_bridge_config();
        let res = bridge_edge_loops(&la, &lb, &cfg);
        assert_eq!(bridge_face_count(&res), 8);
    }

    #[test]
    fn test_empty_loops() {
        let cfg = default_bridge_config();
        let res = bridge_edge_loops(&[], &[], &cfg);
        assert_eq!(bridge_vertex_count(&res), 0);
        assert_eq!(bridge_face_count(&res), 0);
    }

    #[test]
    fn test_validate_loops_ok() {
        let la = square_loop(0.0);
        let lb = square_loop(1.0);
        assert!(bridge_validate_loops(&la, &lb));
    }

    #[test]
    fn test_validate_loops_mismatch() {
        let la = square_loop(0.0);
        let lb = vec![[0.0f32; 3]; 3];
        assert!(!bridge_validate_loops(&la, &lb));
    }

    #[test]
    fn test_to_json() {
        let la = square_loop(0.0);
        let lb = square_loop(1.0);
        let cfg = default_bridge_config();
        let res = bridge_edge_loops(&la, &lb, &cfg);
        let j = bridge_to_json(&res);
        assert!(j.contains("vertex_count"));
        assert!(j.contains("face_count"));
    }

    #[test]
    fn test_bridge_with_twist() {
        let la = square_loop(0.0);
        let lb = square_loop(1.0);
        let mut cfg = default_bridge_config();
        cfg.twist = 1;
        let res = bridge_edge_loops(&la, &lb, &cfg);
        assert_eq!(bridge_face_count(&res), 8);
    }
}
