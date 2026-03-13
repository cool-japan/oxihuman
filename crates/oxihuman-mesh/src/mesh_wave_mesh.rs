// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Generate a planar wave mesh (animated sine-wave surface).

use std::f32::consts::PI;

/// Parameters for the wave mesh generator.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WaveMeshParams {
    pub width: f32,
    pub depth: f32,
    pub cols: u32,
    pub rows: u32,
    pub amplitude: f32,
    pub frequency: f32,
    pub phase: f32,
}

impl Default for WaveMeshParams {
    fn default() -> Self {
        Self {
            width: 2.0,
            depth: 2.0,
            cols: 8,
            rows: 8,
            amplitude: 0.2,
            frequency: 1.0,
            phase: 0.0,
        }
    }
}

/// Result of a wave mesh generation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WaveMeshResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// Evaluate the wave height at position (x, z).
#[allow(dead_code)]
pub fn wave_height(x: f32, z: f32, params: &WaveMeshParams) -> f32 {
    params.amplitude * (2.0 * PI * params.frequency * x + params.phase).sin()
        + params.amplitude * 0.5 * (2.0 * PI * params.frequency * z + params.phase).cos()
}

/// Generate a wave mesh.
#[allow(dead_code)]
pub fn generate_wave_mesh(params: &WaveMeshParams) -> WaveMeshResult {
    let cols = params.cols.max(1);
    let rows = params.rows.max(1);
    let mut positions = Vec::new();
    for row in 0..=(rows) {
        for col in 0..=(cols) {
            let x = (col as f32 / cols as f32) * params.width - params.width * 0.5;
            let z = (row as f32 / rows as f32) * params.depth - params.depth * 0.5;
            let y = wave_height(x, z, params);
            positions.push([x, y, z]);
        }
    }
    let stride = cols + 1;
    let mut indices = Vec::new();
    for row in 0..rows {
        for col in 0..cols {
            let tl = row * stride + col;
            let tr = tl + 1;
            let bl = tl + stride;
            let br = bl + 1;
            indices.extend_from_slice(&[tl, bl, tr, tr, bl, br]);
        }
    }
    WaveMeshResult { positions, indices }
}

/// Count vertices in the wave mesh.
#[allow(dead_code)]
pub fn wave_vertex_count(params: &WaveMeshParams) -> usize {
    ((params.cols + 1) * (params.rows + 1)) as usize
}

/// Count triangles in the wave mesh.
#[allow(dead_code)]
pub fn wave_triangle_count(params: &WaveMeshParams) -> usize {
    (params.cols * params.rows * 2) as usize
}

/// Return the minimum and maximum Y of a wave mesh.
#[allow(dead_code)]
pub fn wave_y_range(result: &WaveMeshResult) -> (f32, f32) {
    let mut mn = f32::INFINITY;
    let mut mx = f32::NEG_INFINITY;
    for p in &result.positions {
        if p[1] < mn {
            mn = p[1];
        }
        if p[1] > mx {
            mx = p[1];
        }
    }
    (mn, mx)
}

/// Serialise params to JSON.
#[allow(dead_code)]
pub fn wave_params_to_json(params: &WaveMeshParams) -> String {
    format!(
        "{{\"cols\":{},\"rows\":{},\"amplitude\":{:.4}}}",
        params.cols, params.rows, params.amplitude
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wave_height_zero_amplitude() {
        let p = WaveMeshParams {
            amplitude: 0.0,
            ..Default::default()
        };
        assert!(wave_height(1.0, 1.0, &p).abs() < 1e-6);
    }

    #[test]
    fn test_wave_vertex_count() {
        let p = WaveMeshParams {
            cols: 4,
            rows: 4,
            ..Default::default()
        };
        assert_eq!(wave_vertex_count(&p), 25);
    }

    #[test]
    fn test_wave_triangle_count() {
        let p = WaveMeshParams {
            cols: 4,
            rows: 4,
            ..Default::default()
        };
        assert_eq!(wave_triangle_count(&p), 32);
    }

    #[test]
    fn test_generate_wave_mesh_vertex_count() {
        let p = WaveMeshParams::default();
        let r = generate_wave_mesh(&p);
        assert_eq!(r.positions.len(), wave_vertex_count(&p));
    }

    #[test]
    fn test_generate_wave_mesh_index_count() {
        let p = WaveMeshParams::default();
        let r = generate_wave_mesh(&p);
        assert_eq!(r.indices.len(), wave_triangle_count(&p) * 3);
    }

    #[test]
    fn test_wave_y_range_finite() {
        let r = generate_wave_mesh(&WaveMeshParams::default());
        let (mn, mx) = wave_y_range(&r);
        assert!(mn.is_finite());
        assert!(mx.is_finite());
        assert!(mx >= mn);
    }

    #[test]
    fn test_wave_height_not_nan() {
        let p = WaveMeshParams::default();
        let h = wave_height(0.5, 0.5, &p);
        assert!(!h.is_nan());
    }

    #[test]
    fn test_wave_params_to_json() {
        let p = WaveMeshParams::default();
        let j = wave_params_to_json(&p);
        assert!(j.contains("amplitude"));
    }

    #[test]
    fn test_generate_wave_mesh_indices_in_bounds() {
        let p = WaveMeshParams {
            cols: 2,
            rows: 2,
            ..Default::default()
        };
        let r = generate_wave_mesh(&p);
        let n = r.positions.len() as u32;
        assert!(r.indices.iter().all(|&i| i < n));
    }

    #[test]
    fn test_phase_shift_changes_height() {
        let p0 = WaveMeshParams {
            phase: 0.0,
            ..Default::default()
        };
        let p1 = WaveMeshParams {
            phase: 1.0,
            ..Default::default()
        };
        let h0 = wave_height(0.5, 0.5, &p0);
        let h1 = wave_height(0.5, 0.5, &p1);
        assert!((h0 - h1).abs() > 1e-3);
    }
}
