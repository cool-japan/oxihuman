// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Constraint indicator overlay view.

/// A single constraint indicator entry.
#[derive(Debug, Clone)]
pub struct ConstraintIndicator {
    pub object_id: u32,
    pub constraint_type: String,
    pub influence: f32,
    pub muted: bool,
}

/// Constraint indicator overlay state.
#[derive(Debug, Clone)]
pub struct ConstraintIndicatorView {
    pub indicators: Vec<ConstraintIndicator>,
    pub visible: bool,
    pub show_muted: bool,
}

impl Default for ConstraintIndicatorView {
    fn default() -> Self {
        Self {
            indicators: Vec::new(),
            visible: true,
            show_muted: false,
        }
    }
}

/// Create a new ConstraintIndicatorView.
pub fn new_constraint_indicator_view() -> ConstraintIndicatorView {
    ConstraintIndicatorView::default()
}

/// Add a constraint indicator.
pub fn constraint_ind_add(
    view: &mut ConstraintIndicatorView,
    object_id: u32,
    constraint_type: &str,
    influence: f32,
) {
    view.indicators.push(ConstraintIndicator {
        object_id,
        constraint_type: constraint_type.to_string(),
        influence: influence.clamp(0.0, 1.0),
        muted: false,
    });
}

/// Mute a constraint for a given object.
pub fn constraint_ind_mute(view: &mut ConstraintIndicatorView, object_id: u32) {
    for ind in &mut view.indicators {
        if ind.object_id == object_id {
            ind.muted = true;
        }
    }
}

/// Count active (non-muted) constraints.
pub fn constraint_ind_active_count(view: &ConstraintIndicatorView) -> usize {
    view.indicators.iter().filter(|i| !i.muted).count()
}

/// Serialize to JSON.
pub fn constraint_indicator_to_json(view: &ConstraintIndicatorView) -> String {
    format!(
        r#"{{"visible":{},"count":{},"active":{}}}"#,
        view.visible,
        view.indicators.len(),
        constraint_ind_active_count(view),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let v = new_constraint_indicator_view();
        assert!(v.indicators.is_empty() /* empty */);
    }

    #[test]
    fn test_add() {
        let mut v = new_constraint_indicator_view();
        constraint_ind_add(&mut v, 1, "COPY_LOC", 1.0);
        assert_eq!(v.indicators.len(), 1 /* one entry */);
    }

    #[test]
    fn test_influence_clamp() {
        let mut v = new_constraint_indicator_view();
        constraint_ind_add(&mut v, 1, "IK", 5.0);
        assert!((v.indicators[0].influence - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_mute() {
        let mut v = new_constraint_indicator_view();
        constraint_ind_add(&mut v, 1, "COPY_ROT", 1.0);
        constraint_ind_mute(&mut v, 1);
        assert!(v.indicators[0].muted /* muted */);
    }

    #[test]
    fn test_active_count() {
        let mut v = new_constraint_indicator_view();
        constraint_ind_add(&mut v, 1, "IK", 1.0);
        constraint_ind_add(&mut v, 2, "TRACK_TO", 1.0);
        constraint_ind_mute(&mut v, 1);
        assert_eq!(constraint_ind_active_count(&v), 1 /* one active */);
    }

    #[test]
    fn test_active_count_all_active() {
        let mut v = new_constraint_indicator_view();
        constraint_ind_add(&mut v, 1, "IK", 1.0);
        constraint_ind_add(&mut v, 2, "IK", 1.0);
        assert_eq!(constraint_ind_active_count(&v), 2 /* both active */);
    }

    #[test]
    fn test_json_keys() {
        let v = new_constraint_indicator_view();
        let j = constraint_indicator_to_json(&v);
        assert!(j.contains("visible") /* key */);
    }

    #[test]
    fn test_default_visible() {
        let v = ConstraintIndicatorView::default();
        assert!(v.visible /* visible */);
    }

    #[test]
    fn test_clone() {
        let mut v = new_constraint_indicator_view();
        constraint_ind_add(&mut v, 1, "IK", 1.0);
        let c = v.clone();
        assert_eq!(c.indicators.len(), 1 /* clone ok */);
    }
}
