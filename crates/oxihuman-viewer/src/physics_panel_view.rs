#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Physics properties panel view.

/// Physics panel view state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PhysicsPanelView {
    /// Active physics type: 0=none, 1=rigid, 2=cloth, 3=fluid, 4=softbody.
    pub active_type: u8,
    pub show_rigid: bool,
    pub show_cloth: bool,
    pub show_fluid: bool,
    pub show_softbody: bool,
    pub expanded: bool,
}

/// Create a default `PhysicsPanelView` (no physics active).
#[allow(dead_code)]
pub fn default_physics_panel_view() -> PhysicsPanelView {
    PhysicsPanelView {
        active_type: 0,
        show_rigid: false,
        show_cloth: false,
        show_fluid: false,
        show_softbody: false,
        expanded: true,
    }
}

/// Switch the active physics type and update the show flags.
#[allow(dead_code)]
pub fn switch_physics_type(view: &mut PhysicsPanelView, type_: u8) {
    let t = type_.min(4);
    view.active_type = t;
    view.show_rigid = t == 1;
    view.show_cloth = t == 2;
    view.show_fluid = t == 3;
    view.show_softbody = t == 4;
}

/// Toggle the expanded state of the panel.
#[allow(dead_code)]
pub fn toggle_panel(view: &mut PhysicsPanelView) {
    view.expanded = !view.expanded;
}

/// Return a static name for a physics type code.
///
/// * 0 → "None"
/// * 1 → "Rigid Body"
/// * 2 → "Cloth"
/// * 3 → "Fluid"
/// * 4 → "Soft Body"
/// * _ → "Unknown"
#[allow(dead_code)]
pub fn physics_type_name(type_: u8) -> &'static str {
    match type_ {
        0 => "None",
        1 => "Rigid Body",
        2 => "Cloth",
        3 => "Fluid",
        4 => "Soft Body",
        _ => "Unknown",
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_active_type_none() {
        let v = default_physics_panel_view();
        assert_eq!(v.active_type, 0);
    }

    #[test]
    fn default_show_flags_false() {
        let v = default_physics_panel_view();
        assert!(!v.show_rigid);
        assert!(!v.show_cloth);
        assert!(!v.show_fluid);
        assert!(!v.show_softbody);
    }

    #[test]
    fn switch_rigid() {
        let mut v = default_physics_panel_view();
        switch_physics_type(&mut v, 1);
        assert!(v.show_rigid);
        assert!(!v.show_cloth);
    }

    #[test]
    fn switch_cloth() {
        let mut v = default_physics_panel_view();
        switch_physics_type(&mut v, 2);
        assert!(v.show_cloth);
    }

    #[test]
    fn switch_clamps_to_four() {
        let mut v = default_physics_panel_view();
        switch_physics_type(&mut v, 99);
        assert_eq!(v.active_type, 4);
    }

    #[test]
    fn toggle_panel_collapses() {
        let mut v = default_physics_panel_view();
        toggle_panel(&mut v);
        assert!(!v.expanded);
    }

    #[test]
    fn toggle_panel_reopens() {
        let mut v = default_physics_panel_view();
        toggle_panel(&mut v);
        toggle_panel(&mut v);
        assert!(v.expanded);
    }

    #[test]
    fn physics_type_name_none() {
        assert_eq!(physics_type_name(0), "None");
    }

    #[test]
    fn physics_type_name_cloth() {
        assert_eq!(physics_type_name(2), "Cloth");
    }

    #[test]
    fn physics_type_name_unknown() {
        assert_eq!(physics_type_name(200), "Unknown");
    }
}
