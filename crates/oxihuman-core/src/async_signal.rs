#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Async signal primitive for cooperative signaling.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AsyncSignal {
    name: String,
    is_set: bool,
    count: usize,
    metadata: HashMap<String, String>,
}

#[allow(dead_code)]
pub fn new_async_signal(name: &str) -> AsyncSignal {
    AsyncSignal {
        name: name.to_string(),
        is_set: false,
        count: 0,
        metadata: HashMap::new(),
    }
}

#[allow(dead_code)]
pub fn signal_set(sig: &mut AsyncSignal) {
    sig.is_set = true;
    sig.count += 1;
}

#[allow(dead_code)]
pub fn signal_wait_stub(sig: &AsyncSignal) -> bool {
    sig.is_set
}

#[allow(dead_code)]
pub fn signal_is_set(sig: &AsyncSignal) -> bool {
    sig.is_set
}

#[allow(dead_code)]
pub fn signal_reset(sig: &mut AsyncSignal) {
    sig.is_set = false;
}

#[allow(dead_code)]
pub fn signal_name_as(sig: &AsyncSignal) -> &str {
    &sig.name
}

#[allow(dead_code)]
pub fn signal_to_json(sig: &AsyncSignal) -> String {
    format!(
        r#"{{"name":"{}","is_set":{},"count":{}}}"#,
        sig.name, sig.is_set, sig.count
    )
}

#[allow(dead_code)]
pub fn signal_count_as(sig: &AsyncSignal) -> usize {
    sig.count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_async_signal() {
        let s = new_async_signal("test");
        assert_eq!(signal_name_as(&s), "test");
        assert!(!signal_is_set(&s));
    }

    #[test]
    fn test_signal_set_and_check() {
        let mut s = new_async_signal("sig");
        signal_set(&mut s);
        assert!(signal_is_set(&s));
    }

    #[test]
    fn test_signal_reset() {
        let mut s = new_async_signal("sig");
        signal_set(&mut s);
        signal_reset(&mut s);
        assert!(!signal_is_set(&s));
    }

    #[test]
    fn test_signal_wait_stub() {
        let mut s = new_async_signal("sig");
        assert!(!signal_wait_stub(&s));
        signal_set(&mut s);
        assert!(signal_wait_stub(&s));
    }

    #[test]
    fn test_signal_count() {
        let mut s = new_async_signal("sig");
        signal_set(&mut s);
        signal_set(&mut s);
        assert_eq!(signal_count_as(&s), 2);
    }

    #[test]
    fn test_signal_name() {
        let s = new_async_signal("my_signal");
        assert_eq!(signal_name_as(&s), "my_signal");
    }

    #[test]
    fn test_signal_to_json() {
        let s = new_async_signal("j");
        let json = signal_to_json(&s);
        assert!(json.contains("\"name\":\"j\""));
        assert!(json.contains("\"is_set\":false"));
    }

    #[test]
    fn test_signal_to_json_after_set() {
        let mut s = new_async_signal("j");
        signal_set(&mut s);
        let json = signal_to_json(&s);
        assert!(json.contains("\"is_set\":true"));
        assert!(json.contains("\"count\":1"));
    }

    #[test]
    fn test_multiple_reset_cycles() {
        let mut s = new_async_signal("cycle");
        signal_set(&mut s);
        signal_reset(&mut s);
        signal_set(&mut s);
        assert!(signal_is_set(&s));
        assert_eq!(signal_count_as(&s), 2);
    }

    #[test]
    fn test_initial_count_zero() {
        let s = new_async_signal("z");
        assert_eq!(signal_count_as(&s), 0);
    }
}
