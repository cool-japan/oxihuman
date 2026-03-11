// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Geometry delta export: position/normal delta arrays between two mesh states.

/// A geometry delta between two mesh states.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GeometryDelta {
    pub position_deltas: Vec<[f32; 3]>,
    pub normal_deltas: Vec<[f32; 3]>,
    pub nonzero_count: usize,
}

/// Compute geometry delta from two position arrays.
#[allow(dead_code)]
pub fn compute_geometry_delta(base: &[[f32; 3]], target: &[[f32; 3]]) -> GeometryDelta {
    let n = base.len().min(target.len());
    let mut pos_deltas = Vec::with_capacity(n);
    let mut nonzero = 0;
    for i in 0..n {
        let d = [
            target[i][0] - base[i][0],
            target[i][1] - base[i][1],
            target[i][2] - base[i][2],
        ];
        if d[0].abs() > 1e-7 || d[1].abs() > 1e-7 || d[2].abs() > 1e-7 {
            nonzero += 1;
        }
        pos_deltas.push(d);
    }
    GeometryDelta {
        position_deltas: pos_deltas,
        normal_deltas: Vec::new(),
        nonzero_count: nonzero,
    }
}

/// Maximum delta magnitude.
#[allow(dead_code)]
pub fn max_delta_magnitude(delta: &GeometryDelta) -> f32 {
    delta
        .position_deltas
        .iter()
        .map(|d| (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt())
        .fold(0.0_f32, f32::max)
}

/// Average delta magnitude.
#[allow(dead_code)]
pub fn avg_delta_magnitude(delta: &GeometryDelta) -> f32 {
    if delta.position_deltas.is_empty() {
        return 0.0;
    }
    let sum: f32 = delta
        .position_deltas
        .iter()
        .map(|d| (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt())
        .sum();
    sum / delta.position_deltas.len() as f32
}

/// Vertex count in delta.
#[allow(dead_code)]
pub fn delta_vertex_count(delta: &GeometryDelta) -> usize {
    delta.position_deltas.len()
}

/// Sparsity ratio: nonzero / total.
#[allow(dead_code)]
pub fn delta_sparsity(delta: &GeometryDelta) -> f32 {
    if delta.position_deltas.is_empty() {
        return 0.0;
    }
    delta.nonzero_count as f32 / delta.position_deltas.len() as f32
}

/// Scale all deltas by a factor.
#[allow(dead_code)]
pub fn scale_delta(delta: &mut GeometryDelta, scale: f32) {
    for d in &mut delta.position_deltas {
        d[0] *= scale;
        d[1] *= scale;
        d[2] *= scale;
    }
}

/// Add normal deltas from two normal arrays.
#[allow(dead_code)]
pub fn add_normal_deltas(delta: &mut GeometryDelta, base: &[[f32; 3]], target: &[[f32; 3]]) {
    let n = base.len().min(target.len());
    delta.normal_deltas = (0..n)
        .map(|i| {
            [
                target[i][0] - base[i][0],
                target[i][1] - base[i][1],
                target[i][2] - base[i][2],
            ]
        })
        .collect();
}

/// Serialise to JSON.
#[allow(dead_code)]
pub fn geometry_delta_to_json(delta: &GeometryDelta) -> String {
    format!(
        "{{\"vertex_count\":{},\"nonzero_count\":{},\"max_magnitude\":{}}}",
        delta_vertex_count(delta),
        delta.nonzero_count,
        max_delta_magnitude(delta)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_delta_nonzero_zero() {
        let base = vec![[0.0f32; 3]; 3];
        let d = compute_geometry_delta(&base, &base);
        assert_eq!(d.nonzero_count, 0);
    }

    #[test]
    fn nonzero_count_correct() {
        let base = vec![[0.0f32; 3], [0.0, 0.0, 0.0]];
        let target = vec![[1.0f32, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let d = compute_geometry_delta(&base, &target);
        assert_eq!(d.nonzero_count, 1);
    }

    #[test]
    fn max_delta_correct() {
        let base = vec![[0.0f32; 3]];
        let target = vec![[3.0f32, 4.0, 0.0]];
        let d = compute_geometry_delta(&base, &target);
        assert!((max_delta_magnitude(&d) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn vertex_count_min() {
        let base = vec![[0.0f32; 3]; 4];
        let target = vec![[1.0f32, 0.0, 0.0]; 2];
        let d = compute_geometry_delta(&base, &target);
        assert_eq!(delta_vertex_count(&d), 2);
    }

    #[test]
    fn scale_delta_doubles() {
        let base = vec![[0.0f32; 3]];
        let target = vec![[1.0f32, 0.0, 0.0]];
        let mut d = compute_geometry_delta(&base, &target);
        scale_delta(&mut d, 2.0);
        assert!((d.position_deltas[0][0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn sparsity_all_nonzero() {
        let base = vec![[0.0f32; 3]];
        let target = vec![[1.0f32, 0.0, 0.0]];
        let d = compute_geometry_delta(&base, &target);
        assert!((delta_sparsity(&d) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn normal_deltas_added() {
        let base = vec![[0.0f32; 3]];
        let target = vec![[1.0f32, 0.0, 0.0]];
        let mut d = compute_geometry_delta(&base, &target);
        add_normal_deltas(&mut d, &[[0.0f32; 3]], &[[0.0f32, 1.0, 0.0]]);
        assert_eq!(d.normal_deltas.len(), 1);
    }

    #[test]
    fn json_contains_vertex_count() {
        let d = compute_geometry_delta(&[[0.0f32; 3]], &[[1.0f32, 0.0, 0.0]]);
        let j = geometry_delta_to_json(&d);
        assert!(j.contains("vertex_count"));
    }

    #[test]
    fn empty_delta_max_zero() {
        let base: Vec<[f32; 3]> = Vec::new();
        let d = compute_geometry_delta(&base, &base);
        assert!((max_delta_magnitude(&d)).abs() < 1e-6);
    }

    #[test]
    fn contains_range() {
        let v = 0.5_f32;
        assert!((0.0..=1.0).contains(&v));
    }
}
