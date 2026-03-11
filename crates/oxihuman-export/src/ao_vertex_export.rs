// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Ambient occlusion per-vertex export.

/// Per-vertex AO export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AoVertexExport {
    pub values: Vec<f32>,
}

/// Create AO export from a list of per-vertex occlusion values [0..1].
#[allow(dead_code)]
pub fn new_ao_vertex_export(values: &[f32]) -> AoVertexExport {
    AoVertexExport {
        values: values.to_vec(),
    }
}

/// Vertex count.
#[allow(dead_code)]
pub fn ao_vertex_count(e: &AoVertexExport) -> usize {
    e.values.len()
}

/// Get AO value at a vertex.
#[allow(dead_code)]
pub fn ao_value_at(e: &AoVertexExport, idx: usize) -> Option<f32> {
    e.values.get(idx).copied()
}

/// Average AO across all vertices.
#[allow(dead_code)]
pub fn ao_average(e: &AoVertexExport) -> f32 {
    if e.values.is_empty() {
        return 0.0;
    }
    e.values.iter().sum::<f32>() / e.values.len() as f32
}

/// Clamp all values to [0, 1].
#[allow(dead_code)]
pub fn ao_clamp(e: &mut AoVertexExport) {
    for v in &mut e.values {
        *v = v.clamp(0.0, 1.0);
    }
}

/// Invert AO (1 - value).
#[allow(dead_code)]
pub fn ao_invert(e: &mut AoVertexExport) {
    for v in &mut e.values {
        *v = 1.0 - *v;
    }
}

/// Export to JSON string.
#[allow(dead_code)]
pub fn ao_vertex_to_json(e: &AoVertexExport) -> String {
    format!(
        "{{\"vertex_count\":{},\"average\":{:.6}}}",
        e.values.len(),
        ao_average(e)
    )
}

/// Export to CSV string.
#[allow(dead_code)]
pub fn ao_vertex_to_csv(e: &AoVertexExport) -> String {
    let mut s = "index,ao\n".to_string();
    for (i, v) in e.values.iter().enumerate() {
        s.push_str(&format!("{},{:.6}\n", i, v));
    }
    s
}

/// Validate: all values in [0, 1].
#[allow(dead_code)]
pub fn ao_validate(e: &AoVertexExport) -> bool {
    e.values.iter().all(|v| (0.0..=1.0).contains(v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let e = new_ao_vertex_export(&[0.5, 0.3, 0.8]);
        assert_eq!(ao_vertex_count(&e), 3);
    }

    #[test]
    fn test_ao_value_at() {
        let e = new_ao_vertex_export(&[0.1, 0.9]);
        assert!((ao_value_at(&e, 0).unwrap() - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_ao_value_at_oob() {
        let e = new_ao_vertex_export(&[]);
        assert!(ao_value_at(&e, 0).is_none());
    }

    #[test]
    fn test_average() {
        let e = new_ao_vertex_export(&[0.0, 1.0]);
        assert!((ao_average(&e) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clamp() {
        let mut e = new_ao_vertex_export(&[-0.5, 1.5]);
        ao_clamp(&mut e);
        assert!((e.values[0]).abs() < 1e-6);
        assert!((e.values[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_invert() {
        let mut e = new_ao_vertex_export(&[0.3]);
        ao_invert(&mut e);
        assert!((e.values[0] - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let e = new_ao_vertex_export(&[0.5]);
        assert!(ao_vertex_to_json(&e).contains("\"vertex_count\":1"));
    }

    #[test]
    fn test_to_csv() {
        let e = new_ao_vertex_export(&[0.25]);
        let csv = ao_vertex_to_csv(&e);
        assert!(csv.contains("0,0.250000"));
    }

    #[test]
    fn test_validate() {
        assert!(ao_validate(&new_ao_vertex_export(&[0.0, 0.5, 1.0])));
        assert!(!ao_validate(&new_ao_vertex_export(&[-0.1])));
    }

    #[test]
    fn test_empty_average() {
        let e = new_ao_vertex_export(&[]);
        assert!((ao_average(&e)).abs() < 1e-6);
    }
}
