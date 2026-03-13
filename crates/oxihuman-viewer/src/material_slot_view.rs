#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Material slot UI state.

/// A single material slot.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MaterialSlot {
    pub slot_id: u32,
    pub material_name: String,
    pub active: bool,
}

/// The full material-slot panel state.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct MaterialSlotView {
    pub slots: Vec<MaterialSlot>,
    pub active_slot: u32,
}

/// Create a new empty `MaterialSlotView`.
#[allow(dead_code)]
pub fn new_material_slot_view() -> MaterialSlotView {
    MaterialSlotView::default()
}

/// Add a material slot with the given id and name.
#[allow(dead_code)]
pub fn add_slot(view: &mut MaterialSlotView, id: u32, name: &str) {
    view.slots.push(MaterialSlot {
        slot_id: id,
        material_name: name.to_string(),
        active: false,
    });
}

/// Set the active slot by id.
#[allow(dead_code)]
pub fn set_active_slot(view: &mut MaterialSlotView, id: u32) {
    view.active_slot = id;
    for slot in view.slots.iter_mut() {
        slot.active = slot.slot_id == id;
    }
}

/// Return the number of material slots.
#[allow(dead_code)]
pub fn slot_count(view: &MaterialSlotView) -> usize {
    view.slots.len()
}

/// Return the name of the currently active material, if any.
#[allow(dead_code)]
pub fn active_material_name(view: &MaterialSlotView) -> Option<&str> {
    view.slots
        .iter()
        .find(|s| s.slot_id == view.active_slot)
        .map(|s| s.material_name.as_str())
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_view_is_empty() {
        let v = new_material_slot_view();
        assert_eq!(slot_count(&v), 0);
    }

    #[test]
    fn add_slot_increments_count() {
        let mut v = new_material_slot_view();
        add_slot(&mut v, 0, "DefaultMat");
        assert_eq!(slot_count(&v), 1);
    }

    #[test]
    fn active_material_name_none_when_empty() {
        let v = new_material_slot_view();
        assert!(active_material_name(&v).is_none());
    }

    #[test]
    fn set_active_slot_updates_active_flag() {
        let mut v = new_material_slot_view();
        add_slot(&mut v, 0, "A");
        add_slot(&mut v, 1, "B");
        set_active_slot(&mut v, 1);
        assert!(v.slots[1].active);
        assert!(!v.slots[0].active);
    }

    #[test]
    fn active_material_name_returns_name() {
        let mut v = new_material_slot_view();
        add_slot(&mut v, 5, "Skin");
        set_active_slot(&mut v, 5);
        assert_eq!(active_material_name(&v), Some("Skin"));
    }

    #[test]
    fn set_active_slot_unknown_id() {
        let mut v = new_material_slot_view();
        add_slot(&mut v, 0, "A");
        set_active_slot(&mut v, 99); // no match — must not panic
        assert!(active_material_name(&v).is_none());
    }

    #[test]
    fn slot_count_multiple() {
        let mut v = new_material_slot_view();
        for i in 0..4u32 {
            add_slot(&mut v, i, "mat");
        }
        assert_eq!(slot_count(&v), 4);
    }

    #[test]
    fn set_active_clears_previous() {
        let mut v = new_material_slot_view();
        add_slot(&mut v, 0, "A");
        add_slot(&mut v, 1, "B");
        set_active_slot(&mut v, 0);
        set_active_slot(&mut v, 1);
        assert!(!v.slots[0].active);
        assert!(v.slots[1].active);
    }

    #[test]
    fn add_slot_stores_name() {
        let mut v = new_material_slot_view();
        add_slot(&mut v, 7, "SpecialMat");
        assert_eq!(v.slots[0].material_name, "SpecialMat");
    }

    #[test]
    fn active_slot_default_zero() {
        let v = new_material_slot_view();
        assert_eq!(v.active_slot, 0);
    }
}
