// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! PCF (Percentage Closer Filtering) shadow parameters.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PCFShadowV2 {
    pub kernel_size: u32,
    pub bias: f32,
    pub samples: u32,
}

#[allow(dead_code)]
pub fn new_pcf_shadow_v2(kernel_size: u32, bias: f32) -> PCFShadowV2 {
    let samples = kernel_size * kernel_size;
    PCFShadowV2 { kernel_size, bias, samples }
}

#[allow(dead_code)]
pub fn pcfv2_sample_count(shadow: &PCFShadowV2) -> u32 {
    shadow.kernel_size * shadow.kernel_size
}

#[allow(dead_code)]
pub fn pcfv2_evaluate(shadow: &PCFShadowV2, depth_sample: f32, ref_depth: f32) -> f32 {
    if depth_sample + shadow.bias > ref_depth { 1.0 } else { 0.0 }
}

#[allow(dead_code)]
pub fn pcfv2_kernel_radius(shadow: &PCFShadowV2) -> u32 {
    shadow.kernel_size / 2
}

#[allow(dead_code)]
pub fn pcfv2_set_bias(shadow: &mut PCFShadowV2, bias: f32) {
    shadow.bias = bias;
}

#[allow(dead_code)]
pub fn pcfv2_total_weight(shadow: &PCFShadowV2) -> f32 {
    pcfv2_sample_count(shadow) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_count_3x3() {
        let s = new_pcf_shadow_v2(3, 0.005);
        assert_eq!(pcfv2_sample_count(&s), 9);
    }

    #[test]
    fn test_evaluate_lit() {
        let s = new_pcf_shadow_v2(3, 0.005);
        /* depth_sample (1.0) + bias (0.005) > ref_depth (0.9) → lit */
        let result = pcfv2_evaluate(&s, 1.0, 0.9);
        assert!((result - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_shadow() {
        let s = new_pcf_shadow_v2(3, 0.001);
        /* depth_sample (0.5) + bias (0.001) < ref_depth (0.9) → shadow */
        let result = pcfv2_evaluate(&s, 0.5, 0.9);
        assert!((result - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_kernel_radius() {
        let s = new_pcf_shadow_v2(5, 0.005);
        assert_eq!(pcfv2_kernel_radius(&s), 2);
    }

    #[test]
    fn test_set_bias() {
        let mut s = new_pcf_shadow_v2(3, 0.005);
        pcfv2_set_bias(&mut s, 0.01);
        assert!((s.bias - 0.01).abs() < 1e-6);
    }

    #[test]
    fn test_total_weight_equals_sample_count() {
        let s = new_pcf_shadow_v2(4, 0.005);
        assert!((pcfv2_total_weight(&s) - 16.0).abs() < 1e-6);
    }

    #[test]
    fn test_kernel_size_1x1() {
        let s = new_pcf_shadow_v2(1, 0.001);
        assert_eq!(pcfv2_sample_count(&s), 1);
    }

    #[test]
    fn test_kernel_radius_even() {
        let s = new_pcf_shadow_v2(4, 0.005);
        assert_eq!(pcfv2_kernel_radius(&s), 2);
    }
}
