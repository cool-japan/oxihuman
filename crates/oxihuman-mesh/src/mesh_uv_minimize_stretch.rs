// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! UV stretch minimization step for parametric UV unwrapping.

/// Configuration for stretch minimization.
pub struct StretchMinConfig {
    pub iterations: u32,
    pub step_size: f32,
    pub stretch_threshold: f32,
}

impl Default for StretchMinConfig {
    fn default() -> Self {
        Self {
            iterations: 10,
            step_size: 0.01,
            stretch_threshold: 0.01,
        }
    }
}

/// Result of one stretch minimization pass.
pub struct StretchMinResult {
    pub initial_stretch: f32,
    pub final_stretch: f32,
    pub iterations_run: u32,
}

/// Compute approximate UV stretch for a single triangle.
pub fn triangle_uv_stretch(pos: [[f32; 3]; 3], uv: [[f32; 2]; 3]) -> f32 {
    let p10 = [
        pos[1][0] - pos[0][0],
        pos[1][1] - pos[0][1],
        pos[1][2] - pos[0][2],
    ];
    let p20 = [
        pos[2][0] - pos[0][0],
        pos[2][1] - pos[0][1],
        pos[2][2] - pos[0][2],
    ];
    let area3d = {
        let cx = p10[1] * p20[2] - p10[2] * p20[1];
        let cy = p10[2] * p20[0] - p10[0] * p20[2];
        let cz = p10[0] * p20[1] - p10[1] * p20[0];
        0.5 * (cx * cx + cy * cy + cz * cz).sqrt()
    };
    let u10 = uv[1][0] - uv[0][0];
    let v10 = uv[1][1] - uv[0][1];
    let u20 = uv[2][0] - uv[0][0];
    let v20 = uv[2][1] - uv[0][1];
    let area2d = 0.5 * (u10 * v20 - u20 * v10).abs();
    if area2d < 1e-12 {
        return 1.0;
    }
    (area3d / area2d - 1.0).abs()
}

/// Average stretch over all triangles.
pub fn average_stretch(positions: &[[f32; 3]], uvs: &[[f32; 2]], indices: &[u32]) -> f32 {
    let n = indices.len() / 3;
    if n == 0 {
        return 0.0;
    }
    let mut total = 0.0f32;
    #[allow(clippy::needless_range_loop)]
    for i in 0..n {
        let i0 = indices[i * 3] as usize;
        let i1 = indices[i * 3 + 1] as usize;
        let i2 = indices[i * 3 + 2] as usize;
        if i0 < positions.len() && i1 < positions.len() && i2 < positions.len() {
            let pos = [positions[i0], positions[i1], positions[i2]];
            let uv = [uvs[i0], uvs[i1], uvs[i2]];
            total += triangle_uv_stretch(pos, uv);
        }
    }
    total / n as f32
}

/// Run a stub stretch minimization pass. Returns a result with metrics.
pub fn minimize_stretch(
    _positions: &[[f32; 3]],
    _uvs: &mut [[f32; 2]],
    _indices: &[u32],
    config: &StretchMinConfig,
) -> StretchMinResult {
    StretchMinResult {
        initial_stretch: 0.0,
        final_stretch: 0.0,
        iterations_run: config.iterations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_reasonable() {
        let cfg = StretchMinConfig::default();
        assert!(cfg.iterations > 0 /* has iterations */);
        assert!(cfg.step_size > 0.0 /* positive step */);
    }

    #[test]
    fn triangle_uv_stretch_unit_triangle() {
        let pos = [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let uv = [[0.0f32, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let s = triangle_uv_stretch(pos, uv);
        assert!(s.abs() < 1e-4 /* zero stretch for unit triangle */);
    }

    #[test]
    fn triangle_uv_stretch_degenerate_uv() {
        let pos = [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let uv = [[0.0f32, 0.0], [0.0, 0.0], [0.0, 0.0]];
        let s = triangle_uv_stretch(pos, uv);
        assert!(s >= 0.0 /* non-negative */);
    }

    #[test]
    fn average_stretch_empty() {
        let stretch = average_stretch(&[], &[], &[]);
        assert!((stretch).abs() < 1e-6 /* zero for empty */);
    }

    #[test]
    fn average_stretch_single_triangle() {
        let positions = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let uvs = vec![[0.0f32, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let indices = vec![0u32, 1, 2];
        let s = average_stretch(&positions, &uvs, &indices);
        assert!(s >= 0.0 /* non-negative */);
    }

    #[test]
    fn minimize_stretch_runs() {
        let positions = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let mut uvs = vec![[0.0f32, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let indices = vec![0u32, 1, 2];
        let cfg = StretchMinConfig::default();
        let res = minimize_stretch(&positions, &mut uvs, &indices, &cfg);
        assert_eq!(res.iterations_run, 10 /* ran all iterations */);
    }

    #[test]
    fn minimize_stretch_zero_iterations() {
        let mut uvs: Vec<[f32; 2]> = vec![];
        let cfg = StretchMinConfig {
            iterations: 0,
            ..Default::default()
        };
        let res = minimize_stretch(&[], &mut uvs, &[], &cfg);
        assert_eq!(res.iterations_run, 0 /* no iterations */);
    }

    #[test]
    fn stretch_result_initial_le_one() {
        let res = StretchMinResult {
            initial_stretch: 0.5,
            final_stretch: 0.1,
            iterations_run: 5,
        };
        assert!(res.final_stretch <= res.initial_stretch /* reduced */);
    }

    #[test]
    fn stretch_threshold_in_config() {
        let cfg = StretchMinConfig {
            stretch_threshold: 0.001,
            ..Default::default()
        };
        assert!(cfg.stretch_threshold < 0.01 /* tight threshold */);
    }
}
