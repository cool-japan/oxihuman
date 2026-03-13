// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct FlowMap {
    pub tangent_dirs: Vec<[f32; 2]>,
    pub weights: Vec<f32>,
    pub vertex_count: usize,
}

pub fn new_flow_map(n: usize) -> FlowMap {
    FlowMap {
        tangent_dirs: vec![[1.0, 0.0]; n],
        weights: vec![0.0; n],
        vertex_count: n,
    }
}

pub fn flow_map_set(f: &mut FlowMap, i: usize, dir: [f32; 2], w: f32) {
    f.tangent_dirs[i] = dir;
    f.weights[i] = w;
}

pub fn flow_map_get(f: &FlowMap, i: usize) -> ([f32; 2], f32) {
    (f.tangent_dirs[i], f.weights[i])
}

pub fn flow_map_mean_speed(f: &FlowMap) -> f32 {
    if f.weights.is_empty() {
        return 0.0;
    }
    let total: f32 = f
        .tangent_dirs
        .iter()
        .zip(f.weights.iter())
        .map(|(d, &w)| w * (d[0] * d[0] + d[1] * d[1]).sqrt())
        .sum();
    total / f.vertex_count as f32
}

pub fn flow_map_to_color(dir: [f32; 2]) -> [f32; 3] {
    /* remap [-1,1] to [0,1] */
    let rx = (dir[0] * 0.5 + 0.5).clamp(0.0, 1.0);
    let ry = (dir[1] * 0.5 + 0.5).clamp(0.0, 1.0);
    [rx, ry, 0.5]
}

pub fn flow_map_normalize_dir(dir: [f32; 2]) -> [f32; 2] {
    let len = (dir[0] * dir[0] + dir[1] * dir[1]).sqrt();
    if len < 1e-8 {
        [0.0, 0.0]
    } else {
        [dir[0] / len, dir[1] / len]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_flow_map() {
        /* construction */
        let f = new_flow_map(4);
        assert_eq!(f.vertex_count, 4);
    }

    #[test]
    fn test_set_get() {
        /* round-trip */
        let mut f = new_flow_map(3);
        flow_map_set(&mut f, 1, [0.0, 1.0], 0.8);
        let (d, w) = flow_map_get(&f, 1);
        assert!((d[1] - 1.0).abs() < 1e-6);
        assert!((w - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_dir() {
        /* normalization */
        let d = flow_map_normalize_dir([3.0, 4.0]);
        let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
        assert!((len - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_zero() {
        /* zero stays zero */
        let d = flow_map_normalize_dir([0.0, 0.0]);
        assert!(d[0].abs() < 1e-6);
        assert!(d[1].abs() < 1e-6);
    }

    #[test]
    fn test_to_color() {
        /* right direction -> [1, 0.5, 0.5] */
        let c = flow_map_to_color([1.0, 0.0]);
        assert!((c[0] - 1.0).abs() < 1e-6);
        assert!((c[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_mean_speed_zero_weights() {
        /* zero weights => zero mean */
        let f = new_flow_map(3);
        assert!((flow_map_mean_speed(&f)).abs() < 1e-6);
    }
}
