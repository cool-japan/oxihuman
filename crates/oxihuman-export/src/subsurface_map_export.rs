// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct SubsurfaceMap {
    pub width: u32,
    pub height: u32,
    pub radius: Vec<[f32; 3]>,
    pub color: Vec<[f32; 3]>,
}

pub fn new_subsurface_map(w: u32, h: u32) -> SubsurfaceMap {
    let n = (w * h) as usize;
    SubsurfaceMap {
        width: w,
        height: h,
        radius: vec![[1.0, 0.2, 0.1]; n],
        color: vec![[1.0, 0.8, 0.7]; n],
    }
}

pub fn sss_set(m: &mut SubsurfaceMap, x: u32, y: u32, radius: [f32; 3], color: [f32; 3]) {
    if x < m.width && y < m.height {
        let idx = (y * m.width + x) as usize;
        m.radius[idx] = radius;
        m.color[idx] = color;
    }
}

pub fn sss_get_radius(m: &SubsurfaceMap, x: u32, y: u32) -> [f32; 3] {
    if x < m.width && y < m.height {
        m.radius[(y * m.width + x) as usize]
    } else {
        [0.0; 3]
    }
}

pub fn sss_get_color(m: &SubsurfaceMap, x: u32, y: u32) -> [f32; 3] {
    if x < m.width && y < m.height {
        m.color[(y * m.width + x) as usize]
    } else {
        [0.0; 3]
    }
}

pub fn sss_to_bytes(m: &SubsurfaceMap) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&m.width.to_le_bytes());
    b.extend_from_slice(&m.height.to_le_bytes());
    for (r, c) in m.radius.iter().zip(m.color.iter()) {
        for &v in r {
            b.extend_from_slice(&v.to_le_bytes());
        }
        for &v in c {
            b.extend_from_slice(&v.to_le_bytes());
        }
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_subsurface_map() {
        /* correct size */
        let m = new_subsurface_map(4, 4);
        assert_eq!(m.radius.len(), 16);
    }

    #[test]
    fn test_sss_set_get_radius() {
        /* set and get radius */
        let mut m = new_subsurface_map(4, 4);
        sss_set(&mut m, 1, 1, [2.0, 1.0, 0.5], [1.0, 0.5, 0.3]);
        let r = sss_get_radius(&m, 1, 1);
        assert!((r[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_sss_get_color() {
        /* color retrieved correctly */
        let mut m = new_subsurface_map(2, 2);
        sss_set(&mut m, 0, 0, [1.0; 3], [0.5, 0.6, 0.7]);
        let c = sss_get_color(&m, 0, 0);
        assert!((c[1] - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_sss_to_bytes() {
        /* bytes non-empty */
        let m = new_subsurface_map(2, 2);
        let b = sss_to_bytes(&m);
        assert!(!b.is_empty());
    }

    #[test]
    fn test_sss_get_oob() {
        /* out-of-bounds returns zeros */
        let m = new_subsurface_map(2, 2);
        assert_eq!(sss_get_radius(&m, 99, 99), [0.0f32; 3]);
    }
}
