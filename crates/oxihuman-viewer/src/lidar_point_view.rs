// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct LidarPointCloud {
    pub points: Vec<[f32; 3]>,
    pub intensities: Vec<f32>,
    pub return_number: Vec<u8>,
}

pub fn new_lidar_point_cloud() -> LidarPointCloud {
    LidarPointCloud {
        points: Vec::new(),
        intensities: Vec::new(),
        return_number: Vec::new(),
    }
}

pub fn lidar_push_point(pc: &mut LidarPointCloud, pos: [f32; 3], intensity: f32, ret: u8) {
    pc.points.push(pos);
    pc.intensities.push(intensity);
    pc.return_number.push(ret);
}

pub fn lidar_point_count(pc: &LidarPointCloud) -> usize {
    pc.points.len()
}

pub fn lidar_bounding_box(pc: &LidarPointCloud) -> ([f32; 3], [f32; 3]) {
    if pc.points.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = pc.points[0];
    let mut mx = pc.points[0];
    for p in &pc.points {
        for i in 0..3 {
            if p[i] < mn[i] {
                mn[i] = p[i];
            }
            if p[i] > mx[i] {
                mx[i] = p[i];
            }
        }
    }
    (mn, mx)
}

pub fn lidar_mean_intensity(pc: &LidarPointCloud) -> f32 {
    if pc.intensities.is_empty() {
        return 0.0;
    }
    pc.intensities.iter().sum::<f32>() / pc.intensities.len() as f32
}

pub fn lidar_filter_by_return(pc: &LidarPointCloud, ret: u8) -> Vec<usize> {
    pc.return_number
        .iter()
        .enumerate()
        .filter(|(_, &r)| r == ret)
        .map(|(i, _)| i)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        /* empty on creation */
        let pc = new_lidar_point_cloud();
        assert_eq!(lidar_point_count(&pc), 0);
    }

    #[test]
    fn test_push_point() {
        /* push adds a point */
        let mut pc = new_lidar_point_cloud();
        lidar_push_point(&mut pc, [1.0, 2.0, 3.0], 0.5, 1);
        assert_eq!(lidar_point_count(&pc), 1);
    }

    #[test]
    fn test_bounding_box_empty() {
        /* empty cloud returns zeros */
        let pc = new_lidar_point_cloud();
        let (mn, mx) = lidar_bounding_box(&pc);
        assert_eq!(mn, [0.0; 3]);
        assert_eq!(mx, [0.0; 3]);
    }

    #[test]
    fn test_bounding_box_single() {
        /* single point => min == max */
        let mut pc = new_lidar_point_cloud();
        lidar_push_point(&mut pc, [5.0, 6.0, 7.0], 1.0, 1);
        let (mn, mx) = lidar_bounding_box(&pc);
        assert_eq!(mn, [5.0, 6.0, 7.0]);
        assert_eq!(mx, [5.0, 6.0, 7.0]);
    }

    #[test]
    fn test_mean_intensity() {
        /* mean of [0.5, 1.0] = 0.75 */
        let mut pc = new_lidar_point_cloud();
        lidar_push_point(&mut pc, [0.0; 3], 0.5, 1);
        lidar_push_point(&mut pc, [1.0; 3], 1.0, 1);
        assert!((lidar_mean_intensity(&pc) - 0.75).abs() < 1e-5);
    }

    #[test]
    fn test_filter_by_return() {
        /* filter keeps matching return numbers */
        let mut pc = new_lidar_point_cloud();
        lidar_push_point(&mut pc, [0.0; 3], 1.0, 1);
        lidar_push_point(&mut pc, [1.0; 3], 1.0, 2);
        lidar_push_point(&mut pc, [2.0; 3], 1.0, 1);
        let indices = lidar_filter_by_return(&pc, 1);
        assert_eq!(indices, vec![0, 2]);
    }

    #[test]
    fn test_mean_intensity_empty() {
        /* empty returns 0 */
        let pc = new_lidar_point_cloud();
        assert!((lidar_mean_intensity(&pc) - 0.0).abs() < 1e-6);
    }
}
