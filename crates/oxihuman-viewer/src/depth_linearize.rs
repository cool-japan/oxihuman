// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Depth linearization: converts non-linear depth buffer values to linear eye-space depth.

/// Camera projection parameters needed for depth linearization.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct DepthLinearizeParams {
    pub near: f32,
    pub far: f32,
    pub reversed_z: bool,
}

#[allow(dead_code)]
pub fn default_depth_params() -> DepthLinearizeParams {
    DepthLinearizeParams {
        near: 0.1,
        far: 1000.0,
        reversed_z: false,
    }
}

/// Linearize a single depth value from `[0,1]` to eye-space distance.
#[allow(dead_code)]
pub fn linearize_depth(z_ndc: f32, params: &DepthLinearizeParams) -> f32 {
    let z = if params.reversed_z {
        1.0 - z_ndc
    } else {
        z_ndc
    };
    let z = z.clamp(0.0, 1.0);
    let n = params.near;
    let f = params.far;
    if (f - n).abs() < 1e-9 {
        return n;
    }
    n * f / (f - z * (f - n))
}

/// Linearize an entire depth buffer.
#[allow(dead_code)]
pub fn linearize_buffer(buffer: &[f32], params: &DepthLinearizeParams) -> Vec<f32> {
    buffer.iter().map(|&z| linearize_depth(z, params)).collect()
}

/// Convert linear depth back to non-linear NDC depth.
#[allow(dead_code)]
pub fn depth_to_ndc(linear: f32, params: &DepthLinearizeParams) -> f32 {
    let n = params.near;
    let f = params.far;
    if linear.abs() < 1e-9 || (f - n).abs() < 1e-9 {
        return 0.0;
    }
    let z = (f * (linear - n)) / (linear * (f - n));
    if params.reversed_z {
        1.0 - z
    } else {
        z
    }
}

/// Compute depth precision at a given linear depth.
#[allow(dead_code)]
pub fn depth_precision(linear: f32, params: &DepthLinearizeParams) -> f32 {
    let n = params.near;
    let f = params.far;
    if linear.abs() < 1e-9 || (f - n).abs() < 1e-9 {
        return 0.0;
    }
    n * f / (linear * linear * (f - n))
}

#[allow(dead_code)]
pub fn is_reversed_z(params: &DepthLinearizeParams) -> bool {
    params.reversed_z
}

#[allow(dead_code)]
pub fn depth_range(params: &DepthLinearizeParams) -> f32 {
    params.far - params.near
}

#[allow(dead_code)]
pub fn depth_params_to_json(params: &DepthLinearizeParams) -> String {
    format!(
        r#"{{"near":{:.4},"far":{:.4},"reversed_z":{}}}"#,
        params.near, params.far, params.reversed_z
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let p = default_depth_params();
        assert!((p.near - 0.1).abs() < 1e-6);
        assert!((p.far - 1000.0).abs() < 1e-3);
    }

    #[test]
    fn test_linearize_at_near() {
        let p = default_depth_params();
        let d = linearize_depth(0.0, &p);
        assert!((d - 0.1).abs() < 1e-3);
    }

    #[test]
    fn test_linearize_at_far() {
        let p = default_depth_params();
        let d = linearize_depth(1.0, &p);
        assert!((d - 1000.0).abs() < 1.0);
    }

    #[test]
    fn test_linearize_buffer() {
        let p = default_depth_params();
        let buf = vec![0.0, 0.5, 1.0];
        let lin = linearize_buffer(&buf, &p);
        assert_eq!(lin.len(), 3);
        assert!(lin[0] < lin[1]);
    }

    #[test]
    fn test_depth_to_ndc_roundtrip() {
        let p = default_depth_params();
        let linear = 50.0;
        let ndc = depth_to_ndc(linear, &p);
        let back = linearize_depth(ndc, &p);
        assert!((back - linear).abs() < 0.1);
    }

    #[test]
    fn test_reversed_z() {
        let p = DepthLinearizeParams {
            near: 0.1,
            far: 100.0,
            reversed_z: true,
        };
        let d_near = linearize_depth(1.0, &p);
        assert!((d_near - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_depth_precision() {
        let p = default_depth_params();
        let near_prec = depth_precision(1.0, &p);
        let far_prec = depth_precision(500.0, &p);
        assert!(near_prec > far_prec);
    }

    #[test]
    fn test_depth_range() {
        let p = default_depth_params();
        assert!((depth_range(&p) - 999.9).abs() < 0.1);
    }

    #[test]
    fn test_is_reversed_z() {
        let p = default_depth_params();
        assert!(!is_reversed_z(&p));
    }

    #[test]
    fn test_depth_params_to_json() {
        let p = default_depth_params();
        let j = depth_params_to_json(&p);
        assert!(j.contains("near"));
    }
}
