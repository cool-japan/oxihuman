// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Depth range utilities — near/far clip management and depth buffer precision analysis.

use std::f32::consts::LN_2;

/// Depth range descriptor.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct DepthRange {
    pub near: f32,
    pub far: f32,
    pub reversed_z: bool,
}

impl Default for DepthRange {
    fn default() -> Self {
        Self {
            near: 0.1,
            far: 1000.0,
            reversed_z: false,
        }
    }
}

/// Depth precision stats.
#[derive(Debug, Clone, PartialEq, Default)]
#[allow(dead_code)]
pub struct DepthPrecisionStats {
    pub range_ratio: f32,
    pub log_depth_range: f32,
    pub recommended_near: f32,
    pub bits_needed: u32,
}

/// Create default depth range.
#[allow(dead_code)]
pub fn default_depth_range() -> DepthRange {
    DepthRange::default()
}

/// Set near and far clipping planes.
#[allow(dead_code)]
pub fn dr_set_planes(dr: &mut DepthRange, near: f32, far: f32) {
    dr.near = near.max(1e-4);
    dr.far = far.max(dr.near + 1e-3);
}

/// Enable/disable reversed-Z depth.
#[allow(dead_code)]
pub fn dr_set_reversed(dr: &mut DepthRange, reversed: bool) {
    dr.reversed_z = reversed;
}

/// Compute depth buffer range ratio (far/near).
#[allow(dead_code)]
pub fn depth_range_ratio(dr: &DepthRange) -> f32 {
    dr.far / dr.near
}

/// Linearize a depth value `[0,1]` to view-space distance.
#[allow(dead_code)]
pub fn linearize_depth_value(depth: f32, dr: &DepthRange) -> f32 {
    let d = if dr.reversed_z { 1.0 - depth } else { depth };
    let z_ndc = d * 2.0 - 1.0;
    2.0 * dr.near * dr.far / (dr.far + dr.near - z_ndc * (dr.far - dr.near))
}

/// Compute precision stats using LN_2 for log-depth analysis.
#[allow(dead_code)]
pub fn compute_precision_stats(dr: &DepthRange) -> DepthPrecisionStats {
    let ratio = depth_range_ratio(dr);
    let log_range = ratio.ln() / LN_2;
    let bits_needed = log_range.ceil() as u32;
    DepthPrecisionStats {
        range_ratio: ratio,
        log_depth_range: log_range,
        recommended_near: dr.far / 1_000_000.0_f32,
        bits_needed,
    }
}

/// Convert NDC z to view-space depth.
#[allow(dead_code)]
pub fn ndc_to_view_depth(z_ndc: f32, dr: &DepthRange) -> f32 {
    2.0 * dr.near * dr.far / (dr.far + dr.near - z_ndc * (dr.far - dr.near))
}

/// Convert view-space depth to NDC z.
#[allow(dead_code)]
pub fn view_to_ndc_depth(view_z: f32, dr: &DepthRange) -> f32 {
    let a = -(dr.far + dr.near) / (dr.far - dr.near);
    let b = -2.0 * dr.far * dr.near / (dr.far - dr.near);
    a + b / view_z
}

/// Check if a depth value is within valid range.
#[allow(dead_code)]
pub fn is_valid_depth(depth: f32) -> bool {
    (0.0..=1.0).contains(&depth)
}

/// Export to JSON-like string.
#[allow(dead_code)]
pub fn depth_range_to_json(dr: &DepthRange) -> String {
    format!(
        r#"{{"near":{:.4},"far":{:.4},"reversed_z":{}}}"#,
        dr.near, dr.far, dr.reversed_z
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_range() {
        let dr = default_depth_range();
        assert!((dr.near - 0.1).abs() < 1e-6);
        assert!((dr.far - 1000.0).abs() < 1e-6);
    }

    #[test]
    fn set_planes() {
        let mut dr = default_depth_range();
        dr_set_planes(&mut dr, 0.5, 500.0);
        assert!((dr.near - 0.5).abs() < 1e-6);
        assert!((dr.far - 500.0).abs() < 1e-6);
    }

    #[test]
    fn range_ratio() {
        let dr = DepthRange {
            near: 1.0,
            far: 1000.0,
            reversed_z: false,
        };
        assert!((depth_range_ratio(&dr) - 1000.0).abs() < 1e-4);
    }

    #[test]
    fn linearize_near() {
        let dr = DepthRange {
            near: 1.0,
            far: 100.0,
            reversed_z: false,
        };
        let d = linearize_depth_value(0.0, &dr);
        assert!(d.is_finite());
    }

    #[test]
    fn precision_stats_bits() {
        let dr = DepthRange {
            near: 0.1,
            far: 1000.0,
            reversed_z: false,
        };
        let stats = compute_precision_stats(&dr);
        assert!(stats.bits_needed > 0);
    }

    #[test]
    fn is_valid_depth_zero() {
        assert!(is_valid_depth(0.0));
    }

    #[test]
    fn is_valid_depth_one() {
        assert!(is_valid_depth(1.0));
    }

    #[test]
    fn is_valid_depth_out_of_range() {
        assert!(!is_valid_depth(1.5));
    }

    #[test]
    fn json_contains_near() {
        let dr = default_depth_range();
        assert!(depth_range_to_json(&dr).contains("near"));
    }

    #[test]
    fn ln2_used_in_stats() {
        let dr = DepthRange {
            near: 1.0,
            far: 2.0,
            reversed_z: false,
        };
        let stats = compute_precision_stats(&dr);
        assert!((stats.log_depth_range - 1.0).abs() < 1e-4);
    }
}
