// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Draco mesh compression export stub.

#[allow(dead_code)]
pub struct DracoExportV2 {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub quantization_bits: u8,
}

#[allow(dead_code)]
pub fn new_draco_export_v2(quantization_bits: u8) -> DracoExportV2 {
    DracoExportV2 { positions: Vec::new(), indices: Vec::new(), quantization_bits }
}

#[allow(dead_code)]
pub fn draco_add_vertex_v2(exp: &mut DracoExportV2, pos: [f32; 3]) {
    exp.positions.push(pos);
}

#[allow(dead_code)]
pub fn draco_add_triangle_v2(exp: &mut DracoExportV2, a: u32, b: u32, c: u32) {
    exp.indices.push(a);
    exp.indices.push(b);
    exp.indices.push(c);
}

#[allow(dead_code)]
pub fn draco_vertex_count_v2(exp: &DracoExportV2) -> usize {
    exp.positions.len()
}

#[allow(dead_code)]
pub fn draco_triangle_count_v2(exp: &DracoExportV2) -> usize {
    exp.indices.len() / 3
}

#[allow(dead_code)]
pub fn draco_estimated_bits_per_vertex_v2(exp: &DracoExportV2) -> u32 {
    3 * exp.quantization_bits as u32
}

#[allow(dead_code)]
pub fn draco_to_json_meta_v2(exp: &DracoExportV2) -> String {
    format!(r#"{{"vertices":{},"triangles":{},"quantBits":{}}}"#,
        draco_vertex_count_v2(exp), draco_triangle_count_v2(exp), exp.quantization_bits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let exp = new_draco_export_v2(11);
        assert_eq!(draco_vertex_count_v2(&exp), 0);
        assert_eq!(draco_triangle_count_v2(&exp), 0);
    }

    #[test]
    fn test_add_vertex() {
        let mut exp = new_draco_export_v2(8);
        draco_add_vertex_v2(&mut exp, [1.0, 2.0, 3.0]);
        assert_eq!(draco_vertex_count_v2(&exp), 1);
    }

    #[test]
    fn test_add_triangle() {
        let mut exp = new_draco_export_v2(8);
        draco_add_vertex_v2(&mut exp, [0.0; 3]);
        draco_add_vertex_v2(&mut exp, [1.0, 0.0, 0.0]);
        draco_add_vertex_v2(&mut exp, [0.0, 1.0, 0.0]);
        draco_add_triangle_v2(&mut exp, 0, 1, 2);
        assert_eq!(draco_triangle_count_v2(&exp), 1);
    }

    #[test]
    fn test_estimated_bits() {
        let exp = new_draco_export_v2(11);
        assert_eq!(draco_estimated_bits_per_vertex_v2(&exp), 33);
    }

    #[test]
    fn test_to_json_meta_contains_vertices() {
        let exp = new_draco_export_v2(8);
        assert!(draco_to_json_meta_v2(&exp).contains("vertices"));
    }

    #[test]
    fn test_to_json_meta_contains_quant() {
        let exp = new_draco_export_v2(8);
        assert!(draco_to_json_meta_v2(&exp).contains("quantBits"));
    }

    #[test]
    fn test_vertex_count_multiple() {
        let mut exp = new_draco_export_v2(8);
        for _ in 0..5 {
            draco_add_vertex_v2(&mut exp, [0.0; 3]);
        }
        assert_eq!(draco_vertex_count_v2(&exp), 5);
    }

    #[test]
    fn test_triangle_count_two() {
        let mut exp = new_draco_export_v2(8);
        draco_add_triangle_v2(&mut exp, 0, 1, 2);
        draco_add_triangle_v2(&mut exp, 1, 2, 3);
        assert_eq!(draco_triangle_count_v2(&exp), 2);
    }
}
