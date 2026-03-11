// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Bitfield-based permission set for role-based access control.

/// Predefined permission bits.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    Read = 0,
    Write = 1,
    Execute = 2,
    Delete = 3,
    Admin = 4,
    Export = 5,
    Import = 6,
    Share = 7,
}

/// A set of permissions stored as a 64-bit bitfield.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermissionSet {
    bits: u64,
}

#[allow(dead_code)]
impl PermissionSet {
    pub fn none() -> Self {
        Self { bits: 0 }
    }

    pub fn all() -> Self {
        Self { bits: u64::MAX }
    }

    pub fn from_bits(bits: u64) -> Self {
        Self { bits }
    }

    pub fn bits(&self) -> u64 {
        self.bits
    }

    pub fn grant(&mut self, perm: Permission) {
        self.bits |= 1u64 << (perm as u32);
    }

    pub fn revoke(&mut self, perm: Permission) {
        self.bits &= !(1u64 << (perm as u32));
    }

    pub fn has(&self, perm: Permission) -> bool {
        (self.bits & (1u64 << (perm as u32))) != 0
    }

    pub fn grant_bit(&mut self, bit: u32) {
        self.bits |= 1u64 << bit;
    }

    pub fn revoke_bit(&mut self, bit: u32) {
        self.bits &= !(1u64 << bit);
    }

    pub fn has_bit(&self, bit: u32) -> bool {
        (self.bits & (1u64 << bit)) != 0
    }

    pub fn union(&self, other: &PermissionSet) -> PermissionSet {
        PermissionSet { bits: self.bits | other.bits }
    }

    pub fn intersection(&self, other: &PermissionSet) -> PermissionSet {
        PermissionSet { bits: self.bits & other.bits }
    }

    pub fn difference(&self, other: &PermissionSet) -> PermissionSet {
        PermissionSet { bits: self.bits & !other.bits }
    }

    pub fn is_subset_of(&self, other: &PermissionSet) -> bool {
        (self.bits & other.bits) == self.bits
    }

    pub fn count(&self) -> u32 {
        self.bits.count_ones()
    }

    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }

    pub fn clear(&mut self) {
        self.bits = 0;
    }
}

/// A named role with a permission set.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Role {
    pub name: String,
    pub permissions: PermissionSet,
}

#[allow(dead_code)]
impl Role {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), permissions: PermissionSet::none() }
    }

    pub fn with_permissions(name: &str, perms: PermissionSet) -> Self {
        Self { name: name.to_string(), permissions: perms }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grant_has() {
        let mut ps = PermissionSet::none();
        ps.grant(Permission::Read);
        assert!(ps.has(Permission::Read));
        assert!(!ps.has(Permission::Write));
    }

    #[test]
    fn test_revoke() {
        let mut ps = PermissionSet::none();
        ps.grant(Permission::Admin);
        ps.revoke(Permission::Admin);
        assert!(!ps.has(Permission::Admin));
    }

    #[test]
    fn test_union() {
        let mut a = PermissionSet::none();
        a.grant(Permission::Read);
        let mut b = PermissionSet::none();
        b.grant(Permission::Write);
        let c = a.union(&b);
        assert!(c.has(Permission::Read));
        assert!(c.has(Permission::Write));
    }

    #[test]
    fn test_intersection() {
        let mut a = PermissionSet::none();
        a.grant(Permission::Read);
        a.grant(Permission::Write);
        let mut b = PermissionSet::none();
        b.grant(Permission::Write);
        let c = a.intersection(&b);
        assert!(!c.has(Permission::Read));
        assert!(c.has(Permission::Write));
    }

    #[test]
    fn test_difference() {
        let mut a = PermissionSet::none();
        a.grant(Permission::Read);
        a.grant(Permission::Write);
        let mut b = PermissionSet::none();
        b.grant(Permission::Read);
        let c = a.difference(&b);
        assert!(!c.has(Permission::Read));
        assert!(c.has(Permission::Write));
    }

    #[test]
    fn test_subset() {
        let mut a = PermissionSet::none();
        a.grant(Permission::Read);
        let mut b = PermissionSet::none();
        b.grant(Permission::Read);
        b.grant(Permission::Write);
        assert!(a.is_subset_of(&b));
        assert!(!b.is_subset_of(&a));
    }

    #[test]
    fn test_count() {
        let mut ps = PermissionSet::none();
        ps.grant(Permission::Read);
        ps.grant(Permission::Execute);
        ps.grant(Permission::Share);
        assert_eq!(ps.count(), 3);
    }

    #[test]
    fn test_clear() {
        let mut ps = PermissionSet::all();
        ps.clear();
        assert!(ps.is_empty());
    }

    #[test]
    fn test_role() {
        let role = Role::new("viewer");
        assert_eq!(role.name, "viewer");
        assert!(role.permissions.is_empty());
    }

    #[test]
    fn test_custom_bit() {
        let mut ps = PermissionSet::none();
        ps.grant_bit(32);
        assert!(ps.has_bit(32));
        assert!(!ps.has_bit(33));
    }
}
