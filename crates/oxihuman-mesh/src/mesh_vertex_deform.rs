// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Per-vertex deformation utilities: displacement, lattice, and blend-based.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexDeform {
    pub deltas: Vec<[f32; 3]>,
}

#[allow(dead_code)]
pub fn new_vertex_deform(vertex_count: usize) -> VertexDeform {
    VertexDeform {
        deltas: vec![[0.0; 3]; vertex_count],
    }
}

#[allow(dead_code)]
pub fn set_delta(deform: &mut VertexDeform, idx: usize, delta: [f32; 3]) {
    if idx < deform.deltas.len() {
        deform.deltas[idx] = delta;
    }
}

#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn apply_deform(positions: &mut [[f32; 3]], deform: &VertexDeform, weight: f32) {
    let n = positions.len().min(deform.deltas.len());
    for i in 0..n {
        positions[i][0] += deform.deltas[i][0] * weight;
        positions[i][1] += deform.deltas[i][1] * weight;
        positions[i][2] += deform.deltas[i][2] * weight;
    }
}

#[allow(dead_code)]
pub fn deform_magnitude(deform: &VertexDeform, idx: usize) -> f32 {
    if idx >= deform.deltas.len() {
        return 0.0;
    }
    let d = deform.deltas[idx];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

#[allow(dead_code)]
pub fn max_deform_magnitude(deform: &VertexDeform) -> f32 {
    deform
        .deltas
        .iter()
        .map(|d| (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt())
        .fold(0.0_f32, f32::max)
}

#[allow(dead_code)]
pub fn scale_deform(deform: &mut VertexDeform, factor: f32) {
    for d in &mut deform.deltas {
        d[0] *= factor;
        d[1] *= factor;
        d[2] *= factor;
    }
}

#[allow(dead_code)]
pub fn nonzero_delta_count(deform: &VertexDeform) -> usize {
    deform
        .deltas
        .iter()
        .filter(|d| d[0] != 0.0 || d[1] != 0.0 || d[2] != 0.0)
        .count()
}

#[allow(dead_code)]
pub fn deform_to_json(deform: &VertexDeform) -> String {
    format!(
        "{{\"vertex_count\":{},\"nonzero\":{}}}",
        deform.deltas.len(),
        nonzero_delta_count(deform)
    )
}

/// Sinusoidal deform along Y using PI.
#[allow(dead_code)]
pub fn sine_deform_y(deform: &mut VertexDeform, amplitude: f32, frequency: f32) {
    for (i, d) in deform.deltas.iter_mut().enumerate() {
        d[1] += amplitude * (frequency * PI * i as f32).sin();
    }
}

#[allow(dead_code)]
pub fn clear_deform(deform: &mut VertexDeform) {
    for d in &mut deform.deltas {
        *d = [0.0; 3];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_zero_deltas() {
        let deform = new_vertex_deform(5);
        assert_eq!(nonzero_delta_count(&deform), 0);
    }

    #[test]
    fn test_set_delta() {
        let mut deform = new_vertex_deform(3);
        set_delta(&mut deform, 1, [1.0, 0.0, 0.0]);
        assert_eq!(nonzero_delta_count(&deform), 1);
    }

    #[test]
    fn test_apply_deform() {
        let mut positions = vec![[0.0f32; 3]; 2];
        let mut deform = new_vertex_deform(2);
        set_delta(&mut deform, 0, [1.0, 0.0, 0.0]);
        apply_deform(&mut positions, &deform, 1.0);
        assert!((positions[0][0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_deform_magnitude() {
        let mut deform = new_vertex_deform(2);
        set_delta(&mut deform, 0, [3.0, 4.0, 0.0]);
        assert!((deform_magnitude(&deform, 0) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_max_magnitude() {
        let mut deform = new_vertex_deform(3);
        set_delta(&mut deform, 2, [0.0, 0.0, 1.0]);
        assert!((max_deform_magnitude(&deform) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_scale_deform() {
        let mut deform = new_vertex_deform(1);
        set_delta(&mut deform, 0, [2.0, 0.0, 0.0]);
        scale_deform(&mut deform, 0.5);
        assert!((deform.deltas[0][0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_clear_deform() {
        let mut deform = new_vertex_deform(3);
        set_delta(&mut deform, 0, [1.0, 2.0, 3.0]);
        clear_deform(&mut deform);
        assert_eq!(nonzero_delta_count(&deform), 0);
    }

    #[test]
    fn test_json_output() {
        let deform = new_vertex_deform(4);
        let j = deform_to_json(&deform);
        assert!(j.contains("vertex_count"));
    }

    #[test]
    fn test_sine_deform_non_zero() {
        let mut deform = new_vertex_deform(4);
        // Use 0.5 frequency so i=1 gives sin(0.5*PI) = 1.0 (non-zero)
        sine_deform_y(&mut deform, 1.0, 0.5);
        let any_nz = deform.deltas.iter().any(|d| d[1].abs() > 1e-6);
        assert!(any_nz);
    }

    #[test]
    fn test_out_of_bounds_magnitude() {
        let deform = new_vertex_deform(2);
        assert!((deform_magnitude(&deform, 99)).abs() < 1e-6);
    }
}
