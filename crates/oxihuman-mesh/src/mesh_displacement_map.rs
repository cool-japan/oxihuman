// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Displacement map application to mesh vertices.

#[allow(dead_code)]
pub struct DisplacementMap {
    pub width: u32,
    pub height: u32,
    pub data: Vec<f32>,
    pub scale: f32,
}

#[allow(dead_code)]
pub fn new_displacement_map(width: u32, height: u32, scale: f32) -> DisplacementMap {
    let size = (width * height) as usize;
    DisplacementMap {
        width,
        height,
        data: vec![0.0; size],
        scale,
    }
}

#[allow(dead_code)]
pub fn dm_set(map: &mut DisplacementMap, x: u32, y: u32, value: f32) {
    let idx = (y * map.width + x) as usize;
    if idx < map.data.len() {
        map.data[idx] = value;
    }
}

#[allow(dead_code)]
pub fn dm_get(map: &DisplacementMap, x: u32, y: u32) -> f32 {
    let idx = (y * map.width + x) as usize;
    if idx < map.data.len() {
        map.data[idx]
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn dm_apply_to_vertex(map: &DisplacementMap, uv: [f32; 2], normal: [f32; 3]) -> [f32; 3] {
    let u = uv[0].clamp(0.0, 1.0);
    let v = uv[1].clamp(0.0, 1.0);
    let px = ((u * (map.width.saturating_sub(1)) as f32) as u32).min(map.width.saturating_sub(1));
    let py = ((v * (map.height.saturating_sub(1)) as f32) as u32).min(map.height.saturating_sub(1));
    let d = dm_get(map, px, py) * map.scale;
    [normal[0] * d, normal[1] * d, normal[2] * d]
}

#[allow(dead_code)]
pub fn dm_max_displacement(map: &DisplacementMap) -> f32 {
    map.data.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
}

#[allow(dead_code)]
pub fn dm_avg_displacement(map: &DisplacementMap) -> f32 {
    if map.data.is_empty() {
        return 0.0;
    }
    let sum: f32 = map.data.iter().sum();
    sum / map.data.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get() {
        let mut m = new_displacement_map(4, 4, 1.0);
        dm_set(&mut m, 1, 2, 0.75);
        assert!((dm_get(&m, 1, 2) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_get_default_zero() {
        let m = new_displacement_map(4, 4, 1.0);
        assert_eq!(dm_get(&m, 0, 0), 0.0);
    }

    #[test]
    fn test_apply_to_vertex_direction() {
        let mut m = new_displacement_map(2, 2, 2.0);
        dm_set(&mut m, 0, 0, 1.0);
        let disp = dm_apply_to_vertex(&m, [0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((disp[1] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_apply_to_vertex_zero_when_zero_map() {
        let m = new_displacement_map(2, 2, 1.0);
        let disp = dm_apply_to_vertex(&m, [0.5, 0.5], [1.0, 0.0, 0.0]);
        assert_eq!(disp, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_max_displacement() {
        let mut m = new_displacement_map(2, 2, 1.0);
        dm_set(&mut m, 1, 1, 0.9);
        assert!((dm_max_displacement(&m) - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_avg_displacement() {
        let mut m = new_displacement_map(2, 1, 1.0);
        dm_set(&mut m, 0, 0, 1.0);
        dm_set(&mut m, 1, 0, 0.0);
        assert!((dm_avg_displacement(&m) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_new_all_zero() {
        let m = new_displacement_map(3, 3, 1.0);
        assert_eq!(m.data.len(), 9);
        assert!(m.data.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_scale_applied() {
        let mut m = new_displacement_map(2, 2, 3.0);
        dm_set(&mut m, 0, 0, 1.0);
        let disp = dm_apply_to_vertex(&m, [0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!((disp[0] - 3.0).abs() < 1e-5);
    }
}
