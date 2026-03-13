// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Collision layer/group filtering using bitmasks.

/// Layer and mask for collision filtering.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionLayerFilter {
    pub layer: u32,
    pub mask: u32,
}

#[allow(dead_code)]
pub fn new_collision_layer_filter(layer: u32, mask: u32) -> CollisionLayerFilter {
    CollisionLayerFilter { layer, mask }
}

#[allow(dead_code)]
pub fn collision_layers_can_collide(a: &CollisionLayerFilter, b: &CollisionLayerFilter) -> bool {
    (a.mask & b.layer) != 0 && (b.mask & a.layer) != 0
}

#[allow(dead_code)]
pub fn collision_layer_add(filter: &mut CollisionLayerFilter, layer_bit: u32) {
    filter.layer |= 1 << layer_bit;
}

#[allow(dead_code)]
pub fn collision_layer_remove(filter: &mut CollisionLayerFilter, layer_bit: u32) {
    filter.layer &= !(1 << layer_bit);
}

#[allow(dead_code)]
pub fn collision_mask_add(filter: &mut CollisionLayerFilter, layer_bit: u32) {
    filter.mask |= 1 << layer_bit;
}

#[allow(dead_code)]
pub fn collision_mask_remove(filter: &mut CollisionLayerFilter, layer_bit: u32) {
    filter.mask &= !(1 << layer_bit);
}

#[allow(dead_code)]
pub fn default_collision_layer_filter() -> CollisionLayerFilter {
    CollisionLayerFilter { layer: 1, mask: 0xFFFFFFFF }
}

#[allow(dead_code)]
pub fn collision_layer_filter_to_json(f: &CollisionLayerFilter) -> String {
    format!("{{\"layer\":{},\"mask\":{}}}", f.layer, f.mask)
}

#[allow(dead_code)]
pub fn collision_layer_matches_all(f: &CollisionLayerFilter) -> bool {
    f.mask == 0xFFFFFFFF
}

#[allow(dead_code)]
pub fn collision_layer_matches_none(f: &CollisionLayerFilter) -> bool {
    f.mask == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_filter() {
        let f = default_collision_layer_filter();
        assert_eq!(f.layer, 1);
        assert_eq!(f.mask, 0xFFFFFFFF);
    }

    #[test]
    fn test_can_collide_default() {
        let a = default_collision_layer_filter();
        let b = default_collision_layer_filter();
        assert!(collision_layers_can_collide(&a, &b));
    }

    #[test]
    fn test_cannot_collide_no_mask() {
        let a = new_collision_layer_filter(1, 0);
        let b = new_collision_layer_filter(1, 0xFFFFFFFF);
        assert!(!collision_layers_can_collide(&a, &b));
    }

    #[test]
    fn test_can_collide_specific_layers() {
        let a = new_collision_layer_filter(0b01, 0b10);
        let b = new_collision_layer_filter(0b10, 0b01);
        assert!(collision_layers_can_collide(&a, &b));
    }

    #[test]
    fn test_add_layer_bit() {
        let mut f = new_collision_layer_filter(0, 0xFFFFFFFF);
        collision_layer_add(&mut f, 3);
        assert_eq!(f.layer, 1 << 3);
    }

    #[test]
    fn test_remove_layer_bit() {
        let mut f = new_collision_layer_filter(0xFF, 0xFFFFFFFF);
        collision_layer_remove(&mut f, 0);
        assert_eq!(f.layer & 1, 0);
    }

    #[test]
    fn test_add_mask_bit() {
        let mut f = new_collision_layer_filter(1, 0);
        collision_mask_add(&mut f, 2);
        assert_eq!(f.mask, 1 << 2);
    }

    #[test]
    fn test_remove_mask_bit() {
        let mut f = new_collision_layer_filter(1, 0xFFFFFFFF);
        collision_mask_remove(&mut f, 0);
        assert_eq!(f.mask & 1, 0);
    }

    #[test]
    fn test_to_json() {
        let f = new_collision_layer_filter(1, 2);
        let j = collision_layer_filter_to_json(&f);
        assert!(j.contains("\"layer\":1"));
        assert!(j.contains("\"mask\":2"));
    }

    #[test]
    fn test_matches_all() {
        let f = default_collision_layer_filter();
        assert!(collision_layer_matches_all(&f));
    }

    #[test]
    fn test_matches_none() {
        let f = new_collision_layer_filter(1, 0);
        assert!(collision_layer_matches_none(&f));
    }
}
