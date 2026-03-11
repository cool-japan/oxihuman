#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// A deferred action that can be scheduled for later execution.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DeferredAction {
    name: String,
    pending: bool,
    payload: String,
}

#[allow(dead_code)]
pub fn new_deferred(name: &str, payload: &str) -> DeferredAction {
    DeferredAction {
        name: name.to_string(),
        pending: true,
        payload: payload.to_string(),
    }
}

#[allow(dead_code)]
pub fn execute_deferred(action: &mut DeferredAction) -> Option<String> {
    if action.pending {
        action.pending = false;
        Some(action.payload.clone())
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn cancel_deferred(action: &mut DeferredAction) {
    action.pending = false;
}

#[allow(dead_code)]
pub fn is_pending(action: &DeferredAction) -> bool {
    action.pending
}

#[allow(dead_code)]
pub fn deferred_count(actions: &[DeferredAction]) -> usize {
    actions.iter().filter(|a| a.pending).count()
}

#[allow(dead_code)]
pub fn deferred_name(action: &DeferredAction) -> &str {
    &action.name
}

#[allow(dead_code)]
pub fn clear_deferred(actions: &mut Vec<DeferredAction>) {
    actions.clear();
}

#[allow(dead_code)]
pub fn deferred_to_json(action: &DeferredAction) -> String {
    format!(
        "{{\"name\":\"{}\",\"pending\":{},\"payload\":\"{}\"}}",
        action.name, action.pending, action.payload
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_deferred() {
        let a = new_deferred("save", "data123");
        assert_eq!(a.name, "save");
        assert!(a.pending);
    }

    #[test]
    fn test_execute_deferred() {
        let mut a = new_deferred("save", "data123");
        let result = execute_deferred(&mut a);
        assert_eq!(result, Some("data123".to_string()));
        assert!(!a.pending);
    }

    #[test]
    fn test_execute_already_done() {
        let mut a = new_deferred("save", "data123");
        execute_deferred(&mut a);
        assert_eq!(execute_deferred(&mut a), None);
    }

    #[test]
    fn test_cancel_deferred() {
        let mut a = new_deferred("save", "data123");
        cancel_deferred(&mut a);
        assert!(!is_pending(&a));
    }

    #[test]
    fn test_is_pending() {
        let a = new_deferred("save", "data123");
        assert!(is_pending(&a));
    }

    #[test]
    fn test_deferred_count() {
        let mut v = vec![new_deferred("a", "1"), new_deferred("b", "2")];
        assert_eq!(deferred_count(&v), 2);
        cancel_deferred(&mut v[0]);
        assert_eq!(deferred_count(&v), 1);
    }

    #[test]
    fn test_deferred_name() {
        let a = new_deferred("load", "x");
        assert_eq!(deferred_name(&a), "load");
    }

    #[test]
    fn test_clear_deferred() {
        let mut v = vec![new_deferred("a", "1"), new_deferred("b", "2")];
        clear_deferred(&mut v);
        assert!(v.is_empty());
    }

    #[test]
    fn test_deferred_to_json() {
        let a = new_deferred("save", "data");
        let json = deferred_to_json(&a);
        assert!(json.contains("\"name\":\"save\""));
        assert!(json.contains("\"pending\":true"));
    }

    #[test]
    fn test_execute_returns_payload() {
        let mut a = new_deferred("act", "payload_val");
        let r = execute_deferred(&mut a);
        assert_eq!(r.unwrap(), "payload_val");
    }
}
