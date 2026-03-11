// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Ankle joint morph control — thickness, rotation-based correctives, and bone protrusion.

use std::f32::consts::PI;

/// Ankle morph parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct AnkleParams {
    /// Overall thickness multiplier (0.5 = thin, 1.5 = thick).
    pub thickness: f32,
    /// Lateral malleolus prominence (outer ankle bone), 0..=1.
    pub lateral_prominence: f32,
    /// Medial malleolus prominence (inner ankle bone), 0..=1.
    pub medial_prominence: f32,
    /// Achilles tendon definition, 0..=1.
    pub achilles_definition: f32,
    /// Dorsiflexion angle in radians (foot up).
    pub dorsiflexion: f32,
    /// Plantarflexion angle in radians (foot down).
    pub plantarflexion: f32,
}

impl Default for AnkleParams {
    fn default() -> Self {
        Self {
            thickness: 1.0,
            lateral_prominence: 0.5,
            medial_prominence: 0.5,
            achilles_definition: 0.3,
            dorsiflexion: 0.0,
            plantarflexion: 0.0,
        }
    }
}

/// Corrective weight for ankle rotation.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct AnkleCorrective {
    pub dorsi_weight: f32,
    pub plantar_weight: f32,
    pub inversion_weight: f32,
    pub eversion_weight: f32,
}

/// Compute the corrective blend weight based on joint angle.
///
/// Uses a smooth hermite ramp from `start_angle` to `end_angle`.
#[allow(dead_code)]
pub fn corrective_weight(angle: f32, start_angle: f32, end_angle: f32) -> f32 {
    if (end_angle - start_angle).abs() < 1e-6 {
        return 0.0;
    }
    let t = ((angle - start_angle) / (end_angle - start_angle)).clamp(0.0, 1.0);
    // Smoothstep
    t * t * (3.0 - 2.0 * t)
}

/// Compute dorsiflexion corrective weight.
#[allow(dead_code)]
pub fn dorsiflexion_corrective(angle: f32) -> f32 {
    corrective_weight(angle, 0.0, PI / 6.0)
}

/// Compute plantarflexion corrective weight.
#[allow(dead_code)]
pub fn plantarflexion_corrective(angle: f32) -> f32 {
    corrective_weight(-angle, 0.0, PI / 4.0)
}

/// Thickness profile: radial scaling around ankle centre.
#[allow(dead_code)]
pub fn thickness_scale(base_radius: f32, thickness_param: f32) -> f32 {
    base_radius * thickness_param.max(0.1)
}

/// Malleolus bump profile — raised cosine on angle around ankle axis.
#[allow(dead_code)]
pub fn malleolus_bump(
    angle_around_axis: f32,
    centre_angle: f32,
    spread: f32,
    prominence: f32,
) -> f32 {
    if spread <= 0.0 {
        return 0.0;
    }
    let diff = (angle_around_axis - centre_angle).abs();
    let diff = if diff > PI { 2.0 * PI - diff } else { diff };
    let t = (diff / spread).clamp(0.0, 1.0);
    prominence * 0.5 * (1.0 + (PI * t).cos())
}

/// Achilles tendon groove depth.
#[allow(dead_code)]
pub fn achilles_groove(definition: f32) -> f32 {
    let definition = definition.clamp(0.0, 1.0);
    -0.005 * definition
}

/// Evaluate full ankle morph returning per-vertex weights.
#[allow(dead_code)]
pub fn evaluate_ankle_morph(
    positions: &[[f32; 3]],
    ankle_centre: [f32; 3],
    params: &AnkleParams,
) -> Vec<(usize, [f32; 3])> {
    let mut deltas = Vec::new();

    for (i, pos) in positions.iter().enumerate() {
        let dx = pos[0] - ankle_centre[0];
        let dy = pos[1] - ankle_centre[1];
        let dz = pos[2] - ankle_centre[2];

        let dist_xz = (dx * dx + dz * dz).sqrt();
        if dist_xz < 1e-6 {
            continue;
        }

        let angle = dz.atan2(dx);

        // Lateral malleolus at ~0 rad, medial at ~PI
        let lat_bump = malleolus_bump(angle, 0.0, 0.5, params.lateral_prominence);
        let med_bump = malleolus_bump(angle, PI, 0.5, params.medial_prominence);

        // Radial displacement
        let radial = (params.thickness - 1.0) * 0.01 + (lat_bump + med_bump) * 0.005;

        // Height falloff
        let height_falloff = (-dy * dy * 100.0).exp();

        let scale = radial * height_falloff;
        if scale.abs() > 1e-6 {
            let nx = dx / dist_xz;
            let nz = dz / dist_xz;
            deltas.push((i, [nx * scale, 0.0, nz * scale]));
        }
    }

    deltas
}

/// Blend two AnkleParams by `t`.
#[allow(dead_code)]
pub fn blend_ankle_params(a: &AnkleParams, b: &AnkleParams, t: f32) -> AnkleParams {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    AnkleParams {
        thickness: a.thickness * inv + b.thickness * t,
        lateral_prominence: a.lateral_prominence * inv + b.lateral_prominence * t,
        medial_prominence: a.medial_prominence * inv + b.medial_prominence * t,
        achilles_definition: a.achilles_definition * inv + b.achilles_definition * t,
        dorsiflexion: a.dorsiflexion * inv + b.dorsiflexion * t,
        plantarflexion: a.plantarflexion * inv + b.plantarflexion * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default_params() {
        let p = AnkleParams::default();
        assert!((p.thickness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_corrective_weight_zero() {
        assert!((corrective_weight(0.0, 0.0, 1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_corrective_weight_full() {
        let w = corrective_weight(1.0, 0.0, 1.0);
        assert!((w - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_corrective_weight_equal_angles() {
        assert_eq!(corrective_weight(0.5, 0.5, 0.5), 0.0);
    }

    #[test]
    fn test_dorsiflexion_corrective() {
        let w = dorsiflexion_corrective(PI / 6.0);
        assert!((w - 1.0).abs() < 1e-5);
        let w0 = dorsiflexion_corrective(0.0);
        assert!(w0.abs() < 1e-5);
    }

    #[test]
    fn test_malleolus_bump_centre() {
        let b = malleolus_bump(0.5, 0.5, 0.3, 0.8);
        assert!((b - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_malleolus_bump_zero_spread() {
        assert_eq!(malleolus_bump(0.0, 0.0, 0.0, 1.0), 0.0);
    }

    #[test]
    fn test_achilles_groove_range() {
        let g0 = achilles_groove(0.0);
        let g1 = achilles_groove(1.0);
        assert!(g0.abs() < 1e-6);
        assert!(g1 < 0.0);
    }

    #[test]
    fn test_evaluate_empty() {
        let params = AnkleParams::default();
        let deltas = evaluate_ankle_morph(&[], [0.0, 0.0, 0.0], &params);
        assert!(deltas.is_empty());
    }

    #[test]
    fn test_blend_endpoints() {
        let a = AnkleParams::default();
        let b = AnkleParams {
            thickness: 1.5,
            ..Default::default()
        };
        let r0 = blend_ankle_params(&a, &b, 0.0);
        let r1 = blend_ankle_params(&a, &b, 1.0);
        assert!((r0.thickness - a.thickness).abs() < 1e-6);
        assert!((r1.thickness - b.thickness).abs() < 1e-6);
    }
}
