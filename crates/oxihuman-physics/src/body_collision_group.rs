// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Body collision group: bitmask-based layer filtering for physics bodies.

/// Collision layer bitmask (up to 32 layers).
pub type CollisionMask = u32;

/// Collision group assigned to a body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub struct CollisionGroup {
    /// Which layers this body belongs to.
    pub membership: CollisionMask,
    /// Which layers this body collides with.
    pub filter: CollisionMask,
}

/// Create a collision group.
#[allow(dead_code)]
pub fn new_collision_group(membership: CollisionMask, filter: CollisionMask) -> CollisionGroup {
    CollisionGroup { membership, filter }
}

/// Default group: layer 0, collides with everything.
#[allow(dead_code)]
pub fn default_collision_group() -> CollisionGroup {
    CollisionGroup {
        membership: 1,
        filter: !0,
    }
}

/// Whether two bodies should collide.
#[allow(dead_code)]
pub fn groups_collide(a: CollisionGroup, b: CollisionGroup) -> bool {
    (a.filter & b.membership) != 0 && (b.filter & a.membership) != 0
}

/// Add a layer to membership.
#[allow(dead_code)]
pub fn cg_add_layer(g: &mut CollisionGroup, layer: u8) {
    g.membership |= 1u32 << (layer & 31);
}

/// Remove a layer from membership.
#[allow(dead_code)]
pub fn cg_remove_layer(g: &mut CollisionGroup, layer: u8) {
    g.membership &= !(1u32 << (layer & 31));
}

/// Enable collision with a layer.
#[allow(dead_code)]
pub fn cg_enable_filter(g: &mut CollisionGroup, layer: u8) {
    g.filter |= 1u32 << (layer & 31);
}

/// Disable collision with a layer.
#[allow(dead_code)]
pub fn cg_disable_filter(g: &mut CollisionGroup, layer: u8) {
    g.filter &= !(1u32 << (layer & 31));
}

/// Number of membership layers active.
#[allow(dead_code)]
pub fn cg_layer_count(g: CollisionGroup) -> u32 {
    g.membership.count_ones()
}

/// Whether body is a member of a specific layer.
#[allow(dead_code)]
pub fn cg_in_layer(g: CollisionGroup, layer: u8) -> bool {
    (g.membership & (1u32 << (layer & 31))) != 0
}

/// Collision group that collides with nothing.
#[allow(dead_code)]
pub fn ghost_collision_group() -> CollisionGroup {
    CollisionGroup {
        membership: 0,
        filter: 0,
    }
}

/// Merge two groups (union of memberships, intersection of filters).
#[allow(dead_code)]
pub fn cg_merge(a: CollisionGroup, b: CollisionGroup) -> CollisionGroup {
    CollisionGroup {
        membership: a.membership | b.membership,
        filter: a.filter & b.filter,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_groups_collide_default() {
        let a = default_collision_group();
        let b = default_collision_group();
        assert!(groups_collide(a, b));
    }

    #[test]
    fn test_ghost_no_collide() {
        let a = ghost_collision_group();
        let b = default_collision_group();
        assert!(!groups_collide(a, b));
    }

    #[test]
    fn test_add_remove_layer() {
        let mut g = ghost_collision_group();
        cg_add_layer(&mut g, 3);
        assert!(cg_in_layer(g, 3));
        cg_remove_layer(&mut g, 3);
        assert!(!cg_in_layer(g, 3));
    }

    #[test]
    fn test_enable_disable_filter() {
        let mut g = new_collision_group(1, 0);
        cg_enable_filter(&mut g, 2);
        assert_eq!(g.filter & 4, 4);
        cg_disable_filter(&mut g, 2);
        assert_eq!(g.filter & 4, 0);
    }

    #[test]
    fn test_layer_count() {
        let g = new_collision_group(0b1011, !0);
        assert_eq!(cg_layer_count(g), 3);
    }

    #[test]
    fn test_layer_isolation() {
        let a = new_collision_group(0b0001, 0b0010);
        let b = new_collision_group(0b0010, 0b0001);
        assert!(groups_collide(a, b));
        let c = new_collision_group(0b0100, 0b0100);
        assert!(!groups_collide(a, c));
    }

    #[test]
    fn test_merge() {
        let a = new_collision_group(0b0001, 0b1111);
        let b = new_collision_group(0b0010, 0b0011);
        let m = cg_merge(a, b);
        assert_eq!(m.membership, 0b0011);
        assert_eq!(m.filter, 0b0011);
    }

    #[test]
    fn test_asymmetric_no_collide() {
        // a filters b but b does not filter a
        let a = new_collision_group(0b0001, 0b0010);
        let b = new_collision_group(0b0010, 0b0100);
        assert!(!groups_collide(a, b));
    }

    #[test]
    fn test_in_layer_boundary() {
        let g = new_collision_group(1u32 << 31, !0);
        assert!(cg_in_layer(g, 31));
        assert!(!cg_in_layer(g, 0));
    }
}
