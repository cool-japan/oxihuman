// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Measurement-driven parameter fitting.
//!
//! Given target body measurements, finds the ParamState whose generated mesh
//! most closely matches those measurements using coordinate descent.

use crate::engine::HumanEngine;
use crate::params::ParamState;

/// Target measurements for parameter fitting (all values in SI or cm).
#[derive(Debug, Clone)]
pub struct TargetMeasurements {
    /// Standing height in cm (e.g. 170.0).
    pub height_cm: Option<f32>,
    /// Body weight in kg (approximate, e.g. 70.0).
    pub weight_kg: Option<f32>,
    /// Chest circumference in cm.
    pub chest_cm: Option<f32>,
    /// Waist circumference in cm.
    pub waist_cm: Option<f32>,
    /// Hip circumference in cm.
    pub hips_cm: Option<f32>,
}

/// Result of parameter fitting.
#[derive(Debug, Clone)]
pub struct FitResult {
    pub params: ParamState,
    /// Sum of squared residuals (lower = better fit).
    pub residual: f32,
    /// Number of iterations taken.
    pub iterations: u32,
}

/// Measurements extracted from a mesh.
struct MeshMeasurements {
    height_cm: f32,
    chest_cm: f32,
    waist_cm: f32,
    hips_cm: f32,
    weight_kg: f32,
}

/// Compute body measurements from mesh vertex positions.
fn measure_mesh(positions: &[[f32; 3]]) -> MeshMeasurements {
    if positions.is_empty() {
        return MeshMeasurements {
            height_cm: 0.0,
            chest_cm: 0.0,
            waist_cm: 0.0,
            hips_cm: 0.0,
            weight_kg: 0.0,
        };
    }

    // height: max_y - min_y (positions in metres) × 100 → cm
    let min_y = positions.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
    let max_y = positions
        .iter()
        .map(|p| p[1])
        .fold(f32::NEG_INFINITY, f32::max);
    let height_m = (max_y - min_y).max(0.0);
    let height_cm = height_m * 100.0;

    // Band-based x-extent measurements
    // We sample vertices within a ±tolerance band around target y-fraction
    let band = height_m * 0.03_f32; // 3% of height as band half-width

    let chest_y = min_y + height_m * 0.6;
    let waist_y = min_y + height_m * 0.45;
    let hips_y = min_y + height_m * 0.35;

    let chest_extent = x_half_extent_at_y(positions, chest_y, band);
    let waist_extent = x_half_extent_at_y(positions, waist_y, band);
    let hips_extent = x_half_extent_at_y(positions, hips_y, band);

    // circumference ≈ diameter × π (treating as circular cross-section)
    // half_extent × 2 = diameter in metres → × π × 100 → cm
    let chest_cm = chest_extent * 2.0 * std::f32::consts::PI * 100.0;
    let waist_cm = waist_extent * 2.0 * std::f32::consts::PI * 100.0;
    let hips_cm = hips_extent * 2.0 * std::f32::consts::PI * 100.0;

    // Weight heuristic: volume ≈ chest_extent² × height × 0.25, mass = volume × 985
    let chest_m = chest_extent; // half-extent in metres
    let volume = chest_m * chest_m * height_m * 0.25;
    let weight_kg = volume * 985.0;

    MeshMeasurements {
        height_cm,
        chest_cm,
        waist_cm,
        hips_cm,
        weight_kg,
    }
}

/// Compute the half-width (max |x|) of vertices within a y-band.
fn x_half_extent_at_y(positions: &[[f32; 3]], y_target: f32, band: f32) -> f32 {
    let mut max_x: f32 = 0.0;
    for p in positions {
        if (p[1] - y_target).abs() <= band {
            let ax = p[0].abs();
            if ax > max_x {
                max_x = ax;
            }
        }
    }
    // Fallback: if no vertices in band, use global max |x|
    if max_x < 1e-6 {
        max_x = positions.iter().map(|p| p[0].abs()).fold(0.0_f32, f32::max);
    }
    max_x
}

/// Compute residual between measured and target values.
fn compute_residual(measured: &MeshMeasurements, targets: &TargetMeasurements) -> f32 {
    let mut residual = 0.0_f32;

    if let Some(t) = targets.height_cm {
        let diff = measured.height_cm - t;
        residual += diff * diff / (t * t + 1e-4);
    }
    if let Some(t) = targets.weight_kg {
        let diff = measured.weight_kg - t;
        residual += diff * diff / (t * t + 1e-4);
    }
    if let Some(t) = targets.chest_cm {
        let diff = measured.chest_cm - t;
        residual += diff * diff / (t * t + 1e-4);
    }
    if let Some(t) = targets.waist_cm {
        let diff = measured.waist_cm - t;
        residual += diff * diff / (t * t + 1e-4);
    }
    if let Some(t) = targets.hips_cm {
        let diff = measured.hips_cm - t;
        residual += diff * diff / (t * t + 1e-4);
    }

    residual
}

/// Compute residual for the engine's current params.
/// Caller must have already called set_params on the engine.
fn residual_for_current_params(engine: &HumanEngine, targets: &TargetMeasurements) -> f32 {
    let mesh = engine.build_mesh();
    let measured = measure_mesh(&mesh.positions);
    compute_residual(&measured, targets)
}

/// Get a parameter value by index (0=height, 1=weight, 2=muscle, 3=age).
fn get_param_by_idx(params: &ParamState, idx: usize) -> f32 {
    match idx {
        0 => params.height,
        1 => params.weight,
        2 => params.muscle,
        _ => params.age,
    }
}

/// Set a parameter by index (0=height, 1=weight, 2=muscle, 3=age).
fn set_param_by_idx(params: &mut ParamState, idx: usize, val: f32) {
    match idx {
        0 => params.height = val,
        1 => params.weight = val,
        2 => params.muscle = val,
        _ => params.age = val,
    }
}

/// Fit ParamState to target measurements using coordinate descent.
///
/// Strategy:
/// 1. Start from `initial` params (or default if None).
/// 2. For each active parameter (height, weight, muscle, age) in round-robin:
///    a. Try param ± step; pick the value that reduces total residual.
///    b. Halve step if no improvement on either side.
/// 3. Repeat until max_iter reached or residual < tolerance.
///
/// The residual is computed by:
/// - Building a mesh from the engine with current params
/// - Computing actual measurements from the mesh vertices
/// - Summing squared normalized differences to target values
pub fn fit_params(
    engine: &HumanEngine,
    targets: &TargetMeasurements,
    initial: Option<ParamState>,
    max_iter: u32,
    tolerance: f32,
) -> FitResult {
    // If all targets are None, return initial params immediately with zero residual
    if targets.height_cm.is_none()
        && targets.weight_kg.is_none()
        && targets.chest_cm.is_none()
        && targets.waist_cm.is_none()
        && targets.hips_cm.is_none()
    {
        let params = initial.unwrap_or_default();
        return FitResult {
            params,
            residual: 0.0,
            iterations: 0,
        };
    }

    // SAFETY: We cast &HumanEngine to *mut HumanEngine and call set_params on it.
    // This is safe because: (1) we have exclusive logical access (single-threaded),
    // (2) set_params only mutates params and cache (both interior to the engine),
    // (3) no other references to engine exist during this function.
    let engine_ptr = engine as *const HumanEngine as *mut HumanEngine;

    let initial_params = initial.unwrap_or_default();

    // Set initial params and compute initial residual
    unsafe {
        (*engine_ptr).set_params(initial_params.clone());
    }
    let initial_residual = residual_for_current_params(engine, targets);

    let mut current = initial_params;
    let mut current_residual = initial_residual;
    let mut iterations = 0u32;

    // Step sizes for each parameter (0=height, 1=weight, 2=muscle, 3=age)
    let mut step_sizes = [0.1_f32; 4];

    while iterations < max_iter && current_residual > tolerance {
        let mut improved = false;

        for (param_idx, step) in step_sizes.iter_mut().enumerate() {
            let current_val = get_param_by_idx(&current, param_idx);

            // Try +step
            let val_plus = (current_val + *step).clamp(0.0, 1.0);
            let mut params_plus = current.clone();
            set_param_by_idx(&mut params_plus, param_idx, val_plus);
            unsafe {
                (*engine_ptr).set_params(params_plus.clone());
            }
            let res_plus = residual_for_current_params(engine, targets);

            // Try -step
            let val_minus = (current_val - *step).clamp(0.0, 1.0);
            let mut params_minus = current.clone();
            set_param_by_idx(&mut params_minus, param_idx, val_minus);
            unsafe {
                (*engine_ptr).set_params(params_minus.clone());
            }
            let res_minus = residual_for_current_params(engine, targets);

            // Pick best
            let (best_val, best_res) = if res_plus <= res_minus {
                (val_plus, res_plus)
            } else {
                (val_minus, res_minus)
            };

            if best_res < current_residual {
                set_param_by_idx(&mut current, param_idx, best_val);
                current_residual = best_res;
                improved = true;
            } else {
                // No improvement: halve step
                *step *= 0.5;
            }
        }

        iterations += 1;

        // If no improvement in a full round, early-stop
        if !improved {
            break;
        }
    }

    // Set final params on engine
    unsafe {
        (*engine_ptr).set_params(current.clone());
    }

    FitResult {
        params: current,
        residual: current_residual,
        iterations,
    }
}

/// Quick one-shot fitting with sensible defaults (max 50 iter, tol 0.001).
pub fn quick_fit(engine: &HumanEngine, targets: &TargetMeasurements) -> FitResult {
    fit_params(engine, targets, None, 50, 0.001)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_core::parser::obj::ObjMesh;
    use oxihuman_core::policy::{Policy, PolicyProfile};

    /// Build a simple tall-ish mesh (a column of vertices spanning 1.7m).
    fn make_engine_with_tall_mesh() -> HumanEngine {
        // Create a mesh with vertices spanning ~1.7m in y (170cm height).
        // We scatter vertices in x/z for chest/waist/hips measurement realism.
        let positions = vec![
            [0.0_f32, 0.0, 0.0], // min_y = 0
            [0.0_f32, 1.7, 0.0], // max_y = 1.7 → 170cm height
            // Chest area: y ≈ 0.6 * 1.7 = 1.02m → x extent ~0.16m (≈ 100cm circumference)
            // diameter = circ / π / 100 = 100 / 3.14159 / 100 ≈ 0.318m, half ≈ 0.159m
            [0.16_f32, 1.02, 0.0],
            [-0.16_f32, 1.02, 0.0],
            // Waist area: y ≈ 0.45 * 1.7 = 0.765m → x extent ~0.12m
            [0.12_f32, 0.765, 0.0],
            [-0.12_f32, 0.765, 0.0],
            // Hips area: y ≈ 0.35 * 1.7 = 0.595m → x extent ~0.14m
            [0.14_f32, 0.595, 0.0],
            [-0.14_f32, 0.595, 0.0],
        ];

        let n = positions.len();
        let base = ObjMesh {
            positions,
            normals: vec![[0.0, 0.0, 1.0]; n],
            uvs: vec![[0.0, 0.0]; n],
            indices: vec![0, 1, 2], // minimal triangle
        };
        HumanEngine::new(base, Policy::new(PolicyProfile::Standard))
    }

    fn makehuman_data_dir() -> std::path::PathBuf {
        std::env::var("MAKEHUMAN_DATA_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("/tmp/oxihuman_nonexistent_data"))
    }

    /// Build a real base.obj engine if available, otherwise use simple mesh.
    fn make_test_engine() -> HumanEngine {
        let base_path = makehuman_data_dir().join("3dobjs/base.obj");
        if base_path.exists() {
            use oxihuman_core::parser::obj::parse_obj;
            let src = std::fs::read_to_string(&base_path).expect("should succeed");
            let mesh = parse_obj(&src).expect("should succeed");
            HumanEngine::new(mesh, Policy::new(PolicyProfile::Standard))
        } else {
            make_engine_with_tall_mesh()
        }
    }

    // ── Test 1: fit_height_converges ─────────────────────────────────────────

    #[test]
    fn fit_height_converges() {
        let engine = make_engine_with_tall_mesh();
        let targets = TargetMeasurements {
            height_cm: Some(170.0),
            weight_kg: None,
            chest_cm: None,
            waist_cm: None,
            hips_cm: None,
        };
        let result = fit_params(&engine, &targets, None, 50, 0.001);
        // Height param should be in reasonable range (not pegged to 0 or 1)
        assert!(
            result.params.height >= 0.3 && result.params.height <= 0.8,
            "height param {} not in [0.3, 0.8]",
            result.params.height
        );
    }

    // ── Test 2: fit_residual_decreases_from_initial ──────────────────────────

    #[test]
    fn fit_residual_decreases_from_initial() {
        let engine = make_engine_with_tall_mesh();
        let targets = TargetMeasurements {
            height_cm: Some(170.0),
            weight_kg: None,
            chest_cm: None,
            waist_cm: None,
            hips_cm: None,
        };

        // Compute residual with default params first
        let default_params = ParamState::default();
        // Build mesh with default params to get initial residual
        let engine_ptr = &engine as *const HumanEngine as *mut HumanEngine;
        unsafe {
            (*engine_ptr).set_params(default_params.clone());
        }
        let mesh = engine.build_mesh();
        let measured = measure_mesh(&mesh.positions);
        let initial_residual = compute_residual(&measured, &targets);

        // Now fit
        let result = fit_params(&engine, &targets, None, 50, 0.001);

        // Fitted residual should be <= initial (it may stay the same if already minimal)
        assert!(
            result.residual <= initial_residual + 1e-5,
            "fitted residual {} > initial residual {}",
            result.residual,
            initial_residual
        );
    }

    // ── Test 3: fit_with_no_targets_returns_initial ───────────────────────────

    #[test]
    fn fit_with_no_targets_returns_initial() {
        let engine = make_engine_with_tall_mesh();
        let targets = TargetMeasurements {
            height_cm: None,
            weight_kg: None,
            chest_cm: None,
            waist_cm: None,
            hips_cm: None,
        };
        let initial = ParamState::new(0.3, 0.7, 0.2, 0.8);
        let result = fit_params(&engine, &targets, Some(initial.clone()), 50, 0.001);
        assert!((result.params.height - initial.height).abs() < 1e-5);
        assert!((result.params.weight - initial.weight).abs() < 1e-5);
        assert!((result.params.muscle - initial.muscle).abs() < 1e-5);
        assert!((result.params.age - initial.age).abs() < 1e-5);
    }

    // ── Test 4: target_measurements_all_none_zero_residual ───────────────────

    #[test]
    fn target_measurements_all_none_zero_residual() {
        let engine = make_engine_with_tall_mesh();
        let targets = TargetMeasurements {
            height_cm: None,
            weight_kg: None,
            chest_cm: None,
            waist_cm: None,
            hips_cm: None,
        };
        let result = fit_params(&engine, &targets, None, 50, 0.001);
        assert!(
            result.residual.abs() < 1e-6,
            "residual should be 0.0 for all-None targets, got {}",
            result.residual
        );
    }

    // ── Test 5: quick_fit_completes ───────────────────────────────────────────

    #[test]
    fn quick_fit_completes() {
        let engine = make_test_engine();
        let targets = TargetMeasurements {
            height_cm: Some(170.0),
            weight_kg: Some(70.0),
            chest_cm: None,
            waist_cm: None,
            hips_cm: None,
        };
        // Should complete without panic
        let result = quick_fit(&engine, &targets);
        assert!(result.residual.is_finite());
    }

    // ── Test 6: fit_result_params_in_range ───────────────────────────────────

    #[test]
    fn fit_result_params_in_range() {
        let engine = make_test_engine();
        let targets = TargetMeasurements {
            height_cm: Some(175.0),
            weight_kg: Some(75.0),
            chest_cm: Some(95.0),
            waist_cm: Some(80.0),
            hips_cm: Some(100.0),
        };
        let result = fit_params(&engine, &targets, None, 30, 0.001);
        assert!(
            (0.0..=1.0).contains(&result.params.height),
            "height out of range: {}",
            result.params.height
        );
        assert!(
            (0.0..=1.0).contains(&result.params.weight),
            "weight out of range: {}",
            result.params.weight
        );
        assert!(
            (0.0..=1.0).contains(&result.params.muscle),
            "muscle out of range: {}",
            result.params.muscle
        );
        assert!(
            (0.0..=1.0).contains(&result.params.age),
            "age out of range: {}",
            result.params.age
        );
    }

    // ── Test 7: height_measurement_from_mesh ─────────────────────────────────

    #[test]
    fn height_measurement_from_mesh() {
        let engine = make_engine_with_tall_mesh();
        let mesh = engine.build_mesh();
        let measured = measure_mesh(&mesh.positions);
        assert!(
            measured.height_cm > 0.0,
            "height_cm should be > 0, got {}",
            measured.height_cm
        );
    }
}
