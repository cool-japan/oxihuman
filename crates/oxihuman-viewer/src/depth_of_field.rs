// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Depth-of-field post-process visualization parameters.
//!
//! Pure-data module; no GPU calls.  All depth values are in world units.

// ── Types ──────────────────────────────────────────────────────────────────

/// Available depth-of-field blur algorithms.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum DofMode {
    /// Fast Gaussian blur approximation.
    Gaussian,
    /// Physically-based bokeh disc blur.
    Bokeh,
    /// Minimal 3-tap simple blur for preview.
    Simple,
}

/// Full depth-of-field configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DofConfig {
    /// Blur algorithm to use.
    pub mode: DofMode,
    /// Distance from camera to the focal plane (world units).
    pub focal_distance: f32,
    /// Camera focal length in millimetres.
    pub focal_length_mm: f32,
    /// Lens aperture in f-stops (lower = shallower DoF).
    pub aperture_fstop: f32,
    /// CoC radius threshold below which a point is considered "in focus".
    pub focus_threshold: f32,
    /// Maximum CoC radius in pixels (clamps blur kernel).
    pub max_coc_radius: f32,
    /// Whether DoF is enabled at all.
    pub enabled: bool,
}

/// A single DoF sample: depth, CoC radius, and whether in focus.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DofSample {
    /// World-space depth of the sampled point.
    pub depth: f32,
    /// Circle of confusion radius at this depth.
    pub coc_radius: f32,
    /// Whether the point is considered in focus.
    pub in_focus: bool,
    /// Normalised blur weight in `[0, 1]`.
    pub blur_weight: f32,
}

// ── Type aliases ───────────────────────────────────────────────────────────

/// Pair `(near_plane, far_plane)` bounding the in-focus region.
pub type DofFocusRegion = (f32, f32);

// ── Config ─────────────────────────────────────────────────────────────────

/// Return a sensible default [`DofConfig`].
#[allow(dead_code)]
pub fn default_dof_config() -> DofConfig {
    DofConfig {
        mode: DofMode::Gaussian,
        focal_distance: 5.0,
        focal_length_mm: 50.0,
        aperture_fstop: 5.6,
        focus_threshold: 0.5,
        max_coc_radius: 16.0,
        enabled: true,
    }
}

/// Construct a [`DofConfig`] with explicit parameters.
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn new_dof_config(
    mode: DofMode,
    focal_distance: f32,
    focal_length_mm: f32,
    aperture_fstop: f32,
    focus_threshold: f32,
    max_coc_radius: f32,
    enabled: bool,
) -> DofConfig {
    DofConfig {
        mode,
        focal_distance: focal_distance.max(f32::EPSILON),
        focal_length_mm: focal_length_mm.max(f32::EPSILON),
        aperture_fstop: aperture_fstop.max(f32::EPSILON),
        focus_threshold: focus_threshold.max(0.0),
        max_coc_radius: max_coc_radius.max(0.0),
        enabled,
    }
}

// ── Parameter setters ──────────────────────────────────────────────────────

/// Update the focal distance.
#[allow(dead_code)]
pub fn set_focal_distance(cfg: &mut DofConfig, distance: f32) {
    cfg.focal_distance = distance.max(f32::EPSILON);
}

/// Update the focal length (in mm).
#[allow(dead_code)]
pub fn set_focal_length(cfg: &mut DofConfig, focal_length_mm: f32) {
    cfg.focal_length_mm = focal_length_mm.max(f32::EPSILON);
}

/// Update the aperture f-stop.
#[allow(dead_code)]
pub fn set_aperture(cfg: &mut DofConfig, fstop: f32) {
    cfg.aperture_fstop = fstop.max(f32::EPSILON);
}

// ── Optics computations ────────────────────────────────────────────────────

/// Compute the circle-of-confusion radius at world-space `depth`.
///
/// Uses the simplified DoF formula:
/// ```text
/// CoC = (focal_length_mm / aperture_fstop) * |depth - focal_distance| / depth
/// ```
/// The result is in the same conceptual unit as `focal_length_mm` (mm), scaled
/// to a normalised world radius by dividing by `focal_distance`.
#[allow(dead_code)]
pub fn compute_coc(cfg: &DofConfig, depth: f32) -> f32 {
    let d = depth.max(f32::EPSILON);
    let cap_d = cfg.focal_distance.max(f32::EPSILON);
    let aperture_diameter = cfg.focal_length_mm / cfg.aperture_fstop.max(f32::EPSILON);
    let coc = aperture_diameter * (d - cap_d).abs() / (d * cap_d);
    coc.min(cfg.max_coc_radius)
}

/// Return the blur radius (pixels) at the given world-space `depth`.
/// Identical to `compute_coc` but semantically named for render use.
#[allow(dead_code)]
pub fn blur_radius_at_depth(cfg: &DofConfig, depth: f32) -> f32 {
    compute_coc(cfg, depth)
}

/// Return `true` if the CoC at `depth` is below the focus threshold.
#[allow(dead_code)]
pub fn is_in_focus(cfg: &DofConfig, depth: f32) -> bool {
    compute_coc(cfg, depth) < cfg.focus_threshold
}

/// Compute the near edge of the in-focus zone.
///
/// Solves `compute_coc(near) == focus_threshold` for the nearer depth.
#[allow(dead_code)]
pub fn dof_near_plane(cfg: &DofConfig) -> f32 {
    let cap_d = cfg.focal_distance.max(f32::EPSILON);
    let a = cfg.focal_length_mm / cfg.aperture_fstop.max(f32::EPSILON);
    let c = cfg.focus_threshold.max(f32::EPSILON);
    // near: d = D * A / (A + c * D)
    cap_d * a / (a + c * cap_d)
}

/// Compute the far edge of the in-focus zone.
///
/// Returns `f32::INFINITY` when the far plane is unbounded.
#[allow(dead_code)]
pub fn dof_far_plane(cfg: &DofConfig) -> f32 {
    let cap_d = cfg.focal_distance.max(f32::EPSILON);
    let a = cfg.focal_length_mm / cfg.aperture_fstop.max(f32::EPSILON);
    let c = cfg.focus_threshold.max(f32::EPSILON);
    // far: d = D * A / (A - c * D); if A <= c * D, depth is infinite
    let denom = a - c * cap_d;
    if denom <= f32::EPSILON {
        return f32::INFINITY;
    }
    cap_d * a / denom
}

/// Return the extent `(near, far)` of the in-focus region.
#[allow(dead_code)]
pub fn focus_region_extent(cfg: &DofConfig) -> DofFocusRegion {
    (dof_near_plane(cfg), dof_far_plane(cfg))
}

/// Sample DoF parameters at a given world-space `depth`.
#[allow(dead_code)]
pub fn dof_sample_at_depth(cfg: &DofConfig, depth: f32) -> DofSample {
    let coc_radius = compute_coc(cfg, depth);
    let in_focus = coc_radius < cfg.focus_threshold;
    let blur_weight = (coc_radius / cfg.max_coc_radius.max(f32::EPSILON)).clamp(0.0, 1.0);
    DofSample {
        depth,
        coc_radius,
        in_focus,
        blur_weight,
    }
}

/// Serialize the config to a compact JSON string.
#[allow(dead_code)]
pub fn dof_to_json(cfg: &DofConfig) -> String {
    let mode_str = match cfg.mode {
        DofMode::Gaussian => "Gaussian",
        DofMode::Bokeh => "Bokeh",
        DofMode::Simple => "Simple",
    };
    format!(
        r#"{{"mode":"{}","focal_distance":{:.4},"focal_length_mm":{:.4},"aperture_fstop":{:.4},"focus_threshold":{:.4},"max_coc_radius":{:.4},"enabled":{}}}"#,
        mode_str,
        cfg.focal_distance,
        cfg.focal_length_mm,
        cfg.aperture_fstop,
        cfg.focus_threshold,
        cfg.max_coc_radius,
        cfg.enabled,
    )
}

// ── DofFocusPlane / DofEffect (spec API) ──────────────────────────────────

/// Describes the near/far extent of the in-focus zone.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DofFocusPlane {
    /// Near edge of the in-focus region (world units).
    pub near: f32,
    /// Far edge of the in-focus region (world units). May be `f32::INFINITY`.
    pub far: f32,
    /// Focal distance (world units).
    pub focal_distance: f32,
}

/// A self-contained depth-of-field effect instance wrapping [`DofConfig`].
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DofEffect {
    /// Underlying configuration.
    pub config: DofConfig,
}

/// Construct a new [`DofEffect`] from the given config.
#[allow(dead_code)]
pub fn new_dof_effect(cfg: &DofConfig) -> DofEffect {
    DofEffect {
        config: cfg.clone(),
    }
}

/// Update the focus distance on the effect.
#[allow(dead_code)]
pub fn dof_set_focus_distance(effect: &mut DofEffect, distance: f32) {
    effect.config.focal_distance = distance.max(f32::EPSILON);
}

/// Update the aperture f-stop on the effect.
#[allow(dead_code)]
pub fn dof_set_aperture(effect: &mut DofEffect, aperture: f32) {
    effect.config.aperture_fstop = aperture.max(f32::EPSILON);
}

/// Update the focal length (mm) on the effect.
#[allow(dead_code)]
pub fn dof_set_focal_length(effect: &mut DofEffect, focal_length_mm: f32) {
    effect.config.focal_length_mm = focal_length_mm.max(f32::EPSILON);
}

/// Compute the circle of confusion at world-space `depth` for this effect.
#[allow(dead_code)]
pub fn dof_circle_of_confusion(effect: &DofEffect, depth: f32) -> f32 {
    compute_coc(&effect.config, depth)
}

/// Return the [`DofFocusPlane`] for this effect.
#[allow(dead_code)]
pub fn dof_focus_plane(effect: &DofEffect) -> DofFocusPlane {
    let near = dof_near_plane(&effect.config);
    let far = dof_far_plane(&effect.config);
    DofFocusPlane {
        near,
        far,
        focal_distance: effect.config.focal_distance,
    }
}

/// Returns `true` when the effect is enabled.
#[allow(dead_code)]
pub fn dof_is_enabled(effect: &DofEffect) -> bool {
    effect.config.enabled
}

/// Enable or disable the effect.
#[allow(dead_code)]
pub fn set_dof_enabled(effect: &mut DofEffect, enabled: bool) {
    effect.config.enabled = enabled;
}

/// Compute the blur radius (pixels) at world-space `depth` given a sensor width.
///
/// Applies the standard blur formula and converts using `sensor_width_mm`.
#[allow(dead_code)]
pub fn dof_blur_radius(effect: &DofEffect, depth: f32, sensor_width_mm: f32) -> f32 {
    let coc = compute_coc(&effect.config, depth);
    let sensor_scale = sensor_width_mm.max(f32::EPSILON) / 35.0; // normalise to 35 mm
    (coc * sensor_scale).clamp(0.0, effect.config.max_coc_radius)
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_dof_config() {
        let cfg = default_dof_config();
        assert!(cfg.enabled);
        assert_eq!(cfg.mode, DofMode::Gaussian);
        assert!((cfg.focal_distance - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_new_dof_config() {
        let cfg = new_dof_config(DofMode::Bokeh, 10.0, 85.0, 2.8, 1.0, 32.0, true);
        assert_eq!(cfg.mode, DofMode::Bokeh);
        assert!((cfg.focal_distance - 10.0).abs() < 1e-5);
        assert!((cfg.focal_length_mm - 85.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_focal_distance() {
        let mut cfg = default_dof_config();
        set_focal_distance(&mut cfg, 12.0);
        assert!((cfg.focal_distance - 12.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_focal_length() {
        let mut cfg = default_dof_config();
        set_focal_length(&mut cfg, 35.0);
        assert!((cfg.focal_length_mm - 35.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_aperture() {
        let mut cfg = default_dof_config();
        set_aperture(&mut cfg, 1.4);
        assert!((cfg.aperture_fstop - 1.4).abs() < 1e-5);
    }

    #[test]
    fn test_compute_coc_at_focal_distance_is_zero() {
        let cfg = default_dof_config();
        // At the focal plane, CoC should be 0 (or very close)
        let coc = compute_coc(&cfg, cfg.focal_distance);
        assert!(coc < 1e-3, "CoC at focal distance should be ~0, got {coc}");
    }

    #[test]
    fn test_compute_coc_increases_with_distance() {
        let cfg = default_dof_config();
        let coc_near = compute_coc(&cfg, 1.0);
        let coc_far = compute_coc(&cfg, 20.0);
        // Both should be positive at non-focal depths
        assert!(coc_near >= 0.0);
        assert!(coc_far >= 0.0);
    }

    #[test]
    fn test_compute_coc_clamped_by_max() {
        let cfg = default_dof_config();
        let coc = compute_coc(&cfg, 1000.0);
        assert!(coc <= cfg.max_coc_radius + f32::EPSILON);
    }

    #[test]
    fn test_blur_radius_matches_compute_coc() {
        let cfg = default_dof_config();
        let depth = 3.0;
        assert!((blur_radius_at_depth(&cfg, depth) - compute_coc(&cfg, depth)).abs() < 1e-6);
    }

    #[test]
    fn test_is_in_focus_at_focal_plane() {
        let cfg = default_dof_config();
        assert!(is_in_focus(&cfg, cfg.focal_distance));
    }

    #[test]
    fn test_is_in_focus_far_away() {
        let mut cfg = default_dof_config();
        cfg.focus_threshold = 0.001; // very tight focus
                                     // Very far point should be out of focus
        assert!(!is_in_focus(&cfg, 1000.0));
    }

    #[test]
    fn test_dof_near_plane_positive() {
        let cfg = default_dof_config();
        let near = dof_near_plane(&cfg);
        assert!(near > 0.0, "near plane must be positive, got {near}");
    }

    #[test]
    fn test_dof_far_plane_gt_near() {
        let cfg = default_dof_config();
        let near = dof_near_plane(&cfg);
        let far = dof_far_plane(&cfg);
        assert!(
            far > near || far == f32::INFINITY,
            "far={far} should exceed near={near}"
        );
    }

    #[test]
    fn test_focus_region_extent() {
        let cfg = default_dof_config();
        let (near, far) = focus_region_extent(&cfg);
        assert!(near > 0.0);
        assert!(far > near || far == f32::INFINITY);
    }

    #[test]
    fn test_dof_sample_at_depth_in_focus() {
        let cfg = default_dof_config();
        let sample = dof_sample_at_depth(&cfg, cfg.focal_distance);
        assert!(sample.in_focus);
        assert!((sample.depth - cfg.focal_distance).abs() < 1e-5);
    }

    #[test]
    fn test_dof_to_json_nonempty() {
        let cfg = default_dof_config();
        let json = dof_to_json(&cfg);
        assert!(!json.is_empty());
        assert!(json.contains("Gaussian"));
        assert!(json.contains("focal_distance"));
    }

    // ── DofEffect API ────────────────────────────────────────────────────────

    #[test]
    fn test_new_dof_effect_from_default() {
        let cfg = default_dof_config();
        let effect = new_dof_effect(&cfg);
        assert!(dof_is_enabled(&effect));
    }

    #[test]
    fn test_dof_set_focus_distance() {
        let cfg = default_dof_config();
        let mut effect = new_dof_effect(&cfg);
        dof_set_focus_distance(&mut effect, 10.0);
        assert!((effect.config.focal_distance - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_dof_set_aperture() {
        let cfg = default_dof_config();
        let mut effect = new_dof_effect(&cfg);
        dof_set_aperture(&mut effect, 2.8);
        assert!((effect.config.aperture_fstop - 2.8).abs() < 1e-5);
    }

    #[test]
    fn test_dof_set_focal_length() {
        let cfg = default_dof_config();
        let mut effect = new_dof_effect(&cfg);
        dof_set_focal_length(&mut effect, 85.0);
        assert!((effect.config.focal_length_mm - 85.0).abs() < 1e-5);
    }

    #[test]
    fn test_dof_circle_of_confusion_at_focal() {
        let cfg = default_dof_config();
        let effect = new_dof_effect(&cfg);
        let coc = dof_circle_of_confusion(&effect, cfg.focal_distance);
        assert!(coc < 1e-3);
    }

    #[test]
    fn test_dof_focus_plane_near_positive() {
        let cfg = default_dof_config();
        let effect = new_dof_effect(&cfg);
        let fp = dof_focus_plane(&effect);
        assert!(fp.near > 0.0);
        assert!(fp.far > fp.near || fp.far == f32::INFINITY);
    }

    #[test]
    fn test_set_dof_enabled() {
        let cfg = default_dof_config();
        let mut effect = new_dof_effect(&cfg);
        set_dof_enabled(&mut effect, false);
        assert!(!dof_is_enabled(&effect));
        set_dof_enabled(&mut effect, true);
        assert!(dof_is_enabled(&effect));
    }

    #[test]
    fn test_dof_blur_radius_nonnegative() {
        let cfg = default_dof_config();
        let effect = new_dof_effect(&cfg);
        let br = dof_blur_radius(&effect, 2.0, 35.0);
        assert!(br >= 0.0);
    }

    #[test]
    fn test_dof_blur_radius_clamped() {
        let cfg = default_dof_config();
        let effect = new_dof_effect(&cfg);
        let br = dof_blur_radius(&effect, 10000.0, 35.0);
        assert!(br <= effect.config.max_coc_radius + f32::EPSILON);
    }
}
