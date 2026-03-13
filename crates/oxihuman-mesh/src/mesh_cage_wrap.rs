// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cage-wrap deformation: offset a high-res mesh using cage displacement.

/// Configuration for cage-wrap.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CageWrapConfig {
    pub influence_radius: f32,
    pub smooth_iterations: u32,
}

/// Result of a cage-wrap operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CageWrapResult {
    pub positions: Vec<[f32; 3]>,
    pub max_displacement: f32,
}

#[allow(dead_code)]
pub fn default_cage_wrap_config() -> CageWrapConfig {
    CageWrapConfig {
        influence_radius: 0.5,
        smooth_iterations: 2,
    }
}

#[inline]
fn dist_sq3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

/// Apply cage-wrap: displace `target_positions` based on cage displacement.
#[allow(dead_code)]
pub fn cage_wrap(
    target_positions: &[[f32; 3]],
    cage_rest: &[[f32; 3]],
    cage_deformed: &[[f32; 3]],
    config: &CageWrapConfig,
) -> CageWrapResult {
    assert_eq!(cage_rest.len(), cage_deformed.len());
    let r2 = config.influence_radius * config.influence_radius;
    let mut out = Vec::with_capacity(target_positions.len());
    let mut max_disp = 0.0_f32;
    for &tp in target_positions {
        let mut wsum = 0.0_f32;
        let mut dx = 0.0_f32;
        let mut dy = 0.0_f32;
        let mut dz = 0.0_f32;
        for (cr, cd) in cage_rest.iter().zip(cage_deformed.iter()) {
            let d2 = dist_sq3(tp, *cr);
            if d2 < r2 {
                let w = 1.0 - d2 / r2;
                wsum += w;
                dx += w * (cd[0] - cr[0]);
                dy += w * (cd[1] - cr[1]);
                dz += w * (cd[2] - cr[2]);
            }
        }
        let disp = if wsum > 0.0 {
            [dx / wsum, dy / wsum, dz / wsum]
        } else {
            [0.0; 3]
        };
        let mag = (disp[0] * disp[0] + disp[1] * disp[1] + disp[2] * disp[2]).sqrt();
        if mag > max_disp {
            max_disp = mag;
        }
        out.push([tp[0] + disp[0], tp[1] + disp[1], tp[2] + disp[2]]);
    }
    CageWrapResult {
        positions: out,
        max_displacement: max_disp,
    }
}

/// Vertex count of wrapped mesh.
#[allow(dead_code)]
pub fn wrapped_vertex_count(result: &CageWrapResult) -> usize {
    result.positions.len()
}

/// Average position of wrapped mesh.
#[allow(dead_code)]
pub fn wrapped_centroid(result: &CageWrapResult) -> [f32; 3] {
    if result.positions.is_empty() {
        return [0.0; 3];
    }
    let n = result.positions.len() as f32;
    let mut sum = [0.0_f32; 3];
    for p in &result.positions {
        sum[0] += p[0];
        sum[1] += p[1];
        sum[2] += p[2];
    }
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

/// Serialise result to JSON string.
#[allow(dead_code)]
pub fn cage_wrap_to_json(result: &CageWrapResult) -> String {
    format!(
        "{{\"vertex_count\":{},\"max_displacement\":{}}}",
        result.positions.len(),
        result.max_displacement
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_displacement_when_cage_same() {
        let target = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let cage_rest = vec![[0.0, 0.0, 0.0]];
        let cage_def = vec![[0.0, 0.0, 0.0]];
        let cfg = default_cage_wrap_config();
        let res = cage_wrap(&target, &cage_rest, &cage_def, &cfg);
        assert!((res.max_displacement).abs() < 1e-5);
    }

    #[test]
    fn displacement_applied() {
        let target = vec![[0.0, 0.0, 0.0]];
        let cage_rest = vec![[0.0, 0.0, 0.0]];
        let cage_def = vec![[0.0, 1.0, 0.0]];
        let cfg = CageWrapConfig {
            influence_radius: 1.0,
            smooth_iterations: 0,
        };
        let res = cage_wrap(&target, &cage_rest, &cage_def, &cfg);
        assert!((res.positions[0][1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn vertex_count_preserved() {
        let target = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let cage_rest = vec![[0.5, 0.0, 0.0]];
        let cage_def = vec![[0.5, 0.5, 0.0]];
        let cfg = default_cage_wrap_config();
        let res = cage_wrap(&target, &cage_rest, &cage_def, &cfg);
        assert_eq!(wrapped_vertex_count(&res), 3);
    }

    #[test]
    fn max_displacement_nonneg() {
        let target = vec![[0.0, 0.0, 0.0]];
        let cage_rest = vec![[0.0, 0.0, 0.0]];
        let cage_def = vec![[1.0, 0.0, 0.0]];
        let cfg = CageWrapConfig {
            influence_radius: 2.0,
            smooth_iterations: 0,
        };
        let res = cage_wrap(&target, &cage_rest, &cage_def, &cfg);
        assert!(res.max_displacement >= 0.0);
    }

    #[test]
    fn no_influence_outside_radius() {
        let target = vec![[10.0, 0.0, 0.0]];
        let cage_rest = vec![[0.0, 0.0, 0.0]];
        let cage_def = vec![[0.0, 1.0, 0.0]];
        let cfg = CageWrapConfig {
            influence_radius: 0.1,
            smooth_iterations: 0,
        };
        let res = cage_wrap(&target, &cage_rest, &cage_def, &cfg);
        assert!((res.positions[0][1]).abs() < 1e-5);
    }

    #[test]
    fn centroid_reasonable() {
        let target = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let cage_rest = vec![[1.0, 0.0, 0.0]];
        let cage_def = vec![[1.0, 0.0, 0.0]];
        let cfg = default_cage_wrap_config();
        let res = cage_wrap(&target, &cage_rest, &cage_def, &cfg);
        let c = wrapped_centroid(&res);
        assert!((c[0] - 1.0).abs() < 0.1);
    }

    #[test]
    fn json_contains_vertex_count() {
        let res = CageWrapResult {
            positions: vec![[0.0; 3]; 5],
            max_displacement: 0.1,
        };
        let j = cage_wrap_to_json(&res);
        assert!(j.contains("5"));
    }

    #[test]
    fn default_config_radius_positive() {
        let cfg = default_cage_wrap_config();
        assert!(cfg.influence_radius > 0.0);
    }

    #[test]
    fn empty_target_returns_empty() {
        let res = cage_wrap(
            &[],
            &[[0.0; 3]],
            &[[1.0, 0.0, 0.0]],
            &default_cage_wrap_config(),
        );
        assert!(res.positions.is_empty());
    }

    #[test]
    fn contains_check() {
        let v = 0.5_f32;
        assert!((0.0..=1.0).contains(&v));
    }
}
