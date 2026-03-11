#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Scope guard / deferred action utility.

/// An action to run (or cancel) later.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ScopeAction {
    name: String,
    cancelled: bool,
    executed: bool,
}

#[allow(dead_code)]
pub fn new_scope_action(name: &str) -> ScopeAction {
    ScopeAction {
        name: name.to_string(),
        cancelled: false,
        executed: false,
    }
}

#[allow(dead_code)]
pub fn scope_action_run(action: &mut ScopeAction) -> bool {
    if action.cancelled {
        return false;
    }
    action.executed = true;
    true
}

#[allow(dead_code)]
pub fn scope_action_cancel(action: &mut ScopeAction) {
    action.cancelled = true;
}

#[allow(dead_code)]
pub fn scope_action_is_cancelled(action: &ScopeAction) -> bool {
    action.cancelled
}

#[allow(dead_code)]
pub fn scope_action_name(action: &ScopeAction) -> &str {
    &action.name
}

#[allow(dead_code)]
pub fn scope_action_count(actions: &[ScopeAction]) -> usize {
    actions.len()
}

#[allow(dead_code)]
pub fn scope_action_pending(actions: &[ScopeAction]) -> usize {
    actions.iter().filter(|a| !a.cancelled && !a.executed).count()
}

#[allow(dead_code)]
pub fn scope_action_executed(actions: &[ScopeAction]) -> usize {
    actions.iter().filter(|a| a.executed).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let a = new_scope_action("cleanup");
        assert_eq!(scope_action_name(&a), "cleanup");
        assert!(!scope_action_is_cancelled(&a));
    }

    #[test]
    fn test_run() {
        let mut a = new_scope_action("run_me");
        assert!(scope_action_run(&mut a));
    }

    #[test]
    fn test_cancel() {
        let mut a = new_scope_action("x");
        scope_action_cancel(&mut a);
        assert!(scope_action_is_cancelled(&a));
    }

    #[test]
    fn test_cancel_prevents_run() {
        let mut a = new_scope_action("x");
        scope_action_cancel(&mut a);
        assert!(!scope_action_run(&mut a));
    }

    #[test]
    fn test_count() {
        let actions = vec![new_scope_action("a"), new_scope_action("b")];
        assert_eq!(scope_action_count(&actions), 2);
    }

    #[test]
    fn test_pending() {
        let mut actions = vec![new_scope_action("a"), new_scope_action("b")];
        scope_action_run(&mut actions[0]);
        assert_eq!(scope_action_pending(&actions), 1);
    }

    #[test]
    fn test_executed() {
        let mut actions = vec![new_scope_action("a"), new_scope_action("b")];
        scope_action_run(&mut actions[0]);
        scope_action_run(&mut actions[1]);
        assert_eq!(scope_action_executed(&actions), 2);
    }

    #[test]
    fn test_name() {
        let a = new_scope_action("my_action");
        assert_eq!(scope_action_name(&a), "my_action");
    }

    #[test]
    fn test_empty_slice() {
        let actions: Vec<ScopeAction> = vec![];
        assert_eq!(scope_action_count(&actions), 0);
        assert_eq!(scope_action_pending(&actions), 0);
        assert_eq!(scope_action_executed(&actions), 0);
    }

    #[test]
    fn test_cancel_and_executed() {
        let mut a = new_scope_action("x");
        scope_action_cancel(&mut a);
        let actions = vec![a];
        assert_eq!(scope_action_pending(&actions), 0);
        assert_eq!(scope_action_executed(&actions), 0);
    }
}
