#![allow(dead_code)]
//! State machine for expression transitions.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ExprState {
    pub name: String,
    pub weight: f32,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ExprStateMachine {
    states: Vec<ExprState>,
    transitions: HashMap<String, Vec<String>>,
    current: usize,
}

#[allow(dead_code)]
pub fn new_expr_state_machine() -> ExprStateMachine {
    ExprStateMachine {
        states: Vec::new(),
        transitions: HashMap::new(),
        current: 0,
    }
}

#[allow(dead_code)]
pub fn add_expr_state(sm: &mut ExprStateMachine, name: &str, weight: f32) {
    sm.states.push(ExprState {
        name: name.to_string(),
        weight: weight.clamp(0.0, 1.0),
    });
}

#[allow(dead_code)]
pub fn set_expr_transition(sm: &mut ExprStateMachine, from: &str, to: &str) {
    sm.transitions
        .entry(from.to_string())
        .or_default()
        .push(to.to_string());
}

#[allow(dead_code)]
pub fn current_expr_state(sm: &ExprStateMachine) -> Option<&ExprState> {
    sm.states.get(sm.current)
}

#[allow(dead_code)]
pub fn transition_to(sm: &mut ExprStateMachine, target: &str) -> bool {
    let current_name = match sm.states.get(sm.current) {
        Some(s) => s.name.clone(),
        None => return false,
    };
    let allowed = sm
        .transitions
        .get(&current_name)
        .map(|v| v.iter().any(|t| t == target))
        .unwrap_or(false);
    if allowed {
        if let Some(idx) = sm.states.iter().position(|s| s.name == target) {
            sm.current = idx;
            return true;
        }
    }
    false
}

#[allow(dead_code)]
pub fn state_count(sm: &ExprStateMachine) -> usize {
    sm.states.len()
}

#[allow(dead_code)]
pub fn transitions_from<'a>(sm: &'a ExprStateMachine, state: &str) -> Vec<&'a str> {
    sm.transitions
        .get(state)
        .map(|v| v.iter().map(|s| s.as_str()).collect())
        .unwrap_or_default()
}

#[allow(dead_code)]
pub fn state_machine_to_json(sm: &ExprStateMachine) -> String {
    let states: Vec<String> = sm
        .states
        .iter()
        .map(|s| format!("{{\"name\":\"{}\",\"weight\":{}}}", s.name, s.weight))
        .collect();
    let current = sm
        .states
        .get(sm.current)
        .map(|s| s.name.as_str())
        .unwrap_or("");
    format!(
        "{{\"states\":[{}],\"current\":\"{}\"}}",
        states.join(","),
        current
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_expr_state_machine() {
        let sm = new_expr_state_machine();
        assert_eq!(state_count(&sm), 0);
    }

    #[test]
    fn test_add_expr_state() {
        let mut sm = new_expr_state_machine();
        add_expr_state(&mut sm, "idle", 0.0);
        assert_eq!(state_count(&sm), 1);
    }

    #[test]
    fn test_current_expr_state() {
        let mut sm = new_expr_state_machine();
        add_expr_state(&mut sm, "idle", 0.0);
        let s = current_expr_state(&sm).unwrap();
        assert_eq!(s.name, "idle");
    }

    #[test]
    fn test_transition_to() {
        let mut sm = new_expr_state_machine();
        add_expr_state(&mut sm, "idle", 0.0);
        add_expr_state(&mut sm, "smile", 0.8);
        set_expr_transition(&mut sm, "idle", "smile");
        assert!(transition_to(&mut sm, "smile"));
        assert_eq!(current_expr_state(&sm).unwrap().name, "smile");
    }

    #[test]
    fn test_transition_not_allowed() {
        let mut sm = new_expr_state_machine();
        add_expr_state(&mut sm, "idle", 0.0);
        add_expr_state(&mut sm, "smile", 0.8);
        assert!(!transition_to(&mut sm, "smile"));
    }

    #[test]
    fn test_transitions_from() {
        let mut sm = new_expr_state_machine();
        add_expr_state(&mut sm, "idle", 0.0);
        add_expr_state(&mut sm, "smile", 0.8);
        set_expr_transition(&mut sm, "idle", "smile");
        let t = transitions_from(&sm, "idle");
        assert_eq!(t, vec!["smile"]);
    }

    #[test]
    fn test_state_machine_to_json() {
        let mut sm = new_expr_state_machine();
        add_expr_state(&mut sm, "idle", 0.0);
        let json = state_machine_to_json(&sm);
        assert!(json.contains("\"current\":\"idle\""));
    }

    #[test]
    fn test_empty_transitions() {
        let sm = new_expr_state_machine();
        assert!(transitions_from(&sm, "none").is_empty());
    }

    #[test]
    fn test_current_state_empty() {
        let sm = new_expr_state_machine();
        assert!(current_expr_state(&sm).is_none());
    }

    #[test]
    fn test_multiple_transitions() {
        let mut sm = new_expr_state_machine();
        add_expr_state(&mut sm, "a", 0.0);
        add_expr_state(&mut sm, "b", 0.5);
        add_expr_state(&mut sm, "c", 1.0);
        set_expr_transition(&mut sm, "a", "b");
        set_expr_transition(&mut sm, "a", "c");
        let t = transitions_from(&sm, "a");
        assert_eq!(t.len(), 2);
    }
}
