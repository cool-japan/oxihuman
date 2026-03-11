#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Modifier stack UI state.

/// A single modifier slot in the stack.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ModifierSlot {
    pub id: u32,
    pub name: String,
    pub enabled: bool,
    pub expanded: bool,
    pub order: u32,
}

/// The modifier stack panel state.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ModifierStackView {
    pub modifiers: Vec<ModifierSlot>,
}

/// Create a new empty `ModifierStackView`.
#[allow(dead_code)]
pub fn new_modifier_stack_view() -> ModifierStackView {
    ModifierStackView::default()
}

/// Add a modifier slot.  Order defaults to the current length.
#[allow(dead_code)]
pub fn add_modifier_slot(view: &mut ModifierStackView, id: u32, name: &str) {
    let order = view.modifiers.len() as u32;
    view.modifiers.push(ModifierSlot {
        id,
        name: name.to_string(),
        enabled: true,
        expanded: false,
        order,
    });
}

/// Toggle the `enabled` flag of the modifier with the given id.
#[allow(dead_code)]
pub fn toggle_modifier(view: &mut ModifierStackView, id: u32) {
    if let Some(slot) = view.modifiers.iter_mut().find(|m| m.id == id) {
        slot.enabled = !slot.enabled;
    }
}

/// Move the modifier with `id` one step up (lower order index) in the stack.
#[allow(dead_code)]
pub fn move_modifier_up(view: &mut ModifierStackView, id: u32) {
    if let Some(pos) = view.modifiers.iter().position(|m| m.id == id) {
        if pos > 0 {
            view.modifiers.swap(pos, pos - 1);
            // update order fields
            for (i, m) in view.modifiers.iter_mut().enumerate() {
                m.order = i as u32;
            }
        }
    }
}

/// Return the number of modifier slots.
#[allow(dead_code)]
pub fn modifier_count(view: &ModifierStackView) -> usize {
    view.modifiers.len()
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_view_is_empty() {
        let v = new_modifier_stack_view();
        assert_eq!(modifier_count(&v), 0);
    }

    #[test]
    fn add_modifier_increments_count() {
        let mut v = new_modifier_stack_view();
        add_modifier_slot(&mut v, 0, "Subsurf");
        assert_eq!(modifier_count(&v), 1);
    }

    #[test]
    fn added_modifier_is_enabled() {
        let mut v = new_modifier_stack_view();
        add_modifier_slot(&mut v, 0, "Armature");
        assert!(v.modifiers[0].enabled);
    }

    #[test]
    fn toggle_modifier_flips_enabled() {
        let mut v = new_modifier_stack_view();
        add_modifier_slot(&mut v, 1, "Mirror");
        toggle_modifier(&mut v, 1);
        assert!(!v.modifiers[0].enabled);
    }

    #[test]
    fn toggle_modifier_unknown_id_no_panic() {
        let mut v = new_modifier_stack_view();
        toggle_modifier(&mut v, 99); // must not panic
    }

    #[test]
    fn move_modifier_up_changes_order() {
        let mut v = new_modifier_stack_view();
        add_modifier_slot(&mut v, 10, "A");
        add_modifier_slot(&mut v, 11, "B");
        move_modifier_up(&mut v, 11);
        assert_eq!(v.modifiers[0].id, 11);
    }

    #[test]
    fn move_modifier_up_top_no_change() {
        let mut v = new_modifier_stack_view();
        add_modifier_slot(&mut v, 10, "A");
        move_modifier_up(&mut v, 10); // already at top, must not panic
        assert_eq!(v.modifiers[0].id, 10);
    }

    #[test]
    fn move_modifier_up_unknown_no_panic() {
        let mut v = new_modifier_stack_view();
        move_modifier_up(&mut v, 99); // must not panic
    }

    #[test]
    fn modifier_count_multiple() {
        let mut v = new_modifier_stack_view();
        for i in 0..5u32 {
            add_modifier_slot(&mut v, i, "mod");
        }
        assert_eq!(modifier_count(&v), 5);
    }

    #[test]
    fn order_assigned_sequentially() {
        let mut v = new_modifier_stack_view();
        add_modifier_slot(&mut v, 0, "A");
        add_modifier_slot(&mut v, 1, "B");
        assert_eq!(v.modifiers[0].order, 0);
        assert_eq!(v.modifiers[1].order, 1);
    }
}
