#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Constraint list UI panel state.

/// A single constraint slot.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConstraintSlot {
    pub id: u32,
    pub name: String,
    pub constraint_type: String,
    pub influence: f32,
    pub enabled: bool,
}

/// The constraint list panel state.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ConstraintListView {
    pub constraints: Vec<ConstraintSlot>,
}

/// Create a new empty `ConstraintListView`.
#[allow(dead_code)]
pub fn new_constraint_list_view() -> ConstraintListView {
    ConstraintListView::default()
}

/// Add a constraint slot.
#[allow(dead_code)]
pub fn add_constraint_slot(view: &mut ConstraintListView, id: u32, name: &str, type_: &str) {
    view.constraints.push(ConstraintSlot {
        id,
        name: name.to_string(),
        constraint_type: type_.to_string(),
        influence: 1.0,
        enabled: true,
    });
}

/// Set the influence of the constraint with the given id.
#[allow(dead_code)]
pub fn set_influence(view: &mut ConstraintListView, id: u32, inf: f32) {
    if let Some(slot) = view.constraints.iter_mut().find(|c| c.id == id) {
        slot.influence = inf.clamp(0.0, 1.0);
    }
}

/// Toggle the `enabled` flag of the constraint with the given id.
#[allow(dead_code)]
pub fn toggle_constraint(view: &mut ConstraintListView, id: u32) {
    if let Some(slot) = view.constraints.iter_mut().find(|c| c.id == id) {
        slot.enabled = !slot.enabled;
    }
}

/// Return the number of constraints.
#[allow(dead_code)]
pub fn constraint_count(view: &ConstraintListView) -> usize {
    view.constraints.len()
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_view_is_empty() {
        let v = new_constraint_list_view();
        assert_eq!(constraint_count(&v), 0);
    }

    #[test]
    fn add_constraint_increments_count() {
        let mut v = new_constraint_list_view();
        add_constraint_slot(&mut v, 0, "IK", "INVERSE_KINEMATICS");
        assert_eq!(constraint_count(&v), 1);
    }

    #[test]
    fn added_constraint_defaults_enabled() {
        let mut v = new_constraint_list_view();
        add_constraint_slot(&mut v, 0, "CopyLoc", "COPY_LOCATION");
        assert!(v.constraints[0].enabled);
    }

    #[test]
    fn added_constraint_influence_one() {
        let mut v = new_constraint_list_view();
        add_constraint_slot(&mut v, 0, "X", "Y");
        assert!((v.constraints[0].influence - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_influence_clamps() {
        let mut v = new_constraint_list_view();
        add_constraint_slot(&mut v, 0, "X", "Y");
        set_influence(&mut v, 0, 2.5);
        assert!((v.constraints[0].influence - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_influence_valid() {
        let mut v = new_constraint_list_view();
        add_constraint_slot(&mut v, 0, "X", "Y");
        set_influence(&mut v, 0, 0.6);
        assert!((v.constraints[0].influence - 0.6).abs() < 1e-6);
    }

    #[test]
    fn toggle_constraint_disables() {
        let mut v = new_constraint_list_view();
        add_constraint_slot(&mut v, 1, "A", "T");
        toggle_constraint(&mut v, 1);
        assert!(!v.constraints[0].enabled);
    }

    #[test]
    fn toggle_constraint_unknown_id_no_panic() {
        let mut v = new_constraint_list_view();
        toggle_constraint(&mut v, 99);
    }

    #[test]
    fn constraint_count_multiple() {
        let mut v = new_constraint_list_view();
        for i in 0..4u32 {
            add_constraint_slot(&mut v, i, "c", "t");
        }
        assert_eq!(constraint_count(&v), 4);
    }

    #[test]
    fn constraint_stores_type_name() {
        let mut v = new_constraint_list_view();
        add_constraint_slot(&mut v, 0, "c", "STRETCH_TO");
        assert_eq!(v.constraints[0].constraint_type, "STRETCH_TO");
    }
}
