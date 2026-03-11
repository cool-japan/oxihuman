// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export quantized mesh data for bandwidth-efficient transfer.

/// Quantized position.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct QuantizedVertex {
    pub x: u16,
    pub y: u16,
    pub z: u16,
}

/// Quantized mesh export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct QuantizedMeshExport {
    pub vertices: Vec<QuantizedVertex>,
    pub indices: Vec<u32>,
    pub min_bounds: [f32; 3],
    pub max_bounds: [f32; 3],
    pub bits: u32,
}

#[allow(dead_code)]
pub fn quantize_mesh_export(positions: &[[f32; 3]], indices: &[u32]) -> QuantizedMeshExport {
    let (min, max) = bounds(positions);
    let vertices: Vec<QuantizedVertex> = positions.iter().map(|p| {
        QuantizedVertex {
            x: quantize_f32(p[0], min[0], max[0]),
            y: quantize_f32(p[1], min[1], max[1]),
            z: quantize_f32(p[2], min[2], max[2]),
        }
    }).collect();
    QuantizedMeshExport { vertices, indices: indices.to_vec(), min_bounds: min, max_bounds: max, bits: 16 }
}

#[allow(dead_code)]
pub fn dequantize_mesh_export(mesh: &QuantizedMeshExport) -> Vec<[f32; 3]> {
    mesh.vertices.iter().map(|v| {
        [
            dequantize_f32(v.x, mesh.min_bounds[0], mesh.max_bounds[0]),
            dequantize_f32(v.y, mesh.min_bounds[1], mesh.max_bounds[1]),
            dequantize_f32(v.z, mesh.min_bounds[2], mesh.max_bounds[2]),
        ]
    }).collect()
}

#[allow(dead_code)]
pub fn qm_vertex_count(mesh: &QuantizedMeshExport) -> usize { mesh.vertices.len() }

#[allow(dead_code)]
pub fn qm_face_count(mesh: &QuantizedMeshExport) -> usize { mesh.indices.len() / 3 }

#[allow(dead_code)]
pub fn qm_data_size(mesh: &QuantizedMeshExport) -> usize {
    mesh.vertices.len() * 6 + mesh.indices.len() * 4 + 24
}

#[allow(dead_code)]
pub fn qm_compression_ratio(original_size: usize, mesh: &QuantizedMeshExport) -> f32 {
    if original_size == 0 { return 0.0; }
    qm_data_size(mesh) as f32 / original_size as f32
}

#[allow(dead_code)]
pub fn qm_to_json(mesh: &QuantizedMeshExport) -> String {
    format!(r#"{{"vertices":{},"faces":{},"bits":{},"bytes":{}}}"#,
        qm_vertex_count(mesh), qm_face_count(mesh), mesh.bits, qm_data_size(mesh))
}

fn quantize_f32(v: f32, min: f32, max: f32) -> u16 {
    let range = max - min;
    if range < 1e-12 { return 0; }
    ((v - min) / range * 65535.0).clamp(0.0, 65535.0) as u16
}

fn dequantize_f32(q: u16, min: f32, max: f32) -> f32 {
    min + (q as f32 / 65535.0) * (max - min)
}

fn bounds(positions: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for p in positions {
        for k in 0..3 {
            if p[k] < min[k] { min[k] = p[k]; }
            if p[k] > max[k] { max[k] = p[k]; }
        }
    }
    if min[0].is_infinite() { min = [0.0; 3]; max = [0.0; 3]; }
    (min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri() -> (Vec<[f32; 3]>, Vec<u32>) {
        (vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]], vec![0, 1, 2])
    }

    #[test]
    fn test_quantize() {
        let (pos, idx) = tri();
        let qm = quantize_mesh_export(&pos, &idx);
        assert_eq!(qm_vertex_count(&qm), 3);
    }

    #[test]
    fn test_dequantize_roundtrip() {
        let (pos, idx) = tri();
        let qm = quantize_mesh_export(&pos, &idx);
        let deq = dequantize_mesh_export(&qm);
        for i in 0..3 {
            for k in 0..3 {
                assert!((deq[i][k] - pos[i][k]).abs() < 0.001);
            }
        }
    }

    #[test]
    fn test_face_count() {
        let (pos, idx) = tri();
        let qm = quantize_mesh_export(&pos, &idx);
        assert_eq!(qm_face_count(&qm), 1);
    }

    #[test]
    fn test_data_size() {
        let (pos, idx) = tri();
        let qm = quantize_mesh_export(&pos, &idx);
        assert!(qm_data_size(&qm) > 0);
    }

    #[test]
    fn test_compression_ratio() {
        let (pos, idx) = tri();
        let orig = pos.len() * 12 + idx.len() * 4;
        let qm = quantize_mesh_export(&pos, &idx);
        let ratio = qm_compression_ratio(orig, &qm);
        assert!(ratio > 0.0);
    }

    #[test]
    fn test_to_json() {
        let (pos, idx) = tri();
        let qm = quantize_mesh_export(&pos, &idx);
        let json = qm_to_json(&qm);
        assert!(json.contains("vertices"));
    }

    #[test]
    fn test_empty() {
        let qm = quantize_mesh_export(&[], &[]);
        assert_eq!(qm_vertex_count(&qm), 0);
    }

    #[test]
    fn test_single_point() {
        let qm = quantize_mesh_export(&[[5.0, 5.0, 5.0]], &[]);
        assert_eq!(qm.vertices[0].x, 0); // single point, range is 0
    }

    #[test]
    fn test_compression_ratio_empty() {
        let qm = quantize_mesh_export(&[], &[]);
        assert!((qm_compression_ratio(0, &qm) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_bits() {
        let (pos, idx) = tri();
        let qm = quantize_mesh_export(&pos, &idx);
        assert_eq!(qm.bits, 16);
    }

}
