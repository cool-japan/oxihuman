// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! View-dependent LOD selection — chooses detail level based on camera parameters.

/// Camera parameters used for LOD selection.
#[derive(Debug, Clone, Copy)]
pub struct CameraParams {
    pub position: [f32; 3],
    pub fov_radians: f32,
    pub screen_height_px: u32,
}

/// A bounding sphere for an object.
#[derive(Debug, Clone, Copy)]
pub struct BoundingSphere {
    pub center: [f32; 3],
    pub radius: f32,
}

/// Result of a view-dependent LOD query.
#[derive(Debug, Clone, Copy)]
pub struct LodQuery {
    pub screen_coverage: f32,
    pub recommended_level: u32,
    pub distance: f32,
}

/// Computes the distance from camera to sphere center.
pub fn camera_distance(cam: &CameraParams, sphere: &BoundingSphere) -> f32 {
    let dx = cam.position[0] - sphere.center[0];
    let dy = cam.position[1] - sphere.center[1];
    let dz = cam.position[2] - sphere.center[2];
    (dx * dx + dy * dy + dz * dz).sqrt().max(f32::EPSILON)
}

/// Computes screen-space coverage (projected radius / screen height).
pub fn screen_coverage(cam: &CameraParams, sphere: &BoundingSphere) -> f32 {
    let dist = camera_distance(cam, sphere);
    let projected_radius = sphere.radius / (dist * (cam.fov_radians * 0.5).tan());
    (projected_radius * cam.screen_height_px as f32).clamp(0.0, 1.0)
}

/// Selects an LOD level from 0..`max_levels` based on screen coverage.
pub fn select_lod_level(coverage: f32, max_levels: u32) -> u32 {
    let coverage = coverage.clamp(0.0, 1.0);
    let inv = 1.0 - coverage;
    ((inv * max_levels as f32) as u32).min(max_levels.saturating_sub(1))
}

/// Runs a full LOD query combining distance, coverage, and level selection.
pub fn query_lod(cam: &CameraParams, sphere: &BoundingSphere, max_levels: u32) -> LodQuery {
    let distance = camera_distance(cam, sphere);
    let coverage = screen_coverage(cam, sphere);
    let recommended_level = select_lod_level(coverage, max_levels);
    LodQuery {
        screen_coverage: coverage,
        recommended_level,
        distance,
    }
}

/// Checks if an object is within a given distance budget.
pub fn within_distance_budget(cam: &CameraParams, sphere: &BoundingSphere, max_dist: f32) -> bool {
    camera_distance(cam, sphere) <= max_dist
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_4;

    fn default_cam() -> CameraParams {
        CameraParams {
            position: [0.0, 0.0, 10.0],
            fov_radians: FRAC_PI_4,
            screen_height_px: 1080,
        }
    }

    fn default_sphere() -> BoundingSphere {
        BoundingSphere {
            center: [0.0, 0.0, 0.0],
            radius: 1.0,
        }
    }

    #[test]
    fn test_camera_distance_basic() {
        /* Distance should be roughly 10 for default params */
        let d = camera_distance(&default_cam(), &default_sphere());
        assert!((d - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_screen_coverage_positive() {
        /* Coverage should be positive for visible sphere */
        let cov = screen_coverage(&default_cam(), &default_sphere());
        assert!(cov > 0.0);
    }

    #[test]
    fn test_select_lod_zero_coverage() {
        /* Zero coverage → max detail (farthest LOD level) */
        let level = select_lod_level(0.0, 4);
        assert_eq!(level, 3);
    }

    #[test]
    fn test_select_lod_full_coverage() {
        /* Full coverage → level 0 */
        let level = select_lod_level(1.0, 4);
        assert_eq!(level, 0);
    }

    #[test]
    fn test_select_lod_clamps() {
        /* Level should never exceed max_levels - 1 */
        let level = select_lod_level(-5.0, 3);
        assert!(level < 3);
    }

    #[test]
    fn test_query_lod_returns_valid_level() {
        /* Recommended level must be within [0, max_levels) */
        let q = query_lod(&default_cam(), &default_sphere(), 5);
        assert!(q.recommended_level < 5);
    }

    #[test]
    fn test_within_distance_budget_true() {
        /* Default setup: distance ~10, budget 20 → true */
        assert!(within_distance_budget(
            &default_cam(),
            &default_sphere(),
            20.0
        ));
    }

    #[test]
    fn test_within_distance_budget_false() {
        /* Budget smaller than distance → false */
        assert!(!within_distance_budget(
            &default_cam(),
            &default_sphere(),
            5.0
        ));
    }

    #[test]
    fn test_query_lod_distance_positive() {
        /* Distance in query result should be positive */
        let q = query_lod(&default_cam(), &default_sphere(), 4);
        assert!(q.distance > 0.0);
    }

    #[test]
    fn test_coverage_clamps_to_one() {
        /* Very large sphere very close should clamp to 1.0 */
        let big_sphere = BoundingSphere {
            center: [0.0, 0.0, 9.99],
            radius: 100.0,
        };
        let cov = screen_coverage(&default_cam(), &big_sphere);
        assert!((0.0..=1.0).contains(&cov));
    }
}
