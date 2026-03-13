// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Simple observer list with string-keyed callbacks (stored as labels).
#[allow(dead_code)]
pub struct ObserverList {
    observers: Vec<(u64, String)>,
    next_id: u64,
    notify_count: u64,
}

#[allow(dead_code)]
impl ObserverList {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
            next_id: 1,
            notify_count: 0,
        }
    }
    pub fn subscribe(&mut self, label: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.observers.push((id, label.to_string()));
        id
    }
    pub fn unsubscribe(&mut self, id: u64) -> bool {
        if let Some(pos) = self.observers.iter().position(|(i, _)| *i == id) {
            self.observers.remove(pos);
            true
        } else {
            false
        }
    }
    pub fn notify(&mut self, _event: &str) {
        self.notify_count += 1;
    }
    pub fn count(&self) -> usize {
        self.observers.len()
    }
    pub fn is_empty(&self) -> bool {
        self.observers.is_empty()
    }
    pub fn notify_count(&self) -> u64 {
        self.notify_count
    }
    pub fn labels(&self) -> Vec<&str> {
        self.observers.iter().map(|(_, l)| l.as_str()).collect()
    }
    pub fn has_label(&self, label: &str) -> bool {
        self.observers.iter().any(|(_, l)| l.as_str() == label)
    }
    pub fn clear(&mut self) {
        self.observers.clear();
    }
    pub fn ids(&self) -> Vec<u64> {
        self.observers.iter().map(|(id, _)| *id).collect()
    }
}

impl Default for ObserverList {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub fn new_observer_list() -> ObserverList {
    ObserverList::new()
}
#[allow(dead_code)]
pub fn ol_subscribe(o: &mut ObserverList, label: &str) -> u64 {
    o.subscribe(label)
}
#[allow(dead_code)]
pub fn ol_unsubscribe(o: &mut ObserverList, id: u64) -> bool {
    o.unsubscribe(id)
}
#[allow(dead_code)]
pub fn ol_notify(o: &mut ObserverList, event: &str) {
    o.notify(event);
}
#[allow(dead_code)]
pub fn ol_count(o: &ObserverList) -> usize {
    o.count()
}
#[allow(dead_code)]
pub fn ol_is_empty(o: &ObserverList) -> bool {
    o.is_empty()
}
#[allow(dead_code)]
pub fn ol_notify_count(o: &ObserverList) -> u64 {
    o.notify_count()
}
#[allow(dead_code)]
pub fn ol_clear(o: &mut ObserverList) {
    o.clear();
}
#[allow(dead_code)]
pub fn ol_has_label(o: &ObserverList, label: &str) -> bool {
    o.has_label(label)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_subscribe_count() {
        let mut o = new_observer_list();
        ol_subscribe(&mut o, "a");
        ol_subscribe(&mut o, "b");
        assert_eq!(ol_count(&o), 2);
    }
    #[test]
    fn test_unsubscribe() {
        let mut o = new_observer_list();
        let id = ol_subscribe(&mut o, "x");
        assert!(ol_unsubscribe(&mut o, id));
        assert_eq!(ol_count(&o), 0);
    }
    #[test]
    fn test_unsubscribe_missing() {
        let mut o = new_observer_list();
        assert!(!ol_unsubscribe(&mut o, 999));
    }
    #[test]
    fn test_notify_count() {
        let mut o = new_observer_list();
        ol_subscribe(&mut o, "a");
        ol_notify(&mut o, "ev");
        ol_notify(&mut o, "ev2");
        assert_eq!(ol_notify_count(&o), 2);
    }
    #[test]
    fn test_is_empty() {
        let o = new_observer_list();
        assert!(ol_is_empty(&o));
    }
    #[test]
    fn test_has_label() {
        let mut o = new_observer_list();
        ol_subscribe(&mut o, "listener");
        assert!(ol_has_label(&o, "listener"));
        assert!(!ol_has_label(&o, "other"));
    }
    #[test]
    fn test_clear() {
        let mut o = new_observer_list();
        ol_subscribe(&mut o, "a");
        ol_clear(&mut o);
        assert!(ol_is_empty(&o));
    }
    #[test]
    fn test_labels() {
        let mut o = new_observer_list();
        ol_subscribe(&mut o, "foo");
        let labels = o.labels();
        assert!(labels.contains(&"foo"));
    }
    #[test]
    fn test_ids_unique() {
        let mut o = new_observer_list();
        let id1 = ol_subscribe(&mut o, "a");
        let id2 = ol_subscribe(&mut o, "b");
        assert_ne!(id1, id2);
    }
    #[test]
    fn test_multiple_unsub() {
        let mut o = new_observer_list();
        let id = ol_subscribe(&mut o, "a");
        ol_subscribe(&mut o, "b");
        ol_unsubscribe(&mut o, id);
        assert_eq!(ol_count(&o), 1);
    }
}
