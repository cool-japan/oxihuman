#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::f32::consts::PI;

/// Apply a sine-wave deformation along the given axis (0=X, 1=Y, 2=Z).
#[allow(dead_code)]
pub fn sine_wave_morph(
    verts: &[[f32; 3]],
    axis: u8,
    freq: f32,
    amplitude: f32,
) -> Vec<[f32; 3]> {
    verts
        .iter()
        .map(|&v| {
            let t = v[axis.min(2) as usize];
            let offset = amplitude * (2.0 * PI * freq * t).sin();
            let mut out = v;
            out[axis.min(2) as usize] += offset;
            out
        })
        .collect()
}

/// Apply a radial deformation away from a center point.
#[allow(dead_code)]
pub fn radial_morph(verts: &[[f32; 3]], center: [f32; 3], strength: f32) -> Vec<[f32; 3]> {
    verts
        .iter()
        .map(|&v| {
            let dx = v[0] - center[0];
            let dy = v[1] - center[1];
            let dz = v[2] - center[2];
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            if dist < 1e-9 {
                return v;
            }
            let scale = strength / dist;
            [v[0] + dx * scale, v[1] + dy * scale, v[2] + dz * scale]
        })
        .collect()
}

/// Displace all vertices by `amount` along a direction vector.
#[allow(dead_code)]
pub fn directional_morph(verts: &[[f32; 3]], direction: [f32; 3], amount: f32) -> Vec<[f32; 3]> {
    let len = (direction[0] * direction[0]
        + direction[1] * direction[1]
        + direction[2] * direction[2])
        .sqrt();
    let norm = if len > 1e-9 {
        [
            direction[0] / len,
            direction[1] / len,
            direction[2] / len,
        ]
    } else {
        [0.0, 0.0, 0.0]
    };
    verts
        .iter()
        .map(|&v| {
            [
                v[0] + norm[0] * amount,
                v[1] + norm[1] * amount,
                v[2] + norm[2] * amount,
            ]
        })
        .collect()
}

/// Scale all vertices by a per-axis scale.
#[allow(dead_code)]
pub fn scale_morph(verts: &[[f32; 3]], scale: [f32; 3]) -> Vec<[f32; 3]> {
    verts
        .iter()
        .map(|&v| [v[0] * scale[0], v[1] * scale[1], v[2] * scale[2]])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sine_wave_empty() {
        let out = sine_wave_morph(&[], 0, 1.0, 0.1);
        assert!(out.is_empty());
    }

    #[test]
    fn test_sine_wave_zero_amplitude() {
        let verts = vec![[1.0f32, 0.0, 0.0]];
        let out = sine_wave_morph(&verts, 0, 1.0, 0.0);
        assert!((out[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_radial_morph_empty() {
        let out = radial_morph(&[], [0.0, 0.0, 0.0], 1.0);
        assert!(out.is_empty());
    }

    #[test]
    fn test_radial_morph_moves_away() {
        let verts = vec![[1.0f32, 0.0, 0.0]];
        let out = radial_morph(&verts, [0.0, 0.0, 0.0], 1.0);
        assert!(out[0][0] > 1.0);
    }

    #[test]
    fn test_directional_morph_empty() {
        let out = directional_morph(&[], [0.0, 1.0, 0.0], 1.0);
        assert!(out.is_empty());
    }

    #[test]
    fn test_directional_morph_y_axis() {
        let verts = vec![[0.0f32, 0.0, 0.0]];
        let out = directional_morph(&verts, [0.0, 1.0, 0.0], 2.0);
        assert!((out[0][1] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_scale_morph_empty() {
        let out = scale_morph(&[], [2.0, 2.0, 2.0]);
        assert!(out.is_empty());
    }

    #[test]
    fn test_scale_morph_doubles() {
        let verts = vec![[1.0f32, 2.0, 3.0]];
        let out = scale_morph(&verts, [2.0, 2.0, 2.0]);
        assert!((out[0][0] - 2.0).abs() < 1e-5);
        assert!((out[0][1] - 4.0).abs() < 1e-5);
        assert!((out[0][2] - 6.0).abs() < 1e-5);
    }

    #[test]
    fn test_scale_morph_identity() {
        let verts = vec![[3.0f32, 4.0, 5.0]];
        let out = scale_morph(&verts, [1.0, 1.0, 1.0]);
        assert!((out[0][0] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_directional_zero_direction() {
        let verts = vec![[1.0f32, 1.0, 1.0]];
        let out = directional_morph(&verts, [0.0, 0.0, 0.0], 5.0);
        // Should not move
        assert!((out[0][0] - 1.0).abs() < 1e-5);
    }
}
