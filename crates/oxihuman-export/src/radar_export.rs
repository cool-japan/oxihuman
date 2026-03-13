// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Radar point cloud export stub.

#[allow(dead_code)]
pub struct RadarPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub velocity: f32,
    pub snr: f32,
}

#[allow(dead_code)]
pub struct RadarExport {
    pub points: Vec<RadarPoint>,
    pub frequency_hz: f64,
}

#[allow(dead_code)]
pub fn new_radar_export(frequency_hz: f64) -> RadarExport {
    RadarExport { points: Vec::new(), frequency_hz }
}

#[allow(dead_code)]
pub fn radar_add_point(exp: &mut RadarExport, x: f32, y: f32, z: f32, velocity: f32, snr: f32) {
    exp.points.push(RadarPoint { x, y, z, velocity, snr });
}

#[allow(dead_code)]
pub fn radar_point_count(exp: &RadarExport) -> usize {
    exp.points.len()
}

#[allow(dead_code)]
pub fn radar_avg_snr(exp: &RadarExport) -> f32 {
    let n = exp.points.len();
    if n == 0 { return 0.0; }
    exp.points.iter().map(|p| p.snr).sum::<f32>() / n as f32
}

#[allow(dead_code)]
pub fn radar_max_velocity(exp: &RadarExport) -> f32 {
    exp.points.iter().map(|p| p.velocity.abs()).fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn radar_to_header(exp: &RadarExport) -> String {
    format!("RADAR freq={:.1}Hz points={}", exp.frequency_hz, exp.points.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let exp = new_radar_export(77e9);
        assert_eq!(radar_point_count(&exp), 0);
    }

    #[test]
    fn test_add_point() {
        let mut exp = new_radar_export(77e9);
        radar_add_point(&mut exp, 1.0, 2.0, 0.0, 5.0, 20.0);
        assert_eq!(radar_point_count(&exp), 1);
    }

    #[test]
    fn test_avg_snr_empty() {
        let exp = new_radar_export(77e9);
        assert_eq!(radar_avg_snr(&exp), 0.0);
    }

    #[test]
    fn test_avg_snr() {
        let mut exp = new_radar_export(77e9);
        radar_add_point(&mut exp, 0.0, 0.0, 0.0, 1.0, 10.0);
        radar_add_point(&mut exp, 0.0, 0.0, 0.0, 1.0, 30.0);
        assert!((radar_avg_snr(&exp) - 20.0).abs() < 1e-4);
    }

    #[test]
    fn test_max_velocity() {
        let mut exp = new_radar_export(77e9);
        radar_add_point(&mut exp, 0.0, 0.0, 0.0, 3.0, 10.0);
        radar_add_point(&mut exp, 0.0, 0.0, 0.0, -7.0, 10.0);
        radar_add_point(&mut exp, 0.0, 0.0, 0.0, 2.0, 10.0);
        assert!((radar_max_velocity(&exp) - 7.0).abs() < 1e-4);
    }

    #[test]
    fn test_to_header_contains_freq() {
        let exp = new_radar_export(77.0e9);
        let h = radar_to_header(&exp);
        assert!(h.contains("RADAR"));
        assert!(h.contains("freq"));
    }

    #[test]
    fn test_to_header_contains_count() {
        let mut exp = new_radar_export(24e9);
        radar_add_point(&mut exp, 0.0, 0.0, 0.0, 0.0, 0.0);
        assert!(radar_to_header(&exp).contains('1'));
    }

    #[test]
    fn test_frequency_stored() {
        let exp = new_radar_export(24.0e9);
        assert!((exp.frequency_hz - 24.0e9).abs() < 1.0);
    }

    #[test]
    fn test_max_velocity_empty() {
        let exp = new_radar_export(77e9);
        assert_eq!(radar_max_velocity(&exp), 0.0);
    }
}
