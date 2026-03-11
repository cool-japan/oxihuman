// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! RBF (radial basis function) deformer stub.

/// RBF kernel type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RbfKernel {
    Gaussian,
    Multiquadric,
    InverseQuadratic,
}

/// An RBF control point.
#[derive(Debug, Clone)]
pub struct RbfControlPoint {
    pub center: Vec<f32>,
    pub coefficient: f32,
}

/// RBF deformer.
#[derive(Debug, Clone)]
pub struct RbfDeformer {
    pub kernel: RbfKernel,
    pub epsilon: f32,
    pub control_points: Vec<RbfControlPoint>,
}

impl RbfDeformer {
    pub fn new(kernel: RbfKernel) -> Self {
        RbfDeformer {
            kernel,
            epsilon: 1.0,
            control_points: Vec::new(),
        }
    }
}

/// Create a new RBF deformer with given kernel.
pub fn new_rbf_deformer(kernel: RbfKernel) -> RbfDeformer {
    RbfDeformer::new(kernel)
}

/// Compute the RBF kernel value for a given distance.
pub fn rbf_kernel_value(deformer: &RbfDeformer, distance: f32) -> f32 {
    let r = distance * deformer.epsilon;
    match deformer.kernel {
        RbfKernel::Gaussian => (-r * r).exp(),
        RbfKernel::Multiquadric => (1.0 + r * r).sqrt(),
        RbfKernel::InverseQuadratic => 1.0 / (1.0 + r * r),
    }
}

/// Add a control point.
pub fn rbf_add_control_point(deformer: &mut RbfDeformer, center: Vec<f32>, coefficient: f32) {
    deformer.control_points.push(RbfControlPoint {
        center,
        coefficient,
    });
}

/// Return control point count.
pub fn rbf_point_count(deformer: &RbfDeformer) -> usize {
    deformer.control_points.len()
}

/// Evaluate the RBF at a given query point.
pub fn rbf_evaluate(deformer: &RbfDeformer, query: &[f32]) -> f32 {
    deformer
        .control_points
        .iter()
        .map(|cp| {
            let n = cp.center.len().min(query.len());
            let dist: f32 = (0..n)
                .map(|i| (cp.center[i] - query[i]).powi(2))
                .sum::<f32>()
                .sqrt();
            cp.coefficient * rbf_kernel_value(deformer, dist)
        })
        .sum()
}

/// Return a JSON-like string.
pub fn rbf_to_json(deformer: &RbfDeformer) -> String {
    format!(
        r#"{{"kernel":"{}","epsilon":{:.4},"points":{}}}"#,
        match deformer.kernel {
            RbfKernel::Gaussian => "gaussian",
            RbfKernel::Multiquadric => "multiquadric",
            RbfKernel::InverseQuadratic => "inverse_quadratic",
        },
        deformer.epsilon,
        deformer.control_points.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_rbf_deformer_no_points() {
        let d = new_rbf_deformer(RbfKernel::Gaussian);
        assert_eq!(
            rbf_point_count(&d),
            0, /* new deformer has no control points */
        );
    }

    #[test]
    fn test_add_control_point_increases_count() {
        let mut d = new_rbf_deformer(RbfKernel::Gaussian);
        rbf_add_control_point(&mut d, vec![0.0, 0.0], 1.0);
        assert_eq!(rbf_point_count(&d), 1 /* count should increase */,);
    }

    #[test]
    fn test_gaussian_at_zero_distance_is_one() {
        let d = new_rbf_deformer(RbfKernel::Gaussian);
        let v = rbf_kernel_value(&d, 0.0);
        assert!((v - 1.0).abs() < 1e-5, /* Gaussian at zero distance should be 1 */);
    }

    #[test]
    fn test_inverse_quadratic_at_zero_distance_is_one() {
        let d = new_rbf_deformer(RbfKernel::InverseQuadratic);
        let v = rbf_kernel_value(&d, 0.0);
        assert!((v - 1.0).abs() < 1e-5, /* InverseQuadratic at zero distance should be 1 */);
    }

    #[test]
    fn test_multiquadric_at_zero_distance_is_one() {
        let d = new_rbf_deformer(RbfKernel::Multiquadric);
        let v = rbf_kernel_value(&d, 0.0);
        assert!((v - 1.0).abs() < 1e-5, /* Multiquadric at zero distance should be 1 */);
    }

    #[test]
    fn test_evaluate_empty_is_zero() {
        let d = new_rbf_deformer(RbfKernel::Gaussian);
        let v = rbf_evaluate(&d, &[0.0, 0.0]);
        assert!((v).abs() < 1e-6 /* empty deformer evaluates to 0 */,);
    }

    #[test]
    fn test_evaluate_at_control_point() {
        let mut d = new_rbf_deformer(RbfKernel::Gaussian);
        rbf_add_control_point(&mut d, vec![0.0, 0.0], 1.0);
        let v = rbf_evaluate(&d, &[0.0, 0.0]);
        assert!(
            (v - 1.0).abs() < 1e-5, /* evaluating exactly at control point with coeff=1 should give 1 */
        );
    }

    #[test]
    fn test_to_json_contains_kernel() {
        let d = new_rbf_deformer(RbfKernel::Gaussian);
        let j = rbf_to_json(&d);
        assert!(j.contains("gaussian"), /* JSON must contain kernel type */);
    }

    #[test]
    fn test_epsilon_default_one() {
        let d = new_rbf_deformer(RbfKernel::Gaussian);
        assert!((d.epsilon - 1.0).abs() < 1e-5, /* default epsilon is 1.0 */);
    }

    #[test]
    fn test_gaussian_decays_with_distance() {
        let d = new_rbf_deformer(RbfKernel::Gaussian);
        let near = rbf_kernel_value(&d, 0.1);
        let far = rbf_kernel_value(&d, 2.0);
        assert!(near > far /* Gaussian should decay with distance */,);
    }
}
