// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct WrinkleDriver {
    pub weights: Vec<f32>,
    pub min_angle_rad: f32,
    pub max_angle_rad: f32,
}

pub fn new_wrinkle_driver(weights: Vec<f32>, min_a: f32, max_a: f32) -> WrinkleDriver {
    WrinkleDriver {
        weights,
        min_angle_rad: min_a,
        max_angle_rad: max_a,
    }
}

pub fn wrinkle_factor(d: &WrinkleDriver, angle_rad: f32) -> f32 {
    let range = d.max_angle_rad - d.min_angle_rad;
    if range.abs() < 1e-8 {
        return 0.0;
    }
    ((angle_rad - d.min_angle_rad) / range).clamp(0.0, 1.0)
}

pub fn wrinkle_weights_at(d: &WrinkleDriver, angle_rad: f32) -> Vec<f32> {
    let f = wrinkle_factor(d, angle_rad);
    d.weights.iter().map(|&w| w * f).collect()
}

pub fn wrinkle_peak_count(d: &WrinkleDriver, threshold: f32) -> usize {
    d.weights.iter().filter(|&&w| w >= threshold).count()
}

pub fn wrinkle_mean_weight(d: &WrinkleDriver) -> f32 {
    if d.weights.is_empty() {
        return 0.0;
    }
    d.weights.iter().sum::<f32>() / d.weights.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    #[test]
    fn test_new_wrinkle_driver() {
        /* basic construction */
        let d = new_wrinkle_driver(vec![0.5, 1.0], 0.0, FRAC_PI_2);
        assert_eq!(d.weights.len(), 2);
    }

    #[test]
    fn test_factor_at_min() {
        /* factor is 0 at min */
        let d = new_wrinkle_driver(vec![], 0.0, 1.0);
        assert!((wrinkle_factor(&d, 0.0)).abs() < 1e-6);
    }

    #[test]
    fn test_factor_at_max() {
        /* factor is 1 at max */
        let d = new_wrinkle_driver(vec![], 0.0, 1.0);
        assert!((wrinkle_factor(&d, 1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_weights_at_halfway() {
        /* halfway factor => half weights */
        let d = new_wrinkle_driver(vec![1.0, 0.8], 0.0, 2.0);
        let w = wrinkle_weights_at(&d, 1.0);
        assert!((w[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_peak_count() {
        /* count above threshold */
        let d = new_wrinkle_driver(vec![0.2, 0.8, 0.9], 0.0, 1.0);
        assert_eq!(wrinkle_peak_count(&d, 0.7), 2);
    }

    #[test]
    fn test_mean_weight() {
        /* mean of 0,1 = 0.5 */
        let d = new_wrinkle_driver(vec![0.0, 1.0], 0.0, 1.0);
        assert!((wrinkle_mean_weight(&d) - 0.5).abs() < 1e-6);
    }
}
