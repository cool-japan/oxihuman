// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// 3D velocity/flow field.
pub struct FlowField {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    pub velocities: Vec<[f32; 3]>,
}

pub fn new_flow_field(w: usize, h: usize, d: usize) -> FlowField {
    FlowField {
        width: w,
        height: h,
        depth: d,
        velocities: vec![[0.0; 3]; w * h * d],
    }
}

fn flow_index(f: &FlowField, x: usize, y: usize, z: usize) -> usize {
    z * f.height * f.width + y * f.width + x
}

pub fn flow_set(f: &mut FlowField, x: usize, y: usize, z: usize, v: [f32; 3]) {
    if x < f.width && y < f.height && z < f.depth {
        let idx = flow_index(f, x, y, z);
        f.velocities[idx] = v;
    }
}

pub fn flow_get(f: &FlowField, x: usize, y: usize, z: usize) -> [f32; 3] {
    if x < f.width && y < f.height && z < f.depth {
        f.velocities[flow_index(f, x, y, z)]
    } else {
        [0.0; 3]
    }
}

pub fn flow_max_speed(f: &FlowField) -> f32 {
    f.velocities
        .iter()
        .map(|&v| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt())
        .fold(0.0f32, f32::max)
}

pub fn flow_divergence_at(f: &FlowField, x: usize, y: usize, z: usize) -> f32 {
    if x == 0 || x + 1 >= f.width || y == 0 || y + 1 >= f.height || z == 0 || z + 1 >= f.depth {
        return 0.0;
    }
    let dvx = flow_get(f, x + 1, y, z)[0] - flow_get(f, x - 1, y, z)[0];
    let dvy = flow_get(f, x, y + 1, z)[1] - flow_get(f, x, y - 1, z)[1];
    let dvz = flow_get(f, x, y, z + 1)[2] - flow_get(f, x, y, z - 1)[2];
    (dvx + dvy + dvz) * 0.5
}

pub fn flow_to_bytes(f: &FlowField) -> Vec<u8> {
    let mut out = Vec::with_capacity(f.velocities.len() * 12);
    for &v in &f.velocities {
        out.extend_from_slice(&v[0].to_le_bytes());
        out.extend_from_slice(&v[1].to_le_bytes());
        out.extend_from_slice(&v[2].to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_flow_field_size() {
        let f = new_flow_field(2, 3, 4);
        assert_eq!(f.velocities.len(), 24);
    }

    #[test]
    fn test_flow_set_get() {
        let mut f = new_flow_field(4, 4, 4);
        flow_set(&mut f, 1, 2, 3, [1.0, 2.0, 3.0]);
        let v = flow_get(&f, 1, 2, 3);
        assert!((v[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_flow_get_oob() {
        let f = new_flow_field(4, 4, 4);
        let v = flow_get(&f, 10, 10, 10);
        assert_eq!(v, [0.0; 3]);
    }

    #[test]
    fn test_flow_max_speed() {
        let mut f = new_flow_field(3, 1, 1);
        flow_set(&mut f, 0, 0, 0, [3.0, 4.0, 0.0]);
        flow_set(&mut f, 1, 0, 0, [1.0, 0.0, 0.0]);
        assert!((flow_max_speed(&f) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_flow_divergence_at_interior() {
        /* uniform flow has zero divergence */
        let mut f = new_flow_field(5, 5, 5);
        for z in 0..5 {
            for y in 0..5 {
                for x in 0..5 {
                    flow_set(&mut f, x, y, z, [1.0, 0.0, 0.0]);
                }
            }
        }
        let div = flow_divergence_at(&f, 2, 2, 2);
        assert!(div.abs() < 1e-4);
    }

    #[test]
    fn test_flow_to_bytes_len() {
        let f = new_flow_field(2, 2, 2);
        assert_eq!(flow_to_bytes(&f).len(), 8 * 12);
    }

    #[test]
    fn test_flow_divergence_at_boundary() {
        let f = new_flow_field(4, 4, 4);
        /* boundary should return 0 without panic */
        assert!((flow_divergence_at(&f, 0, 0, 0) - 0.0).abs() < 1e-6);
    }
}
