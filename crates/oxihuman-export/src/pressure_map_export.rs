// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// 2D pressure distribution map.
pub struct PressureMap {
    pub width: usize,
    pub height: usize,
    pub pressures: Vec<f32>,
}

pub fn new_pressure_map(w: usize, h: usize) -> PressureMap {
    PressureMap {
        width: w,
        height: h,
        pressures: vec![0.0; w * h],
    }
}

pub fn pressure_set(m: &mut PressureMap, x: usize, y: usize, p: f32) {
    if x < m.width && y < m.height {
        m.pressures[y * m.width + x] = p;
    }
}

pub fn pressure_get(m: &PressureMap, x: usize, y: usize) -> f32 {
    if x < m.width && y < m.height {
        m.pressures[y * m.width + x]
    } else {
        0.0
    }
}

pub fn pressure_total_force(m: &PressureMap, cell_area: f32) -> f32 {
    m.pressures.iter().sum::<f32>() * cell_area
}

pub fn pressure_center_of_pressure(m: &PressureMap) -> [f32; 2] {
    let total: f32 = m.pressures.iter().sum();
    if total < 1e-9 {
        return [m.width as f32 * 0.5, m.height as f32 * 0.5];
    }
    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    for y in 0..m.height {
        for x in 0..m.width {
            let p = m.pressures[y * m.width + x];
            cx += p * x as f32;
            cy += p * y as f32;
        }
    }
    [cx / total, cy / total]
}

pub fn pressure_max(m: &PressureMap) -> f32 {
    m.pressures
        .iter()
        .cloned()
        .fold(f32::NEG_INFINITY, f32::max)
}

pub fn pressure_to_bytes(m: &PressureMap) -> Vec<u8> {
    let mut out = Vec::with_capacity(m.pressures.len() * 4);
    for &p in &m.pressures {
        out.extend_from_slice(&p.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pressure_map_size() {
        let m = new_pressure_map(5, 5);
        assert_eq!(m.pressures.len(), 25);
    }

    #[test]
    fn test_pressure_set_get() {
        let mut m = new_pressure_map(4, 4);
        pressure_set(&mut m, 2, 1, 100.0);
        assert!((pressure_get(&m, 2, 1) - 100.0).abs() < 1e-4);
    }

    #[test]
    fn test_pressure_get_oob() {
        let m = new_pressure_map(4, 4);
        assert!((pressure_get(&m, 10, 10) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_pressure_total_force() {
        let mut m = new_pressure_map(2, 1);
        pressure_set(&mut m, 0, 0, 10.0);
        pressure_set(&mut m, 1, 0, 20.0);
        assert!((pressure_total_force(&m, 0.01) - 0.3).abs() < 1e-5);
    }

    #[test]
    fn test_pressure_center_of_pressure() {
        let mut m = new_pressure_map(3, 1);
        pressure_set(&mut m, 0, 0, 0.0);
        pressure_set(&mut m, 1, 0, 1.0);
        pressure_set(&mut m, 2, 0, 0.0);
        let cop = pressure_center_of_pressure(&m);
        assert!((cop[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_pressure_max() {
        let mut m = new_pressure_map(3, 1);
        pressure_set(&mut m, 0, 0, 50.0);
        pressure_set(&mut m, 1, 0, 200.0);
        pressure_set(&mut m, 2, 0, 75.0);
        assert!((pressure_max(&m) - 200.0).abs() < 1e-4);
    }

    #[test]
    fn test_pressure_to_bytes_len() {
        let m = new_pressure_map(4, 4);
        assert_eq!(pressure_to_bytes(&m).len(), 64);
    }

    #[test]
    fn test_pressure_center_empty() {
        let m = new_pressure_map(4, 4);
        /* should not panic with all zeros */
        let _ = pressure_center_of_pressure(&m);
    }
}
