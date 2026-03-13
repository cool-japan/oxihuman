// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Collision filtering masks — bitmask-based layer system that controls which
//! bodies collide with which.
//!
//! A body belongs to one or more *layers* (encoded as set bits in a `u32`) and
//! carries a *mask* that lists the layers it can collide with.  Two bodies
//! collide only when each body's layer is present in the other's mask.

// ── types ────────────────────────────────────────────────────────────────────

/// A named collision layer bitmask constant.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CollisionLayer {
    /// Bitmask for this layer (single bit recommended, e.g. `1 << n`).
    pub bits: u32,
    /// Human-readable name used for debugging.
    pub name: &'static str,
}

/// Global configuration for the collision filter system.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct CollisionFilterConfig {
    /// Default layer assigned to new filters.
    pub default_layer: u32,
    /// Default mask assigned to new filters.
    pub default_mask: u32,
}

/// A per-body collision filter consisting of a layer and a mask.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollisionFilter {
    /// The layer(s) this body belongs to.
    pub layer: u32,
    /// The layer(s) this body can collide with.
    pub mask: u32,
}

// ── public API ────────────────────────────────────────────────────────────────

/// Return a [`CollisionFilterConfig`] with sensible defaults.
///
/// By default all layers are active (all bits set).
#[allow(dead_code)]
pub fn default_collision_filter_config() -> CollisionFilterConfig {
    CollisionFilterConfig {
        default_layer: 0x0000_0001,
        default_mask: 0xFFFF_FFFF,
    }
}

/// Create a new [`CollisionFilter`] with the specified layer and mask.
#[allow(dead_code)]
pub fn new_collision_filter(layer: u32, mask: u32) -> CollisionFilter {
    CollisionFilter { layer, mask }
}

/// Return `true` if two filters allow collision between their respective bodies.
///
/// Collision occurs when `a.layer & b.mask != 0` **and** `b.layer & a.mask != 0`.
#[allow(dead_code)]
pub fn filters_collide(a: &CollisionFilter, b: &CollisionFilter) -> bool {
    (a.layer & b.mask) != 0 && (b.layer & a.mask) != 0
}

/// Return the layer bitmask of a filter.
#[allow(dead_code)]
pub fn collision_filter_layer(filter: &CollisionFilter) -> u32 {
    filter.layer
}

/// Return the mask bitmask of a filter.
#[allow(dead_code)]
pub fn collision_filter_mask(filter: &CollisionFilter) -> u32 {
    filter.mask
}

/// Set the layer on a filter.
#[allow(dead_code)]
pub fn set_collision_layer(filter: &mut CollisionFilter, layer: u32) {
    filter.layer = layer;
}

/// Set the mask on a filter.
#[allow(dead_code)]
pub fn set_collision_mask(filter: &mut CollisionFilter, mask: u32) {
    filter.mask = mask;
}

/// Return a human-readable name for a layer bitmask.
///
/// Recognises the lowest set bit and maps it to a conventional name.  Unknown
/// combinations are returned as `"custom"`.
#[allow(dead_code)]
pub fn collision_layer_name(layer: u32) -> String {
    match layer {
        0 => "none".to_string(),
        0x0000_0001 => "default".to_string(),
        0x0000_0002 => "player".to_string(),
        0x0000_0004 => "npc".to_string(),
        0x0000_0008 => "projectile".to_string(),
        0x0000_0010 => "trigger".to_string(),
        0x0000_0020 => "terrain".to_string(),
        0x0000_0040 => "debris".to_string(),
        0x0000_0080 => "ragdoll".to_string(),
        0xFFFF_FFFF => "all".to_string(),
        _ => "custom".to_string(),
    }
}

/// Return a [`CollisionFilter`] that collides with every layer.
#[allow(dead_code)]
pub fn collision_filter_matches_all() -> CollisionFilter {
    CollisionFilter {
        layer: 0xFFFF_FFFF,
        mask: 0xFFFF_FFFF,
    }
}

/// Return a [`CollisionFilter`] that collides with no layer.
#[allow(dead_code)]
pub fn collision_filter_matches_none() -> CollisionFilter {
    CollisionFilter { layer: 0, mask: 0 }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_fields() {
        let cfg = default_collision_filter_config();
        assert_ne!(cfg.default_layer, 0);
        assert_ne!(cfg.default_mask, 0);
    }

    #[test]
    fn test_new_collision_filter() {
        let f = new_collision_filter(0b0001, 0b1111);
        assert_eq!(collision_filter_layer(&f), 0b0001);
        assert_eq!(collision_filter_mask(&f), 0b1111);
    }

    #[test]
    fn test_filters_collide_both_match() {
        let a = new_collision_filter(0b0001, 0b0010);
        let b = new_collision_filter(0b0010, 0b0001);
        assert!(filters_collide(&a, &b));
    }

    #[test]
    fn test_filters_collide_one_sided_no_collision() {
        // a's mask includes b's layer, but b's mask does NOT include a's layer.
        let a = new_collision_filter(0b0001, 0b0010);
        let b = new_collision_filter(0b0010, 0b0100); // mask excludes a's layer
        assert!(!filters_collide(&a, &b));
    }

    #[test]
    fn test_filters_collide_none_never_collide() {
        let a = collision_filter_matches_all();
        let b = collision_filter_matches_none();
        assert!(!filters_collide(&a, &b));
    }

    #[test]
    fn test_filters_collide_all_always_collide() {
        let a = collision_filter_matches_all();
        let b = collision_filter_matches_all();
        assert!(filters_collide(&a, &b));
    }

    #[test]
    fn test_set_collision_layer() {
        let mut f = new_collision_filter(0b0001, 0b1111);
        set_collision_layer(&mut f, 0b1000);
        assert_eq!(collision_filter_layer(&f), 0b1000);
    }

    #[test]
    fn test_set_collision_mask() {
        let mut f = new_collision_filter(0b0001, 0b1111);
        set_collision_mask(&mut f, 0b0010);
        assert_eq!(collision_filter_mask(&f), 0b0010);
    }

    #[test]
    fn test_collision_layer_name_known() {
        assert_eq!(collision_layer_name(0x0000_0001), "default");
        assert_eq!(collision_layer_name(0x0000_0002), "player");
        assert_eq!(collision_layer_name(0), "none");
        assert_eq!(collision_layer_name(0xFFFF_FFFF), "all");
    }

    #[test]
    fn test_collision_layer_name_custom() {
        assert_eq!(collision_layer_name(0x0000_0100), "custom");
    }

    #[test]
    fn test_matches_all_layer_and_mask() {
        let f = collision_filter_matches_all();
        assert_eq!(f.layer, 0xFFFF_FFFF);
        assert_eq!(f.mask, 0xFFFF_FFFF);
    }

    #[test]
    fn test_matches_none_layer_and_mask() {
        let f = collision_filter_matches_none();
        assert_eq!(f.layer, 0);
        assert_eq!(f.mask, 0);
    }

    #[test]
    fn test_same_layer_collide_when_mask_includes_own_layer() {
        // Two bodies on the same layer, mask includes that layer.
        let a = new_collision_filter(0b0100, 0b0100);
        let b = new_collision_filter(0b0100, 0b0100);
        assert!(filters_collide(&a, &b));
    }

    #[test]
    fn test_asymmetric_mask_no_collision() {
        // a can see b but b cannot see a.
        let a = new_collision_filter(1, 2);
        let b = new_collision_filter(2, 4); // b's mask is 4, not 1
        assert!(!filters_collide(&a, &b));
    }
}
