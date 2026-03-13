// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Body calibration workflows — fit a body model to tape measurements.
//!
//! Provides a [`BodyCalibrator`] that uses Nelder–Mead simplex optimisation to
//! find morph-target weights whose resulting mesh best matches a set of
//! [`TapeMeasurement`] entries (circumferences, lengths, widths, depths).

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Domain enums
// ---------------------------------------------------------------------------

/// Body section used for circumference / width / depth measurements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BodySection {
    Neck,
    Chest,
    Underbust,
    Waist,
    Hips,
    UpperArm,
    Forearm,
    Wrist,
    Thigh,
    Knee,
    Calf,
    Ankle,
    Head,
    Shoulder,
}

/// Named anatomical landmarks used for length measurements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BodyLandmark {
    TopOfHead,
    Chin,
    ShoulderLeft,
    ShoulderRight,
    ElbowLeft,
    ElbowRight,
    WristLeft,
    WristRight,
    FingertipLeft,
    FingertipRight,
    Navel,
    Crotch,
    KneeLeft,
    KneeRight,
    AnkleLeft,
    AnkleRight,
    FloorLeft,
    FloorRight,
    NapeOfNeck,
    Acromion,
}

/// The kind of measurement.
#[derive(Debug, Clone, Copy)]
pub enum MeasurementKind {
    /// Circumference around a body section (plane perpendicular to body axis).
    Circumference(BodySection),
    /// Straight-line distance between two landmarks.
    Length(BodyLandmark, BodyLandmark),
    /// Left-right extent of a section.
    Width(BodySection),
    /// Front-back extent of a section.
    Depth(BodySection),
}

// ---------------------------------------------------------------------------
// Tape measurement
// ---------------------------------------------------------------------------

/// A single tape-measure entry used as calibration input.
#[derive(Debug, Clone)]
pub struct TapeMeasurement {
    /// Human-readable name (e.g. "chest_circumference").
    pub name: String,
    /// What kind of measurement this is.
    pub kind: MeasurementKind,
    /// Measured value in centimetres.
    pub value_cm: f64,
    /// Acceptable tolerance in centimetres (used for reporting only).
    pub tolerance_cm: f64,
    /// Importance weight for the optimisation objective.
    pub weight: f64,
}

// ---------------------------------------------------------------------------
// Measurement function
// ---------------------------------------------------------------------------

/// Describes how to compute a measurement from mesh vertices.
#[derive(Debug, Clone)]
pub struct MeasurementFunction {
    /// Name matching the corresponding [`TapeMeasurement::name`].
    pub name: String,
    /// Kind of measurement.
    pub kind: MeasurementKind,
    /// Vertex indices involved in computing this measurement.
    pub vertex_indices: Vec<usize>,
}

// ---------------------------------------------------------------------------
// Configuration & result
// ---------------------------------------------------------------------------

/// Optimiser configuration.
#[derive(Debug, Clone)]
pub struct CalibrationConfig {
    /// Maximum number of Nelder–Mead iterations.
    pub max_iterations: usize,
    /// Stop when simplex diameter falls below this threshold.
    pub convergence_threshold: f64,
    /// Initial perturbation size when constructing the simplex.
    pub learning_rate: f64,
    /// L2 regularisation coefficient on morph weights.
    pub regularization: f64,
}

impl Default for CalibrationConfig {
    fn default() -> Self {
        Self {
            max_iterations: 2000,
            convergence_threshold: 1e-8,
            learning_rate: 0.05,
            regularization: 1e-4,
        }
    }
}

/// Outcome of the calibration procedure.
#[derive(Debug, Clone)]
pub struct CalibrationResult {
    /// Best-fit morph parameters `(name, weight)`.
    pub morph_parameters: Vec<(String, f64)>,
    /// Per-measurement residuals `(name, error_cm)`.
    pub residuals: Vec<(String, f64)>,
    /// Root-mean-square error across all measurements (cm).
    pub total_error: f64,
    /// Iterations consumed.
    pub iterations: usize,
    /// Whether the simplex converged within the threshold.
    pub converged: bool,
}

// ---------------------------------------------------------------------------
// Body calibrator
// ---------------------------------------------------------------------------

/// Fits morph-target weights to match a set of tape measurements.
pub struct BodyCalibrator {
    measurements: Vec<TapeMeasurement>,
    measurement_functions: HashMap<String, MeasurementFunction>,
    config: CalibrationConfig,
}

impl BodyCalibrator {
    /// Create a new calibrator with the given configuration.
    pub fn new(config: CalibrationConfig) -> Self {
        Self {
            measurements: Vec::new(),
            measurement_functions: HashMap::new(),
            config,
        }
    }

    /// Add a tape measurement to the calibration set.
    pub fn add_measurement(&mut self, measurement: TapeMeasurement) {
        self.measurements.push(measurement);
    }

    /// Associate vertex indices with a named measurement.
    pub fn set_measurement_vertices(
        &mut self,
        name: &str,
        vertex_indices: Vec<usize>,
    ) -> anyhow::Result<()> {
        let m = self
            .measurements
            .iter()
            .find(|m| m.name == name)
            .ok_or_else(|| anyhow::anyhow!("measurement '{}' not found", name))?;

        let func = MeasurementFunction {
            name: name.to_string(),
            kind: m.kind,
            vertex_indices,
        };
        self.measurement_functions.insert(name.to_string(), func);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Public measurement geometry helpers
    // -----------------------------------------------------------------------

    /// Compute the perimeter of the cross-section obtained by slicing a set of
    /// vertices with an infinite plane.
    ///
    /// Steps:
    /// 1. Project vertices onto the plane.
    /// 2. Compute the 2-D convex hull of the projected points.
    /// 3. Return the perimeter of the hull (in the same units as the input).
    pub fn compute_circumference(
        vertices: &[[f64; 3]],
        plane_normal: &[f64; 3],
        plane_point: &[f64; 3],
    ) -> anyhow::Result<f64> {
        if vertices.is_empty() {
            return Ok(0.0);
        }

        // Normalise the plane normal.
        let n_len = vec3_len(plane_normal);
        if n_len < 1e-15 {
            return Err(anyhow::anyhow!("plane_normal has near-zero length"));
        }
        let n = [
            plane_normal[0] / n_len,
            plane_normal[1] / n_len,
            plane_normal[2] / n_len,
        ];

        // Build two orthonormal tangent vectors on the plane (u, v).
        let (u, v) = plane_tangent_frame(&n);

        // Project each vertex onto the plane and express in (u, v) coordinates.
        let pts_2d: Vec<[f64; 2]> = vertices
            .iter()
            .map(|vtx| {
                let d = [
                    vtx[0] - plane_point[0],
                    vtx[1] - plane_point[1],
                    vtx[2] - plane_point[2],
                ];
                [dot3(&d, &u), dot3(&d, &v)]
            })
            .collect();

        let hull = convex_hull_2d(&pts_2d);
        if hull.len() < 2 {
            return Ok(0.0);
        }

        let mut perimeter = 0.0_f64;
        for i in 0..hull.len() {
            let j = (i + 1) % hull.len();
            let dx = hull[j][0] - hull[i][0];
            let dy = hull[j][1] - hull[i][1];
            perimeter += (dx * dx + dy * dy).sqrt();
        }
        Ok(perimeter)
    }

    /// Compute the Euclidean distance between the centroids of two vertex
    /// groups.
    pub fn compute_distance(vertices_a: &[[f64; 3]], vertices_b: &[[f64; 3]]) -> f64 {
        let centroid = |vs: &[[f64; 3]]| -> [f64; 3] {
            if vs.is_empty() {
                return [0.0; 3];
            }
            let inv_n = 1.0 / vs.len() as f64;
            let mut c = [0.0_f64; 3];
            for v in vs {
                c[0] += v[0];
                c[1] += v[1];
                c[2] += v[2];
            }
            c[0] *= inv_n;
            c[1] *= inv_n;
            c[2] *= inv_n;
            c
        };
        let ca = centroid(vertices_a);
        let cb = centroid(vertices_b);
        let d = [cb[0] - ca[0], cb[1] - ca[1], cb[2] - ca[2]];
        vec3_len(&d)
    }

    // -----------------------------------------------------------------------
    // Calibration entry point
    // -----------------------------------------------------------------------

    /// Run the Nelder–Mead simplex optimisation to find morph weights that best
    /// reproduce the tape measurements.
    ///
    /// # Arguments
    ///
    /// * `initial_vertices` — base mesh vertex positions `[x, y, z]`.
    /// * `morph_targets` — named morph targets, each a full-mesh displacement
    ///   array of the same length as `initial_vertices`.
    /// * `initial_weights` — starting morph weights (one per morph target).
    pub fn calibrate(
        &self,
        initial_vertices: &[[f64; 3]],
        morph_targets: &[(String, Vec<[f64; 3]>)],
        initial_weights: &[f64],
    ) -> anyhow::Result<CalibrationResult> {
        let n = morph_targets.len();
        if initial_weights.len() != n {
            return Err(anyhow::anyhow!(
                "initial_weights length ({}) != morph_targets count ({})",
                initial_weights.len(),
                n
            ));
        }
        if self.measurements.is_empty() {
            return Err(anyhow::anyhow!("no measurements registered"));
        }

        // Objective function closure ----------------------------------------
        let objective = |weights: &[f64]| -> f64 {
            self.evaluate_objective(initial_vertices, morph_targets, weights)
        };

        // Nelder–Mead -------------------------------------------------------
        let result = nelder_mead(
            &objective,
            initial_weights,
            self.config.learning_rate,
            self.config.max_iterations,
            self.config.convergence_threshold,
        );

        // Build result -------------------------------------------------------
        let best_weights = &result.best_point;
        let deformed = Self::deform_mesh(initial_vertices, morph_targets, best_weights);

        let mut residuals = Vec::with_capacity(self.measurements.len());
        let mut total_sq = 0.0_f64;
        for m in &self.measurements {
            let computed = self.compute_measurement_value(&deformed, &m.name);
            let err = computed - m.value_cm;
            residuals.push((m.name.clone(), err));
            total_sq += err * err;
        }
        let rms = if self.measurements.is_empty() {
            0.0
        } else {
            (total_sq / self.measurements.len() as f64).sqrt()
        };

        let morph_parameters: Vec<(String, f64)> = morph_targets
            .iter()
            .zip(best_weights.iter())
            .map(|((name, _), &w)| (name.clone(), w))
            .collect();

        Ok(CalibrationResult {
            morph_parameters,
            residuals,
            total_error: rms,
            iterations: result.iterations,
            converged: result.converged,
        })
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Evaluate the weighted sum-of-squared-errors objective with optional L2
    /// regularisation on the weights.
    fn evaluate_objective(
        &self,
        base: &[[f64; 3]],
        targets: &[(String, Vec<[f64; 3]>)],
        weights: &[f64],
    ) -> f64 {
        let deformed = Self::deform_mesh(base, targets, weights);

        let mut cost = 0.0_f64;
        for m in &self.measurements {
            let computed = self.compute_measurement_value(&deformed, &m.name);
            let err = computed - m.value_cm;
            cost += m.weight * err * err;
        }

        // L2 regularisation
        if self.config.regularization > 0.0 {
            let reg: f64 = weights.iter().map(|w| w * w).sum();
            cost += self.config.regularization * reg;
        }

        cost
    }

    /// Apply morph-target displacements to the base mesh.
    fn deform_mesh(
        base: &[[f64; 3]],
        targets: &[(String, Vec<[f64; 3]>)],
        weights: &[f64],
    ) -> Vec<[f64; 3]> {
        let mut result: Vec<[f64; 3]> = base.to_vec();
        for (i, (_name, deltas)) in targets.iter().enumerate() {
            let w = weights.get(i).copied().unwrap_or(0.0);
            if w.abs() < 1e-15 {
                continue;
            }
            let len = result.len().min(deltas.len());
            for j in 0..len {
                result[j][0] += w * deltas[j][0];
                result[j][1] += w * deltas[j][1];
                result[j][2] += w * deltas[j][2];
            }
        }
        result
    }

    /// Compute a single named measurement from deformed vertices.
    fn compute_measurement_value(&self, vertices: &[[f64; 3]], name: &str) -> f64 {
        let func = match self.measurement_functions.get(name) {
            Some(f) => f,
            None => return 0.0,
        };

        let selected: Vec<[f64; 3]> = func
            .vertex_indices
            .iter()
            .filter_map(|&idx| vertices.get(idx).copied())
            .collect();

        if selected.is_empty() {
            return 0.0;
        }

        match func.kind {
            MeasurementKind::Circumference(section) => {
                let (normal, point) =
                    Self::circumference_plane_for_section(&selected, section);
                Self::compute_circumference(&selected, &normal, &point).unwrap_or(0.0)
            }
            MeasurementKind::Length(_, _) => {
                // Split vertex set into two halves: first half -> group A, second -> group B.
                let mid = selected.len() / 2;
                let (a, b) = selected.split_at(mid.max(1));
                if b.is_empty() {
                    0.0
                } else {
                    Self::compute_distance(a, b) * 100.0 // metres to cm
                }
            }
            MeasurementKind::Width(_section) => {
                Self::compute_extent(&selected, 0) * 100.0 // x-axis extent
            }
            MeasurementKind::Depth(_section) => {
                Self::compute_extent(&selected, 2) * 100.0 // z-axis extent
            }
        }
    }

    /// Determine the cutting plane for a circumference measurement.
    ///
    /// The plane passes through the centroid of the selected vertices with a
    /// normal aligned to the body axis (Y-up convention).
    fn circumference_plane_for_section(
        vertices: &[[f64; 3]],
        _section: BodySection,
    ) -> ([f64; 3], [f64; 3]) {
        let centroid = Self::centroid(vertices);
        // Y-up: circumference plane has normal along Y.
        ([0.0, 1.0, 0.0], centroid)
    }

    /// Compute the extent (max − min) of vertices along one axis.
    fn compute_extent(vertices: &[[f64; 3]], axis: usize) -> f64 {
        if vertices.is_empty() {
            return 0.0;
        }
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for v in vertices {
            let c = v[axis];
            if c < lo {
                lo = c;
            }
            if c > hi {
                hi = c;
            }
        }
        (hi - lo).max(0.0)
    }

    /// Centroid of a vertex set.
    fn centroid(vertices: &[[f64; 3]]) -> [f64; 3] {
        if vertices.is_empty() {
            return [0.0; 3];
        }
        let inv = 1.0 / vertices.len() as f64;
        let mut c = [0.0_f64; 3];
        for v in vertices {
            c[0] += v[0];
            c[1] += v[1];
            c[2] += v[2];
        }
        c[0] *= inv;
        c[1] *= inv;
        c[2] *= inv;
        c
    }
}

// ===========================================================================
// Nelder–Mead simplex optimiser (gradient-free)
// ===========================================================================

/// Outcome of the Nelder–Mead run.
struct NelderMeadResult {
    best_point: Vec<f64>,
    iterations: usize,
    converged: bool,
}

/// Standard Nelder–Mead coefficients.
const NM_ALPHA: f64 = 1.0; // reflection
const NM_GAMMA: f64 = 2.0; // expansion
const NM_RHO: f64 = 0.5; // contraction
const NM_SIGMA: f64 = 0.5; // shrink

/// Run Nelder–Mead simplex optimisation on an n-dimensional objective.
fn nelder_mead<F>(
    objective: &F,
    initial: &[f64],
    step_size: f64,
    max_iter: usize,
    tol: f64,
) -> NelderMeadResult
where
    F: Fn(&[f64]) -> f64,
{
    let n = initial.len();
    if n == 0 {
        return NelderMeadResult {
            best_point: Vec::new(),
            iterations: 0,
            converged: true,
        };
    }

    // 1. Initialise simplex with n+1 vertices.
    let mut simplex: Vec<Vec<f64>> = Vec::with_capacity(n + 1);
    simplex.push(initial.to_vec());
    for i in 0..n {
        let mut pt = initial.to_vec();
        pt[i] += step_size;
        simplex.push(pt);
    }

    let mut values: Vec<f64> = simplex.iter().map(|p| objective(p)).collect();
    let mut converged = false;

    let mut iter = 0_usize;
    while iter < max_iter {
        iter += 1;

        // Sort simplex by objective value.
        let mut order: Vec<usize> = (0..=n).collect();
        order.sort_by(|&a, &b| {
            values[a]
                .partial_cmp(&values[b])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let sorted_simplex: Vec<Vec<f64>> = order.iter().map(|&i| simplex[i].clone()).collect();
        let sorted_values: Vec<f64> = order.iter().map(|&i| values[i]).collect();
        simplex = sorted_simplex;
        values = sorted_values;

        // Check convergence: simplex diameter.
        let diameter = simplex_diameter(&simplex);
        if diameter < tol {
            converged = true;
            break;
        }

        // Also check if the range of objective values is negligible.
        let f_range = values[n] - values[0];
        if f_range.abs() < tol * 1e-2 {
            converged = true;
            break;
        }

        // Centroid of all points except worst.
        let centroid = nm_centroid(&simplex, n);

        // Reflection.
        let x_r = nm_reflect(&centroid, &simplex[n], NM_ALPHA);
        let f_r = objective(&x_r);

        if f_r < values[0] {
            // Try expansion.
            let x_e = nm_reflect(&centroid, &simplex[n], NM_GAMMA);
            let f_e = objective(&x_e);
            if f_e < f_r {
                simplex[n] = x_e;
                values[n] = f_e;
            } else {
                simplex[n] = x_r;
                values[n] = f_r;
            }
        } else if f_r < values[n - 1] {
            // Accept reflection.
            simplex[n] = x_r;
            values[n] = f_r;
        } else {
            // Contraction.
            let (x_c, f_c) = if f_r < values[n] {
                // Outside contraction.
                let x_c = nm_contract(&centroid, &x_r, NM_RHO);
                let f_c = objective(&x_c);
                (x_c, f_c)
            } else {
                // Inside contraction.
                let x_c = nm_contract(&centroid, &simplex[n], NM_RHO);
                let f_c = objective(&x_c);
                (x_c, f_c)
            };

            if f_c < values[n].min(f_r) {
                simplex[n] = x_c;
                values[n] = f_c;
            } else {
                // Shrink: move all points toward best.
                nm_shrink(&mut simplex, &mut values, objective, NM_SIGMA);
            }
        }
    }

    // Final sort.
    let mut order: Vec<usize> = (0..=n).collect();
    order.sort_by(|&a, &b| {
        values[a]
            .partial_cmp(&values[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    NelderMeadResult {
        best_point: simplex[order[0]].clone(),
        iterations: iter,
        converged,
    }
}

/// Compute centroid of the first `count` simplex vertices.
fn nm_centroid(simplex: &[Vec<f64>], count: usize) -> Vec<f64> {
    let n = simplex[0].len();
    let mut c = vec![0.0_f64; n];
    for pt in simplex.iter().take(count) {
        for (ci, &pi) in c.iter_mut().zip(pt.iter()) {
            *ci += pi;
        }
    }
    let inv = 1.0 / count as f64;
    for ci in &mut c {
        *ci *= inv;
    }
    c
}

/// Reflect / expand: `centroid + alpha * (centroid - worst)`.
fn nm_reflect(centroid: &[f64], worst: &[f64], alpha: f64) -> Vec<f64> {
    centroid
        .iter()
        .zip(worst.iter())
        .map(|(&c, &w)| c + alpha * (c - w))
        .collect()
}

/// Contract: point between `centroid` and `point` at ratio `rho`.
fn nm_contract(centroid: &[f64], point: &[f64], rho: f64) -> Vec<f64> {
    centroid
        .iter()
        .zip(point.iter())
        .map(|(&c, &p)| c + rho * (p - c))
        .collect()
}

/// Shrink all simplex vertices (except the best) toward the best vertex.
fn nm_shrink<F>(simplex: &mut [Vec<f64>], values: &mut [f64], objective: &F, sigma: f64)
where
    F: Fn(&[f64]) -> f64,
{
    let best = simplex[0].clone();
    for i in 1..simplex.len() {
        for j in 0..simplex[i].len() {
            simplex[i][j] = best[j] + sigma * (simplex[i][j] - best[j]);
        }
        values[i] = objective(&simplex[i]);
    }
}

/// Maximum distance between any two simplex vertices (diameter).
fn simplex_diameter(simplex: &[Vec<f64>]) -> f64 {
    let mut max_d = 0.0_f64;
    for i in 0..simplex.len() {
        for j in (i + 1)..simplex.len() {
            let d_sq: f64 = simplex[i]
                .iter()
                .zip(simplex[j].iter())
                .map(|(&a, &b)| (a - b) * (a - b))
                .sum();
            let d = d_sq.sqrt();
            if d > max_d {
                max_d = d;
            }
        }
    }
    max_d
}

// ===========================================================================
// 2-D convex hull (Andrew's monotone chain)
// ===========================================================================

/// Compute the convex hull of a set of 2-D points using Andrew's monotone
/// chain algorithm. Returns vertices in counter-clockwise order.
fn convex_hull_2d(points: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let mut pts: Vec<[f64; 2]> = points.to_vec();
    pts.sort_by(|a, b| {
        a[0].partial_cmp(&b[0])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a[1].partial_cmp(&b[1])
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    pts.dedup_by(|a, b| (a[0] - b[0]).abs() < 1e-15 && (a[1] - b[1]).abs() < 1e-15);

    let n = pts.len();
    if n <= 1 {
        return pts;
    }
    if n == 2 {
        return pts;
    }

    let mut hull: Vec<[f64; 2]> = Vec::with_capacity(2 * n);

    // Lower hull.
    for &p in &pts {
        while hull.len() >= 2 && cross_2d(hull[hull.len() - 2], hull[hull.len() - 1], p) <= 0.0 {
            hull.pop();
        }
        hull.push(p);
    }

    // Upper hull.
    let lower_len = hull.len() + 1;
    for &p in pts.iter().rev() {
        while hull.len() >= lower_len
            && cross_2d(hull[hull.len() - 2], hull[hull.len() - 1], p) <= 0.0
        {
            hull.pop();
        }
        hull.push(p);
    }

    hull.pop(); // remove duplicate of first point
    hull
}

/// 2-D cross product of vectors `OA` and `OB`.
fn cross_2d(o: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
}

// ===========================================================================
// 3-D vector helpers
// ===========================================================================

fn dot3(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn vec3_len(v: &[f64; 3]) -> f64 {
    dot3(v, v).sqrt()
}

/// Build an orthonormal pair of tangent vectors on a plane with the given
/// unit normal.
fn plane_tangent_frame(n: &[f64; 3]) -> ([f64; 3], [f64; 3]) {
    // Choose a vector not parallel to n.
    let seed = if n[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };

    // u = normalise(seed - (seed . n) * n)
    let d = dot3(&seed, n);
    let mut u = [seed[0] - d * n[0], seed[1] - d * n[1], seed[2] - d * n[2]];
    let u_len = vec3_len(&u);
    if u_len < 1e-15 {
        return ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    }
    u[0] /= u_len;
    u[1] /= u_len;
    u[2] /= u_len;

    // v = n × u
    let v = [
        n[1] * u[2] - n[2] * u[1],
        n[2] * u[0] - n[0] * u[2],
        n[0] * u[1] - n[1] * u[0],
    ];

    (u, v)
}

// ===========================================================================
// Pre-built measurement sets (convenience constructors)
// ===========================================================================

/// Build a standard set of tape measurements from common body stats.
///
/// All values in centimetres except where noted. Pass `0.0` for any
/// measurement you do not have; it will be omitted.
pub fn standard_measurements(
    neck_cm: f64,
    chest_cm: f64,
    waist_cm: f64,
    hips_cm: f64,
    upper_arm_cm: f64,
    thigh_cm: f64,
    inseam_cm: f64,
    height_cm: f64,
) -> Vec<TapeMeasurement> {
    let mut out = Vec::new();

    let mut push = |name: &str, kind: MeasurementKind, value: f64, weight: f64| {
        if value > 0.0 {
            out.push(TapeMeasurement {
                name: name.to_string(),
                kind,
                value_cm: value,
                tolerance_cm: 1.0,
                weight,
            });
        }
    };

    push(
        "neck",
        MeasurementKind::Circumference(BodySection::Neck),
        neck_cm,
        1.0,
    );
    push(
        "chest",
        MeasurementKind::Circumference(BodySection::Chest),
        chest_cm,
        2.0,
    );
    push(
        "waist",
        MeasurementKind::Circumference(BodySection::Waist),
        waist_cm,
        2.0,
    );
    push(
        "hips",
        MeasurementKind::Circumference(BodySection::Hips),
        hips_cm,
        2.0,
    );
    push(
        "upper_arm",
        MeasurementKind::Circumference(BodySection::UpperArm),
        upper_arm_cm,
        0.5,
    );
    push(
        "thigh",
        MeasurementKind::Circumference(BodySection::Thigh),
        thigh_cm,
        1.0,
    );
    push(
        "inseam",
        MeasurementKind::Length(BodyLandmark::Crotch, BodyLandmark::FloorLeft),
        inseam_cm,
        1.5,
    );
    push(
        "height",
        MeasurementKind::Length(BodyLandmark::TopOfHead, BodyLandmark::FloorLeft),
        height_cm,
        3.0,
    );

    out
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convex_hull_triangle() {
        let pts = [[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let hull = convex_hull_2d(&pts);
        assert_eq!(hull.len(), 3);
    }

    #[test]
    fn test_convex_hull_collinear() {
        let pts = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]];
        let hull = convex_hull_2d(&pts);
        assert_eq!(hull.len(), 2);
    }

    #[test]
    fn test_convex_hull_square() {
        let pts = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.5, 0.5]];
        let hull = convex_hull_2d(&pts);
        assert_eq!(hull.len(), 4);
    }

    #[test]
    fn test_compute_circumference_square() {
        // Unit square in the XZ plane at y=0.
        let vertices = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ];
        let normal = [0.0, 1.0, 0.0];
        let point = [0.5, 0.0, 0.5];
        let circ = BodyCalibrator::compute_circumference(&vertices, &normal, &point)
            .expect("should not fail");
        // Perimeter of a unit square = 4.0
        assert!((circ - 4.0).abs() < 1e-10, "circ = {circ}");
    }

    #[test]
    fn test_compute_distance() {
        let a = [[0.0, 0.0, 0.0]];
        let b = [[3.0, 4.0, 0.0]];
        let d = BodyCalibrator::compute_distance(&a, &b);
        assert!((d - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_nelder_mead_rosenbrock_2d() {
        // Minimise Rosenbrock: f(x,y) = (1−x)² + 100(y−x²)²
        let rosenbrock = |p: &[f64]| -> f64 {
            let x = p[0];
            let y = p[1];
            (1.0 - x).powi(2) + 100.0 * (y - x * x).powi(2)
        };
        let result = nelder_mead(&rosenbrock, &[0.0, 0.0], 0.5, 10_000, 1e-12);
        assert!(
            (result.best_point[0] - 1.0).abs() < 1e-4,
            "x = {}",
            result.best_point[0]
        );
        assert!(
            (result.best_point[1] - 1.0).abs() < 1e-4,
            "y = {}",
            result.best_point[1]
        );
    }

    #[test]
    fn test_nelder_mead_sphere() {
        // Minimise f(x) = sum(x_i^2), minimum at origin.
        let sphere = |p: &[f64]| -> f64 { p.iter().map(|x| x * x).sum() };
        let init = vec![3.0, -2.0, 1.0, -4.0];
        let result = nelder_mead(&sphere, &init, 1.0, 5000, 1e-12);
        for (i, &v) in result.best_point.iter().enumerate() {
            assert!(v.abs() < 1e-4, "dim {i}: {v}");
        }
    }

    #[test]
    fn test_calibrator_basic() {
        // Simple calibration: one morph target that scales X uniformly.
        // We want chest circumference to match a target.
        let base: Vec<[f64; 3]> = (0..8)
            .map(|i| {
                let angle = (i as f64) * std::f64::consts::TAU / 8.0;
                [angle.cos() * 0.15, 1.0, angle.sin() * 0.15]
            })
            .collect();

        // Morph target: scale outward.
        let deltas: Vec<[f64; 3]> = base
            .iter()
            .map(|v| [v[0] * 0.5, 0.0, v[2] * 0.5])
            .collect();

        let morph_targets = vec![("scale_chest".to_string(), deltas)];

        let mut calibrator = BodyCalibrator::new(CalibrationConfig {
            max_iterations: 500,
            convergence_threshold: 1e-10,
            learning_rate: 0.1,
            regularization: 0.0,
        });

        // Target: we want a specific circumference.
        let target_circ = 100.0; // cm
        calibrator.add_measurement(TapeMeasurement {
            name: "chest".to_string(),
            kind: MeasurementKind::Circumference(BodySection::Chest),
            value_cm: target_circ,
            tolerance_cm: 1.0,
            weight: 1.0,
        });

        let indices: Vec<usize> = (0..8).collect();
        calibrator
            .set_measurement_vertices("chest", indices)
            .expect("set_measurement_vertices");

        let result = calibrator
            .calibrate(&base, &morph_targets, &[0.0])
            .expect("calibrate");

        // The optimizer should find a weight that produces something close.
        assert!(
            result.total_error < 5.0,
            "total_error = {}",
            result.total_error
        );
        assert!(!result.morph_parameters.is_empty());
    }

    #[test]
    fn test_standard_measurements() {
        let ms = standard_measurements(38.0, 96.0, 80.0, 100.0, 30.0, 55.0, 80.0, 175.0);
        assert_eq!(ms.len(), 8);

        // Zero values are omitted.
        let ms2 = standard_measurements(0.0, 96.0, 0.0, 100.0, 0.0, 0.0, 0.0, 175.0);
        assert_eq!(ms2.len(), 3);
    }

    #[test]
    fn test_plane_tangent_frame_orthonormal() {
        let normals: &[[f64; 3]] = &[
            [0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.577, 0.577, 0.577],
        ];
        for n_raw in normals {
            let len = vec3_len(n_raw);
            let n = [n_raw[0] / len, n_raw[1] / len, n_raw[2] / len];
            let (u, v) = plane_tangent_frame(&n);
            // u . n ≈ 0
            assert!(dot3(&u, &n).abs() < 1e-10, "u not perpendicular to n");
            // v . n ≈ 0
            assert!(dot3(&v, &n).abs() < 1e-10, "v not perpendicular to n");
            // u . v ≈ 0
            assert!(dot3(&u, &v).abs() < 1e-10, "u not perpendicular to v");
            // |u| ≈ 1
            assert!((vec3_len(&u) - 1.0).abs() < 1e-10);
            // |v| ≈ 1
            assert!((vec3_len(&v) - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_compute_extent() {
        let verts = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [2.0, 3.0, 1.0]];
        let ex = BodyCalibrator::compute_extent(&verts, 0);
        assert!((ex - 3.0).abs() < 1e-10); // 4 - 1
        let ey = BodyCalibrator::compute_extent(&verts, 1);
        assert!((ey - 3.0).abs() < 1e-10); // 5 - 2
        let ez = BodyCalibrator::compute_extent(&verts, 2);
        assert!((ez - 5.0).abs() < 1e-10); // 6 - 1
    }

    #[test]
    fn test_empty_vertices_circumference() {
        let result = BodyCalibrator::compute_circumference(&[], &[0.0, 1.0, 0.0], &[0.0; 3]);
        assert_eq!(result.expect("should succeed"), 0.0);
    }

    #[test]
    fn test_zero_normal_error() {
        let verts = [[0.0, 0.0, 0.0]];
        let result = BodyCalibrator::compute_circumference(&verts, &[0.0, 0.0, 0.0], &[0.0; 3]);
        assert!(result.is_err());
    }

    #[test]
    fn test_nelder_mead_zero_dim() {
        let f = |_p: &[f64]| -> f64 { 0.0 };
        let result = nelder_mead(&f, &[], 1.0, 100, 1e-8);
        assert!(result.best_point.is_empty());
        assert!(result.converged);
    }

    #[test]
    fn test_calibrator_no_measurements_error() {
        let calibrator = BodyCalibrator::new(CalibrationConfig::default());
        let result = calibrator.calibrate(&[], &[], &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_calibrator_weight_mismatch_error() {
        let mut calibrator = BodyCalibrator::new(CalibrationConfig::default());
        calibrator.add_measurement(TapeMeasurement {
            name: "test".to_string(),
            kind: MeasurementKind::Circumference(BodySection::Chest),
            value_cm: 90.0,
            tolerance_cm: 1.0,
            weight: 1.0,
        });
        let targets = vec![("a".to_string(), vec![[0.0; 3]])];
        let result = calibrator.calibrate(&[[0.0; 3]], &targets, &[0.0, 1.0]); // 2 weights, 1 target
        assert!(result.is_err());
    }

    #[test]
    fn test_set_measurement_vertices_not_found() {
        let mut calibrator = BodyCalibrator::new(CalibrationConfig::default());
        let result = calibrator.set_measurement_vertices("nonexistent", vec![0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_deform_mesh() {
        let base = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let deltas = vec![[0.5, 0.0, 0.0], [0.0, 0.5, 0.0]];
        let targets = vec![("t".to_string(), deltas)];
        let deformed = BodyCalibrator::deform_mesh(&base, &targets, &[2.0]);
        assert!((deformed[0][0] - 2.0).abs() < 1e-10); // 1 + 2*0.5
        assert!((deformed[1][1] - 2.0).abs() < 1e-10); // 1 + 2*0.5
    }
}
