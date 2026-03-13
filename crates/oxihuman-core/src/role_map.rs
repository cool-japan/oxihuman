// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Role-based permission map: entities carry named roles.

use std::collections::{HashMap, HashSet};

/// Assigns roles to entity IDs and checks permissions.
pub struct RoleMap {
    entity_roles: HashMap<u64, HashSet<String>>,
    role_perms: HashMap<String, HashSet<String>>,
}

#[allow(dead_code)]
impl RoleMap {
    pub fn new() -> Self {
        RoleMap {
            entity_roles: HashMap::new(),
            role_perms: HashMap::new(),
        }
    }

    pub fn assign_role(&mut self, entity: u64, role: &str) {
        self.entity_roles
            .entry(entity)
            .or_default()
            .insert(role.to_string());
    }

    pub fn remove_role(&mut self, entity: u64, role: &str) -> bool {
        if let Some(roles) = self.entity_roles.get_mut(&entity) {
            roles.remove(role)
        } else {
            false
        }
    }

    pub fn has_role(&self, entity: u64, role: &str) -> bool {
        self.entity_roles
            .get(&entity)
            .is_some_and(|r| r.contains(role))
    }

    pub fn roles_of(&self, entity: u64) -> Vec<String> {
        self.entity_roles
            .get(&entity)
            .map(|r| r.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn define_permission(&mut self, role: &str, perm: &str) {
        self.role_perms
            .entry(role.to_string())
            .or_default()
            .insert(perm.to_string());
    }

    pub fn has_permission(&self, entity: u64, perm: &str) -> bool {
        if let Some(roles) = self.entity_roles.get(&entity) {
            roles.iter().any(|r| {
                self.role_perms
                    .get(r)
                    .is_some_and(|perms| perms.contains(perm))
            })
        } else {
            false
        }
    }

    pub fn entity_count(&self) -> usize {
        self.entity_roles.len()
    }

    pub fn role_count(&self) -> usize {
        let mut all: HashSet<&str> = HashSet::new();
        for roles in self.entity_roles.values() {
            for r in roles {
                all.insert(r.as_str());
            }
        }
        all.len()
    }

    pub fn entities_with_role(&self, role: &str) -> Vec<u64> {
        self.entity_roles
            .iter()
            .filter(|(_, roles)| roles.contains(role))
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn clear_entity(&mut self, entity: u64) {
        self.entity_roles.remove(&entity);
    }

    pub fn clear(&mut self) {
        self.entity_roles.clear();
        self.role_perms.clear();
    }
}

impl Default for RoleMap {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_role_map() -> RoleMap {
    RoleMap::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assign_and_has_role() {
        let mut m = new_role_map();
        m.assign_role(1, "admin");
        assert!(m.has_role(1, "admin"));
        assert!(!m.has_role(1, "guest"));
    }

    #[test]
    fn remove_role() {
        let mut m = new_role_map();
        m.assign_role(1, "admin");
        assert!(m.remove_role(1, "admin"));
        assert!(!m.has_role(1, "admin"));
    }

    #[test]
    fn roles_of_entity() {
        let mut m = new_role_map();
        m.assign_role(1, "admin");
        m.assign_role(1, "editor");
        let roles = m.roles_of(1);
        assert_eq!(roles.len(), 2);
    }

    #[test]
    fn permissions() {
        let mut m = new_role_map();
        m.assign_role(1, "editor");
        m.define_permission("editor", "write");
        assert!(m.has_permission(1, "write"));
        assert!(!m.has_permission(1, "delete"));
    }

    #[test]
    fn entities_with_role() {
        let mut m = new_role_map();
        m.assign_role(1, "admin");
        m.assign_role(2, "admin");
        m.assign_role(3, "user");
        let admins = m.entities_with_role("admin");
        assert_eq!(admins.len(), 2);
    }

    #[test]
    fn entity_count() {
        let mut m = new_role_map();
        m.assign_role(1, "a");
        m.assign_role(2, "b");
        assert_eq!(m.entity_count(), 2);
    }

    #[test]
    fn clear_entity() {
        let mut m = new_role_map();
        m.assign_role(1, "admin");
        m.clear_entity(1);
        assert!(!m.has_role(1, "admin"));
    }

    #[test]
    fn clear_all() {
        let mut m = new_role_map();
        m.assign_role(1, "admin");
        m.clear();
        assert_eq!(m.entity_count(), 0);
    }

    #[test]
    fn permission_via_multiple_roles() {
        let mut m = new_role_map();
        m.assign_role(1, "viewer");
        m.define_permission("viewer", "read");
        assert!(m.has_permission(1, "read"));
    }
}
