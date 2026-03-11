// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Animation state machine view stub.

/// State machine state kind.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StateKind {
    Entry,
    Exit,
    Any,
    Normal,
    SubStateMachine,
}

/// A state machine state node.
#[derive(Debug, Clone)]
pub struct SmState {
    pub id: u32,
    pub kind: StateKind,
    pub label: String,
    pub position: [f32; 2],
    pub is_active: bool,
}

/// A state transition entry.
#[derive(Debug, Clone)]
pub struct SmTransition {
    pub from_id: u32,
    pub to_id: u32,
    pub condition: String,
}

/// State machine view configuration.
#[derive(Debug, Clone)]
pub struct StateMachineView {
    pub states: Vec<SmState>,
    pub transitions: Vec<SmTransition>,
    pub show_conditions: bool,
    pub zoom: f32,
    pub enabled: bool,
}

impl StateMachineView {
    pub fn new() -> Self {
        StateMachineView {
            states: Vec::new(),
            transitions: Vec::new(),
            show_conditions: true,
            zoom: 1.0,
            enabled: true,
        }
    }
}

impl Default for StateMachineView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new state machine view.
pub fn new_state_machine_view() -> StateMachineView {
    StateMachineView::new()
}

/// Add a state node.
pub fn smv_add_state(view: &mut StateMachineView, state: SmState) {
    view.states.push(state);
}

/// Add a transition.
pub fn smv_add_transition(view: &mut StateMachineView, transition: SmTransition) {
    view.transitions.push(transition);
}

/// Clear all states and transitions.
pub fn smv_clear(view: &mut StateMachineView) {
    view.states.clear();
    view.transitions.clear();
}

/// Set zoom level.
pub fn smv_set_zoom(view: &mut StateMachineView, zoom: f32) {
    view.zoom = zoom.max(0.05);
}

/// Toggle condition label display.
pub fn smv_show_conditions(view: &mut StateMachineView, show: bool) {
    view.show_conditions = show;
}

/// Enable or disable.
pub fn smv_set_enabled(view: &mut StateMachineView, enabled: bool) {
    view.enabled = enabled;
}

/// Return state count.
pub fn smv_state_count(view: &StateMachineView) -> usize {
    view.states.len()
}

/// Return transition count.
pub fn smv_transition_count(view: &StateMachineView) -> usize {
    view.transitions.len()
}

/// Serialize to JSON-like string.
pub fn smv_to_json(view: &StateMachineView) -> String {
    format!(
        r#"{{"state_count":{},"transition_count":{},"show_conditions":{},"zoom":{},"enabled":{}}}"#,
        view.states.len(),
        view.transitions.len(),
        view.show_conditions,
        view.zoom,
        view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state(id: u32) -> SmState {
        SmState {
            id,
            kind: StateKind::Normal,
            label: format!("state_{id}"),
            position: [0.0, 0.0],
            is_active: false,
        }
    }

    fn make_transition(from: u32, to: u32) -> SmTransition {
        SmTransition {
            from_id: from,
            to_id: to,
            condition: "speed > 0".to_string(),
        }
    }

    #[test]
    fn test_initial_empty() {
        let v = new_state_machine_view();
        assert_eq!(smv_state_count(&v), 0 /* no states initially */);
    }

    #[test]
    fn test_add_state() {
        let mut v = new_state_machine_view();
        smv_add_state(&mut v, make_state(0));
        assert_eq!(smv_state_count(&v), 1 /* one state after add */);
    }

    #[test]
    fn test_add_transition() {
        let mut v = new_state_machine_view();
        smv_add_transition(&mut v, make_transition(0, 1));
        assert_eq!(
            smv_transition_count(&v),
            1 /* one transition after add */
        );
    }

    #[test]
    fn test_clear() {
        let mut v = new_state_machine_view();
        smv_add_state(&mut v, make_state(0));
        smv_add_transition(&mut v, make_transition(0, 1));
        smv_clear(&mut v);
        assert_eq!(smv_state_count(&v), 0 /* states cleared */);
        assert_eq!(smv_transition_count(&v), 0 /* transitions cleared */);
    }

    #[test]
    fn test_zoom_min() {
        let mut v = new_state_machine_view();
        smv_set_zoom(&mut v, 0.0);
        assert!((v.zoom - 0.05).abs() < 1e-6 /* minimum zoom must be 0.05 */);
    }

    #[test]
    fn test_show_conditions() {
        let mut v = new_state_machine_view();
        smv_show_conditions(&mut v, false);
        assert!(!v.show_conditions /* conditions must be hidden */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_state_machine_view();
        smv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_state_count() {
        let v = new_state_machine_view();
        let j = smv_to_json(&v);
        assert!(j.contains("\"state_count\"") /* JSON must have state_count */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_state_machine_view();
        assert!(v.enabled /* must be enabled by default */);
    }
}
