// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Deformer binding export: records which vertices are bound to which deformer.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DeformBindEntry {
    pub deformer_name: String,
    pub vertex_indices: Vec<u32>,
    pub weights: Vec<f32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DeformBindExport {
    pub bindings: Vec<DeformBindEntry>,
}

#[allow(dead_code)]
pub fn new_deform_bind_export() -> DeformBindExport {
    DeformBindExport {
        bindings: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_binding(exp: &mut DeformBindExport, name: &str, indices: Vec<u32>, weights: Vec<f32>) {
    exp.bindings.push(DeformBindEntry {
        deformer_name: name.to_string(),
        vertex_indices: indices,
        weights,
    });
}

#[allow(dead_code)]
pub fn binding_count(exp: &DeformBindExport) -> usize {
    exp.bindings.len()
}

#[allow(dead_code)]
pub fn find_binding<'a>(exp: &'a DeformBindExport, name: &str) -> Option<&'a DeformBindEntry> {
    exp.bindings.iter().find(|b| b.deformer_name == name)
}

#[allow(dead_code)]
pub fn total_bound_vertices(exp: &DeformBindExport) -> usize {
    exp.bindings.iter().map(|b| b.vertex_indices.len()).sum()
}

#[allow(dead_code)]
pub fn binding_avg_weight(entry: &DeformBindEntry) -> f32 {
    if entry.weights.is_empty() {
        return 0.0;
    }
    entry.weights.iter().sum::<f32>() / entry.weights.len() as f32
}

#[allow(dead_code)]
pub fn binding_is_valid(entry: &DeformBindEntry) -> bool {
    entry.vertex_indices.len() == entry.weights.len()
        && entry.weights.iter().all(|&w| (0.0..=1.0).contains(&w))
}

#[allow(dead_code)]
pub fn deform_bind_to_json(exp: &DeformBindExport) -> String {
    format!(
        "{{\"binding_count\":{},\"total_vertices\":{}}}",
        binding_count(exp),
        total_bound_vertices(exp)
    )
}

#[allow(dead_code)]
pub fn normalize_binding_weights(entry: &mut DeformBindEntry) {
    let sum: f32 = entry.weights.iter().sum();
    if sum > 0.0 {
        for w in &mut entry.weights {
            *w /= sum;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let exp = new_deform_bind_export();
        assert_eq!(binding_count(&exp), 0);
    }

    #[test]
    fn test_add_binding() {
        let mut exp = new_deform_bind_export();
        add_binding(&mut exp, "lattice", vec![0, 1, 2], vec![1.0, 0.8, 0.6]);
        assert_eq!(binding_count(&exp), 1);
    }

    #[test]
    fn test_find_binding() {
        let mut exp = new_deform_bind_export();
        add_binding(&mut exp, "curve", vec![5], vec![1.0]);
        assert!(find_binding(&exp, "curve").is_some());
    }

    #[test]
    fn test_total_bound_vertices() {
        let mut exp = new_deform_bind_export();
        add_binding(&mut exp, "a", vec![0, 1], vec![1.0, 1.0]);
        add_binding(&mut exp, "b", vec![2, 3, 4], vec![0.5, 0.5, 0.5]);
        assert_eq!(total_bound_vertices(&exp), 5);
    }

    #[test]
    fn test_avg_weight() {
        let entry = DeformBindEntry {
            deformer_name: "x".to_string(),
            vertex_indices: vec![0, 1],
            weights: vec![1.0, 0.0],
        };
        assert!((binding_avg_weight(&entry) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_binding_is_valid() {
        let entry = DeformBindEntry {
            deformer_name: "x".to_string(),
            vertex_indices: vec![0, 1],
            weights: vec![0.5, 0.5],
        };
        assert!(binding_is_valid(&entry));
    }

    #[test]
    fn test_binding_invalid_mismatch() {
        let entry = DeformBindEntry {
            deformer_name: "x".to_string(),
            vertex_indices: vec![0, 1],
            weights: vec![0.5],
        };
        assert!(!binding_is_valid(&entry));
    }

    #[test]
    fn test_normalize() {
        let mut entry = DeformBindEntry {
            deformer_name: "x".to_string(),
            vertex_indices: vec![0, 1],
            weights: vec![2.0, 2.0],
        };
        normalize_binding_weights(&mut entry);
        assert!((entry.weights.iter().sum::<f32>() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_json_output() {
        let exp = new_deform_bind_export();
        let j = deform_bind_to_json(&exp);
        assert!(j.contains("binding_count"));
    }

    #[test]
    fn test_find_missing() {
        let exp = new_deform_bind_export();
        assert!(find_binding(&exp, "missing").is_none());
    }
}
