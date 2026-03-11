// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Terrain inclination estimator stub.

/// Terrain surface estimate from foot contact measurements.
#[derive(Debug, Clone)]
pub struct TerrainEstimate {
    pub pitch_deg: f32,
    pub roll_deg: f32,
    pub height: f32,
    pub confidence: f32,
}

impl Default for TerrainEstimate {
    fn default() -> Self {
        Self {
            pitch_deg: 0.0,
            roll_deg: 0.0,
            height: 0.0,
            confidence: 1.0,
        }
    }
}

/// Terrain estimator state.
#[derive(Debug, Clone, Default)]
pub struct TerrainEstimator {
    pub estimate: TerrainEstimate,
    pub alpha: f32,
}

impl TerrainEstimator {
    pub fn new(alpha: f32) -> Self {
        Self {
            estimate: TerrainEstimate::default(),
            alpha: alpha.clamp(0.0, 1.0),
        }
    }

    pub fn default_estimator() -> Self {
        Self::new(0.1)
    }
}

/// Update the terrain estimate from a new foot contact height measurement.
pub fn update_terrain_from_contact(
    estimator: &mut TerrainEstimator,
    left_foot_pos: [f32; 3],
    right_foot_pos: [f32; 3],
) {
    /* stub: estimate pitch from foot height difference */
    let dx = right_foot_pos[0] - left_foot_pos[0];
    let dz = right_foot_pos[2] - left_foot_pos[2];
    let roll_rad = if dx.abs() > 1e-6 { dz.atan2(dx) } else { 0.0 };
    let new_roll = roll_rad.to_degrees();
    let new_height = (left_foot_pos[2] + right_foot_pos[2]) * 0.5;
    let alpha = estimator.alpha;
    estimator.estimate.roll_deg = (1.0 - alpha) * estimator.estimate.roll_deg + alpha * new_roll;
    estimator.estimate.height = (1.0 - alpha) * estimator.estimate.height + alpha * new_height;
}

/// Return the terrain gradient vector (stub).
pub fn terrain_gradient(estimate: &TerrainEstimate) -> [f32; 2] {
    let pitch_rad = estimate.pitch_deg.to_radians();
    let roll_rad = estimate.roll_deg.to_radians();
    [pitch_rad.tan(), roll_rad.tan()]
}

/// Return whether the terrain slope exceeds a threshold (stub safety check).
pub fn terrain_too_steep(estimate: &TerrainEstimate, max_deg: f32) -> bool {
    estimate.pitch_deg.abs() > max_deg || estimate.roll_deg.abs() > max_deg
}

/// Return a flat terrain estimate.
pub fn flat_terrain() -> TerrainEstimate {
    TerrainEstimate::default()
}

/// Return the estimated surface normal (unit vector, stub).
pub fn terrain_normal(estimate: &TerrainEstimate) -> [f32; 3] {
    let p = estimate.pitch_deg.to_radians();
    let r = estimate.roll_deg.to_radians();
    let nx = -p.sin();
    let ny = -r.sin();
    let nz = (1.0 - nx * nx - ny * ny).max(0.0).sqrt();
    [nx, ny, nz]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_flat() {
        /* default terrain is flat */
        let e = TerrainEstimate::default();
        assert_eq!(e.pitch_deg, 0.0);
        assert_eq!(e.roll_deg, 0.0);
    }

    #[test]
    fn test_update_flat_feet() {
        /* equal foot heights → flat terrain */
        let mut est = TerrainEstimator::default_estimator();
        update_terrain_from_contact(&mut est, [0.0, 0.0, 0.0], [0.0, 0.1, 0.0]);
        assert!(est.estimate.roll_deg.abs() < 1e-3);
    }

    #[test]
    fn test_gradient_flat() {
        /* flat terrain has zero gradient */
        let e = flat_terrain();
        let g = terrain_gradient(&e);
        assert!(g[0].abs() < 1e-6 && g[1].abs() < 1e-6);
    }

    #[test]
    fn test_too_steep_false() {
        /* flat terrain not too steep */
        let e = flat_terrain();
        assert!(!terrain_too_steep(&e, 30.0));
    }

    #[test]
    fn test_too_steep_true() {
        /* steep terrain triggers flag */
        let e = TerrainEstimate {
            pitch_deg: 45.0,
            ..Default::default()
        };
        assert!(terrain_too_steep(&e, 30.0));
    }

    #[test]
    fn test_normal_flat_points_up() {
        /* flat terrain normal is [0,0,1] */
        let e = flat_terrain();
        let n = terrain_normal(&e);
        assert!((n[2] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_normal_unit_length() {
        /* normal has unit length */
        let e = TerrainEstimate {
            pitch_deg: 20.0,
            roll_deg: 10.0,
            ..Default::default()
        };
        let n = terrain_normal(&e);
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_alpha_clamped() {
        /* alpha is clamped to [0,1] */
        let est = TerrainEstimator::new(5.0);
        assert!(est.alpha <= 1.0);
    }

    #[test]
    fn test_confidence_default() {
        /* default confidence is 1.0 */
        assert_eq!(TerrainEstimate::default().confidence, 1.0);
    }
}
