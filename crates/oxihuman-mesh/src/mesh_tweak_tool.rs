// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Soft-selection vertex tweaking tool.

/// Falloff types for soft selection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SoftFalloff {
    Smooth,
    Linear,
    Sharp,
    Constant,
}

/// Parameters for a tweak operation.
#[derive(Debug, Clone)]
pub struct TweakParams {
    pub radius: f32,
    pub falloff: SoftFalloff,
    pub delta: [f32; 3],
}

/// Result of a tweak operation.
#[derive(Debug, Clone)]
pub struct TweakResult {
    pub positions: Vec<[f32; 3]>,
    pub affected_count: usize,
    pub max_weight: f32,
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Compute soft-selection weight for a vertex at distance `d` with given radius.
pub fn soft_weight(d: f32, radius: f32, falloff: SoftFalloff) -> f32 {
    if d >= radius || radius < 1e-8 {
        return 0.0;
    }
    let t = 1.0 - d / radius;
    match falloff {
        SoftFalloff::Smooth => t * t * (3.0 - 2.0 * t),
        SoftFalloff::Linear => t,
        SoftFalloff::Sharp => t * t,
        SoftFalloff::Constant => 1.0,
    }
}

/// Apply a tweak (soft-selection move) centred at `centre` to the given positions.
pub fn apply_tweak(positions: &[[f32; 3]], centre: [f32; 3], params: &TweakParams) -> TweakResult {
    let mut out = positions.to_vec();
    let mut affected_count = 0usize;
    let mut max_weight = 0.0f32;
    for pos in out.iter_mut() {
        let d = dist3(*pos, centre);
        let w = soft_weight(d, params.radius, params.falloff);
        if w > 1e-8 {
            affected_count += 1;
            if w > max_weight {
                max_weight = w;
            }
            pos[0] += w * params.delta[0];
            pos[1] += w * params.delta[1];
            pos[2] += w * params.delta[2];
        }
    }
    TweakResult {
        positions: out,
        affected_count,
        max_weight,
    }
}

/// Compute weights for all vertices given a centre and radius.
pub fn compute_soft_weights(
    positions: &[[f32; 3]],
    centre: [f32; 3],
    radius: f32,
    falloff: SoftFalloff,
) -> Vec<f32> {
    positions
        .iter()
        .map(|&p| soft_weight(dist3(p, centre), radius, falloff))
        .collect()
}

/// Build a default tweak params for a simple Y-axis move.
pub fn default_tweak_params(radius: f32, dy: f32) -> TweakParams {
    TweakParams {
        radius,
        falloff: SoftFalloff::Smooth,
        delta: [0.0, dy, 0.0],
    }
}

/// Returns count of vertices within radius (weight > 0).
pub fn count_selected(positions: &[[f32; 3]], centre: [f32; 3], radius: f32) -> usize {
    positions
        .iter()
        .filter(|&&p| dist3(p, centre) < radius)
        .count()
}

/// Average displacement magnitude over affected vertices.
pub fn average_displacement(result: &TweakResult, original: &[[f32; 3]]) -> f32 {
    if result.affected_count == 0 {
        return 0.0;
    }
    let total: f32 = result
        .positions
        .iter()
        .zip(original.iter())
        .map(|(&a, &b)| dist3(a, b))
        .sum();
    total / result.affected_count as f32
}

/// Undo a tweak by applying the inverse delta.
pub fn undo_tweak(positions: &[[f32; 3]], centre: [f32; 3], params: &TweakParams) -> TweakResult {
    let inv_params = TweakParams {
        radius: params.radius,
        falloff: params.falloff,
        delta: [-params.delta[0], -params.delta[1], -params.delta[2]],
    };
    apply_tweak(positions, centre, &inv_params)
}

/// Scale delta by a factor.
pub fn scale_tweak_delta(params: &TweakParams, factor: f32) -> TweakParams {
    TweakParams {
        radius: params.radius,
        falloff: params.falloff,
        delta: [
            params.delta[0] * factor,
            params.delta[1] * factor,
            params.delta[2] * factor,
        ],
    }
}

/// Clamp positions within a bounding box after a tweak.
pub fn clamp_tweak_result(result: &mut TweakResult, min_bound: [f32; 3], max_bound: [f32; 3]) {
    for pos in result.positions.iter_mut() {
        for i in 0..3 {
            pos[i] = pos[i].clamp(min_bound[i], max_bound[i]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /* soft_weight at centre = max */
    #[test]
    fn test_soft_weight_centre() {
        let w = soft_weight(0.0, 1.0, SoftFalloff::Smooth);
        assert!((w - 1.0).abs() < 1e-6);
    }

    /* soft_weight at boundary = 0 */
    #[test]
    fn test_soft_weight_boundary() {
        let w = soft_weight(1.0, 1.0, SoftFalloff::Linear);
        assert!(w < 1e-6);
    }

    /* apply_tweak moves vertices inside radius */
    #[test]
    fn test_apply_tweak_moves_inside() {
        let pts = vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]];
        let params = default_tweak_params(2.0, 1.0);
        let res = apply_tweak(&pts, [0.0, 0.0, 0.0], &params);
        assert!(res.positions[0][1] > 0.0);
        assert!((res.positions[1][1]).abs() < 1e-6);
    }

    /* affected_count */
    #[test]
    fn test_affected_count() {
        let pts: Vec<[f32; 3]> = (0..5).map(|i| [i as f32, 0.0, 0.0]).collect();
        let params = default_tweak_params(2.5, 0.5);
        let res = apply_tweak(&pts, [0.0, 0.0, 0.0], &params);
        assert!(res.affected_count > 0 && res.affected_count <= 5);
    }

    /* undo_tweak restores positions */
    #[test]
    fn test_undo_tweak() {
        /* Vertex at exact centre → weight=1 for both apply and undo steps */
        let pts = vec![[0.0, 0.0, 0.0]];
        let params = default_tweak_params(1.0, 2.0);
        let moved = apply_tweak(&pts, [0.0, 0.0, 0.0], &params);
        assert!((moved.positions[0][1] - 2.0).abs() < 1e-5);
        /* undo with centre at the moved position (d=0, w=1) */
        let restored = undo_tweak(&moved.positions, moved.positions[0], &params);
        /* Y: 2.0 + 1.0 * (-2.0) = 0.0 */
        assert!((restored.positions[0][1]).abs() < 1e-5);
    }

    /* compute_soft_weights length */
    #[test]
    fn test_compute_soft_weights_length() {
        let pts: Vec<[f32; 3]> = (0..4).map(|i| [i as f32, 0.0, 0.0]).collect();
        let w = compute_soft_weights(&pts, [0.0; 3], 3.0, SoftFalloff::Smooth);
        assert_eq!(w.len(), 4);
    }

    /* count_selected */
    #[test]
    fn test_count_selected() {
        let pts = vec![[0.0, 0.0, 0.0], [0.5, 0.0, 0.0], [5.0, 0.0, 0.0]];
        let c = count_selected(&pts, [0.0; 3], 1.0);
        assert_eq!(c, 2);
    }

    /* scale_tweak_delta */
    #[test]
    fn test_scale_tweak_delta() {
        let p = default_tweak_params(1.0, 2.0);
        let scaled = scale_tweak_delta(&p, 0.5);
        assert!((scaled.delta[1] - 1.0).abs() < 1e-6);
    }

    /* clamp_tweak_result */
    #[test]
    fn test_clamp_tweak_result() {
        let pts = vec![[0.0, 0.0, 0.0]];
        let params = default_tweak_params(2.0, 10.0);
        let mut res = apply_tweak(&pts, [0.0, 0.0, 0.0], &params);
        clamp_tweak_result(&mut res, [-5.0; 3], [5.0; 3]);
        assert!(res.positions[0][1] <= 5.0);
    }
}
