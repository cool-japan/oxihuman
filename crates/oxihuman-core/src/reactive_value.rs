//! Reactive/observable `f32` value that notifies listeners when it changes.
//!
//! Listeners are identified by a monotonically-increasing `u32` ID so they can
//! be selectively removed. The value stores a generation counter and a
//! notification count for diagnostics.

/// Configuration for a reactive value.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ReactiveConfig {
    /// Initial value.
    pub initial: f32,
    /// Minimum change magnitude that triggers a notification.
    pub dead_zone: f32,
}

/// A listener: a name tag plus a stored target value (simplified; no closures).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChangeListener {
    /// Opaque listener ID.
    pub id: u32,
    /// Human-readable tag.
    pub tag: String,
    /// The value observed when the listener was last notified.
    pub last_seen: f32,
}

/// A reactive `f32` value.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ReactiveValue {
    config: ReactiveConfig,
    value: f32,
    listeners: Vec<ChangeListener>,
    next_id: u32,
    notify_count: u32,
}

/// Return a default [`ReactiveConfig`].
#[allow(dead_code)]
pub fn default_reactive_config() -> ReactiveConfig {
    ReactiveConfig {
        initial: 0.0,
        dead_zone: 1e-6,
    }
}

/// Create a new [`ReactiveValue`].
#[allow(dead_code)]
pub fn new_reactive_value(config: ReactiveConfig) -> ReactiveValue {
    let initial = config.initial;
    ReactiveValue {
        config,
        value: initial,
        listeners: Vec::new(),
        next_id: 1,
        notify_count: 0,
    }
}

/// Read the current value.
#[allow(dead_code)]
pub fn reactive_get(rv: &ReactiveValue) -> f32 {
    rv.value
}

/// Set a new value.  If the change exceeds `dead_zone`, all listeners are
/// updated and the notification counter is incremented.  Returns `true` if
/// listeners were notified.
#[allow(dead_code)]
pub fn reactive_set(rv: &mut ReactiveValue, new_val: f32) -> bool {
    if (new_val - rv.value).abs() <= rv.config.dead_zone {
        return false;
    }
    rv.value = new_val;
    rv.notify_count += 1;
    for l in &mut rv.listeners {
        l.last_seen = new_val;
    }
    true
}

/// Register a new listener.  Returns the listener ID.
#[allow(dead_code)]
pub fn reactive_subscribe(rv: &mut ReactiveValue, tag: &str) -> u32 {
    let id = rv.next_id;
    rv.next_id += 1;
    rv.listeners.push(ChangeListener {
        id,
        tag: tag.to_owned(),
        last_seen: rv.value,
    });
    id
}

/// Remove the listener with the given ID.  Returns `true` if it was found.
#[allow(dead_code)]
pub fn reactive_unsubscribe(rv: &mut ReactiveValue, id: u32) -> bool {
    let before = rv.listeners.len();
    rv.listeners.retain(|l| l.id != id);
    rv.listeners.len() < before
}

/// Return the number of registered listeners.
#[allow(dead_code)]
pub fn reactive_listener_count(rv: &ReactiveValue) -> usize {
    rv.listeners.len()
}

/// Return the total number of times listeners were notified.
#[allow(dead_code)]
pub fn reactive_notify_count(rv: &ReactiveValue) -> u32 {
    rv.notify_count
}

/// Reset the value to `config.initial` and zero the notification counter.
/// Listeners are preserved but their `last_seen` is updated.
#[allow(dead_code)]
pub fn reactive_reset(rv: &mut ReactiveValue) {
    rv.value = rv.config.initial;
    rv.notify_count = 0;
    for l in &mut rv.listeners {
        l.last_seen = rv.value;
    }
}

/// Serialize the reactive value to a compact JSON string.
#[allow(dead_code)]
pub fn reactive_to_json(rv: &ReactiveValue) -> String {
    format!(
        r#"{{"value":{:.6},"listener_count":{},"notify_count":{}}}"#,
        rv.value,
        rv.listeners.len(),
        rv.notify_count,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rv() -> ReactiveValue {
        new_reactive_value(default_reactive_config())
    }

    #[test]
    fn test_initial_value() {
        let rv = make_rv();
        assert!((reactive_get(&rv) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_set_triggers_notify() {
        let mut rv = make_rv();
        assert!(reactive_set(&mut rv, 1.0));
        assert_eq!(reactive_notify_count(&rv), 1);
    }

    #[test]
    fn test_set_within_dead_zone_no_notify() {
        let mut rv = make_rv();
        assert!(!reactive_set(&mut rv, 1e-9)); // within dead_zone of 1e-6
        assert_eq!(reactive_notify_count(&rv), 0);
    }

    #[test]
    fn test_subscribe_returns_id() {
        let mut rv = make_rv();
        let id = reactive_subscribe(&mut rv, "test");
        assert!(id > 0);
        assert_eq!(reactive_listener_count(&rv), 1);
    }

    #[test]
    fn test_unsubscribe_removes_listener() {
        let mut rv = make_rv();
        let id = reactive_subscribe(&mut rv, "listener_a");
        assert!(reactive_unsubscribe(&mut rv, id));
        assert_eq!(reactive_listener_count(&rv), 0);
    }

    #[test]
    fn test_unsubscribe_unknown_returns_false() {
        let mut rv = make_rv();
        assert!(!reactive_unsubscribe(&mut rv, 999));
    }

    #[test]
    fn test_listener_last_seen_updated_on_set() {
        let mut rv = make_rv();
        reactive_subscribe(&mut rv, "a");
        reactive_set(&mut rv, 0.7);
        assert!((rv.listeners[0].last_seen - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_reset_restores_initial() {
        let mut rv = make_rv();
        reactive_set(&mut rv, 5.0);
        reactive_reset(&mut rv);
        assert!((reactive_get(&rv) - 0.0).abs() < 1e-9);
        assert_eq!(reactive_notify_count(&rv), 0);
    }

    #[test]
    fn test_to_json_fields() {
        let rv = make_rv();
        let json = reactive_to_json(&rv);
        assert!(json.contains("value"));
        assert!(json.contains("listener_count"));
        assert!(json.contains("notify_count"));
    }

    #[test]
    fn test_multiple_listeners() {
        let mut rv = make_rv();
        reactive_subscribe(&mut rv, "a");
        reactive_subscribe(&mut rv, "b");
        reactive_subscribe(&mut rv, "c");
        assert_eq!(reactive_listener_count(&rv), 3);
        reactive_set(&mut rv, 2.0);
        for l in &rv.listeners {
            assert!((l.last_seen - 2.0).abs() < 1e-6);
        }
    }
}
