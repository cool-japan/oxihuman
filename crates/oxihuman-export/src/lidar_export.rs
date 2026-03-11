// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! LiDAR scan export (range + angle pairs).

#[allow(dead_code)]
pub struct LidarScan {
    pub ranges: Vec<f32>,
    pub angle_min: f32,
    pub angle_increment: f32,
}

#[allow(dead_code)]
pub fn new_lidar_scan(angle_min: f32, angle_max: f32, n_rays: usize) -> LidarScan {
    let increment = if n_rays > 1 { (angle_max - angle_min) / (n_rays - 1) as f32 } else { 0.0 };
    LidarScan { ranges: vec![0.0; n_rays], angle_min, angle_increment: increment }
}

#[allow(dead_code)]
pub fn ls_set_range(scan: &mut LidarScan, i: usize, r: f32) {
    if i < scan.ranges.len() { scan.ranges[i] = r; }
}

#[allow(dead_code)]
pub fn ls_get_range(scan: &LidarScan, i: usize) -> f32 {
    if i < scan.ranges.len() { scan.ranges[i] } else { 0.0 }
}

#[allow(dead_code)]
pub fn ls_ray_angle(scan: &LidarScan, i: usize) -> f32 {
    scan.angle_min + i as f32 * scan.angle_increment
}

#[allow(dead_code)]
pub fn ls_to_cartesian(scan: &LidarScan) -> Vec<[f32; 2]> {
    scan.ranges.iter().enumerate().map(|(i, &r)| {
        let angle = ls_ray_angle(scan, i);
        [r * angle.cos(), r * angle.sin()]
    }).collect()
}

#[allow(dead_code)]
pub fn ls_max_range(scan: &LidarScan) -> f32 {
    scan.ranges.iter().copied().fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn ls_min_range(scan: &LidarScan) -> f32 {
    scan.ranges.iter().copied().fold(f32::MAX, f32::min)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_new() {
        let scan = new_lidar_scan(0.0, PI, 5);
        assert_eq!(scan.ranges.len(), 5);
    }

    #[test]
    fn test_set_get_range() {
        let mut scan = new_lidar_scan(0.0, PI, 5);
        ls_set_range(&mut scan, 2, 3.5);
        assert!((ls_get_range(&scan, 2) - 3.5).abs() < 1e-5);
    }

    #[test]
    fn test_ray_angle_first() {
        let scan = new_lidar_scan(0.0, PI, 3);
        assert!(ls_ray_angle(&scan, 0).abs() < 1e-5);
    }

    #[test]
    fn test_ray_angle_last() {
        let scan = new_lidar_scan(0.0, PI, 3);
        assert!((ls_ray_angle(&scan, 2) - PI).abs() < 1e-4);
    }

    #[test]
    fn test_to_cartesian_count() {
        let scan = new_lidar_scan(0.0, PI, 4);
        let pts = ls_to_cartesian(&scan);
        assert_eq!(pts.len(), 4);
    }

    #[test]
    fn test_max_range() {
        let mut scan = new_lidar_scan(0.0, PI, 3);
        ls_set_range(&mut scan, 0, 1.0);
        ls_set_range(&mut scan, 1, 5.0);
        ls_set_range(&mut scan, 2, 2.0);
        assert!((ls_max_range(&scan) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_min_range() {
        let mut scan = new_lidar_scan(0.0, PI, 3);
        ls_set_range(&mut scan, 0, 3.0);
        ls_set_range(&mut scan, 1, 1.5);
        ls_set_range(&mut scan, 2, 2.0);
        assert!((ls_min_range(&scan) - 1.5).abs() < 1e-5);
    }

    #[test]
    fn test_cartesian_angle_zero() {
        let mut scan = new_lidar_scan(0.0, PI, 3);
        ls_set_range(&mut scan, 0, 5.0);
        let pts = ls_to_cartesian(&scan);
        assert!((pts[0][0] - 5.0).abs() < 1e-4);
        assert!(pts[0][1].abs() < 1e-4);
    }
}
