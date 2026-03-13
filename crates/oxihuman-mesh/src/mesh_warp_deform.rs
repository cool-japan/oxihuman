// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Warp-based mesh deformation using radial basis functions.

/// A handle point: original position and target position.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WarpHandle2 {
    pub origin: [f32; 3],
    pub target: [f32; 3],
}

/// Configuration for warp deformation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WarpDeformConfig {
    pub radius: f32,
    pub falloff_power: f32,
}

impl Default for WarpDeformConfig {
    fn default() -> Self {
        Self {
            radius: 1.0,
            falloff_power: 2.0,
        }
    }
}

/// Euclidean distance between two 3-D points.
#[allow(dead_code)]
pub fn dist3_wd(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Compute the RBF weight for a given distance and radius.
#[allow(dead_code)]
pub fn rbf_weight(dist: f32, radius: f32, falloff_power: f32) -> f32 {
    if radius <= 0.0 || dist >= radius {
        return 0.0;
    }
    let t = 1.0 - dist / radius;
    t.powf(falloff_power)
}

/// Apply warp deformation to a set of positions.
#[allow(dead_code)]
pub fn apply_warp_deform(
    positions: &[[f32; 3]],
    handles: &[WarpHandle2],
    config: &WarpDeformConfig,
) -> Vec<[f32; 3]> {
    positions
        .iter()
        .map(|&p| {
            let mut delta = [0.0f32; 3];
            let mut total_w = 0.0f32;
            for h in handles {
                let d = dist3_wd(p, h.origin);
                let w = rbf_weight(d, config.radius, config.falloff_power);
                let disp = [
                    h.target[0] - h.origin[0],
                    h.target[1] - h.origin[1],
                    h.target[2] - h.origin[2],
                ];
                delta[0] += w * disp[0];
                delta[1] += w * disp[1];
                delta[2] += w * disp[2];
                total_w += w;
            }
            if total_w > 0.0 {
                [p[0] + delta[0], p[1] + delta[1], p[2] + delta[2]]
            } else {
                p
            }
        })
        .collect()
}

/// Compute the average displacement magnitude of a warp result.
#[allow(dead_code)]
pub fn avg_warp_displacement(original: &[[f32; 3]], warped: &[[f32; 3]]) -> f32 {
    let n = original.len().min(warped.len());
    if n == 0 {
        return 0.0;
    }
    original
        .iter()
        .zip(warped.iter())
        .map(|(a, b)| dist3_wd(*a, *b))
        .sum::<f32>()
        / n as f32
}

/// Count handles that produce non-zero displacement.
#[allow(dead_code)]
pub fn active_handle_count(handles: &[WarpHandle2]) -> usize {
    handles
        .iter()
        .filter(|h| dist3_wd(h.origin, h.target) > 1e-8)
        .count()
}

/// Serialise config to JSON.
#[allow(dead_code)]
pub fn warp_config_to_json(config: &WarpDeformConfig) -> String {
    format!(
        "{{\"radius\":{:.4},\"falloff_power\":{:.4}}}",
        config.radius, config.falloff_power
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dist3_wd_basic() {
        let d = dist3_wd([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!((d - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_rbf_weight_at_origin() {
        let w = rbf_weight(0.0, 1.0, 2.0);
        assert!((w - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_rbf_weight_at_radius() {
        let w = rbf_weight(1.0, 1.0, 2.0);
        assert!(w.abs() < 1e-6);
    }

    #[test]
    fn test_rbf_weight_beyond_radius() {
        let w = rbf_weight(2.0, 1.0, 2.0);
        assert!(w.abs() < 1e-6);
    }

    #[test]
    fn test_apply_warp_no_handles() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let cfg = WarpDeformConfig::default();
        let result = apply_warp_deform(&pos, &[], &cfg);
        assert_eq!(result[0], pos[0]);
    }

    #[test]
    fn test_apply_warp_moves_position() {
        let pos = vec![[0.0, 0.0, 0.0]];
        let handles = vec![WarpHandle2 {
            origin: [0.0, 0.0, 0.0],
            target: [1.0, 0.0, 0.0],
        }];
        let cfg = WarpDeformConfig {
            radius: 2.0,
            falloff_power: 1.0,
        };
        let result = apply_warp_deform(&pos, &handles, &cfg);
        assert!(result[0][0] > 0.5);
    }

    #[test]
    fn test_avg_warp_displacement_zero() {
        let pos = vec![[0.0, 0.0, 0.0]];
        let d = avg_warp_displacement(&pos, &pos);
        assert!(d.abs() < 1e-6);
    }

    #[test]
    fn test_active_handle_count() {
        let handles = vec![
            WarpHandle2 {
                origin: [0.0, 0.0, 0.0],
                target: [1.0, 0.0, 0.0],
            },
            WarpHandle2 {
                origin: [1.0, 0.0, 0.0],
                target: [1.0, 0.0, 0.0],
            },
        ];
        assert_eq!(active_handle_count(&handles), 1);
    }

    #[test]
    fn test_warp_config_to_json() {
        let cfg = WarpDeformConfig::default();
        let j = warp_config_to_json(&cfg);
        assert!(j.contains("radius"));
    }

    #[test]
    fn test_empty_positions() {
        let cfg = WarpDeformConfig::default();
        let result = apply_warp_deform(&[], &[], &cfg);
        assert!(result.is_empty());
    }
}
