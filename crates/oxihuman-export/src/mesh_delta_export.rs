// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Mesh delta export: compressed sparse vertex delta arrays for shape keys.

/// A sparse vertex delta entry.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct VertexDeltaEntry {
    pub vertex_index: u32,
    pub delta: [f32; 3],
}

/// Mesh delta export container.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshDeltaExport {
    pub name: String,
    pub deltas: Vec<VertexDeltaEntry>,
    pub total_vertices: usize,
    pub weight: f32,
}

/// Create a new mesh delta export.
#[allow(dead_code)]
pub fn new_mesh_delta_export(name: &str, total_vertices: usize) -> MeshDeltaExport {
    MeshDeltaExport {
        name: name.to_string(),
        deltas: Vec::new(),
        total_vertices,
        weight: 1.0,
    }
}

/// Add a vertex delta (skips near-zero).
#[allow(dead_code)]
pub fn add_vertex_delta(exp: &mut MeshDeltaExport, vertex: u32, delta: [f32; 3]) {
    if delta[0].abs() > 1e-7 || delta[1].abs() > 1e-7 || delta[2].abs() > 1e-7 {
        exp.deltas.push(VertexDeltaEntry {
            vertex_index: vertex,
            delta,
        });
    }
}

/// Delta entry count.
#[allow(dead_code)]
pub fn delta_entry_count(exp: &MeshDeltaExport) -> usize {
    exp.deltas.len()
}

/// Maximum delta magnitude.
#[allow(dead_code)]
pub fn mesh_delta_max_magnitude(exp: &MeshDeltaExport) -> f32 {
    exp.deltas
        .iter()
        .map(|e| {
            let d = e.delta;
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        })
        .fold(0.0_f32, f32::max)
}

/// Sparsity: delta_count / total_vertices.
#[allow(dead_code)]
pub fn mesh_delta_sparsity(exp: &MeshDeltaExport) -> f32 {
    if exp.total_vertices == 0 {
        return 0.0;
    }
    exp.deltas.len() as f32 / exp.total_vertices as f32
}

/// Apply delta to a positions array with weight.
#[allow(dead_code)]
pub fn apply_mesh_delta(positions: &mut [[f32; 3]], exp: &MeshDeltaExport, weight: f32) {
    for e in &exp.deltas {
        let i = e.vertex_index as usize;
        if i < positions.len() {
            positions[i][0] += e.delta[0] * weight;
            positions[i][1] += e.delta[1] * weight;
            positions[i][2] += e.delta[2] * weight;
        }
    }
}

/// Serialise to JSON.
#[allow(dead_code)]
pub fn mesh_delta_to_json(exp: &MeshDeltaExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"delta_count\":{},\"weight\":{}}}",
        exp.name,
        delta_entry_count(exp),
        exp.weight
    )
}

/// Validate: all vertex indices in range.
#[allow(dead_code)]
pub fn validate_mesh_delta(exp: &MeshDeltaExport) -> bool {
    exp.deltas
        .iter()
        .all(|e| (e.vertex_index as usize) < exp.total_vertices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let exp = new_mesh_delta_export("smile", 100);
        assert_eq!(delta_entry_count(&exp), 0);
    }

    #[test]
    fn add_nonzero_delta() {
        let mut exp = new_mesh_delta_export("smile", 10);
        add_vertex_delta(&mut exp, 3, [0.1, 0.0, 0.0]);
        assert_eq!(delta_entry_count(&exp), 1);
    }

    #[test]
    fn skip_zero_delta() {
        let mut exp = new_mesh_delta_export("smile", 10);
        add_vertex_delta(&mut exp, 0, [0.0; 3]);
        assert_eq!(delta_entry_count(&exp), 0);
    }

    #[test]
    fn max_magnitude_correct() {
        let mut exp = new_mesh_delta_export("s", 5);
        add_vertex_delta(&mut exp, 0, [3.0, 4.0, 0.0]);
        assert!((mesh_delta_max_magnitude(&exp) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn sparsity_correct() {
        let mut exp = new_mesh_delta_export("s", 10);
        add_vertex_delta(&mut exp, 0, [1.0, 0.0, 0.0]);
        assert!((mesh_delta_sparsity(&exp) - 0.1).abs() < 1e-5);
    }

    #[test]
    fn apply_delta_works() {
        let mut exp = new_mesh_delta_export("s", 1);
        add_vertex_delta(&mut exp, 0, [1.0, 0.0, 0.0]);
        let mut pos = vec![[0.0f32; 3]];
        apply_mesh_delta(&mut pos, &exp, 0.5);
        assert!((pos[0][0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn validate_in_range() {
        let mut exp = new_mesh_delta_export("s", 5);
        add_vertex_delta(&mut exp, 4, [0.1, 0.0, 0.0]);
        assert!(validate_mesh_delta(&exp));
    }

    #[test]
    fn json_contains_name() {
        let exp = new_mesh_delta_export("blink", 100);
        let j = mesh_delta_to_json(&exp);
        assert!(j.contains("blink"));
    }

    #[test]
    fn weight_in_range() {
        let exp = new_mesh_delta_export("s", 5);
        assert!((0.0..=1.0).contains(&exp.weight));
    }

    #[test]
    fn empty_max_zero() {
        let exp = new_mesh_delta_export("s", 5);
        assert!((mesh_delta_max_magnitude(&exp)).abs() < 1e-6);
    }
}
