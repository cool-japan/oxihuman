// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Moving Least Squares (MLS) mesh deformation.
#[allow(dead_code)]
pub struct MlsHandle {
    pub source: [f32; 3],
    pub target: [f32; 3],
    pub weight: f32,
}

#[allow(dead_code)]
pub struct MlsConfig {
    pub alpha: f32,
    pub falloff_radius: f32,
}

#[allow(dead_code)]
pub fn default_mls_config() -> MlsConfig {
    MlsConfig {
        alpha: 2.0,
        falloff_radius: 1.0,
    }
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn wij(q: [f32; 3], pi: [f32; 3], alpha: f32) -> f32 {
    let d = dist3(q, pi);
    if d < 1e-8 {
        return 1e8;
    }
    1.0 / d.powf(alpha * 2.0)
}

/// Apply MLS rigid deformation to a set of query points.
#[allow(dead_code)]
pub fn mls_deform(positions: &[[f32; 3]], handles: &[MlsHandle], cfg: &MlsConfig) -> Vec<[f32; 3]> {
    if handles.is_empty() {
        return positions.to_vec();
    }

    positions
        .iter()
        .map(|&q| {
            let weights: Vec<f32> = handles
                .iter()
                .map(|h| wij(q, h.source, cfg.alpha) * h.weight)
                .collect();
            let w_sum: f32 = weights.iter().sum();
            if w_sum < 1e-10 {
                return q;
            }

            // Weighted centroid of sources
            let mut p_star = [0.0f32; 3];
            let mut q_star = [0.0f32; 3];
            for (h, &w) in handles.iter().zip(weights.iter()) {
                for k in 0..3 {
                    p_star[k] += w * h.source[k];
                    q_star[k] += w * h.target[k];
                }
            }
            for k in 0..3 {
                p_star[k] /= w_sum;
                q_star[k] /= w_sum;
            }

            // Simple translation-only approximation
            let delta = [
                q_star[0] - p_star[0],
                q_star[1] - p_star[1],
                q_star[2] - p_star[2],
            ];

            // Blend influence by proximity
            let total_w: f32 = handles.iter().map(|h| wij(q, h.source, cfg.alpha)).sum();
            let blend = if total_w < 1e-10 {
                0.0
            } else {
                let max_w = handles
                    .iter()
                    .map(|h| wij(q, h.source, cfg.alpha))
                    .fold(0.0f32, f32::max);
                (max_w / (total_w + 1e-10)).min(1.0)
            };

            [
                q[0] + delta[0] * blend,
                q[1] + delta[1] * blend,
                q[2] + delta[2] * blend,
            ]
        })
        .collect()
}

#[allow(dead_code)]
pub fn mls_displacement_magnitude(original: &[[f32; 3]], deformed: &[[f32; 3]]) -> f32 {
    original
        .iter()
        .zip(deformed.iter())
        .map(|(&a, &b)| dist3(a, b))
        .fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn mls_avg_displacement(original: &[[f32; 3]], deformed: &[[f32; 3]]) -> f32 {
    let n = original.len().min(deformed.len());
    if n == 0 {
        return 0.0;
    }
    let sum: f32 = original
        .iter()
        .zip(deformed.iter())
        .map(|(&a, &b)| dist3(a, b))
        .sum();
    sum / n as f32
}

#[allow(dead_code)]
pub fn mls_result_to_json(deformed: &[[f32; 3]]) -> String {
    format!("{{\"vertex_count\":{}}}", deformed.len())
}

#[allow(dead_code)]
pub fn handles_valid(handles: &[MlsHandle]) -> bool {
    handles
        .iter()
        .all(|h| (0.0..=1.0).contains(&h.weight) && h.weight.is_finite())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_grid() -> Vec<[f32; 3]> {
        let mut pts = vec![];
        for i in 0..4 {
            for j in 0..4 {
                pts.push([i as f32, 0.0, j as f32]);
            }
        }
        pts
    }

    fn one_handle() -> Vec<MlsHandle> {
        vec![MlsHandle {
            source: [0.0, 0.0, 0.0],
            target: [0.5, 0.5, 0.0],
            weight: 1.0,
        }]
    }

    #[test]
    fn test_no_handles_no_change() {
        let grid = flat_grid();
        let cfg = default_mls_config();
        let deformed = mls_deform(&grid, &[], &cfg);
        assert_eq!(deformed, grid);
    }

    #[test]
    fn test_deform_vertex_count() {
        let grid = flat_grid();
        let cfg = default_mls_config();
        let deformed = mls_deform(&grid, &one_handle(), &cfg);
        assert_eq!(deformed.len(), grid.len());
    }

    #[test]
    fn test_deform_moves_nearby_verts() {
        let grid = flat_grid();
        let cfg = default_mls_config();
        let deformed = mls_deform(&grid, &one_handle(), &cfg);
        let max_d = mls_displacement_magnitude(&grid, &deformed);
        assert!(max_d > 0.0);
    }

    #[test]
    fn test_empty_positions() {
        let cfg = default_mls_config();
        let d = mls_deform(&[], &one_handle(), &cfg);
        assert_eq!(d.len(), 0);
    }

    #[test]
    fn test_avg_displacement_zero_no_handles() {
        let grid = flat_grid();
        let cfg = default_mls_config();
        let d = mls_deform(&grid, &[], &cfg);
        assert!((mls_avg_displacement(&grid, &d)).abs() < 1e-6);
    }

    #[test]
    fn test_deformed_positions_finite() {
        let grid = flat_grid();
        let cfg = default_mls_config();
        let d = mls_deform(&grid, &one_handle(), &cfg);
        for p in &d {
            assert!(p.iter().all(|v| v.is_finite()));
        }
    }

    #[test]
    fn test_handles_valid_true() {
        assert!(handles_valid(&one_handle()));
    }

    #[test]
    fn test_handles_valid_false() {
        let bad = vec![MlsHandle {
            source: [0.0; 3],
            target: [0.0; 3],
            weight: 2.0,
        }];
        assert!(!handles_valid(&bad));
    }

    #[test]
    fn test_result_to_json() {
        let grid = flat_grid();
        let j = mls_result_to_json(&grid);
        assert!(j.contains("vertex_count"));
    }

    #[test]
    fn test_default_config() {
        let cfg = default_mls_config();
        assert!(cfg.alpha > 0.0);
        assert!(cfg.falloff_radius > 0.0);
    }
}
