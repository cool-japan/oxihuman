// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Thermal image buffer (temperatures in Celsius).
pub struct ThermalMap {
    pub width: usize,
    pub height: usize,
    pub temperatures_c: Vec<f32>,
}

pub fn new_thermal_map(width: usize, height: usize) -> ThermalMap {
    ThermalMap {
        width,
        height,
        temperatures_c: vec![20.0; width * height],
    }
}

pub fn thermal_set(m: &mut ThermalMap, x: usize, y: usize, temp_c: f32) {
    if x < m.width && y < m.height {
        m.temperatures_c[y * m.width + x] = temp_c;
    }
}

pub fn thermal_get(m: &ThermalMap, x: usize, y: usize) -> f32 {
    if x < m.width && y < m.height {
        m.temperatures_c[y * m.width + x]
    } else {
        0.0
    }
}

pub fn thermal_to_false_color(m: &ThermalMap) -> Vec<[u8; 3]> {
    let mn = m
        .temperatures_c
        .iter()
        .cloned()
        .fold(f32::INFINITY, f32::min);
    let mx = m
        .temperatures_c
        .iter()
        .cloned()
        .fold(f32::NEG_INFINITY, f32::max);
    let range = (mx - mn).max(1e-9);
    m.temperatures_c
        .iter()
        .map(|&t| {
            let v = ((t - mn) / range).clamp(0.0, 1.0);
            // blue=cold -> red=hot
            let r = (v * 255.0) as u8;
            let b = ((1.0 - v) * 255.0) as u8;
            [r, 0, b]
        })
        .collect()
}

pub fn thermal_mean_temp(m: &ThermalMap) -> f32 {
    let n = m.temperatures_c.len();
    if n == 0 {
        return 0.0;
    }
    m.temperatures_c.iter().sum::<f32>() / n as f32
}

pub fn thermal_to_bytes(m: &ThermalMap) -> Vec<u8> {
    let mut out = Vec::with_capacity(m.temperatures_c.len() * 4);
    for &t in &m.temperatures_c {
        out.extend_from_slice(&t.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_thermal_map_size() {
        let m = new_thermal_map(8, 8);
        assert_eq!(m.temperatures_c.len(), 64);
    }

    #[test]
    fn test_thermal_set_get() {
        let mut m = new_thermal_map(4, 4);
        thermal_set(&mut m, 1, 2, 37.5);
        assert!((thermal_get(&m, 1, 2) - 37.5).abs() < 1e-5);
    }

    #[test]
    fn test_thermal_get_oob() {
        let m = new_thermal_map(4, 4);
        assert!((thermal_get(&m, 10, 10) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_thermal_false_color_count() {
        let m = new_thermal_map(3, 3);
        let colors = thermal_to_false_color(&m);
        assert_eq!(colors.len(), 9);
    }

    #[test]
    fn test_thermal_mean_temp() {
        let mut m = new_thermal_map(2, 1);
        thermal_set(&mut m, 0, 0, 10.0);
        thermal_set(&mut m, 1, 0, 20.0);
        assert!((thermal_mean_temp(&m) - 15.0).abs() < 1e-4);
    }

    #[test]
    fn test_thermal_to_bytes_len() {
        /* 4x4 = 16 pixels * 4 bytes each = 64 bytes */
        let m = new_thermal_map(4, 4);
        assert_eq!(thermal_to_bytes(&m).len(), 16 * 4);
    }

    #[test]
    fn test_thermal_false_color_hot_red() {
        let mut m = new_thermal_map(2, 1);
        thermal_set(&mut m, 0, 0, 0.0);
        thermal_set(&mut m, 1, 0, 100.0);
        let colors = thermal_to_false_color(&m);
        /* hottest pixel should have high R, low B */
        assert!(colors[1][0] > 200);
        assert!(colors[1][2] < 50);
    }
}
