// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Per-vertex scalar weight buffers with normalisation and blending support.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexWeightBuffer {
    pub weights: Vec<f32>,
    pub name: String,
}

#[allow(dead_code)]
pub fn new_weight_buffer(name: &str, vertex_count: usize) -> VertexWeightBuffer {
    VertexWeightBuffer {
        weights: vec![0.0; vertex_count],
        name: name.to_string(),
    }
}

#[allow(dead_code)]
pub fn set_weight(buf: &mut VertexWeightBuffer, idx: usize, weight: f32) {
    if idx < buf.weights.len() {
        buf.weights[idx] = weight;
    }
}

#[allow(dead_code)]
pub fn get_weight(buf: &VertexWeightBuffer, idx: usize) -> f32 {
    if idx < buf.weights.len() {
        buf.weights[idx]
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn normalize_weights(buf: &mut VertexWeightBuffer) {
    let max = buf.weights.iter().cloned().fold(0.0_f32, f32::max);
    if max > 0.0 {
        for w in &mut buf.weights {
            *w /= max;
        }
    }
}

#[allow(dead_code)]
pub fn average_weight(buf: &VertexWeightBuffer) -> f32 {
    if buf.weights.is_empty() {
        return 0.0;
    }
    buf.weights.iter().sum::<f32>() / buf.weights.len() as f32
}

#[allow(dead_code)]
pub fn clamp_weights(buf: &mut VertexWeightBuffer) {
    for w in &mut buf.weights {
        *w = w.clamp(0.0, 1.0);
    }
}

#[allow(dead_code)]
pub fn blend_weight_buffers(
    a: &VertexWeightBuffer,
    b: &VertexWeightBuffer,
    t: f32,
) -> VertexWeightBuffer {
    let n = a.weights.len().min(b.weights.len());
    let weights: Vec<f32> = (0..n)
        .map(|i| a.weights[i] * (1.0 - t) + b.weights[i] * t)
        .collect();
    VertexWeightBuffer {
        weights,
        name: a.name.clone(),
    }
}

#[allow(dead_code)]
pub fn weight_vertex_count(buf: &VertexWeightBuffer) -> usize {
    buf.weights.len()
}

#[allow(dead_code)]
pub fn weights_above_threshold(buf: &VertexWeightBuffer, threshold: f32) -> usize {
    buf.weights.iter().filter(|&&w| w > threshold).count()
}

#[allow(dead_code)]
pub fn weight_buffer_to_json(buf: &VertexWeightBuffer) -> String {
    format!(
        "{{\"name\":\"{}\",\"vertex_count\":{}}}",
        buf.name,
        buf.weights.len()
    )
}

#[allow(dead_code)]
pub fn weights_valid(buf: &VertexWeightBuffer) -> bool {
    buf.weights.iter().all(|&w| (0.0..=1.0).contains(&w))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer_zeros() {
        let buf = new_weight_buffer("skin", 5);
        assert!(buf.weights.iter().all(|&w| w == 0.0));
    }

    #[test]
    fn test_set_get() {
        let mut buf = new_weight_buffer("test", 4);
        set_weight(&mut buf, 2, 0.7);
        assert!((get_weight(&buf, 2) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_weights() {
        let mut buf = new_weight_buffer("x", 3);
        set_weight(&mut buf, 0, 2.0);
        normalize_weights(&mut buf);
        assert!((get_weight(&buf, 0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_average_weight() {
        let mut buf = new_weight_buffer("y", 2);
        set_weight(&mut buf, 0, 1.0);
        set_weight(&mut buf, 1, 0.0);
        assert!((average_weight(&buf) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_clamp_weights() {
        let mut buf = new_weight_buffer("z", 2);
        set_weight(&mut buf, 0, 2.0);
        set_weight(&mut buf, 1, -1.0);
        clamp_weights(&mut buf);
        assert!(weights_valid(&buf));
    }

    #[test]
    fn test_blend_buffers() {
        let mut a = new_weight_buffer("a", 2);
        let mut b = new_weight_buffer("b", 2);
        set_weight(&mut a, 0, 0.0);
        set_weight(&mut b, 0, 1.0);
        let blended = blend_weight_buffers(&a, &b, 0.5);
        assert!((blended.weights[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_above_threshold() {
        let mut buf = new_weight_buffer("t", 4);
        set_weight(&mut buf, 0, 0.8);
        set_weight(&mut buf, 1, 0.3);
        assert_eq!(weights_above_threshold(&buf, 0.5), 1);
    }

    #[test]
    fn test_json_output() {
        let buf = new_weight_buffer("skin", 6);
        let j = weight_buffer_to_json(&buf);
        assert!(j.contains("skin"));
    }

    #[test]
    fn test_vertex_count() {
        let buf = new_weight_buffer("x", 10);
        assert_eq!(weight_vertex_count(&buf), 10);
    }

    #[test]
    fn test_weights_valid_after_new() {
        let buf = new_weight_buffer("ok", 5);
        assert!(weights_valid(&buf));
    }
}
