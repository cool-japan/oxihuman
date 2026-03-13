#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Finite state machine builder.

use std::collections::{HashMap, HashSet};

/// A transition in the FSM.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FsmTransition {
    pub from: String,
    pub to: String,
    pub event: String,
}

/// A built FSM: set of states and transitions.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct BuiltFsm {
    pub states: HashSet<String>,
    pub transitions: Vec<FsmTransition>,
}

/// Builder for constructing an FSM.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct FsmBuilder {
    states: HashSet<String>,
    transitions: Vec<FsmTransition>,
    /// Map from event+from -> to for quick lookup after build.
    lookup: HashMap<(String, String), String>,
}

/// Create a new empty `FsmBuilder`.
#[allow(dead_code)]
pub fn new_fsm_builder() -> FsmBuilder {
    FsmBuilder::default()
}

/// Add a state to the builder.
#[allow(dead_code)]
pub fn add_state(builder: &mut FsmBuilder, state: &str) {
    builder.states.insert(state.to_string());
}

/// Add a transition to the builder.
#[allow(dead_code)]
pub fn add_transition(builder: &mut FsmBuilder, from: &str, to: &str, event: &str) {
    builder.states.insert(from.to_string());
    builder.states.insert(to.to_string());
    builder.transitions.push(FsmTransition {
        from: from.to_string(),
        to: to.to_string(),
        event: event.to_string(),
    });
    builder
        .lookup
        .insert((event.to_string(), from.to_string()), to.to_string());
}

/// Build the FSM, consuming the builder.
#[allow(dead_code)]
pub fn build_fsm(builder: FsmBuilder) -> BuiltFsm {
    BuiltFsm {
        states: builder.states,
        transitions: builder.transitions,
    }
}

/// Return the number of states in the builder.
#[allow(dead_code)]
pub fn state_count(builder: &FsmBuilder) -> usize {
    builder.states.len()
}

/// Return the number of transitions in the builder.
#[allow(dead_code)]
pub fn transition_count(builder: &FsmBuilder) -> usize {
    builder.transitions.len()
}

/// Check if a state exists in the builder.
#[allow(dead_code)]
pub fn has_state(builder: &FsmBuilder, state: &str) -> bool {
    builder.states.contains(state)
}

/// Check if a transition exists (by from and event).
#[allow(dead_code)]
pub fn has_transition(builder: &FsmBuilder, from: &str, event: &str) -> bool {
    builder
        .lookup
        .contains_key(&(event.to_string(), from.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_fsm_builder() {
        let b = new_fsm_builder();
        assert_eq!(state_count(&b), 0);
        assert_eq!(transition_count(&b), 0);
    }

    #[test]
    fn test_add_state() {
        let mut b = new_fsm_builder();
        add_state(&mut b, "idle");
        assert!(has_state(&b, "idle"));
        assert_eq!(state_count(&b), 1);
    }

    #[test]
    fn test_add_transition() {
        let mut b = new_fsm_builder();
        add_transition(&mut b, "idle", "running", "start");
        assert_eq!(transition_count(&b), 1);
        assert!(has_state(&b, "idle"));
        assert!(has_state(&b, "running"));
    }

    #[test]
    fn test_has_transition() {
        let mut b = new_fsm_builder();
        add_transition(&mut b, "a", "b", "go");
        assert!(has_transition(&b, "a", "go"));
        assert!(!has_transition(&b, "a", "stop"));
    }

    #[test]
    fn test_build_fsm() {
        let mut b = new_fsm_builder();
        add_state(&mut b, "idle");
        add_transition(&mut b, "idle", "done", "finish");
        let fsm = build_fsm(b);
        assert!(fsm.states.contains("idle"));
        assert!(fsm.states.contains("done"));
        assert_eq!(fsm.transitions.len(), 1);
    }

    #[test]
    fn test_duplicate_states() {
        let mut b = new_fsm_builder();
        add_state(&mut b, "s");
        add_state(&mut b, "s");
        assert_eq!(state_count(&b), 1);
    }

    #[test]
    fn test_multiple_transitions() {
        let mut b = new_fsm_builder();
        add_transition(&mut b, "a", "b", "e1");
        add_transition(&mut b, "b", "c", "e2");
        assert_eq!(transition_count(&b), 2);
    }

    #[test]
    fn test_state_added_via_transition() {
        let mut b = new_fsm_builder();
        add_transition(&mut b, "x", "y", "evt");
        assert!(has_state(&b, "x"));
        assert!(has_state(&b, "y"));
    }
}
