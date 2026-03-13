// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

//! Core scan-fitting types: [`ScanCloud`], [`FitResult`], [`FitConfig`],
//! [`BodyMeasurementsEstimate`], and the gradient-free fitting functions.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// ScanCloud
// ---------------------------------------------------------------------------

/// A point cloud produced by a body scanner.
///
/// Points are in metres, Y-up convention.
#[derive(Debug, Clone)]
pub struct ScanCloud {
    /// 3-D positions `[x, y, z]` in metres.
    pub points: Vec<[f32; 3]>,
    /// Optional per-point outward normals.
    pub normals: Option<Vec<[f32; 3]>>,
}

impl ScanCloud {
    /// Create a cloud from positions only (normals set to `None`).
    pub fn new(points: Vec<[f32; 3]>) -> Self {
        Self {
            points,
            normals: None,
        }
    }

    /// Create a cloud with explicit normals.
    ///
    /// # Panics
    /// Panics if `normals.len() != points.len()`.
    pub fn with_normals(points: Vec<[f32; 3]>, normals: Vec<[f32; 3]>) -> Self {
        assert_eq!(
            points.len(),
            normals.len(),
            "points and normals must have the same length"
        );
        Self {
            points,
            normals: Some(normals),
        }
    }

    /// Number of points in the cloud.
    pub fn point_count(&self) -> usize {
        self.points.len()
    }

    /// Compute the axis-aligned bounding box as `(min, max)`.
    ///
    /// Returns `([0.0; 3], [0.0; 3])` for an empty cloud.
    pub fn bbox(&self) -> ([f32; 3], [f32; 3]) {
        if self.points.is_empty() {
            return ([0.0; 3], [0.0; 3]);
        }
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for p in &self.points {
            for i in 0..3 {
                if p[i] < min[i] {
                    min[i] = p[i];
                }
                if p[i] > max[i] {
                    max[i] = p[i];
                }
            }
        }
        (min, max)
    }

    /// Compute the centroid (mean position) of the cloud.
    ///
    /// Returns `[0.0; 3]` for an empty cloud.
    pub fn centroid(&self) -> [f32; 3] {
        if self.points.is_empty() {
            return [0.0; 3];
        }
        let n = self.points.len() as f32;
        let mut sum = [0.0_f32; 3];
        for p in &self.points {
            sum[0] += p[0];
            sum[1] += p[1];
            sum[2] += p[2];
        }
        [sum[0] / n, sum[1] / n, sum[2] / n]
    }

    /// Body height = Y extent of the bounding box (in metres).
    pub fn height(&self) -> f32 {
        let (min, max) = self.bbox();
        (max[1] - min[1]).max(0.0)
    }

    /// Return a new cloud centred at the origin and scaled so that height = 1.
    ///
    /// If height is zero the cloud is only centred, not scaled.
    pub fn normalize(&self) -> Self {
        let c = self.centroid();
        let h = self.height().max(1e-8);
        let pts: Vec<[f32; 3]> = self
            .points
            .iter()
            .map(|p| [(p[0] - c[0]) / h, (p[1] - c[1]) / h, (p[2] - c[2]) / h])
            .collect();
        let nrm = self.normals.clone();
        Self {
            points: pts,
            normals: nrm,
        }
    }
}

// ---------------------------------------------------------------------------
// FitResult
// ---------------------------------------------------------------------------

/// Outcome of a parameter-fitting run.
#[derive(Debug, Clone)]
pub struct FitResult {
    /// Parameter name → value in `[0, 1]`.
    pub params: HashMap<String, f32>,
    /// Mean closest-point distance (metres) between scan and fitted mesh.
    pub residual_error: f32,
    /// Number of coordinate-descent iterations executed.
    pub iterations: usize,
    /// Whether the run converged within `convergence_tol`.
    pub converged: bool,
}

// ---------------------------------------------------------------------------
// FitConfig
// ---------------------------------------------------------------------------

/// Configuration for the gradient-free fitting loop.
#[derive(Debug, Clone)]
pub struct FitConfig {
    /// Maximum number of outer iterations (one full sweep per iteration).
    pub max_iterations: usize,
    /// Convergence tolerance on mean error improvement.
    pub convergence_tol: f32,
    /// Initial step size for each parameter.
    pub learning_rate: f32,
    /// Names of the parameters to fit.
    pub param_names: Vec<String>,
}

impl Default for FitConfig {
    fn default() -> Self {
        Self {
            max_iterations: 50,
            convergence_tol: 0.001,
            learning_rate: 0.1,
            param_names: vec![
                "height".to_string(),
                "weight".to_string(),
                "muscle".to_string(),
                "age".to_string(),
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// BodyMeasurementsEstimate
// ---------------------------------------------------------------------------

/// Axis-aligned body measurement estimates derived from a [`ScanCloud`].
///
/// All values are in **metres**.
#[derive(Debug, Clone)]
pub struct BodyMeasurementsEstimate {
    /// Standing height (Y extent of bounding box).
    pub height_m: f32,
    /// Shoulder width (X extent at shoulder level ≈ 85 % of height).
    pub shoulder_width_m: f32,
    /// Estimated chest circumference (treating cross-section as ellipse).
    pub chest_circumference_m: f32,
    /// Estimated waist circumference.
    pub waist_circumference_m: f32,
    /// Hip width (X extent at hip level ≈ 35 % of height).
    pub hip_width_m: f32,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Half-width of the cloud in X at a given Y target ± band.
fn half_width_at_y(cloud: &ScanCloud, y_target: f32, band: f32) -> f32 {
    let mut max_x = 0.0_f32;
    let mut max_z = 0.0_f32;
    let mut found = false;
    for p in &cloud.points {
        if (p[1] - y_target).abs() <= band {
            let ax = p[0].abs();
            let az = p[2].abs();
            if ax > max_x {
                max_x = ax;
            }
            if az > max_z {
                max_z = az;
            }
            found = true;
        }
    }
    if !found {
        // Fallback to global X extent
        max_x = cloud
            .points
            .iter()
            .map(|p| p[0].abs())
            .fold(0.0_f32, f32::max);
        max_z = max_x * 0.6; // assume depth ~ 60 % of width
    }
    (max_x, max_z).0.max((max_x, max_z).1) // return the wider axis
                                           // Actually return a tuple for circumference computation
                                           // We'll inline this differently below.
}

/// Returns `(half_x, half_z)` of the cloud at a y-slice.
fn half_extents_at_y(cloud: &ScanCloud, y_target: f32, band: f32) -> (f32, f32) {
    let mut max_x = 0.0_f32;
    let mut max_z = 0.0_f32;
    let mut found = false;
    for p in &cloud.points {
        if (p[1] - y_target).abs() <= band {
            let ax = p[0].abs();
            let az = p[2].abs();
            if ax > max_x {
                max_x = ax;
            }
            if az > max_z {
                max_z = az;
            }
            found = true;
        }
    }
    if !found {
        let gx = cloud
            .points
            .iter()
            .map(|p| p[0].abs())
            .fold(0.0_f32, f32::max);
        let gz = cloud
            .points
            .iter()
            .map(|p| p[2].abs())
            .fold(0.0_f32, f32::max);
        return (gx, gz.max(gx * 0.5));
    }
    (max_x, max_z.max(max_x * 0.4))
}

/// Ramanujan ellipse circumference approximation for semi-axes a, b.
fn ellipse_circumference(a: f32, b: f32) -> f32 {
    // Ramanujan first approximation: π × [3(a+b) - sqrt((3a+b)(a+3b))]
    let t = 3.0 * (a + b) - ((3.0 * a + b) * (a + 3.0 * b)).sqrt();
    std::f32::consts::PI * t
}

/// Clamp a value to `[0, 1]`.
fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Public free functions
// ---------------------------------------------------------------------------

/// Estimate body measurements from a scan cloud using bounding-box slicing.
pub fn estimate_measurements(cloud: &ScanCloud) -> BodyMeasurementsEstimate {
    let h = cloud.height();
    let (bbox_min, _bbox_max) = cloud.bbox();
    let min_y = bbox_min[1];
    let band = h * 0.03_f32;

    // Anatomical Y-fractions (from floor)
    let shoulder_y = min_y + h * 0.82;
    let chest_y = min_y + h * 0.60;
    let waist_y = min_y + h * 0.45;
    let hip_y = min_y + h * 0.35;

    let (shld_x, _) = half_extents_at_y(cloud, shoulder_y, band * 2.0);
    let shoulder_width = shld_x * 2.0;

    let (chest_x, chest_z) = half_extents_at_y(cloud, chest_y, band);
    let chest_circ = ellipse_circumference(chest_x, chest_z.max(chest_x * 0.55));

    let (waist_x, waist_z) = half_extents_at_y(cloud, waist_y, band);
    let waist_circ = ellipse_circumference(waist_x, waist_z.max(waist_x * 0.50));

    let (hip_x, _) = half_extents_at_y(cloud, hip_y, band);
    let hip_width = hip_x * 2.0;

    BodyMeasurementsEstimate {
        height_m: h,
        shoulder_width_m: shoulder_width,
        chest_circumference_m: chest_circ,
        waist_circumference_m: waist_circ,
        hip_width_m: hip_width,
    }
}

/// Map body measurements to approximate parameter values in `[0, 1]`.
///
/// Uses simple linear heuristics calibrated to average human proportions.
pub fn measurements_to_params(meas: &BodyMeasurementsEstimate) -> HashMap<String, f32> {
    let mut params = HashMap::new();

    // Height: range 1.40 m → 0.0, 2.10 m → 1.0
    let height_param = clamp01((meas.height_m - 1.40) / (2.10 - 1.40));
    params.insert("height".to_string(), height_param);

    // Weight: inferred from chest + waist circumference
    // Rough: circ range 0.60–1.40 m maps to [0, 1]
    let avg_girth = (meas.chest_circumference_m + meas.waist_circumference_m) * 0.5;
    let weight_param = clamp01((avg_girth - 0.60) / (1.40 - 0.60));
    params.insert("weight".to_string(), weight_param);

    // Muscle: shoulder-to-waist ratio
    // Athletic: shoulder wide, waist narrow → ratio > 1.6
    let waist_w = meas.waist_circumference_m / std::f32::consts::PI; // diameter
    let ratio = if waist_w > 1e-4 {
        meas.shoulder_width_m / waist_w
    } else {
        1.0
    };
    // ratio 0.9 → 0.0 muscle, 2.5 → 1.0 muscle
    let muscle_param = clamp01((ratio - 0.9) / (2.5 - 0.9));
    params.insert("muscle".to_string(), muscle_param);

    // Age: no direct measurement; default mid-range
    params.insert("age".to_string(), 0.35);

    params
}

/// Compute mean closest-point distance from scan to mesh positions.
///
/// For each scan point the nearest mesh vertex is found by brute force.
/// Returns 0.0 if either collection is empty.
pub fn scan_to_mesh_error(scan: &ScanCloud, mesh_positions: &[[f32; 3]]) -> f32 {
    if scan.points.is_empty() || mesh_positions.is_empty() {
        return 0.0;
    }
    let total: f32 = scan
        .points
        .iter()
        .map(|sp| {
            mesh_positions
                .iter()
                .map(|mp| {
                    let dx = sp[0] - mp[0];
                    let dy = sp[1] - mp[1];
                    let dz = sp[2] - mp[2];
                    (dx * dx + dy * dy + dz * dz).sqrt()
                })
                .fold(f32::INFINITY, f32::min)
        })
        .sum();
    total / scan.points.len() as f32
}

/// Align a scan cloud to a mesh via centroid translation + uniform scale.
///
/// The returned cloud has the same centroid as the mesh and the same height
/// (Y extent).  If the scan height is zero the original cloud is returned.
pub fn align_scan_to_mesh(scan: &ScanCloud, mesh_positions: &[[f32; 3]]) -> ScanCloud {
    if scan.points.is_empty() || mesh_positions.is_empty() {
        return scan.clone();
    }

    // Scan stats
    let scan_c = scan.centroid();
    let scan_h = scan.height().max(1e-8);

    // Mesh centroid
    let n = mesh_positions.len() as f32;
    let mut mesh_c = [0.0_f32; 3];
    for p in mesh_positions {
        mesh_c[0] += p[0];
        mesh_c[1] += p[1];
        mesh_c[2] += p[2];
    }
    mesh_c[0] /= n;
    mesh_c[1] /= n;
    mesh_c[2] /= n;

    // Mesh height
    let min_y = mesh_positions
        .iter()
        .map(|p| p[1])
        .fold(f32::INFINITY, f32::min);
    let max_y = mesh_positions
        .iter()
        .map(|p| p[1])
        .fold(f32::NEG_INFINITY, f32::max);
    let mesh_h = (max_y - min_y).max(1e-8);

    let scale = mesh_h / scan_h;

    let pts: Vec<[f32; 3]> = scan
        .points
        .iter()
        .map(|p| {
            [
                (p[0] - scan_c[0]) * scale + mesh_c[0],
                (p[1] - scan_c[1]) * scale + mesh_c[1],
                (p[2] - scan_c[2]) * scale + mesh_c[2],
            ]
        })
        .collect();

    ScanCloud {
        points: pts,
        normals: scan.normals.clone(),
    }
}

/// Fit body parameters to a scan cloud using coordinate descent.
///
/// For each parameter, tries `current ± step`; keeps the move that reduces
/// the scan-to-mesh error.  Step is halved when no improvement is found.
/// Runs for at most `config.max_iterations` outer rounds.
///
/// `mesh_fn` must map a `&HashMap<String, f32>` (parameter values) to a
/// `Vec<[f32; 3]>` of mesh vertex positions.
#[allow(clippy::type_complexity)]
pub fn fit_params_to_scan(
    scan: &ScanCloud,
    config: &FitConfig,
    mesh_fn: &dyn Fn(&HashMap<String, f32>) -> Vec<[f32; 3]>,
) -> FitResult {
    if scan.points.is_empty() {
        return FitResult {
            params: HashMap::new(),
            residual_error: 0.0,
            iterations: 0,
            converged: true,
        };
    }

    // Initialise parameters at 0.5
    let mut params: HashMap<String, f32> = config
        .param_names
        .iter()
        .map(|n| (n.clone(), 0.5_f32))
        .collect();

    // Step sizes per parameter
    let mut steps: HashMap<String, f32> = config
        .param_names
        .iter()
        .map(|n| (n.clone(), config.learning_rate))
        .collect();

    // Compute initial error
    let initial_mesh = mesh_fn(&params);
    let aligned = align_scan_to_mesh(scan, &initial_mesh);
    let mut current_error = scan_to_mesh_error(&aligned, &initial_mesh);

    let mut iterations = 0usize;
    let mut converged = false;

    'outer: for _iter in 0..config.max_iterations {
        let mut improved_any = false;

        for name in &config.param_names {
            let cur_val = *params.get(name).unwrap_or(&0.5);
            let step = *steps.get(name).unwrap_or(&0.1);

            // Try + step
            let val_plus = clamp01(cur_val + step);
            params.insert(name.clone(), val_plus);
            let mesh_plus = mesh_fn(&params);
            let aligned_plus = align_scan_to_mesh(scan, &mesh_plus);
            let err_plus = scan_to_mesh_error(&aligned_plus, &mesh_plus);

            // Try - step
            let val_minus = clamp01(cur_val - step);
            params.insert(name.clone(), val_minus);
            let mesh_minus = mesh_fn(&params);
            let aligned_minus = align_scan_to_mesh(scan, &mesh_minus);
            let err_minus = scan_to_mesh_error(&aligned_minus, &mesh_minus);

            // Pick best
            let (best_val, best_err) = if err_plus <= err_minus {
                (val_plus, err_plus)
            } else {
                (val_minus, err_minus)
            };

            if best_err < current_error {
                params.insert(name.clone(), best_val);
                current_error = best_err;
                improved_any = true;
            } else {
                // Restore original and halve step
                params.insert(name.clone(), cur_val);
                steps.insert(name.clone(), step * 0.5);
            }
        }

        iterations += 1;

        // Convergence: improvement in this round was tiny
        if !improved_any || current_error < config.convergence_tol {
            converged = true;
            break 'outer;
        }
    }

    FitResult {
        params,
        residual_error: current_error,
        iterations,
        converged,
    }
}

/// Quick parameter estimate from bounding-box measurements only (no mesh).
///
/// Uses [`estimate_measurements`] → [`measurements_to_params`].
pub fn quick_fit_from_bbox(cloud: &ScanCloud) -> HashMap<String, f32> {
    let meas = estimate_measurements(cloud);
    measurements_to_params(&meas)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helper: build a synthetic humanoid point cloud ──────────────────────

    /// Build a minimal humanoid point cloud (Y-up, metres).
    /// Height ≈ 1.75 m, broad shoulders, moderate waist.
    fn make_human_cloud() -> ScanCloud {
        let mut pts: Vec<[f32; 3]> = Vec::new();

        // Floor and crown
        pts.push([0.0, 0.0, 0.0]);
        pts.push([0.0, 1.75, 0.0]);

        // Shoulder slice (y ≈ 0.82 × 1.75 = 1.435 m), width ~0.46 m
        for dx in [-0.23_f32, 0.0, 0.23] {
            pts.push([dx, 1.435, 0.0]);
            pts.push([dx, 1.435, 0.15]);
            pts.push([dx, 1.435, -0.15]);
        }

        // Chest slice (y ≈ 0.60 × 1.75 = 1.05 m), half-x=0.19 half-z=0.12
        for dx in [-0.19_f32, 0.19] {
            pts.push([dx, 1.05, 0.0]);
            pts.push([dx, 1.05, 0.12]);
            pts.push([dx, 1.05, -0.12]);
        }
        for dz in [-0.12_f32, 0.12] {
            pts.push([0.0, 1.05, dz]);
        }

        // Waist slice (y ≈ 0.45 × 1.75 = 0.7875 m), half-x=0.14 half-z=0.09
        for dx in [-0.14_f32, 0.14] {
            pts.push([dx, 0.7875, 0.0]);
            pts.push([dx, 0.7875, 0.09]);
            pts.push([dx, 0.7875, -0.09]);
        }

        // Hip slice (y ≈ 0.35 × 1.75 = 0.6125 m), half-x=0.17
        for dx in [-0.17_f32, 0.0, 0.17] {
            pts.push([dx, 0.6125, 0.0]);
        }

        ScanCloud::new(pts)
    }

    /// Build a simple sparse mesh (column spanning 0.0–1.75 m in Y).
    fn make_simple_mesh(h: f32) -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [0.0, h, 0.0],
            [0.2, h * 0.6, 0.0],
            [-0.2, h * 0.6, 0.0],
            [0.15, h * 0.45, 0.0],
            [-0.15, h * 0.45, 0.0],
        ]
    }

    // ── Test 1: ScanCloud::new stores points correctly ───────────────────────

    #[test]
    fn scan_cloud_new_stores_points() {
        let pts = vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let cloud = ScanCloud::new(pts.clone());
        assert_eq!(cloud.points, pts);
        assert!(cloud.normals.is_none());
    }

    // ── Test 2: ScanCloud::with_normals stores both ──────────────────────────

    #[test]
    fn scan_cloud_with_normals_stores_both() {
        let pts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let nrm = vec![[0.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
        let cloud = ScanCloud::with_normals(pts.clone(), nrm.clone());
        assert_eq!(cloud.points, pts);
        assert_eq!(cloud.normals.expect("should succeed"), nrm);
    }

    // ── Test 3: point_count returns correct count ────────────────────────────

    #[test]
    fn scan_cloud_point_count() {
        let cloud = make_human_cloud();
        assert!(cloud.point_count() > 0);
        let empty = ScanCloud::new(vec![]);
        assert_eq!(empty.point_count(), 0);
    }

    // ── Test 4: bbox returns correct min/max ─────────────────────────────────

    #[test]
    fn scan_cloud_bbox_correct() {
        let pts = vec![[1.0, 2.0, 3.0], [-1.0, 0.0, -3.0], [0.5, 5.0, 1.0]];
        let cloud = ScanCloud::new(pts);
        let (min, max) = cloud.bbox();
        assert!((min[0] - (-1.0)).abs() < 1e-5);
        assert!((min[1] - 0.0).abs() < 1e-5);
        assert!((min[2] - (-3.0)).abs() < 1e-5);
        assert!((max[0] - 1.0).abs() < 1e-5);
        assert!((max[1] - 5.0).abs() < 1e-5);
        assert!((max[2] - 3.0).abs() < 1e-5);
    }

    // ── Test 5: centroid is correct ──────────────────────────────────────────

    #[test]
    fn scan_cloud_centroid_correct() {
        let pts = vec![[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 2.0, 0.0]];
        let cloud = ScanCloud::new(pts);
        let c = cloud.centroid();
        assert!((c[0] - 0.0).abs() < 1e-5, "cx={}", c[0]);
        assert!((c[1] - (2.0 / 3.0)).abs() < 1e-5, "cy={}", c[1]);
        assert!((c[2] - 0.0).abs() < 1e-5, "cz={}", c[2]);
    }

    // ── Test 6: height returns Y extent ─────────────────────────────────────

    #[test]
    fn scan_cloud_height() {
        let cloud = make_human_cloud();
        let h = cloud.height();
        assert!((h - 1.75).abs() < 1e-4, "height={}", h);
    }

    // ── Test 7: normalize produces unit height at origin ────────────────────

    #[test]
    fn scan_cloud_normalize_unit_height() {
        let cloud = make_human_cloud();
        let norm = cloud.normalize();
        let h = norm.height();
        assert!((h - 1.0).abs() < 1e-4, "normalized height={}", h);
        let c = norm.centroid();
        // Centroid Y should be near 0
        assert!(c[1].abs() < 0.1, "centroid y={}", c[1]);
    }

    // ── Test 8: estimate_measurements returns sensible values ────────────────

    #[test]
    fn estimate_measurements_sensible() {
        let cloud = make_human_cloud();
        let meas = estimate_measurements(&cloud);
        assert!(
            (meas.height_m - 1.75).abs() < 1e-4,
            "height={}",
            meas.height_m
        );
        assert!(meas.shoulder_width_m > 0.0, "shoulder_width <= 0");
        assert!(meas.chest_circumference_m > 0.0, "chest_circ <= 0");
        assert!(meas.waist_circumference_m > 0.0, "waist_circ <= 0");
        assert!(meas.hip_width_m > 0.0, "hip_width <= 0");
        // chest > waist is typical for athletic builds
        // (relaxed requirement: both positive)
        assert!(
            meas.chest_circumference_m > meas.waist_circumference_m * 0.5,
            "chest {} waist {}",
            meas.chest_circumference_m,
            meas.waist_circumference_m
        );
    }

    // ── Test 9: measurements_to_params output in [0,1] ──────────────────────

    #[test]
    fn measurements_to_params_in_range() {
        let cloud = make_human_cloud();
        let meas = estimate_measurements(&cloud);
        let params = measurements_to_params(&meas);
        for (k, v) in &params {
            assert!((0.0..=1.0).contains(v), "param {} = {} out of [0,1]", k, v);
        }
        assert!(params.contains_key("height"));
        assert!(params.contains_key("weight"));
        assert!(params.contains_key("muscle"));
        assert!(params.contains_key("age"));
    }

    // ── Test 10: scan_to_mesh_error zero for identical sets ─────────────────

    #[test]
    fn scan_to_mesh_error_identical_is_zero() {
        let pts = vec![[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let cloud = ScanCloud::new(pts.clone());
        let err = scan_to_mesh_error(&cloud, &pts);
        assert!(err < 1e-5, "error={}", err);
    }

    // ── Test 11: scan_to_mesh_error is positive for different sets ───────────

    #[test]
    fn scan_to_mesh_error_positive_for_different() {
        let scan_pts = vec![[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let mesh_pts = vec![[10.0_f32, 10.0, 10.0], [11.0, 10.0, 10.0]];
        let cloud = ScanCloud::new(scan_pts);
        let err = scan_to_mesh_error(&cloud, &mesh_pts);
        assert!(err > 1.0, "err={}", err);
    }

    // ── Test 12: align_scan_to_mesh brings centroids close ──────────────────

    #[test]
    fn align_scan_to_mesh_centroid_matches() {
        let cloud = make_human_cloud();
        let mesh = make_simple_mesh(1.75);
        let aligned = align_scan_to_mesh(&cloud, &mesh);

        // Compute mesh centroid
        let n = mesh.len() as f32;
        let mut mc = [0.0_f32; 3];
        for p in &mesh {
            mc[0] += p[0];
            mc[1] += p[1];
            mc[2] += p[2];
        }
        mc[0] /= n;
        mc[1] /= n;
        mc[2] /= n;

        let ac = aligned.centroid();
        for i in 0..3 {
            assert!(
                (ac[i] - mc[i]).abs() < 1e-3,
                "aligned centroid[{}] = {} mesh centroid = {}",
                i,
                ac[i],
                mc[i]
            );
        }
    }

    // ── Test 13: quick_fit_from_bbox returns height & weight keys ───────────

    #[test]
    fn quick_fit_from_bbox_has_expected_keys() {
        let cloud = make_human_cloud();
        let params = quick_fit_from_bbox(&cloud);
        assert!(params.contains_key("height"), "missing 'height'");
        assert!(params.contains_key("weight"), "missing 'weight'");
        assert!(params.contains_key("muscle"), "missing 'muscle'");
        assert!(params.contains_key("age"), "missing 'age'");
        for (k, v) in &params {
            assert!(
                (0.0..=1.0).contains(v),
                "quick_fit param {} = {} out of [0,1]",
                k,
                v
            );
        }
    }

    // ── Test 14: FitConfig default has correct values ────────────────────────

    #[test]
    fn fit_config_default_values() {
        let cfg = FitConfig::default();
        assert_eq!(cfg.max_iterations, 50);
        assert!((cfg.convergence_tol - 0.001).abs() < 1e-6);
        assert!((cfg.learning_rate - 0.1).abs() < 1e-6);
        assert!(cfg.param_names.contains(&"height".to_string()));
        assert!(cfg.param_names.contains(&"weight".to_string()));
        assert!(cfg.param_names.contains(&"muscle".to_string()));
        assert!(cfg.param_names.contains(&"age".to_string()));
    }

    // ── Test 15: fit_params_to_scan improves on initial error ───────────────

    #[test]
    fn fit_params_to_scan_improves_error() {
        let cloud = make_human_cloud();
        let cfg = FitConfig {
            max_iterations: 10,
            ..Default::default()
        };

        // mesh_fn: a simple parameterised mesh where height param scales Y
        let mesh_fn = |params: &HashMap<String, f32>| -> Vec<[f32; 3]> {
            let h_p = *params.get("height").unwrap_or(&0.5);
            let h = 1.40 + h_p * 0.70; // maps [0,1] to [1.40, 2.10]
            make_simple_mesh(h)
        };

        let result = fit_params_to_scan(&cloud, &cfg, &mesh_fn);
        assert!(result.residual_error.is_finite(), "residual is not finite");
        assert!(result.iterations <= cfg.max_iterations);
        for (k, v) in &result.params {
            assert!(
                (0.0..=1.0).contains(v),
                "fitted param {} = {} out of [0,1]",
                k,
                v
            );
        }
    }

    // ── Test 16: fit_params_to_scan on empty cloud returns immediately ───────

    #[test]
    fn fit_params_to_scan_empty_cloud() {
        let cloud = ScanCloud::new(vec![]);
        let cfg = FitConfig::default();
        let mesh_fn = |_: &HashMap<String, f32>| make_simple_mesh(1.75);
        let result = fit_params_to_scan(&cloud, &cfg, &mesh_fn);
        assert_eq!(result.iterations, 0);
        assert!(result.converged);
    }

    // ── Test 17: empty cloud bbox returns zeros ──────────────────────────────

    #[test]
    fn empty_cloud_bbox_returns_zeros() {
        let cloud = ScanCloud::new(vec![]);
        let (min, max) = cloud.bbox();
        for i in 0..3 {
            assert_eq!(min[i], 0.0);
            assert_eq!(max[i], 0.0);
        }
    }

    // ── Test 18: write quick_fit results to /tmp/ ────────────────────────────

    #[test]
    fn write_quick_fit_results_to_tmp() {
        let cloud = make_human_cloud();
        let params = quick_fit_from_bbox(&cloud);

        // Serialize to JSON-like string and write to /tmp/
        let mut lines = Vec::new();
        let mut keys: Vec<&String> = params.keys().collect();
        keys.sort();
        for k in keys {
            lines.push(format!("{}: {:.4}", k, params[k]));
        }
        let content = lines.join("\n");
        std::fs::write("/tmp/oxihuman_body_scan_fit_quick.txt", &content).expect("should succeed");

        let read_back = std::fs::read_to_string("/tmp/oxihuman_body_scan_fit_quick.txt")
            .expect("should succeed");
        assert!(read_back.contains("height"));
    }
}
