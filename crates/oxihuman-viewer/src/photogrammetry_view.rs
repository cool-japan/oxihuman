// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct PhotogrammetryModel {
    pub point_count: usize,
    pub camera_count: usize,
    pub reprojection_error: f32,
    pub scale_m_per_unit: f32,
}

pub fn new_photogrammetry_model(points: usize, cameras: usize) -> PhotogrammetryModel {
    PhotogrammetryModel {
        point_count: points,
        camera_count: cameras,
        reprojection_error: 0.5,
        scale_m_per_unit: 1.0,
    }
}

pub fn photogram_quality_score(m: &PhotogrammetryModel) -> f32 {
    1.0 / (1.0 + m.reprojection_error)
}

pub fn photogram_real_scale_factor(m: &PhotogrammetryModel) -> f32 {
    m.scale_m_per_unit
}

pub fn photogram_is_calibrated(m: &PhotogrammetryModel) -> bool {
    m.reprojection_error < 1.0
}

pub fn photogram_point_density(m: &PhotogrammetryModel, volume_m3: f32) -> f32 {
    if volume_m3 <= 0.0 {
        return 0.0;
    }
    m.point_count as f32 / volume_m3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_model() {
        /* point count is set */
        let m = new_photogrammetry_model(1000, 20);
        assert_eq!(m.point_count, 1000);
    }

    #[test]
    fn test_quality_score() {
        /* quality = 1/(1+error) */
        let m = new_photogrammetry_model(100, 5);
        let q = photogram_quality_score(&m);
        assert!((q - 1.0 / 1.5).abs() < 1e-5);
    }

    #[test]
    fn test_is_calibrated() {
        /* error 0.5 < 1.0 => calibrated */
        let m = new_photogrammetry_model(100, 5);
        assert!(photogram_is_calibrated(&m));
    }

    #[test]
    fn test_not_calibrated() {
        /* error >= 1.0 => not calibrated */
        let mut m = new_photogrammetry_model(100, 5);
        m.reprojection_error = 2.0;
        assert!(!photogram_is_calibrated(&m));
    }

    #[test]
    fn test_point_density() {
        /* density = points / volume */
        let m = new_photogrammetry_model(1000, 10);
        let d = photogram_point_density(&m, 10.0);
        assert!((d - 100.0).abs() < 1e-5);
    }

    #[test]
    fn test_point_density_zero_volume() {
        /* zero volume => 0 density */
        let m = new_photogrammetry_model(100, 5);
        assert!((photogram_point_density(&m, 0.0) - 0.0).abs() < 1e-6);
    }
}
