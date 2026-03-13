// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A group of constraints solved together.
#[allow(dead_code)]
pub struct ConstraintGroupItem {
    pub id: u32,
    pub constraint_ids: Vec<u32>,
    pub priority: u32,
}

/// A set of constraint groups.
#[allow(dead_code)]
pub struct ConstraintGroupSet {
    pub groups: Vec<ConstraintGroupItem>,
}

/// Create a new empty `ConstraintGroupSet`.
#[allow(dead_code)]
pub fn new_constraint_group_set() -> ConstraintGroupSet {
    ConstraintGroupSet { groups: Vec::new() }
}

/// Add a new group with the given id and priority.
#[allow(dead_code)]
pub fn add_group(cgs: &mut ConstraintGroupSet, id: u32, priority: u32) {
    if cgs.groups.iter().any(|g| g.id == id) {
        return;
    }
    cgs.groups.push(ConstraintGroupItem {
        id,
        constraint_ids: Vec::new(),
        priority,
    });
}

/// Add a constraint to an existing group.
#[allow(dead_code)]
pub fn add_constraint_to_group(cgs: &mut ConstraintGroupSet, group_id: u32, cid: u32) {
    if let Some(g) = cgs.groups.iter_mut().find(|g| g.id == group_id) {
        if !g.constraint_ids.contains(&cid) {
            g.constraint_ids.push(cid);
        }
    }
}

/// Return the number of groups.
#[allow(dead_code)]
pub fn group_count(cgs: &ConstraintGroupSet) -> usize {
    cgs.groups.len()
}

/// Return constraint IDs in a group.
#[allow(dead_code)]
pub fn constraints_in_group(cgs: &ConstraintGroupSet, group_id: u32) -> Vec<u32> {
    cgs.groups
        .iter()
        .find(|g| g.id == group_id)
        .map(|g| g.constraint_ids.clone())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_set_is_empty() {
        let cgs = new_constraint_group_set();
        assert_eq!(group_count(&cgs), 0);
    }

    #[test]
    fn add_group_increments_count() {
        let mut cgs = new_constraint_group_set();
        add_group(&mut cgs, 1, 10);
        assert_eq!(group_count(&cgs), 1);
    }

    #[test]
    fn add_duplicate_group_no_duplicate() {
        let mut cgs = new_constraint_group_set();
        add_group(&mut cgs, 1, 10);
        add_group(&mut cgs, 1, 20);
        assert_eq!(group_count(&cgs), 1);
    }

    #[test]
    fn add_constraint_to_group_basic() {
        let mut cgs = new_constraint_group_set();
        add_group(&mut cgs, 1, 5);
        add_constraint_to_group(&mut cgs, 1, 100);
        let ids = constraints_in_group(&cgs, 1);
        assert!(ids.contains(&100));
    }

    #[test]
    fn add_duplicate_constraint_no_duplicate() {
        let mut cgs = new_constraint_group_set();
        add_group(&mut cgs, 1, 5);
        add_constraint_to_group(&mut cgs, 1, 100);
        add_constraint_to_group(&mut cgs, 1, 100);
        assert_eq!(constraints_in_group(&cgs, 1).len(), 1);
    }

    #[test]
    fn constraints_in_unknown_group_empty() {
        let cgs = new_constraint_group_set();
        assert!(constraints_in_group(&cgs, 999).is_empty());
    }

    #[test]
    fn multiple_groups_independent() {
        let mut cgs = new_constraint_group_set();
        add_group(&mut cgs, 1, 1);
        add_group(&mut cgs, 2, 2);
        add_constraint_to_group(&mut cgs, 1, 10);
        add_constraint_to_group(&mut cgs, 2, 20);
        assert!(!constraints_in_group(&cgs, 1).contains(&20));
        assert!(!constraints_in_group(&cgs, 2).contains(&10));
    }

    #[test]
    fn priority_stored() {
        let mut cgs = new_constraint_group_set();
        add_group(&mut cgs, 42, 99);
        assert_eq!(cgs.groups[0].priority, 99);
    }

    #[test]
    fn multiple_constraints_in_group() {
        let mut cgs = new_constraint_group_set();
        add_group(&mut cgs, 1, 1);
        add_constraint_to_group(&mut cgs, 1, 10);
        add_constraint_to_group(&mut cgs, 1, 20);
        add_constraint_to_group(&mut cgs, 1, 30);
        assert_eq!(constraints_in_group(&cgs, 1).len(), 3);
    }

    #[test]
    fn add_constraint_to_nonexistent_group_no_panic() {
        let mut cgs = new_constraint_group_set();
        add_constraint_to_group(&mut cgs, 999, 1);
        // Should not crash, and group count remains 0
        assert_eq!(group_count(&cgs), 0);
    }
}
