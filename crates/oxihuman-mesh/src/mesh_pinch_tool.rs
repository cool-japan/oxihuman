// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Vertex pinch/squeeze tool.

/// Result of a pinch operation.
#[derive(Debug, Clone)]
pub struct PinchResult {
    pub positions: Vec<[f32; 3]>,
    pub affected_count: usize,
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Smooth falloff weight.
fn smooth_weight(d: f32, radius: f32) -> f32 {
    if d >= radius || radius < 1e-8 {
        return 0.0;
    }
    let t = 1.0 - d / radius;
    t * t * (3.0 - 2.0 * t)
}

/// Apply a pinch: move vertices toward `target` by `strength` * falloff.
pub fn apply_pinch(
    positions: &[[f32; 3]],
    centre: [f32; 3],
    target: [f32; 3],
    radius: f32,
    strength: f32,
) -> PinchResult {
    let mut out = positions.to_vec();
    let mut affected_count = 0usize;
    for pos in out.iter_mut() {
        let d = dist3(*pos, centre);
        let w = smooth_weight(d, radius) * strength;
        if w < 1e-8 {
            continue;
        }
        pos[0] += (target[0] - pos[0]) * w;
        pos[1] += (target[1] - pos[1]) * w;
        pos[2] += (target[2] - pos[2]) * w;
        affected_count += 1;
    }
    PinchResult {
        positions: out,
        affected_count,
    }
}

/// Pinch toward a line (axis squeeze).
pub fn pinch_to_line(
    positions: &[[f32; 3]],
    line_point: [f32; 3],
    line_dir: [f32; 3],
    radius: f32,
    strength: f32,
) -> PinchResult {
    let mut out = positions.to_vec();
    let mut affected_count = 0usize;
    let len2 = line_dir[0] * line_dir[0] + line_dir[1] * line_dir[1] + line_dir[2] * line_dir[2];
    if len2 < 1e-10 {
        return PinchResult {
            positions: out,
            affected_count,
        };
    }
    for pos in out.iter_mut() {
        let v = [
            pos[0] - line_point[0],
            pos[1] - line_point[1],
            pos[2] - line_point[2],
        ];
        let t = (v[0] * line_dir[0] + v[1] * line_dir[1] + v[2] * line_dir[2]) / len2;
        let closest = [
            line_point[0] + t * line_dir[0],
            line_point[1] + t * line_dir[1],
            line_point[2] + t * line_dir[2],
        ];
        let d = dist3(*pos, closest);
        let w = smooth_weight(d, radius) * strength;
        if w < 1e-8 {
            continue;
        }
        pos[0] += (closest[0] - pos[0]) * w;
        pos[1] += (closest[1] - pos[1]) * w;
        pos[2] += (closest[2] - pos[2]) * w;
        affected_count += 1;
    }
    PinchResult {
        positions: out,
        affected_count,
    }
}

/// Expand (reverse pinch): push vertices away from target.
pub fn apply_expand(
    positions: &[[f32; 3]],
    centre: [f32; 3],
    target: [f32; 3],
    radius: f32,
    strength: f32,
) -> PinchResult {
    apply_pinch(positions, centre, target, radius, -strength)
}

/// Compute the centroid of affected vertices.
pub fn affected_centroid(positions: &[[f32; 3]], centre: [f32; 3], radius: f32) -> [f32; 3] {
    let mut sum = [0.0f32; 3];
    let mut count = 0usize;
    for &pos in positions {
        if dist3(pos, centre) < radius {
            sum[0] += pos[0];
            sum[1] += pos[1];
            sum[2] += pos[2];
            count += 1;
        }
    }
    if count == 0 {
        return centre;
    }
    let n = count as f32;
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

/// Count vertices within the pinch radius.
pub fn count_in_radius(positions: &[[f32; 3]], centre: [f32; 3], radius: f32) -> usize {
    positions
        .iter()
        .filter(|&&p| dist3(p, centre) < radius)
        .count()
}

/// Compute the maximum displacement from a pinch result compared to original.
pub fn max_pinch_displacement(original: &[[f32; 3]], result: &PinchResult) -> f32 {
    original
        .iter()
        .zip(result.positions.iter())
        .map(|(&a, &b)| dist3(a, b))
        .fold(0.0f32, f32::max)
}

/// Default strength for a pinch tool.
pub fn default_pinch_strength() -> f32 {
    0.5
}

/// Build a sequence of progressive pinch steps.
pub fn pinch_steps(
    positions: &[[f32; 3]],
    centre: [f32; 3],
    target: [f32; 3],
    radius: f32,
    total_strength: f32,
    steps: usize,
) -> Vec<Vec<[f32; 3]>> {
    let step_strength = total_strength / steps.max(1) as f32;
    let mut current = positions.to_vec();
    let mut frames = Vec::with_capacity(steps);
    for _ in 0..steps {
        let res = apply_pinch(&current, centre, target, radius, step_strength);
        current = res.positions.clone();
        frames.push(current.clone());
    }
    frames
}

#[cfg(test)]
mod tests {
    use super::*;

    /* pinch moves vertices toward target */
    #[test]
    fn test_apply_pinch_moves_toward_target() {
        let pts = vec![[1.0, 0.0, 0.0], [0.5, 0.0, 0.0]];
        let res = apply_pinch(&pts, [0.0; 3], [0.0; 3], 2.0, 1.0);
        assert!(res.positions[0][0] < pts[0][0]);
    }

    /* expand moves away */
    #[test]
    fn test_apply_expand_moves_away() {
        let pts = vec![[0.5, 0.0, 0.0]];
        let res = apply_expand(&pts, [0.0; 3], [0.0; 3], 2.0, 0.5);
        assert!(res.positions[0][0] > 0.5 - 1e-6);
    }

    /* count_in_radius */
    #[test]
    fn test_count_in_radius() {
        let pts = vec![[0.0; 3], [0.5, 0.0, 0.0], [5.0, 0.0, 0.0]];
        assert_eq!(count_in_radius(&pts, [0.0; 3], 1.0), 2);
    }

    /* affected_centroid */
    #[test]
    fn test_affected_centroid() {
        let pts = vec![[0.0; 3], [2.0, 0.0, 0.0]];
        let c = affected_centroid(&pts, [0.0; 3], 5.0);
        assert!((c[0] - 1.0).abs() < 1e-5);
    }

    /* affected_count */
    #[test]
    fn test_affected_count() {
        let pts = vec![[0.0; 3], [10.0, 0.0, 0.0]];
        let res = apply_pinch(&pts, [0.0; 3], [0.0; 3], 1.0, 0.5);
        assert!(res.affected_count < 2);
    }

    /* default_pinch_strength */
    #[test]
    fn test_default_pinch_strength() {
        assert!((default_pinch_strength() - 0.5).abs() < 1e-6);
    }

    /* pinch_to_line */
    #[test]
    fn test_pinch_to_line_count() {
        let pts: Vec<[f32; 3]> = (0..4).map(|i| [i as f32, 0.5, 0.0]).collect();
        let res = pinch_to_line(&pts, [0.0; 3], [1.0, 0.0, 0.0], 2.0, 0.5);
        assert!(res.affected_count > 0);
    }

    /* max_pinch_displacement */
    #[test]
    fn test_max_pinch_displacement() {
        let pts = vec![[1.0, 0.0, 0.0]];
        let res = apply_pinch(&pts, [0.0; 3], [0.0; 3], 2.0, 0.8);
        let d = max_pinch_displacement(&pts, &res);
        assert!(d > 0.0);
    }

    /* pinch_steps count */
    #[test]
    fn test_pinch_steps_count() {
        let pts = vec![[1.0, 0.0, 0.0]];
        let frames = pinch_steps(&pts, [0.0; 3], [0.0; 3], 2.0, 0.8, 4);
        assert_eq!(frames.len(), 4);
    }
}
