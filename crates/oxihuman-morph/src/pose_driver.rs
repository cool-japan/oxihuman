// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! RBF-based pose-driven corrective shapes.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoseDriverSample {
    pub pose_values: Vec<f32>,
    pub deltas: Vec<[f32; 3]>,
    pub weight: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoseDriverConfig {
    pub rbf_radius: f32,
    pub normalize: bool,
    pub falloff: RbfFalloff,
}

impl Default for PoseDriverConfig {
    fn default() -> Self {
        Self {
            rbf_radius: 1.0,
            normalize: true,
            falloff: RbfFalloff::Gaussian,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum RbfFalloff {
    Gaussian,
    InverseDistance,
    ThinPlate,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoseDriver {
    pub samples: Vec<PoseDriverSample>,
    pub config: PoseDriverConfig,
    pub vertex_count: usize,
}

impl PoseDriver {
    #[allow(dead_code)]
    pub fn new(vertex_count: usize, config: PoseDriverConfig) -> Self {
        Self {
            samples: Vec::new(),
            config,
            vertex_count,
        }
    }

    #[allow(dead_code)]
    pub fn add_sample(&mut self, sample: PoseDriverSample) {
        self.samples.push(sample);
    }

    #[allow(dead_code)]
    pub fn evaluate(&self, pose: &[f32]) -> Vec<[f32; 3]> {
        let mut result = vec![[0.0f32; 3]; self.vertex_count];

        if self.samples.is_empty() {
            return result;
        }

        let radius = self.config.rbf_radius;
        let mut rbf_weights: Vec<f32> = self
            .samples
            .iter()
            .map(|s| {
                let dist = pose_distance(pose, &s.pose_values);
                let rbf = match self.config.falloff {
                    RbfFalloff::Gaussian => rbf_gaussian(dist, radius),
                    RbfFalloff::InverseDistance => rbf_inverse_distance(dist, 1e-6),
                    RbfFalloff::ThinPlate => rbf_thin_plate(dist),
                };
                rbf * s.weight
            })
            .collect();

        if self.config.normalize {
            normalize_weights(&mut rbf_weights);
        }

        for (sample, &w) in self.samples.iter().zip(rbf_weights.iter()) {
            let deltas = &sample.deltas;
            let vcount = deltas.len().min(self.vertex_count);
            for i in 0..vcount {
                result[i][0] += w * deltas[i][0];
                result[i][1] += w * deltas[i][1];
                result[i][2] += w * deltas[i][2];
            }
        }

        result
    }
}

#[allow(dead_code)]
pub fn rbf_gaussian(dist: f32, radius: f32) -> f32 {
    let r = if radius.abs() < 1e-10 { 1e-10 } else { radius };
    let x = dist / r;
    (-x * x).exp()
}

#[allow(dead_code)]
pub fn rbf_inverse_distance(dist: f32, eps: f32) -> f32 {
    1.0 / (dist + eps)
}

#[allow(dead_code)]
pub fn rbf_thin_plate(dist: f32) -> f32 {
    dist * dist * (dist + 1e-10_f32).ln()
}

#[allow(dead_code)]
pub fn pose_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let d = x - y;
            d * d
        })
        .sum::<f32>()
        .sqrt()
}

#[allow(dead_code)]
pub fn normalize_weights(weights: &mut [f32]) {
    let sum: f32 = weights.iter().sum();
    if sum.abs() < 1e-10 {
        if !weights.is_empty() {
            let v = 1.0 / weights.len() as f32;
            for w in weights.iter_mut() {
                *w = v;
            }
        }
        return;
    }
    for w in weights.iter_mut() {
        *w /= sum;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rbf_gaussian_zero_distance() {
        let v = rbf_gaussian(0.0, 1.0);
        assert!(
            (v - 1.0).abs() < 1e-6,
            "zero dist should return 1.0, got {v}"
        );
    }

    #[test]
    fn test_rbf_gaussian_decay() {
        let v0 = rbf_gaussian(0.0, 1.0);
        let v1 = rbf_gaussian(1.0, 1.0);
        let v2 = rbf_gaussian(2.0, 1.0);
        assert!(v0 > v1, "should decay with distance");
        assert!(v1 > v2, "should decay with distance");
        // at dist=radius, value = exp(-1)
        let expected = (-1.0_f32).exp();
        assert!((v1 - expected).abs() < 1e-6);
    }

    #[test]
    fn test_rbf_gaussian_large_radius() {
        // with large radius, decay is slow
        let v = rbf_gaussian(1.0, 100.0);
        assert!(v > 0.999, "large radius should give near-1: {v}");
    }

    #[test]
    fn test_rbf_inverse_distance() {
        let v = rbf_inverse_distance(0.0, 1e-6);
        assert!(v > 1e5, "near zero dist should give large value");
        let v1 = rbf_inverse_distance(1.0, 1e-6);
        let v2 = rbf_inverse_distance(2.0, 1e-6);
        assert!(v1 > v2);
    }

    #[test]
    fn test_rbf_thin_plate_zero() {
        let v = rbf_thin_plate(0.0);
        // 0 * 0 * ln(1e-10) — should be near 0
        assert!(v.abs() < 1e-6, "thin plate at zero: {v}");
    }

    #[test]
    fn test_rbf_thin_plate_positive() {
        let v = rbf_thin_plate(2.0);
        let expected = 4.0_f32 * (2.0_f32 + 1e-10_f32).ln();
        assert!((v - expected).abs() < 1e-5);
    }

    #[test]
    fn test_pose_distance_identical() {
        let d = pose_distance(&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0]);
        assert!(d.abs() < 1e-6);
    }

    #[test]
    fn test_pose_distance_basic() {
        let d = pose_distance(&[0.0, 0.0], &[3.0, 4.0]);
        assert!((d - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_weights_sum_one() {
        let mut w = vec![1.0, 2.0, 3.0, 4.0];
        normalize_weights(&mut w);
        let sum: f32 = w.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_weights_zero_sum() {
        let mut w = vec![0.0, 0.0, 0.0];
        normalize_weights(&mut w);
        let sum: f32 = w.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6, "zero-sum should uniform: {sum}");
    }

    #[test]
    fn test_single_sample_evaluate_returns_deltas() {
        let cfg = PoseDriverConfig::default();
        let mut driver = PoseDriver::new(3, cfg);
        let deltas = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        driver.add_sample(PoseDriverSample {
            pose_values: vec![0.0],
            deltas: deltas.clone(),
            weight: 1.0,
        });
        let result = driver.evaluate(&[0.0]);
        // single sample normalized → weight = 1.0, so result == deltas
        for i in 0..3 {
            for j in 0..3 {
                assert!((result[i][j] - deltas[i][j]).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn test_multiple_samples_interpolation() {
        let cfg = PoseDriverConfig {
            rbf_radius: 1.0,
            normalize: true,
            falloff: RbfFalloff::Gaussian,
        };
        let mut driver = PoseDriver::new(1, cfg);
        driver.add_sample(PoseDriverSample {
            pose_values: vec![0.0],
            deltas: vec![[1.0, 0.0, 0.0]],
            weight: 1.0,
        });
        driver.add_sample(PoseDriverSample {
            pose_values: vec![2.0],
            deltas: vec![[0.0, 1.0, 0.0]],
            weight: 1.0,
        });
        // Query at midpoint
        let result = driver.evaluate(&[1.0]);
        // Both have equal distance → equal weights after normalize
        assert!(
            (result[0][0] - result[0][1]).abs() < 1e-4,
            "midpoint interpolation: x={} y={}",
            result[0][0],
            result[0][1]
        );
    }

    #[test]
    fn test_evaluate_empty_samples() {
        let cfg = PoseDriverConfig::default();
        let driver = PoseDriver::new(2, cfg);
        let result = driver.evaluate(&[0.0, 1.0]);
        assert_eq!(result.len(), 2);
        for v in &result {
            assert_eq!(*v, [0.0, 0.0, 0.0]);
        }
    }

    #[test]
    fn test_inverse_distance_falloff() {
        let cfg = PoseDriverConfig {
            rbf_radius: 1.0,
            normalize: true,
            falloff: RbfFalloff::InverseDistance,
        };
        let mut driver = PoseDriver::new(1, cfg);
        driver.add_sample(PoseDriverSample {
            pose_values: vec![0.0],
            deltas: vec![[1.0, 0.0, 0.0]],
            weight: 1.0,
        });
        driver.add_sample(PoseDriverSample {
            pose_values: vec![10.0],
            deltas: vec![[0.0, 1.0, 0.0]],
            weight: 1.0,
        });
        let result = driver.evaluate(&[0.0]);
        // Near sample 0 → result[0][0] >> result[0][1]
        assert!(
            result[0][0] > result[0][1],
            "inverse: close sample should dominate"
        );
    }

    #[test]
    fn test_thin_plate_falloff_evaluate() {
        let cfg = PoseDriverConfig {
            rbf_radius: 1.0,
            normalize: true,
            falloff: RbfFalloff::ThinPlate,
        };
        let mut driver = PoseDriver::new(1, cfg);
        driver.add_sample(PoseDriverSample {
            pose_values: vec![0.0],
            deltas: vec![[2.0, 0.0, 0.0]],
            weight: 1.0,
        });
        let result = driver.evaluate(&[0.0]);
        // single sample → gets all weight
        assert!(
            (result[0][0] - 2.0).abs() < 1e-4,
            "thin plate single: {}",
            result[0][0]
        );
    }
}
