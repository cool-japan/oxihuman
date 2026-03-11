// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! 3D Gaussian Splat export stub.

#[allow(dead_code)]
pub struct GaussianSplat {
    pub position: [f32; 3],
    pub scale: [f32; 3],
    pub rotation: [f32; 4],
    pub opacity: f32,
    pub color: [f32; 3],
}

#[allow(dead_code)]
pub struct GaussianSplatExport {
    pub splats: Vec<GaussianSplat>,
}

#[allow(dead_code)]
pub fn new_gaussian_splat_export() -> GaussianSplatExport {
    GaussianSplatExport { splats: Vec::new() }
}

#[allow(dead_code)]
pub fn gs_add(exp: &mut GaussianSplatExport, pos: [f32; 3], scale: [f32; 3], opacity: f32, color: [f32; 3]) {
    exp.splats.push(GaussianSplat {
        position: pos,
        scale,
        rotation: [0.0, 0.0, 0.0, 1.0],
        opacity,
        color,
    });
}

#[allow(dead_code)]
pub fn gs_count(exp: &GaussianSplatExport) -> usize {
    exp.splats.len()
}

#[allow(dead_code)]
pub fn gs_avg_opacity(exp: &GaussianSplatExport) -> f32 {
    let n = exp.splats.len();
    if n == 0 { return 0.0; }
    exp.splats.iter().map(|s| s.opacity).sum::<f32>() / n as f32
}

#[allow(dead_code)]
pub fn gs_to_header(exp: &GaussianSplatExport) -> String {
    format!("gaussian_splat count={}", exp.splats.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let exp = new_gaussian_splat_export();
        assert_eq!(gs_count(&exp), 0);
    }

    #[test]
    fn test_add_splat() {
        let mut exp = new_gaussian_splat_export();
        gs_add(&mut exp, [0.0; 3], [1.0; 3], 0.8, [1.0, 0.0, 0.0]);
        assert_eq!(gs_count(&exp), 1);
    }

    #[test]
    fn test_default_rotation_identity() {
        let mut exp = new_gaussian_splat_export();
        gs_add(&mut exp, [0.0; 3], [1.0; 3], 0.5, [1.0; 3]);
        assert_eq!(exp.splats[0].rotation, [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_avg_opacity_empty() {
        let exp = new_gaussian_splat_export();
        assert_eq!(gs_avg_opacity(&exp), 0.0);
    }

    #[test]
    fn test_avg_opacity() {
        let mut exp = new_gaussian_splat_export();
        gs_add(&mut exp, [0.0; 3], [1.0; 3], 0.4, [0.0; 3]);
        gs_add(&mut exp, [0.0; 3], [1.0; 3], 0.8, [0.0; 3]);
        assert!((gs_avg_opacity(&exp) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_to_header_contains_splat() {
        let exp = new_gaussian_splat_export();
        assert!(gs_to_header(&exp).contains("splat"));
    }

    #[test]
    fn test_to_header_contains_count() {
        let mut exp = new_gaussian_splat_export();
        gs_add(&mut exp, [0.0; 3], [1.0; 3], 1.0, [0.0; 3]);
        assert!(gs_to_header(&exp).contains('1'));
    }

    #[test]
    fn test_multiple_splats() {
        let mut exp = new_gaussian_splat_export();
        for _ in 0..8 {
            gs_add(&mut exp, [0.0; 3], [1.0; 3], 0.5, [0.0; 3]);
        }
        assert_eq!(gs_count(&exp), 8);
    }
}
