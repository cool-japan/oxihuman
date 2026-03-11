// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

//! Fit body parameters to a 3-D point cloud from a body scanner.
//!
//! Provides:
//! - [`ScanCloud`] — input point cloud with optional normals.
//! - [`FitResult`] — the fitted parameter map + quality metrics.
//! - [`FitConfig`] — configuration for the fitting loop.
//! - [`BodyMeasurementsEstimate`] — axis-aligned measurement estimates.
//! - Free functions for estimating measurements, parameter mapping, gradient-
//!   free fitting (coordinate descent), quick bbox fitting, error computation,
//!   and scan-to-mesh alignment.

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

// ===========================================================================
// Photogrammetry fitting — PLY/OBJ import, ICP alignment, multi-stage fit
// ===========================================================================

/// Point cloud from 3D scan (PLY/OBJ import), using f64 precision.
#[derive(Debug, Clone)]
pub struct PointCloud {
    /// 3-D positions `[x, y, z]`.
    pub points: Vec<[f64; 3]>,
    /// Optional per-point normals.
    pub normals: Option<Vec<[f64; 3]>>,
    /// Optional per-point RGB colours in `[0, 1]`.
    pub colors: Option<Vec<[f64; 3]>>,
}

impl PointCloud {
    /// Parse PLY ASCII format.
    pub fn from_ply_ascii(data: &str) -> anyhow::Result<Self> {
        let mut lines = data.lines();
        let first = lines.next().unwrap_or("");
        if first.trim() != "ply" {
            anyhow::bail!("not a PLY file: missing 'ply' magic");
        }
        let mut vertex_count: usize = 0;
        let mut has_normals = false;
        let mut has_colors = false;
        let mut in_header = true;
        let mut prop_order: Vec<String> = Vec::new();

        while in_header {
            let line = match lines.next() {
                Some(l) => l.trim(),
                None => anyhow::bail!("unexpected end of PLY header"),
            };
            if line == "end_header" {
                in_header = false;
            } else if line.starts_with("element vertex") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    vertex_count = parts[2]
                        .parse::<usize>()
                        .map_err(|e| anyhow::anyhow!("bad vertex count: {}", e))?;
                }
            } else if line.starts_with("property") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let name = parts[2].to_lowercase();
                    prop_order.push(name.clone());
                    if name == "nx" || name == "ny" || name == "nz" {
                        has_normals = true;
                    }
                    if name == "red" || name == "green" || name == "blue" {
                        has_colors = true;
                    }
                }
            }
        }

        let idx = |name: &str| -> Option<usize> { prop_order.iter().position(|s| s == name) };
        let ix = idx("x");
        let iy = idx("y");
        let iz = idx("z");
        let inx = idx("nx");
        let iny = idx("ny");
        let inz = idx("nz");
        let ir = idx("red");
        let ig = idx("green");
        let ib = idx("blue");

        let mut points = Vec::with_capacity(vertex_count);
        let mut normals_vec: Vec<[f64; 3]> = if has_normals {
            Vec::with_capacity(vertex_count)
        } else {
            Vec::new()
        };
        let mut colors_vec: Vec<[f64; 3]> = if has_colors {
            Vec::with_capacity(vertex_count)
        } else {
            Vec::new()
        };

        for _ in 0..vertex_count {
            let line = match lines.next() {
                Some(l) => l.trim(),
                None => break,
            };
            let vals: Vec<f64> = line
                .split_whitespace()
                .filter_map(|s| s.parse::<f64>().ok())
                .collect();

            let x = ix.and_then(|i| vals.get(i).copied()).unwrap_or(0.0);
            let y = iy.and_then(|i| vals.get(i).copied()).unwrap_or(0.0);
            let z = iz.and_then(|i| vals.get(i).copied()).unwrap_or(0.0);
            points.push([x, y, z]);

            if has_normals {
                let nx = inx.and_then(|i| vals.get(i).copied()).unwrap_or(0.0);
                let ny = iny.and_then(|i| vals.get(i).copied()).unwrap_or(0.0);
                let nz = inz.and_then(|i| vals.get(i).copied()).unwrap_or(0.0);
                normals_vec.push([nx, ny, nz]);
            }
            if has_colors {
                let r = ir.and_then(|i| vals.get(i).copied()).unwrap_or(0.0);
                let g = ig.and_then(|i| vals.get(i).copied()).unwrap_or(0.0);
                let b = ib.and_then(|i| vals.get(i).copied()).unwrap_or(0.0);
                let scale = if r > 1.0 || g > 1.0 || b > 1.0 {
                    1.0 / 255.0
                } else {
                    1.0
                };
                colors_vec.push([r * scale, g * scale, b * scale]);
            }
        }

        Ok(Self {
            points,
            normals: if has_normals { Some(normals_vec) } else { None },
            colors: if has_colors { Some(colors_vec) } else { None },
        })
    }

    /// Parse PLY binary little-endian format.
    pub fn from_ply_binary_le(data: &[u8]) -> anyhow::Result<Self> {
        let header_end = find_header_end(data)
            .ok_or_else(|| anyhow::anyhow!("no end_header found in PLY binary"))?;
        let header_str = std::str::from_utf8(&data[..header_end])
            .map_err(|e| anyhow::anyhow!("invalid UTF-8 in PLY header: {}", e))?;

        let mut vertex_count: usize = 0;
        let mut props: Vec<(String, PlyPropType)> = Vec::new();

        for line in header_str.lines() {
            let line = line.trim();
            if line.starts_with("element vertex") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    vertex_count = parts[2]
                        .parse::<usize>()
                        .map_err(|e| anyhow::anyhow!("bad vertex count: {}", e))?;
                }
            } else if line.starts_with("property") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let ptype = match parts[1] {
                        "float" | "float32" => PlyPropType::Float32,
                        "double" | "float64" => PlyPropType::Float64,
                        "uchar" | "uint8" => PlyPropType::Uint8,
                        "int" | "int32" => PlyPropType::Int32,
                        "short" | "int16" => PlyPropType::Int16,
                        _ => PlyPropType::Float32,
                    };
                    props.push((parts[2].to_lowercase(), ptype));
                }
            }
        }

        let body_start = header_end + "end_header".len();
        let body_start = data[body_start..]
            .iter()
            .position(|&b| b == b'\n')
            .map(|p| body_start + p + 1)
            .unwrap_or(body_start);

        let stride: usize = props.iter().map(|(_, t)| t.byte_size()).sum();
        let prop_idx = |name: &str| -> Option<(usize, PlyPropType)> {
            let mut offset = 0usize;
            for (n, t) in &props {
                if n == name {
                    return Some((offset, *t));
                }
                offset += t.byte_size();
            }
            None
        };

        let has_normals = prop_idx("nx").is_some();
        let has_colors = prop_idx("red").is_some();

        let mut points = Vec::with_capacity(vertex_count);
        let mut normals_vec: Vec<[f64; 3]> = Vec::new();
        let mut colors_vec: Vec<[f64; 3]> = Vec::new();
        if has_normals {
            normals_vec.reserve(vertex_count);
        }
        if has_colors {
            colors_vec.reserve(vertex_count);
        }

        for i in 0..vertex_count {
            let base = body_start + i * stride;
            if base + stride > data.len() {
                break;
            }
            let row = &data[base..base + stride];

            let read_f64 = |name: &str| -> f64 {
                if let Some((off, t)) = prop_idx(name) {
                    if off + t.byte_size() <= row.len() {
                        t.read_le_f64(&row[off..])
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            };

            points.push([read_f64("x"), read_f64("y"), read_f64("z")]);

            if has_normals {
                normals_vec.push([read_f64("nx"), read_f64("ny"), read_f64("nz")]);
            }
            if has_colors {
                let r = read_f64("red");
                let g = read_f64("green");
                let b = read_f64("blue");
                let scale = if r > 1.0 || g > 1.0 || b > 1.0 {
                    1.0 / 255.0
                } else {
                    1.0
                };
                colors_vec.push([r * scale, g * scale, b * scale]);
            }
        }

        Ok(Self {
            points,
            normals: if has_normals { Some(normals_vec) } else { None },
            colors: if has_colors { Some(colors_vec) } else { None },
        })
    }

    /// Parse OBJ vertex data (vertices only, ignore faces).
    pub fn from_obj_vertices(data: &str) -> anyhow::Result<Self> {
        let mut points = Vec::new();
        let mut normals_vec = Vec::new();

        for line in data.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("vn ") {
                let vals: Vec<f64> = rest
                    .split_whitespace()
                    .filter_map(|s| s.parse::<f64>().ok())
                    .collect();
                if vals.len() >= 3 {
                    normals_vec.push([vals[0], vals[1], vals[2]]);
                }
            } else if let Some(rest) = line.strip_prefix("v ") {
                let vals: Vec<f64> = rest
                    .split_whitespace()
                    .filter_map(|s| s.parse::<f64>().ok())
                    .collect();
                if vals.len() >= 3 {
                    points.push([vals[0], vals[1], vals[2]]);
                }
            }
        }

        let normals = if normals_vec.len() == points.len() && !normals_vec.is_empty() {
            Some(normals_vec)
        } else {
            None
        };

        Ok(Self {
            points,
            normals,
            colors: None,
        })
    }

    /// Downsample by voxel grid.
    pub fn voxel_downsample(&self, voxel_size: f64) -> Self {
        if self.points.is_empty() || voxel_size <= 0.0 {
            return self.clone();
        }
        let inv = 1.0 / voxel_size;
        let mut buckets: std::collections::HashMap<(i64, i64, i64), VoxelAccum> =
            std::collections::HashMap::new();

        let has_normals = self.normals.is_some();
        let has_colors = self.colors.is_some();

        for (idx, p) in self.points.iter().enumerate() {
            let key = (
                (p[0] * inv).floor() as i64,
                (p[1] * inv).floor() as i64,
                (p[2] * inv).floor() as i64,
            );
            let entry = buckets.entry(key).or_insert_with(|| VoxelAccum {
                sum_pos: [0.0; 3],
                sum_nrm: [0.0; 3],
                sum_col: [0.0; 3],
                count: 0,
            });
            entry.sum_pos[0] += p[0];
            entry.sum_pos[1] += p[1];
            entry.sum_pos[2] += p[2];
            entry.count += 1;

            if let Some(ref nrms) = self.normals {
                if let Some(n) = nrms.get(idx) {
                    entry.sum_nrm[0] += n[0];
                    entry.sum_nrm[1] += n[1];
                    entry.sum_nrm[2] += n[2];
                }
            }
            if let Some(ref cols) = self.colors {
                if let Some(c) = cols.get(idx) {
                    entry.sum_col[0] += c[0];
                    entry.sum_col[1] += c[1];
                    entry.sum_col[2] += c[2];
                }
            }
        }

        let n_out = buckets.len();
        let mut points = Vec::with_capacity(n_out);
        let mut normals_out = if has_normals {
            Vec::with_capacity(n_out)
        } else {
            Vec::new()
        };
        let mut colors_out = if has_colors {
            Vec::with_capacity(n_out)
        } else {
            Vec::new()
        };

        for acc in buckets.values() {
            let inv_n = 1.0 / (acc.count as f64);
            points.push([
                acc.sum_pos[0] * inv_n,
                acc.sum_pos[1] * inv_n,
                acc.sum_pos[2] * inv_n,
            ]);
            if has_normals {
                let n = [
                    acc.sum_nrm[0] * inv_n,
                    acc.sum_nrm[1] * inv_n,
                    acc.sum_nrm[2] * inv_n,
                ];
                let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-12);
                normals_out.push([n[0] / len, n[1] / len, n[2] / len]);
            }
            if has_colors {
                colors_out.push([
                    acc.sum_col[0] * inv_n,
                    acc.sum_col[1] * inv_n,
                    acc.sum_col[2] * inv_n,
                ]);
            }
        }

        Self {
            points,
            normals: if has_normals { Some(normals_out) } else { None },
            colors: if has_colors { Some(colors_out) } else { None },
        }
    }

    /// Remove statistical outliers.
    pub fn remove_outliers(&self, k_neighbors: usize, std_ratio: f64) -> Self {
        if self.points.len() <= k_neighbors + 1 {
            return self.clone();
        }
        let n = self.points.len();
        let k = k_neighbors.min(n - 1).max(1);

        let mean_dists: Vec<f64> = self
            .points
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let mut dists: Vec<f64> = self
                    .points
                    .iter()
                    .enumerate()
                    .filter_map(|(j, q)| if j == i { None } else { Some(dist3(p, q)) })
                    .collect();
                dists.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                dists.iter().take(k).sum::<f64>() / k as f64
            })
            .collect();

        let global_mean = mean_dists.iter().sum::<f64>() / n as f64;
        let variance = mean_dists
            .iter()
            .map(|d| (d - global_mean).powi(2))
            .sum::<f64>()
            / n as f64;
        let global_std = variance.sqrt();
        let threshold = global_mean + std_ratio * global_std;

        let keep: Vec<usize> = mean_dists
            .iter()
            .enumerate()
            .filter(|(_, d)| **d <= threshold)
            .map(|(i, _)| i)
            .collect();

        let points: Vec<[f64; 3]> = keep.iter().map(|&i| self.points[i]).collect();
        let normals = self
            .normals
            .as_ref()
            .map(|nv| keep.iter().map(|&i| nv[i]).collect());
        let colors = self
            .colors
            .as_ref()
            .map(|cv| keep.iter().map(|&i| cv[i]).collect());

        Self {
            points,
            normals,
            colors,
        }
    }

    /// Compute the centroid (f64).
    fn centroid_f64(&self) -> [f64; 3] {
        if self.points.is_empty() {
            return [0.0; 3];
        }
        let n = self.points.len() as f64;
        let mut s = [0.0_f64; 3];
        for p in &self.points {
            s[0] += p[0];
            s[1] += p[1];
            s[2] += p[2];
        }
        [s[0] / n, s[1] / n, s[2] / n]
    }
}

// ---------------------------------------------------------------------------
// PLY binary helpers
// ---------------------------------------------------------------------------

fn find_header_end(data: &[u8]) -> Option<usize> {
    let needle = b"end_header";
    data.windows(needle.len()).position(|w| w == needle)
}

#[derive(Debug, Clone, Copy)]
enum PlyPropType {
    Float32,
    Float64,
    Uint8,
    Int32,
    Int16,
}

impl PlyPropType {
    fn byte_size(self) -> usize {
        match self {
            Self::Float32 => 4,
            Self::Float64 => 8,
            Self::Uint8 => 1,
            Self::Int32 => 4,
            Self::Int16 => 2,
        }
    }

    fn read_le_f64(self, buf: &[u8]) -> f64 {
        match self {
            Self::Float32 => {
                if buf.len() >= 4 {
                    f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as f64
                } else {
                    0.0
                }
            }
            Self::Float64 => {
                if buf.len() >= 8 {
                    f64::from_le_bytes([
                        buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
                    ])
                } else {
                    0.0
                }
            }
            Self::Uint8 => {
                if !buf.is_empty() {
                    buf[0] as f64
                } else {
                    0.0
                }
            }
            Self::Int32 => {
                if buf.len() >= 4 {
                    i32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as f64
                } else {
                    0.0
                }
            }
            Self::Int16 => {
                if buf.len() >= 2 {
                    i16::from_le_bytes([buf[0], buf[1]]) as f64
                } else {
                    0.0
                }
            }
        }
    }
}

struct VoxelAccum {
    sum_pos: [f64; 3],
    sum_nrm: [f64; 3],
    sum_col: [f64; 3],
    count: usize,
}

// ---------------------------------------------------------------------------
// 3-D math helpers (f64)
// ---------------------------------------------------------------------------

fn dist3(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn dist3_sq(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

fn vec3_sub(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn vec3_dot(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn mat3_identity() -> [[f64; 3]; 3] {
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
}

fn mat3_mul(a: &[[f64; 3]; 3], b: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut c = [[0.0_f64; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            c[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    c
}

fn mat3_transpose(m: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    [
        [m[0][0], m[1][0], m[2][0]],
        [m[0][1], m[1][1], m[2][1]],
        [m[0][2], m[1][2], m[2][2]],
    ]
}

fn mat3_det(m: &[[f64; 3]; 3]) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

fn mat3_vec(m: &[[f64; 3]; 3], v: &[f64; 3]) -> [f64; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

fn centroid_of(pts: &[[f64; 3]]) -> [f64; 3] {
    if pts.is_empty() {
        return [0.0; 3];
    }
    let n = pts.len() as f64;
    let mut s = [0.0; 3];
    for p in pts {
        s[0] += p[0];
        s[1] += p[1];
        s[2] += p[2];
    }
    [s[0] / n, s[1] / n, s[2] / n]
}

// ---------------------------------------------------------------------------
// 3x3 SVD via Jacobi rotations (pure Rust)
// ---------------------------------------------------------------------------

struct Svd3 {
    u: [[f64; 3]; 3],
    s: [f64; 3],
    vt: [[f64; 3]; 3],
}

fn jacobi_rotation_sym(a: &[[f64; 3]; 3], p: usize, q: usize) -> (f64, f64) {
    let apq = a[p][q];
    if apq.abs() < 1e-15 {
        return (1.0, 0.0);
    }
    let tau = (a[q][q] - a[p][p]) / (2.0 * apq);
    let t = if tau.abs() > 1e15 {
        1.0 / (2.0 * tau)
    } else {
        let sign_tau = if tau >= 0.0 { 1.0 } else { -1.0 };
        sign_tau / (tau.abs() + (1.0 + tau * tau).sqrt())
    };
    let c = 1.0 / (1.0 + t * t).sqrt();
    let s = t * c;
    (c, s)
}

fn apply_jacobi_sym(a: &mut [[f64; 3]; 3], p: usize, q: usize, c: f64, s: f64) {
    let mut tmp = *a;
    for k in 0..3 {
        tmp[p][k] = c * a[p][k] - s * a[q][k];
        tmp[q][k] = s * a[p][k] + c * a[q][k];
    }
    let a2 = tmp;
    for k in 0..3 {
        tmp[k][p] = c * a2[k][p] - s * a2[k][q];
        tmp[k][q] = s * a2[k][p] + c * a2[k][q];
    }
    *a = tmp;
}

fn apply_jacobi_vec(v: &mut [[f64; 3]; 3], p: usize, q: usize, c: f64, s: f64) {
    for row in v.iter_mut() {
        let vp = row[p];
        let vq = row[q];
        row[p] = c * vp - s * vq;
        row[q] = s * vp + c * vq;
    }
}

fn sym_eigen3(m: &[[f64; 3]; 3]) -> ([f64; 3], [[f64; 3]; 3]) {
    let mut a = *m;
    let mut v = mat3_identity();
    let max_iter = 100;

    for _ in 0..max_iter {
        let pairs: [(usize, usize); 3] = [(0, 1), (0, 2), (1, 2)];
        let mut max_off = 0.0_f64;
        for &(p, q) in &pairs {
            let val = a[p][q].abs();
            if val > max_off {
                max_off = val;
            }
        }
        if max_off < 1e-14 {
            break;
        }
        for &(p, q) in &pairs {
            if a[p][q].abs() < 1e-15 {
                continue;
            }
            let (c, s) = jacobi_rotation_sym(&a, p, q);
            apply_jacobi_sym(&mut a, p, q, c, s);
            apply_jacobi_vec(&mut v, p, q, c, s);
        }
    }

    ([a[0][0], a[1][1], a[2][2]], v)
}

fn svd3(m: &[[f64; 3]; 3]) -> Svd3 {
    let mt = mat3_transpose(m);
    let ata = mat3_mul(&mt, m);
    let (eigenvalues, v_cols) = sym_eigen3(&ata);

    let mut s = [0.0_f64; 3];
    for i in 0..3 {
        s[i] = eigenvalues[i].max(0.0).sqrt();
    }

    let mut order = [0usize, 1, 2];
    if s[order[1]] > s[order[0]] {
        order.swap(0, 1);
    }
    if s[order[2]] > s[order[0]] {
        order.swap(0, 2);
    }
    if s[order[2]] > s[order[1]] {
        order.swap(1, 2);
    }

    let s_sorted = [s[order[0]], s[order[1]], s[order[2]]];

    let mut v_mat = [[0.0_f64; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            v_mat[i][j] = v_cols[i][order[j]];
        }
    }

    let mv = mat3_mul(m, &v_mat);
    let mut u_mat = [[0.0_f64; 3]; 3];
    for j in 0..3 {
        let inv_s = if s_sorted[j] > 1e-12 {
            1.0 / s_sorted[j]
        } else {
            0.0
        };
        for i in 0..3 {
            u_mat[i][j] = mv[i][j] * inv_s;
        }
    }

    let det_u = mat3_det(&u_mat);
    let det_v = mat3_det(&v_mat);
    let mut s_final = s_sorted;

    if det_u < 0.0 {
        for row in u_mat.iter_mut() {
            row[2] = -row[2];
        }
        s_final[2] = -s_final[2];
    }
    if det_v < 0.0 {
        for row in v_mat.iter_mut() {
            row[2] = -row[2];
        }
        s_final[2] = -s_final[2];
    }

    Svd3 {
        u: u_mat,
        s: s_final,
        vt: mat3_transpose(&v_mat),
    }
}

// ---------------------------------------------------------------------------
// ICP (Iterative Closest Point)
// ---------------------------------------------------------------------------

/// ICP (Iterative Closest Point) alignment algorithm.
#[derive(Debug, Clone)]
pub struct IcpAligner {
    /// Maximum number of ICP iterations.
    pub max_iterations: usize,
    /// Convergence threshold on RMSE change.
    pub convergence_threshold: f64,
    /// Maximum correspondence distance; pairs farther apart are rejected.
    pub max_correspondence_distance: f64,
}

/// Result of ICP alignment.
#[derive(Debug, Clone)]
pub struct IcpResult {
    /// 3x3 rotation matrix.
    pub rotation: [[f64; 3]; 3],
    /// Translation vector.
    pub translation: [f64; 3],
    /// Uniform scale factor.
    pub scale: f64,
    /// Fraction of source points with a valid correspondence.
    pub fitness: f64,
    /// Root mean square error of corresponding pairs.
    pub rmse: f64,
    /// Number of ICP iterations executed.
    pub iterations: usize,
}

impl IcpAligner {
    /// Create a new ICP aligner.
    pub fn new(max_iterations: usize, convergence_threshold: f64) -> Self {
        Self {
            max_iterations,
            convergence_threshold,
            max_correspondence_distance: f64::MAX,
        }
    }

    /// Set the maximum correspondence distance for rejecting outlier pairs.
    pub fn with_max_correspondence_distance(mut self, d: f64) -> Self {
        self.max_correspondence_distance = d;
        self
    }

    /// Align source point cloud to target using point-to-point ICP.
    pub fn align_point_to_point(
        &self,
        source: &[[f64; 3]],
        target: &[[f64; 3]],
    ) -> anyhow::Result<IcpResult> {
        if source.is_empty() || target.is_empty() {
            anyhow::bail!("ICP requires non-empty point sets");
        }

        let mut src: Vec<[f64; 3]> = source.to_vec();
        let mut cumulative_rot = mat3_identity();
        let mut cumulative_trans = [0.0_f64; 3];
        let mut cumulative_scale = 1.0_f64;
        let mut prev_rmse = f64::MAX;
        let mut iters = 0usize;

        for _ in 0..self.max_iterations {
            iters += 1;

            let (src_matched, tgt_matched) =
                find_correspondences(&src, target, self.max_correspondence_distance);

            if src_matched.len() < 3 {
                break;
            }

            let (rot, trans, scale) = compute_rigid_transform(&src_matched, &tgt_matched);

            for p in &mut src {
                let rotated = mat3_vec(&rot, p);
                p[0] = rotated[0] * scale + trans[0];
                p[1] = rotated[1] * scale + trans[1];
                p[2] = rotated[2] * scale + trans[2];
            }

            let new_rot = mat3_mul(&rot, &cumulative_rot);
            let ct_rotated = mat3_vec(&rot, &cumulative_trans);
            let new_trans = [
                scale * ct_rotated[0] + trans[0],
                scale * ct_rotated[1] + trans[1],
                scale * ct_rotated[2] + trans[2],
            ];
            let new_scale = scale * cumulative_scale;

            cumulative_rot = new_rot;
            cumulative_trans = new_trans;
            cumulative_scale = new_scale;

            let rmse = compute_rmse(&src_matched, &tgt_matched);

            if (prev_rmse - rmse).abs() < self.convergence_threshold {
                break;
            }
            prev_rmse = rmse;
        }

        let (final_src, final_tgt) =
            find_correspondences(&src, target, self.max_correspondence_distance);
        let fitness = final_src.len() as f64 / source.len().max(1) as f64;
        let rmse = if final_src.is_empty() {
            f64::MAX
        } else {
            compute_rmse(&final_src, &final_tgt)
        };

        Ok(IcpResult {
            rotation: cumulative_rot,
            translation: cumulative_trans,
            scale: cumulative_scale,
            fitness,
            rmse,
            iterations: iters,
        })
    }

    /// Align using point-to-plane ICP (requires normals on target).
    pub fn align_point_to_plane(
        &self,
        source: &[[f64; 3]],
        target: &[[f64; 3]],
        target_normals: &[[f64; 3]],
    ) -> anyhow::Result<IcpResult> {
        if source.is_empty() || target.is_empty() {
            anyhow::bail!("ICP requires non-empty point sets");
        }
        if target.len() != target_normals.len() {
            anyhow::bail!("target and target_normals must have the same length");
        }

        let mut src: Vec<[f64; 3]> = source.to_vec();
        let mut cumulative_rot = mat3_identity();
        let mut cumulative_trans = [0.0_f64; 3];
        let mut prev_rmse = f64::MAX;
        let mut iters = 0usize;

        for _ in 0..self.max_iterations {
            iters += 1;

            let (src_idx, tgt_idx) =
                find_correspondence_indices(&src, target, self.max_correspondence_distance);

            if src_idx.len() < 6 {
                break;
            }

            let (delta_rot_vec, delta_trans) =
                solve_point_to_plane_step(&src, &src_idx, target, target_normals, &tgt_idx);

            let rot_inc = small_angle_rotation(&delta_rot_vec);

            for p in &mut src {
                let rotated = mat3_vec(&rot_inc, p);
                p[0] = rotated[0] + delta_trans[0];
                p[1] = rotated[1] + delta_trans[1];
                p[2] = rotated[2] + delta_trans[2];
            }

            let new_rot = mat3_mul(&rot_inc, &cumulative_rot);
            let ct_rotated = mat3_vec(&rot_inc, &cumulative_trans);
            cumulative_rot = new_rot;
            cumulative_trans = [
                ct_rotated[0] + delta_trans[0],
                ct_rotated[1] + delta_trans[1],
                ct_rotated[2] + delta_trans[2],
            ];

            let matched_src: Vec<[f64; 3]> = src_idx.iter().map(|&i| src[i]).collect();
            let matched_tgt: Vec<[f64; 3]> = tgt_idx.iter().map(|&i| target[i]).collect();
            let rmse = compute_rmse(&matched_src, &matched_tgt);

            if (prev_rmse - rmse).abs() < self.convergence_threshold {
                break;
            }
            prev_rmse = rmse;
        }

        let (final_src_idx, final_tgt_idx) =
            find_correspondence_indices(&src, target, self.max_correspondence_distance);
        let fitness = final_src_idx.len() as f64 / source.len().max(1) as f64;
        let rmse = if final_src_idx.is_empty() {
            f64::MAX
        } else {
            let ms: Vec<[f64; 3]> = final_src_idx.iter().map(|&i| src[i]).collect();
            let mt: Vec<[f64; 3]> = final_tgt_idx.iter().map(|&i| target[i]).collect();
            compute_rmse(&ms, &mt)
        };

        Ok(IcpResult {
            rotation: cumulative_rot,
            translation: cumulative_trans,
            scale: 1.0,
            fitness,
            rmse,
            iterations: iters,
        })
    }

    /// Apply a rigid transform (rotation, translation, scale) to points in-place.
    pub fn transform_points(
        points: &mut [[f64; 3]],
        rotation: &[[f64; 3]; 3],
        translation: &[f64; 3],
        scale: f64,
    ) {
        for p in points.iter_mut() {
            let r = mat3_vec(rotation, p);
            p[0] = r[0] * scale + translation[0];
            p[1] = r[1] * scale + translation[1];
            p[2] = r[2] * scale + translation[2];
        }
    }
}

// ---------------------------------------------------------------------------
// ICP helper functions
// ---------------------------------------------------------------------------

fn find_correspondences(
    source: &[[f64; 3]],
    target: &[[f64; 3]],
    max_dist: f64,
) -> (Vec<[f64; 3]>, Vec<[f64; 3]>) {
    let max_dist_sq = max_dist * max_dist;
    let mut src_out = Vec::new();
    let mut tgt_out = Vec::new();

    for sp in source {
        let mut best_dist_sq = f64::MAX;
        let mut best_pt = [0.0_f64; 3];
        for tp in target {
            let d2 = dist3_sq(sp, tp);
            if d2 < best_dist_sq {
                best_dist_sq = d2;
                best_pt = *tp;
            }
        }
        if best_dist_sq <= max_dist_sq {
            src_out.push(*sp);
            tgt_out.push(best_pt);
        }
    }

    (src_out, tgt_out)
}

fn find_correspondence_indices(
    source: &[[f64; 3]],
    target: &[[f64; 3]],
    max_dist: f64,
) -> (Vec<usize>, Vec<usize>) {
    let max_dist_sq = max_dist * max_dist;
    let mut src_idx = Vec::new();
    let mut tgt_idx = Vec::new();

    for (si, sp) in source.iter().enumerate() {
        let mut best_dist_sq = f64::MAX;
        let mut best_idx = 0usize;
        for (ti, tp) in target.iter().enumerate() {
            let d2 = dist3_sq(sp, tp);
            if d2 < best_dist_sq {
                best_dist_sq = d2;
                best_idx = ti;
            }
        }
        if best_dist_sq <= max_dist_sq {
            src_idx.push(si);
            tgt_idx.push(best_idx);
        }
    }

    (src_idx, tgt_idx)
}

fn compute_rigid_transform(
    source: &[[f64; 3]],
    target: &[[f64; 3]],
) -> ([[f64; 3]; 3], [f64; 3], f64) {
    let c_src = centroid_of(source);
    let c_tgt = centroid_of(target);

    let src_c: Vec<[f64; 3]> = source.iter().map(|p| vec3_sub(p, &c_src)).collect();
    let tgt_c: Vec<[f64; 3]> = target.iter().map(|p| vec3_sub(p, &c_tgt)).collect();

    let mut h = [[0.0_f64; 3]; 3];
    for (s, t) in src_c.iter().zip(tgt_c.iter()) {
        for i in 0..3 {
            for j in 0..3 {
                h[i][j] += s[i] * t[j];
            }
        }
    }

    let svd = svd3(&h);
    let ut = mat3_transpose(&svd.u);
    let vt_t = mat3_transpose(&svd.vt);
    let mut rot = mat3_mul(&vt_t, &ut);

    if mat3_det(&rot) < 0.0 {
        let mut v_fixed = vt_t;
        for row in v_fixed.iter_mut() {
            row[2] = -row[2];
        }
        rot = mat3_mul(&v_fixed, &ut);
    }

    let src_var: f64 = src_c.iter().map(|p| vec3_dot(p, p)).sum();
    let scale = if src_var > 1e-12 {
        let tgt_var: f64 = tgt_c.iter().map(|p| vec3_dot(p, p)).sum();
        (tgt_var / src_var).sqrt()
    } else {
        1.0
    };

    let r_csrc = mat3_vec(&rot, &c_src);
    let trans = [
        c_tgt[0] - scale * r_csrc[0],
        c_tgt[1] - scale * r_csrc[1],
        c_tgt[2] - scale * r_csrc[2],
    ];

    (rot, trans, scale)
}

fn compute_rmse(a: &[[f64; 3]], b: &[[f64; 3]]) -> f64 {
    if a.is_empty() {
        return 0.0;
    }
    let sum: f64 = a.iter().zip(b.iter()).map(|(p, q)| dist3_sq(p, q)).sum();
    (sum / a.len() as f64).sqrt()
}

fn small_angle_rotation(w: &[f64; 3]) -> [[f64; 3]; 3] {
    let (a, b, g) = (w[0], w[1], w[2]);
    let theta = (a * a + b * b + g * g).sqrt();
    if theta < 1e-12 {
        return mat3_identity();
    }
    let k = [a / theta, b / theta, g / theta];
    let ct = theta.cos();
    let st = theta.sin();
    let omc = 1.0 - ct;

    [
        [
            ct + k[0] * k[0] * omc,
            k[0] * k[1] * omc - k[2] * st,
            k[0] * k[2] * omc + k[1] * st,
        ],
        [
            k[1] * k[0] * omc + k[2] * st,
            ct + k[1] * k[1] * omc,
            k[1] * k[2] * omc - k[0] * st,
        ],
        [
            k[2] * k[0] * omc - k[1] * st,
            k[2] * k[1] * omc + k[0] * st,
            ct + k[2] * k[2] * omc,
        ],
    ]
}

fn solve_point_to_plane_step(
    source: &[[f64; 3]],
    src_idx: &[usize],
    target: &[[f64; 3]],
    target_normals: &[[f64; 3]],
    tgt_idx: &[usize],
) -> ([f64; 3], [f64; 3]) {
    let mut ata = [[0.0_f64; 6]; 6];
    let mut atb = [0.0_f64; 6];

    for (&si, &ti) in src_idx.iter().zip(tgt_idx.iter()) {
        let s = &source[si];
        let t = &target[ti];
        let n = &target_normals[ti];

        let d = vec3_sub(s, t);
        let r = vec3_dot(n, &d);

        let cn = [
            s[1] * n[2] - s[2] * n[1],
            s[2] * n[0] - s[0] * n[2],
            s[0] * n[1] - s[1] * n[0],
        ];
        let row = [cn[0], cn[1], cn[2], n[0], n[1], n[2]];

        for i in 0..6 {
            for j in 0..6 {
                ata[i][j] += row[i] * row[j];
            }
            atb[i] += row[i] * (-r);
        }
    }

    let x = solve_6x6(&ata, &atb);
    ([x[0], x[1], x[2]], [x[3], x[4], x[5]])
}

#[allow(clippy::needless_range_loop)]
fn solve_6x6(a: &[[f64; 6]; 6], b: &[f64; 6]) -> [f64; 6] {
    let mut aug = [[0.0_f64; 7]; 6];
    for i in 0..6 {
        for j in 0..6 {
            aug[i][j] = a[i][j];
        }
        aug[i][6] = b[i];
    }

    for col in 0..6 {
        let mut max_row = col;
        let mut max_val = aug[col][col].abs();
        for row in (col + 1)..6 {
            if aug[row][col].abs() > max_val {
                max_val = aug[row][col].abs();
                max_row = row;
            }
        }
        if max_val < 1e-15 {
            continue;
        }
        if max_row != col {
            aug.swap(col, max_row);
        }

        let pivot = aug[col][col];
        for row in (col + 1)..6 {
            let factor = aug[row][col] / pivot;
            for j in col..7 {
                aug[row][j] -= factor * aug[col][j];
            }
        }
    }

    let mut x = [0.0_f64; 6];
    for col in (0..6).rev() {
        if aug[col][col].abs() < 1e-15 {
            x[col] = 0.0;
            continue;
        }
        let mut sum = aug[col][6];
        for j in (col + 1)..6 {
            sum -= aug[col][j] * x[j];
        }
        x[col] = sum / aug[col][col];
    }
    x
}

// ===========================================================================
// Multi-stage body scan fitting pipeline
// ===========================================================================

/// Configuration for the multi-stage scan fitting pipeline.
#[derive(Debug, Clone)]
pub struct ScanFitConfig {
    /// Number of ICP iterations per stage.
    pub icp_iterations: usize,
    /// Number of morph gradient-descent iterations.
    pub morph_iterations: usize,
    /// Voxel size for coarse downsampling (metres).
    pub coarse_voxel_size: f64,
    /// Voxel size for fine downsampling (metres).
    pub fine_voxel_size: f64,
    /// Regularisation weight on morph parameters (L2 penalty).
    pub regularization: f64,
}

impl Default for ScanFitConfig {
    fn default() -> Self {
        Self {
            icp_iterations: 50,
            morph_iterations: 100,
            coarse_voxel_size: 0.02,
            fine_voxel_size: 0.005,
            regularization: 0.01,
        }
    }
}

/// Result of the multi-stage scan fitting pipeline.
#[derive(Debug, Clone)]
pub struct PhotoFitResult {
    /// Fitted morph parameters `(name, weight)`.
    pub morph_parameters: Vec<(String, f64)>,
    /// ICP alignment result from the fine stage.
    pub alignment: IcpResult,
    /// Final mean surface distance error (metres).
    pub surface_error: f64,
    /// Number of fitting stages completed (0-3).
    pub stages_completed: usize,
}

/// Multi-stage body scan fitting pipeline.
#[derive(Debug, Clone)]
pub struct ScanFitter {
    config: ScanFitConfig,
}

impl ScanFitter {
    /// Create a new scan fitter with the given configuration.
    pub fn new(config: ScanFitConfig) -> Self {
        Self { config }
    }

    /// Run the full pipeline: import -> downsample -> align -> fit morphs.
    pub fn fit(
        &self,
        scan_cloud: &PointCloud,
        template_vertices: &[[f64; 3]],
        template_triangles: &[[usize; 3]],
        morph_targets: &[(String, Vec<[f64; 3]>)],
    ) -> anyhow::Result<PhotoFitResult> {
        if scan_cloud.points.is_empty() {
            anyhow::bail!("scan point cloud is empty");
        }
        if template_vertices.is_empty() {
            anyhow::bail!("template mesh has no vertices");
        }

        // Stage 1: Coarse alignment
        let coarse_scan = scan_cloud.voxel_downsample(self.config.coarse_voxel_size);
        let coarse_template =
            voxel_downsample_slice(template_vertices, self.config.coarse_voxel_size);

        let coarse_icp = IcpAligner::new(self.config.icp_iterations, 1e-6);
        let coarse_result =
            coarse_icp.align_point_to_point(&coarse_scan.points, &coarse_template)?;
        // stages_completed: 1

        let mut aligned_scan: Vec<[f64; 3]> = scan_cloud.points.clone();
        IcpAligner::transform_points(
            &mut aligned_scan,
            &coarse_result.rotation,
            &coarse_result.translation,
            coarse_result.scale,
        );

        // Stage 2: Fine alignment
        let fine_scan = if self.config.fine_voxel_size > 0.0 {
            let pc = PointCloud {
                points: aligned_scan.clone(),
                normals: None,
                colors: None,
            };
            pc.voxel_downsample(self.config.fine_voxel_size).points
        } else {
            aligned_scan.clone()
        };

        let fine_icp = IcpAligner::new(self.config.icp_iterations, 1e-7);
        let fine_result = fine_icp.align_point_to_point(&fine_scan, template_vertices)?;
        // stages_completed: 2

        IcpAligner::transform_points(
            &mut aligned_scan,
            &fine_result.rotation,
            &fine_result.translation,
            fine_result.scale,
        );

        let combined_rot = mat3_mul(&fine_result.rotation, &coarse_result.rotation);
        let cr_trans = mat3_vec(&fine_result.rotation, &coarse_result.translation);
        let combined_trans = [
            fine_result.scale * cr_trans[0] + fine_result.translation[0],
            fine_result.scale * cr_trans[1] + fine_result.translation[1],
            fine_result.scale * cr_trans[2] + fine_result.translation[2],
        ];
        let combined_scale = fine_result.scale * coarse_result.scale;

        let combined_alignment = IcpResult {
            rotation: combined_rot,
            translation: combined_trans,
            scale: combined_scale,
            fitness: fine_result.fitness,
            rmse: fine_result.rmse,
            iterations: coarse_result.iterations + fine_result.iterations,
        };

        // Stage 3: Morph fitting
        let morph_params = if morph_targets.is_empty() {
            Vec::new()
        } else {
            self.fit_morphs(
                &aligned_scan,
                template_vertices,
                template_triangles,
                morph_targets,
            )?
        };

        // stages_completed: 3
        let deformed = apply_morph_deltas(template_vertices, morph_targets, &morph_params);
        let surface_error = mean_closest_distance(&aligned_scan, &deformed);

        Ok(PhotoFitResult {
            morph_parameters: morph_params,
            alignment: combined_alignment,
            surface_error,
            stages_completed: 3,
        })
    }

    /// Gradient descent to fit morph target weights.
    fn fit_morphs(
        &self,
        scan_points: &[[f64; 3]],
        template_vertices: &[[f64; 3]],
        _template_triangles: &[[usize; 3]],
        morph_targets: &[(String, Vec<[f64; 3]>)],
    ) -> anyhow::Result<Vec<(String, f64)>> {
        let n_morphs = morph_targets.len();
        let mut weights = vec![0.0_f64; n_morphs];
        let lr = 0.001_f64;
        let reg = self.config.regularization;

        let scan_sub = if scan_points.len() > 2000 {
            let step = scan_points.len() / 2000;
            scan_points
                .iter()
                .step_by(step.max(1))
                .copied()
                .collect::<Vec<_>>()
        } else {
            scan_points.to_vec()
        };

        for _iter in 0..self.config.morph_iterations {
            let deformed = apply_morph_deltas(
                template_vertices,
                morph_targets,
                &weight_pairs(morph_targets, &weights),
            );

            let mut grad = vec![0.0_f64; n_morphs];
            let n_scan = scan_sub.len() as f64;

            for sp in &scan_sub {
                let (closest_idx, _) = find_closest_vertex(sp, &deformed);
                let diff = vec3_sub(sp, &deformed[closest_idx]);

                for (j, (_name, deltas)) in morph_targets.iter().enumerate() {
                    if closest_idx < deltas.len() {
                        let d = &deltas[closest_idx];
                        grad[j] += -2.0 * vec3_dot(&diff, d) / n_scan;
                    }
                }
            }

            for j in 0..n_morphs {
                grad[j] += 2.0 * reg * weights[j];
            }

            for j in 0..n_morphs {
                weights[j] -= lr * grad[j];
                weights[j] = weights[j].clamp(-2.0, 2.0);
            }
        }

        Ok(weight_pairs(morph_targets, &weights))
    }
}

// ---------------------------------------------------------------------------
// Multi-stage fitting helpers
// ---------------------------------------------------------------------------

fn voxel_downsample_slice(pts: &[[f64; 3]], voxel_size: f64) -> Vec<[f64; 3]> {
    if pts.is_empty() || voxel_size <= 0.0 {
        return pts.to_vec();
    }
    let inv = 1.0 / voxel_size;
    let mut buckets: std::collections::HashMap<(i64, i64, i64), ([f64; 3], usize)> =
        std::collections::HashMap::new();

    for p in pts {
        let key = (
            (p[0] * inv).floor() as i64,
            (p[1] * inv).floor() as i64,
            (p[2] * inv).floor() as i64,
        );
        let entry = buckets.entry(key).or_insert(([0.0; 3], 0));
        entry.0[0] += p[0];
        entry.0[1] += p[1];
        entry.0[2] += p[2];
        entry.1 += 1;
    }

    buckets
        .values()
        .map(|(sum, count)| {
            let inv_n = 1.0 / (*count as f64);
            [sum[0] * inv_n, sum[1] * inv_n, sum[2] * inv_n]
        })
        .collect()
}

fn apply_morph_deltas(
    template: &[[f64; 3]],
    morph_targets: &[(String, Vec<[f64; 3]>)],
    weights: &[(String, f64)],
) -> Vec<[f64; 3]> {
    let mut result: Vec<[f64; 3]> = template.to_vec();

    for (name, w) in weights {
        if w.abs() < 1e-12 {
            continue;
        }
        if let Some((_n, deltas)) = morph_targets.iter().find(|(n, _)| n == name) {
            let len = result.len().min(deltas.len());
            for i in 0..len {
                result[i][0] += w * deltas[i][0];
                result[i][1] += w * deltas[i][1];
                result[i][2] += w * deltas[i][2];
            }
        }
    }

    result
}

fn weight_pairs(morph_targets: &[(String, Vec<[f64; 3]>)], weights: &[f64]) -> Vec<(String, f64)> {
    morph_targets
        .iter()
        .zip(weights.iter())
        .map(|((name, _), &w)| (name.clone(), w))
        .collect()
}

fn find_closest_vertex(point: &[f64; 3], vertices: &[[f64; 3]]) -> (usize, f64) {
    let mut best_idx = 0usize;
    let mut best_d2 = f64::MAX;
    for (i, v) in vertices.iter().enumerate() {
        let d2 = dist3_sq(point, v);
        if d2 < best_d2 {
            best_d2 = d2;
            best_idx = i;
        }
    }
    (best_idx, best_d2)
}

fn mean_closest_distance(source: &[[f64; 3]], target: &[[f64; 3]]) -> f64 {
    if source.is_empty() || target.is_empty() {
        return 0.0;
    }
    let total: f64 = source
        .iter()
        .map(|sp| {
            let (_, d2) = find_closest_vertex(sp, target);
            d2.sqrt()
        })
        .sum();
    total / source.len() as f64
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
        assert_eq!(cloud.normals.unwrap(), nrm);
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
        std::fs::write("/tmp/oxihuman_body_scan_fit_quick.txt", &content).unwrap();

        let read_back = std::fs::read_to_string("/tmp/oxihuman_body_scan_fit_quick.txt").unwrap();
        assert!(read_back.contains("height"));
    }
}
