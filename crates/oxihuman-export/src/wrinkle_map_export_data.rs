// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct WrinkleMapData {
    pub vertex_count: usize,
    pub weights: Vec<f32>,
    pub normals: Vec<[f32; 3]>,
}

pub fn new_wrinkle_map_data(n: usize) -> WrinkleMapData {
    WrinkleMapData {
        vertex_count: n,
        weights: vec![0.0; n],
        normals: vec![[0.0, 1.0, 0.0]; n],
    }
}

pub fn wrinkle_set(m: &mut WrinkleMapData, i: usize, w: f32, n: [f32; 3]) {
    if i < m.vertex_count {
        m.weights[i] = w.clamp(0.0, 1.0);
        m.normals[i] = n;
    }
}

pub fn wrinkle_get(m: &WrinkleMapData, i: usize) -> (f32, [f32; 3]) {
    if i < m.vertex_count {
        (m.weights[i], m.normals[i])
    } else {
        (0.0, [0.0, 1.0, 0.0])
    }
}

pub fn wrinkle_max_weight(m: &WrinkleMapData) -> f32 {
    m.weights.iter().cloned().fold(0.0f32, f32::max)
}

pub fn wrinkle_active_count(m: &WrinkleMapData, thr: f32) -> usize {
    m.weights.iter().filter(|&&w| w >= thr).count()
}

pub fn wrinkle_to_bytes(m: &WrinkleMapData) -> Vec<u8> {
    let mut b = Vec::new();
    let n = m.vertex_count as u32;
    b.extend_from_slice(&n.to_le_bytes());
    for &w in &m.weights {
        b.extend_from_slice(&w.to_le_bytes());
    }
    for norm in &m.normals {
        for &v in norm {
            b.extend_from_slice(&v.to_le_bytes());
        }
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_wrinkle_map_data() {
        /* vertex count stored */
        let m = new_wrinkle_map_data(5);
        assert_eq!(m.vertex_count, 5);
    }

    #[test]
    fn test_wrinkle_set_get() {
        /* set and get roundtrip */
        let mut m = new_wrinkle_map_data(3);
        wrinkle_set(&mut m, 1, 0.8, [1.0, 0.0, 0.0]);
        let (w, n) = wrinkle_get(&m, 1);
        assert!((w - 0.8).abs() < 1e-6);
        assert!((n[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_wrinkle_max_weight() {
        /* finds max weight */
        let mut m = new_wrinkle_map_data(3);
        wrinkle_set(&mut m, 2, 0.9, [0.0, 1.0, 0.0]);
        assert!((wrinkle_max_weight(&m) - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_wrinkle_active_count() {
        /* counts above threshold */
        let mut m = new_wrinkle_map_data(4);
        wrinkle_set(&mut m, 0, 0.5, [0.0; 3]);
        wrinkle_set(&mut m, 1, 0.8, [0.0; 3]);
        assert_eq!(wrinkle_active_count(&m, 0.5), 2);
    }

    #[test]
    fn test_wrinkle_to_bytes() {
        /* bytes non-empty */
        let m = new_wrinkle_map_data(2);
        let b = wrinkle_to_bytes(&m);
        assert!(!b.is_empty());
    }

    #[test]
    fn test_wrinkle_get_oob() {
        /* out-of-bounds returns defaults */
        let m = new_wrinkle_map_data(2);
        let (w, _) = wrinkle_get(&m, 99);
        assert_eq!(w, 0.0);
    }
}
