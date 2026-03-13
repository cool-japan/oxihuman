// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export vertex delta (morph target difference) data.

/// Vertex delta entry.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct VertexDelta {
    pub vertex_index: u32,
    pub position_delta: [f32; 3],
    pub normal_delta: [f32; 3],
}

/// Vertex delta export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexDeltaExport {
    pub name: String,
    pub deltas: Vec<VertexDelta>,
}

#[allow(dead_code)]
pub fn new_vertex_delta_export(name: &str) -> VertexDeltaExport {
    VertexDeltaExport { name: name.to_string(), deltas: Vec::new() }
}

#[allow(dead_code)]
pub fn vd_add_delta(export: &mut VertexDeltaExport, idx: u32, pos_delta: [f32; 3], norm_delta: [f32; 3]) {
    export.deltas.push(VertexDelta { vertex_index: idx, position_delta: pos_delta, normal_delta: norm_delta });
}

#[allow(dead_code)]
pub fn vd_delta_count(export: &VertexDeltaExport) -> usize { export.deltas.len() }

#[allow(dead_code)]
pub fn vd_max_displacement(export: &VertexDeltaExport) -> f32 {
    export.deltas.iter().map(|d| {
        let p = d.position_delta;
        (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt()
    }).fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn vd_average_displacement(export: &VertexDeltaExport) -> f32 {
    if export.deltas.is_empty() { return 0.0; }
    let sum: f32 = export.deltas.iter().map(|d| {
        let p = d.position_delta;
        (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt()
    }).sum();
    sum / export.deltas.len() as f32
}

#[allow(dead_code)]
pub fn vd_to_bytes(export: &VertexDeltaExport) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(export.deltas.len() as u32).to_le_bytes());
    for d in &export.deltas {
        bytes.extend_from_slice(&d.vertex_index.to_le_bytes());
        for &v in &d.position_delta { bytes.extend_from_slice(&v.to_le_bytes()); }
        for &v in &d.normal_delta { bytes.extend_from_slice(&v.to_le_bytes()); }
    }
    bytes
}

#[allow(dead_code)]
pub fn vd_to_json(export: &VertexDeltaExport) -> String {
    format!(r#"{{"name":"{}","deltas":{},"max_disp":{:.6}}}"#,
        export.name, vd_delta_count(export), vd_max_displacement(export))
}

#[allow(dead_code)]
pub fn vd_prune_small(export: &mut VertexDeltaExport, threshold: f32) {
    export.deltas.retain(|d| {
        let p = d.position_delta;
        (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt() > threshold
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_export() {
        let e = new_vertex_delta_export("smile");
        assert_eq!(e.name, "smile");
        assert_eq!(vd_delta_count(&e), 0);
    }

    #[test]
    fn test_add_delta() {
        let mut e = new_vertex_delta_export("test");
        vd_add_delta(&mut e, 0, [0.1, 0.0, 0.0], [0.0; 3]);
        assert_eq!(vd_delta_count(&e), 1);
    }

    #[test]
    fn test_max_displacement() {
        let mut e = new_vertex_delta_export("t");
        vd_add_delta(&mut e, 0, [3.0, 4.0, 0.0], [0.0; 3]);
        assert!((vd_max_displacement(&e) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_avg_displacement() {
        let mut e = new_vertex_delta_export("t");
        vd_add_delta(&mut e, 0, [1.0, 0.0, 0.0], [0.0; 3]);
        vd_add_delta(&mut e, 1, [0.0, 0.0, 0.0], [0.0; 3]);
        assert!((vd_average_displacement(&e) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_to_bytes() {
        let mut e = new_vertex_delta_export("t");
        vd_add_delta(&mut e, 0, [0.1, 0.0, 0.0], [0.0; 3]);
        let bytes = vd_to_bytes(&e);
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_to_json() {
        let e = new_vertex_delta_export("blink");
        let json = vd_to_json(&e);
        assert!(json.contains("blink"));
    }

    #[test]
    fn test_prune_small() {
        let mut e = new_vertex_delta_export("t");
        vd_add_delta(&mut e, 0, [0.001, 0.0, 0.0], [0.0; 3]);
        vd_add_delta(&mut e, 1, [1.0, 0.0, 0.0], [0.0; 3]);
        vd_prune_small(&mut e, 0.01);
        assert_eq!(vd_delta_count(&e), 1);
    }

    #[test]
    fn test_empty_displacement() {
        let e = new_vertex_delta_export("t");
        assert!((vd_max_displacement(&e) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_empty_avg() {
        let e = new_vertex_delta_export("t");
        assert!((vd_average_displacement(&e) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_prune_keeps_large() {
        let mut e = new_vertex_delta_export("t");
        vd_add_delta(&mut e, 0, [10.0, 0.0, 0.0], [0.0; 3]);
        vd_prune_small(&mut e, 0.01);
        assert_eq!(vd_delta_count(&e), 1);
    }

}
