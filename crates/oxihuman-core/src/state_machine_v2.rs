// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A simple finite state machine with named states and guarded transitions.

/// A guard function: returns true if the transition is allowed.
pub type GuardFn = fn(&str, &str) -> bool;

/// A transition definition.
#[allow(dead_code)]
#[derive(Clone)]
pub struct Transition {
    pub from: String,
    pub to: String,
    pub event: String,
    pub guard: Option<GuardFn>,
}

/// A finite state machine with named states and transitions.
#[allow(dead_code)]
pub struct StateMachineV2 {
    current: String,
    states: Vec<String>,
    transitions: Vec<Transition>,
    history: Vec<String>,
    fire_count: u64,
}

#[allow(dead_code)]
impl StateMachineV2 {
    pub fn new(initial: &str) -> Self {
        Self {
            current: initial.to_string(),
            states: vec![initial.to_string()],
            transitions: Vec::new(),
            history: vec![initial.to_string()],
            fire_count: 0,
        }
    }

    pub fn add_state(&mut self, name: &str) {
        if !self.states.contains(&name.to_string()) {
            self.states.push(name.to_string());
        }
    }

    pub fn add_transition(&mut self, from: &str, to: &str, event: &str) {
        self.transitions.push(Transition {
            from: from.to_string(),
            to: to.to_string(),
            event: event.to_string(),
            guard: None,
        });
    }

    pub fn add_guarded_transition(&mut self, from: &str, to: &str, event: &str, guard: GuardFn) {
        self.transitions.push(Transition {
            from: from.to_string(),
            to: to.to_string(),
            event: event.to_string(),
            guard: Some(guard),
        });
    }

    pub fn fire(&mut self, event: &str) -> bool {
        let current = self.current.clone();
        for t in &self.transitions {
            if t.from != current || t.event != event {
                continue;
            }
            let allowed = t.guard.is_none_or(|g| g(&current, &t.to));
            if allowed {
                self.current = t.to.clone();
                self.history.push(t.to.clone());
                self.fire_count += 1;
                return true;
            }
        }
        false
    }

    pub fn current(&self) -> &str {
        &self.current
    }

    pub fn state_count(&self) -> usize {
        self.states.len()
    }

    pub fn transition_count(&self) -> usize {
        self.transitions.len()
    }

    pub fn history(&self) -> &[String] {
        &self.history
    }

    pub fn fire_count(&self) -> u64 {
        self.fire_count
    }

    pub fn is_in(&self, state: &str) -> bool {
        self.current == state
    }

    pub fn can_fire(&self, event: &str) -> bool {
        let current = &self.current;
        self.transitions.iter().any(|t| {
            t.from == *current && t.event == event && t.guard.is_none_or(|g| g(current, &t.to))
        })
    }

    pub fn reachable_events(&self) -> Vec<String> {
        let current = &self.current;
        let mut evts: Vec<String> = self
            .transitions
            .iter()
            .filter(|t| t.from == *current)
            .map(|t| t.event.clone())
            .collect();
        evts.sort();
        evts.dedup();
        evts
    }
}

pub fn new_state_machine(initial: &str) -> StateMachineV2 {
    StateMachineV2::new(initial)
}

pub fn sm_add_state(sm: &mut StateMachineV2, name: &str) {
    sm.add_state(name);
}

pub fn sm_add_transition(sm: &mut StateMachineV2, from: &str, to: &str, event: &str) {
    sm.add_transition(from, to, event);
}

pub fn sm_fire(sm: &mut StateMachineV2, event: &str) -> bool {
    sm.fire(event)
}

pub fn sm_current(sm: &StateMachineV2) -> &str {
    sm.current()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn traffic_light() -> StateMachineV2 {
        let mut sm = new_state_machine("red");
        sm.add_state("green");
        sm.add_state("yellow");
        sm_add_transition(&mut sm, "red", "green", "go");
        sm_add_transition(&mut sm, "green", "yellow", "slow");
        sm_add_transition(&mut sm, "yellow", "red", "stop");
        sm
    }

    #[test]
    fn initial_state() {
        let sm = traffic_light();
        assert_eq!(sm_current(&sm), "red");
    }

    #[test]
    fn valid_transition() {
        let mut sm = traffic_light();
        assert!(sm_fire(&mut sm, "go"));
        assert_eq!(sm_current(&sm), "green");
    }

    #[test]
    fn invalid_event_returns_false() {
        let mut sm = traffic_light();
        assert!(!sm_fire(&mut sm, "stop"));
        assert_eq!(sm_current(&sm), "red");
    }

    #[test]
    fn full_cycle() {
        let mut sm = traffic_light();
        sm_fire(&mut sm, "go");
        sm_fire(&mut sm, "slow");
        sm_fire(&mut sm, "stop");
        assert!(sm.is_in("red"));
    }

    #[test]
    fn history_tracks_states() {
        let mut sm = traffic_light();
        sm_fire(&mut sm, "go");
        sm_fire(&mut sm, "slow");
        assert_eq!(sm.history().len(), 3);
    }

    #[test]
    fn fire_count() {
        let mut sm = traffic_light();
        sm_fire(&mut sm, "go");
        sm_fire(&mut sm, "slow");
        assert_eq!(sm.fire_count(), 2);
    }

    #[test]
    fn can_fire() {
        let sm = traffic_light();
        assert!(sm.can_fire("go"));
        assert!(!sm.can_fire("slow"));
    }

    #[test]
    fn reachable_events() {
        let sm = traffic_light();
        assert_eq!(sm.reachable_events(), vec!["go".to_string()]);
    }

    #[test]
    fn guarded_transition_blocked() {
        let mut sm = new_state_machine("idle");
        sm.add_state("active");
        sm.add_guarded_transition("idle", "active", "start", |_, _| false);
        assert!(!sm_fire(&mut sm, "start"));
        assert!(sm.is_in("idle"));
    }

    #[test]
    fn state_count() {
        let sm = traffic_light();
        assert_eq!(sm.state_count(), 3);
    }
}
