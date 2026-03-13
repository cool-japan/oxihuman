// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Blend mask export: per-vertex blend weights for shape/morph blending.

/// Blend mask export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendMaskExport {
    pub name: String,
    pub weights: Vec<f32>,
}

/// Create a new blend mask.
#[allow(dead_code)]
pub fn new_blend_mask(name: &str, vertex_count: usize) -> BlendMaskExport {
    BlendMaskExport {
        name: name.to_string(),
        weights: vec![1.0; vertex_count],
    }
}

/// Set weight at a vertex.
#[allow(dead_code)]
pub fn mask_set_weight(m: &mut BlendMaskExport, vertex: usize, weight: f32) {
    if vertex < m.weights.len() {
        m.weights[vertex] = weight;
    }
}

/// Get weight at a vertex.
#[allow(dead_code)]
pub fn mask_get_weight(m: &BlendMaskExport, vertex: usize) -> f32 {
    m.weights.get(vertex).copied().unwrap_or(0.0)
}

/// Vertex count.
#[allow(dead_code)]
pub fn mask_vertex_count(m: &BlendMaskExport) -> usize {
    m.weights.len()
}

/// Average weight.
#[allow(dead_code)]
pub fn mask_average_weight(m: &BlendMaskExport) -> f32 {
    if m.weights.is_empty() {
        return 0.0;
    }
    m.weights.iter().sum::<f32>() / m.weights.len() as f32
}

/// Count vertices with weight > threshold.
#[allow(dead_code)]
pub fn mask_active_count(m: &BlendMaskExport, threshold: f32) -> usize {
    m.weights.iter().filter(|&&w| w > threshold).count()
}

/// Invert mask (1 - weight).
#[allow(dead_code)]
pub fn mask_invert(m: &mut BlendMaskExport) {
    for w in &mut m.weights {
        *w = 1.0 - *w;
    }
}

/// Clamp all to [0, 1].
#[allow(dead_code)]
pub fn mask_clamp(m: &mut BlendMaskExport) {
    for w in &mut m.weights {
        *w = w.clamp(0.0, 1.0);
    }
}

/// Export to JSON.
#[allow(dead_code)]
pub fn blend_mask_to_json(m: &BlendMaskExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"vertex_count\":{},\"avg_weight\":{:.6}}}",
        m.name,
        m.weights.len(),
        mask_average_weight(m)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = new_blend_mask("test", 5);
        assert_eq!(mask_vertex_count(&m), 5);
    }

    #[test]
    fn test_default_weight() {
        let m = new_blend_mask("t", 3);
        assert!((mask_get_weight(&m, 0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_weight() {
        let mut m = new_blend_mask("t", 3);
        mask_set_weight(&mut m, 1, 0.5);
        assert!((mask_get_weight(&m, 1) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_average_weight() {
        let mut m = new_blend_mask("t", 2);
        mask_set_weight(&mut m, 0, 0.0);
        assert!((mask_average_weight(&m) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_active_count() {
        let mut m = new_blend_mask("t", 4);
        mask_set_weight(&mut m, 0, 0.0);
        mask_set_weight(&mut m, 1, 0.0);
        assert_eq!(mask_active_count(&m, 0.5), 2);
    }

    #[test]
    fn test_invert() {
        let mut m = new_blend_mask("t", 1);
        mask_set_weight(&mut m, 0, 0.3);
        mask_invert(&mut m);
        assert!((m.weights[0] - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_clamp() {
        let mut m = BlendMaskExport {
            name: "t".to_string(),
            weights: vec![-0.5, 1.5],
        };
        mask_clamp(&mut m);
        assert!((m.weights[0]).abs() < 1e-6);
        assert!((m.weights[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let m = new_blend_mask("mask1", 3);
        assert!(blend_mask_to_json(&m).contains("\"name\":\"mask1\""));
    }

    #[test]
    fn test_get_oob() {
        let m = new_blend_mask("t", 0);
        assert!((mask_get_weight(&m, 0)).abs() < 1e-6);
    }

    #[test]
    fn test_empty_average() {
        let m = new_blend_mask("t", 0);
        assert!((mask_average_weight(&m)).abs() < 1e-6);
    }
}
