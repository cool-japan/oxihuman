#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Voxel-based collider for terrain/world.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VoxelCollider {
    pub voxels: HashMap<(i32, i32, i32), bool>,
    pub voxel_size: f32,
}

#[allow(dead_code)]
pub fn new_voxel_collider(voxel_size: f32) -> VoxelCollider {
    VoxelCollider {
        voxels: HashMap::new(),
        voxel_size: voxel_size.max(1e-6),
    }
}

#[allow(dead_code)]
pub fn vc_set(vc: &mut VoxelCollider, x: i32, y: i32, z: i32, solid: bool) {
    if solid {
        vc.voxels.insert((x, y, z), true);
    } else {
        vc.voxels.remove(&(x, y, z));
    }
}

#[allow(dead_code)]
pub fn vc_get(vc: &VoxelCollider, x: i32, y: i32, z: i32) -> bool {
    *vc.voxels.get(&(x, y, z)).unwrap_or(&false)
}

#[allow(dead_code)]
pub fn vc_aabb_test(vc: &VoxelCollider, aabb_min: [f32; 3], aabb_max: [f32; 3]) -> bool {
    let vs = vc.voxel_size;
    let ix_min = (aabb_min[0] / vs).floor() as i32;
    let iy_min = (aabb_min[1] / vs).floor() as i32;
    let iz_min = (aabb_min[2] / vs).floor() as i32;
    let ix_max = (aabb_max[0] / vs).ceil() as i32;
    let iy_max = (aabb_max[1] / vs).ceil() as i32;
    let iz_max = (aabb_max[2] / vs).ceil() as i32;

    for x in ix_min..=ix_max {
        for y in iy_min..=iy_max {
            for z in iz_min..=iz_max {
                if vc_get(vc, x, y, z) {
                    return true;
                }
            }
        }
    }
    false
}

#[allow(dead_code)]
pub fn vc_count(vc: &VoxelCollider) -> usize {
    vc.voxels.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() {
        let vc = new_voxel_collider(1.0);
        assert_eq!(vc_count(&vc), 0);
    }

    #[test]
    fn set_solid() {
        let mut vc = new_voxel_collider(1.0);
        vc_set(&mut vc, 0, 0, 0, true);
        assert!(vc_get(&vc, 0, 0, 0));
    }

    #[test]
    fn set_not_solid_removes() {
        let mut vc = new_voxel_collider(1.0);
        vc_set(&mut vc, 1, 2, 3, true);
        vc_set(&mut vc, 1, 2, 3, false);
        assert!(!vc_get(&vc, 1, 2, 3));
    }

    #[test]
    fn get_missing_returns_false() {
        let vc = new_voxel_collider(1.0);
        assert!(!vc_get(&vc, 99, 99, 99));
    }

    #[test]
    fn aabb_test_hits_solid() {
        let mut vc = new_voxel_collider(1.0);
        vc_set(&mut vc, 0, 0, 0, true);
        assert!(vc_aabb_test(&vc, [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]));
    }

    #[test]
    fn aabb_test_misses_empty() {
        let vc = new_voxel_collider(1.0);
        assert!(!vc_aabb_test(&vc, [0.0, 0.0, 0.0], [0.9, 0.9, 0.9]));
    }

    #[test]
    fn count_tracks_inserts() {
        let mut vc = new_voxel_collider(1.0);
        vc_set(&mut vc, 0, 0, 0, true);
        vc_set(&mut vc, 1, 0, 0, true);
        vc_set(&mut vc, 0, 1, 0, true);
        assert_eq!(vc_count(&vc), 3);
    }

    #[test]
    fn duplicate_set_no_count_increase() {
        let mut vc = new_voxel_collider(1.0);
        vc_set(&mut vc, 5, 5, 5, true);
        vc_set(&mut vc, 5, 5, 5, true);
        assert_eq!(vc_count(&vc), 1);
    }

    #[test]
    fn negative_coords() {
        let mut vc = new_voxel_collider(1.0);
        vc_set(&mut vc, -3, -2, -1, true);
        assert!(vc_get(&vc, -3, -2, -1));
    }

    #[test]
    fn voxel_size_stored() {
        let vc = new_voxel_collider(0.5);
        assert!((vc.voxel_size - 0.5).abs() < 1e-5);
    }
}
