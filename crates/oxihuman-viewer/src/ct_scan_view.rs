// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct CtVolume {
    pub width: u16,
    pub height: u16,
    pub depth: u16,
    pub voxels: Vec<i16>,
    pub voxel_spacing_mm: [f32; 3],
}

pub fn new_ct_volume(w: u16, h: u16, d: u16) -> CtVolume {
    let n = (w as usize) * (h as usize) * (d as usize);
    CtVolume {
        width: w,
        height: h,
        depth: d,
        voxels: vec![0; n],
        voxel_spacing_mm: [1.0, 1.0, 1.0],
    }
}

fn ct_index(v: &CtVolume, x: u16, y: u16, z: u16) -> Option<usize> {
    if x < v.width && y < v.height && z < v.depth {
        Some(
            z as usize * v.height as usize * v.width as usize
                + y as usize * v.width as usize
                + x as usize,
        )
    } else {
        None
    }
}

pub fn ct_set_voxel(v: &mut CtVolume, x: u16, y: u16, z: u16, hu: i16) {
    if let Some(idx) = ct_index(v, x, y, z) {
        v.voxels[idx] = hu;
    }
}

pub fn ct_get_voxel(v: &CtVolume, x: u16, y: u16, z: u16) -> i16 {
    ct_index(v, x, y, z).map(|i| v.voxels[i]).unwrap_or(0)
}

pub fn ct_voxel_count(v: &CtVolume) -> usize {
    v.width as usize * v.height as usize * v.depth as usize
}

pub fn ct_volume_mm3(v: &CtVolume) -> f32 {
    ct_voxel_count(v) as f32 * v.voxel_spacing_mm[0] * v.voxel_spacing_mm[1] * v.voxel_spacing_mm[2]
}

pub fn ct_density_above(v: &CtVolume, threshold_hu: i16) -> usize {
    v.voxels.iter().filter(|&&hu| hu > threshold_hu).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_volume() {
        /* voxel count = w*h*d */
        let v = new_ct_volume(4, 4, 4);
        assert_eq!(ct_voxel_count(&v), 64);
    }

    #[test]
    fn test_set_get_voxel() {
        /* set and get voxel */
        let mut v = new_ct_volume(5, 5, 5);
        ct_set_voxel(&mut v, 1, 2, 3, 400);
        assert_eq!(ct_get_voxel(&v, 1, 2, 3), 400);
    }

    #[test]
    fn test_get_voxel_oob() {
        /* out of bounds returns 0 */
        let v = new_ct_volume(3, 3, 3);
        assert_eq!(ct_get_voxel(&v, 100, 100, 100), 0);
    }

    #[test]
    fn test_volume_mm3() {
        /* volume = voxel_count * spacing product */
        let v = new_ct_volume(2, 2, 2);
        assert!((ct_volume_mm3(&v) - 8.0).abs() < 1e-5);
    }

    #[test]
    fn test_density_above_all_zero() {
        /* all zero, threshold 0 => 0 above */
        let v = new_ct_volume(3, 3, 3);
        assert_eq!(ct_density_above(&v, 0), 0);
    }

    #[test]
    fn test_density_above_some() {
        /* set one voxel high */
        let mut v = new_ct_volume(2, 2, 1);
        ct_set_voxel(&mut v, 0, 0, 0, 1000);
        assert_eq!(ct_density_above(&v, 500), 1);
    }

    #[test]
    fn test_voxel_count_correct() {
        /* voxel count matches */
        let v = new_ct_volume(10, 5, 3);
        assert_eq!(ct_voxel_count(&v), 150);
    }
}
