// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A depth image buffer.
pub struct DepthMap {
    pub width: usize,
    pub height: usize,
    pub data: Vec<f32>,
    pub near_plane: f32,
    pub far_plane: f32,
}

pub fn new_depth_map(width: usize, height: usize, near: f32, far: f32) -> DepthMap {
    DepthMap {
        width,
        height,
        data: vec![far; width * height],
        near_plane: near,
        far_plane: far,
    }
}

pub fn depth_map_set(m: &mut DepthMap, x: usize, y: usize, depth: f32) {
    if x < m.width && y < m.height {
        m.data[y * m.width + x] = depth;
    }
}

pub fn depth_map_get(m: &DepthMap, x: usize, y: usize) -> f32 {
    if x < m.width && y < m.height {
        m.data[y * m.width + x]
    } else {
        m.far_plane
    }
}

pub fn depth_map_to_u16(m: &DepthMap) -> Vec<u16> {
    let range = (m.far_plane - m.near_plane).max(1e-9);
    m.data
        .iter()
        .map(|&d| {
            let t = ((d - m.near_plane) / range).clamp(0.0, 1.0);
            (t * 65535.0) as u16
        })
        .collect()
}

pub fn depth_map_min(m: &DepthMap) -> f32 {
    m.data.iter().cloned().fold(f32::INFINITY, f32::min)
}

pub fn depth_map_max(m: &DepthMap) -> f32 {
    m.data.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
}

pub fn depth_map_normalize(m: &DepthMap) -> Vec<f32> {
    let mn = depth_map_min(m);
    let mx = depth_map_max(m);
    let range = (mx - mn).max(1e-9);
    m.data.iter().map(|&d| (d - mn) / range).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_depth_map_size() {
        let m = new_depth_map(4, 4, 0.1, 100.0);
        assert_eq!(m.data.len(), 16);
    }

    #[test]
    fn test_depth_map_set_get() {
        let mut m = new_depth_map(4, 4, 0.1, 100.0);
        depth_map_set(&mut m, 2, 1, 5.0);
        assert!((depth_map_get(&m, 2, 1) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_depth_map_get_oob() {
        let m = new_depth_map(4, 4, 0.1, 100.0);
        assert!((depth_map_get(&m, 10, 10) - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_depth_map_to_u16_range() {
        let mut m = new_depth_map(2, 1, 0.0, 1.0);
        depth_map_set(&mut m, 0, 0, 0.0);
        depth_map_set(&mut m, 1, 0, 1.0);
        let u16s = depth_map_to_u16(&m);
        assert_eq!(u16s[0], 0);
        assert_eq!(u16s[1], 65535);
    }

    #[test]
    fn test_depth_map_min() {
        let mut m = new_depth_map(3, 1, 0.0, 10.0);
        depth_map_set(&mut m, 0, 0, 2.0);
        depth_map_set(&mut m, 1, 0, 5.0);
        depth_map_set(&mut m, 2, 0, 8.0);
        assert!((depth_map_min(&m) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_depth_map_max() {
        let mut m = new_depth_map(3, 1, 0.0, 10.0);
        depth_map_set(&mut m, 0, 0, 2.0);
        depth_map_set(&mut m, 1, 0, 9.0);
        depth_map_set(&mut m, 2, 0, 4.0);
        assert!((depth_map_max(&m) - 9.0).abs() < 1e-5);
    }

    #[test]
    fn test_depth_map_normalize_range() {
        let mut m = new_depth_map(2, 1, 0.0, 10.0);
        depth_map_set(&mut m, 0, 0, 0.0);
        depth_map_set(&mut m, 1, 0, 10.0);
        let n = depth_map_normalize(&m);
        assert!((n[0] - 0.0).abs() < 1e-5);
        assert!((n[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_depth_map_default_far() {
        let m = new_depth_map(2, 2, 0.1, 50.0);
        assert!((m.data[0] - 50.0).abs() < 1e-5);
    }
}
