// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Physics collision layer system using bitmask-based layer filtering.

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicsLayerMask(u32);

#[allow(dead_code)]
impl PhysicsLayerMask {
    pub const NONE: Self = Self(0);
    pub const ALL: Self = Self(u32::MAX);
    pub const DEFAULT: Self = Self(1);

    pub fn new(bits: u32) -> Self { Self(bits) }

    pub fn from_layer(layer: u32) -> Self {
        if layer < 32 { Self(1u32 << layer) } else { Self::NONE }
    }

    pub fn bits(self) -> u32 { self.0 }

    pub fn set_layer(&mut self, layer: u32) {
        if layer < 32 { self.0 |= 1u32 << layer; }
    }

    pub fn clear_layer(&mut self, layer: u32) {
        if layer < 32 { self.0 &= !(1u32 << layer); }
    }

    pub fn has_layer(self, layer: u32) -> bool {
        layer < 32 && (self.0 & (1u32 << layer)) != 0
    }

    pub fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    pub fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    pub fn count_layers(self) -> u32 {
        self.0.count_ones()
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PhysicsLayerConfig {
    pub membership: PhysicsLayerMask,
    pub collision_mask: PhysicsLayerMask,
}

#[allow(dead_code)]
impl PhysicsLayerConfig {
    pub fn new(membership: PhysicsLayerMask, collision_mask: PhysicsLayerMask) -> Self {
        Self { membership, collision_mask }
    }

    pub fn default_config() -> Self {
        Self {
            membership: PhysicsLayerMask::DEFAULT,
            collision_mask: PhysicsLayerMask::ALL,
        }
    }

    pub fn should_collide(&self, other: &PhysicsLayerConfig) -> bool {
        self.collision_mask.intersects(other.membership)
            && other.collision_mask.intersects(self.membership)
    }
}

/// Named layer registry.
#[allow(dead_code)]
pub struct LayerNames {
    names: [Option<String>; 32],
}

#[allow(dead_code)]
impl LayerNames {
    pub fn new() -> Self {
        Self { names: Default::default() }
    }

    pub fn set_name(&mut self, layer: u32, name: &str) {
        if (layer as usize) < 32 {
            self.names[layer as usize] = Some(name.to_string());
        }
    }

    pub fn get_name(&self, layer: u32) -> Option<&str> {
        if (layer as usize) < 32 {
            self.names[layer as usize].as_deref()
        } else {
            None
        }
    }

    pub fn find_layer(&self, name: &str) -> Option<u32> {
        self.names.iter().enumerate()
            .find(|(_, n)| n.as_deref() == Some(name))
            .map(|(i, _)| i as u32)
    }
}

impl Default for LayerNames {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_layer() {
        let m = PhysicsLayerMask::from_layer(3);
        assert!(m.has_layer(3));
        assert!(!m.has_layer(2));
    }

    #[test]
    fn test_set_clear() {
        let mut m = PhysicsLayerMask::NONE;
        m.set_layer(5);
        assert!(m.has_layer(5));
        m.clear_layer(5);
        assert!(!m.has_layer(5));
    }

    #[test]
    fn test_intersects() {
        let a = PhysicsLayerMask::from_layer(1);
        let b = PhysicsLayerMask::from_layer(1);
        let c = PhysicsLayerMask::from_layer(2);
        assert!(a.intersects(b));
        assert!(!a.intersects(c));
    }

    #[test]
    fn test_union() {
        let a = PhysicsLayerMask::from_layer(0);
        let b = PhysicsLayerMask::from_layer(1);
        let u = a.union(b);
        assert_eq!(u.count_layers(), 2);
    }

    #[test]
    fn test_should_collide() {
        let a = PhysicsLayerConfig::new(
            PhysicsLayerMask::from_layer(0),
            PhysicsLayerMask::from_layer(1),
        );
        let b = PhysicsLayerConfig::new(
            PhysicsLayerMask::from_layer(1),
            PhysicsLayerMask::from_layer(0),
        );
        assert!(a.should_collide(&b));
    }

    #[test]
    fn test_should_not_collide() {
        let a = PhysicsLayerConfig::new(
            PhysicsLayerMask::from_layer(0),
            PhysicsLayerMask::from_layer(0),
        );
        let b = PhysicsLayerConfig::new(
            PhysicsLayerMask::from_layer(1),
            PhysicsLayerMask::from_layer(1),
        );
        assert!(!a.should_collide(&b));
    }

    #[test]
    fn test_default_config() {
        let a = PhysicsLayerConfig::default_config();
        let b = PhysicsLayerConfig::default_config();
        assert!(a.should_collide(&b));
    }

    #[test]
    fn test_layer_names() {
        let mut names = LayerNames::new();
        names.set_name(0, "default");
        names.set_name(1, "player");
        assert_eq!(names.get_name(0), Some("default"));
        assert_eq!(names.find_layer("player"), Some(1));
    }

    #[test]
    fn test_count_layers() {
        let m = PhysicsLayerMask::from_layer(0).union(PhysicsLayerMask::from_layer(3));
        assert_eq!(m.count_layers(), 2);
    }

    #[test]
    fn test_empty() {
        assert!(PhysicsLayerMask::NONE.is_empty());
        assert!(!PhysicsLayerMask::DEFAULT.is_empty());
    }
}
