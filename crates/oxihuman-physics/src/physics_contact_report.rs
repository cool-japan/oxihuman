// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Contact event reporting: tracks begin/end/persist contact events per simulation step.

/// Kind of contact event.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContactEventKind {
    Begin,
    Persist,
    End,
}

/// A contact event between two bodies.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContactEvent {
    pub kind: ContactEventKind,
    pub body_a: u32,
    pub body_b: u32,
    pub normal: [f32; 3],
    pub depth: f32,
    pub impulse: f32,
    pub step: u64,
}

/// Tracks contact events across simulation steps.
#[allow(dead_code)]
#[derive(Debug)]
pub struct ContactReport {
    events: Vec<ContactEvent>,
    active_pairs: std::collections::HashSet<(u32, u32)>,
    step: u64,
}

#[allow(dead_code)]
fn pair_key(a: u32, b: u32) -> (u32, u32) {
    (a.min(b), a.max(b))
}

#[allow(dead_code)]
impl ContactReport {
    pub fn new() -> Self {
        Self { events: Vec::new(), active_pairs: std::collections::HashSet::new(), step: 0 }
    }

    /// Begin a new simulation step. Provide the set of currently touching pairs.
    pub fn update_step(&mut self, current_pairs: &[(u32, u32, [f32; 3], f32, f32)]) {
        self.step += 1;
        let mut new_active = std::collections::HashSet::new();

        for &(a, b, normal, depth, impulse) in current_pairs {
            let key = pair_key(a, b);
            new_active.insert(key);
            let kind = if self.active_pairs.contains(&key) {
                ContactEventKind::Persist
            } else {
                ContactEventKind::Begin
            };
            self.events.push(ContactEvent {
                kind, body_a: key.0, body_b: key.1, normal, depth, impulse, step: self.step,
            });
        }

        for &key in &self.active_pairs {
            if !new_active.contains(&key) {
                self.events.push(ContactEvent {
                    kind: ContactEventKind::End,
                    body_a: key.0, body_b: key.1,
                    normal: [0.0; 3], depth: 0.0, impulse: 0.0,
                    step: self.step,
                });
            }
        }

        self.active_pairs = new_active;
    }

    pub fn events(&self) -> &[ContactEvent] {
        &self.events
    }

    pub fn events_at_step(&self, step: u64) -> Vec<&ContactEvent> {
        self.events.iter().filter(|e| e.step == step).collect()
    }

    pub fn begin_events(&self) -> Vec<&ContactEvent> {
        self.events.iter().filter(|e| e.kind == ContactEventKind::Begin).collect()
    }

    pub fn end_events(&self) -> Vec<&ContactEvent> {
        self.events.iter().filter(|e| e.kind == ContactEventKind::End).collect()
    }

    pub fn active_pair_count(&self) -> usize {
        self.active_pairs.len()
    }

    pub fn total_event_count(&self) -> usize {
        self.events.len()
    }

    pub fn current_step(&self) -> u64 {
        self.step
    }

    pub fn clear_history(&mut self) {
        self.events.clear();
    }
}

impl Default for ContactReport {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_begin_event() {
        let mut r = ContactReport::new();
        r.update_step(&[(0, 1, [0.0,1.0,0.0], 0.1, 5.0)]);
        assert_eq!(r.begin_events().len(), 1);
    }

    #[test]
    fn test_persist_event() {
        let mut r = ContactReport::new();
        r.update_step(&[(0, 1, [0.0,1.0,0.0], 0.1, 5.0)]);
        r.update_step(&[(0, 1, [0.0,1.0,0.0], 0.2, 6.0)]);
        let events = r.events_at_step(2);
        assert_eq!(events[0].kind, ContactEventKind::Persist);
    }

    #[test]
    fn test_end_event() {
        let mut r = ContactReport::new();
        r.update_step(&[(0, 1, [0.0,1.0,0.0], 0.1, 5.0)]);
        r.update_step(&[]);
        assert_eq!(r.end_events().len(), 1);
    }

    #[test]
    fn test_active_pairs() {
        let mut r = ContactReport::new();
        r.update_step(&[(0, 1, [0.0;3], 0.1, 1.0), (2, 3, [0.0;3], 0.1, 1.0)]);
        assert_eq!(r.active_pair_count(), 2);
    }

    #[test]
    fn test_step_counter() {
        let mut r = ContactReport::new();
        r.update_step(&[]);
        r.update_step(&[]);
        assert_eq!(r.current_step(), 2);
    }

    #[test]
    fn test_total_events() {
        let mut r = ContactReport::new();
        r.update_step(&[(0, 1, [0.0;3], 0.1, 1.0)]);
        r.update_step(&[]);
        assert_eq!(r.total_event_count(), 2); // begin + end
    }

    #[test]
    fn test_clear_history() {
        let mut r = ContactReport::new();
        r.update_step(&[(0, 1, [0.0;3], 0.1, 1.0)]);
        r.clear_history();
        assert_eq!(r.total_event_count(), 0);
    }

    #[test]
    fn test_pair_ordering() {
        let mut r = ContactReport::new();
        r.update_step(&[(5, 2, [0.0;3], 0.1, 1.0)]);
        let e = &r.events()[0];
        assert_eq!(e.body_a, 2);
        assert_eq!(e.body_b, 5);
    }

    #[test]
    fn test_empty_initial() {
        let r = ContactReport::new();
        assert_eq!(r.active_pair_count(), 0);
        assert_eq!(r.current_step(), 0);
    }

    #[test]
    fn test_multiple_contacts() {
        let mut r = ContactReport::new();
        r.update_step(&[
            (0, 1, [0.0;3], 0.1, 1.0),
            (0, 2, [0.0;3], 0.1, 1.0),
            (1, 2, [0.0;3], 0.1, 1.0),
        ]);
        assert_eq!(r.begin_events().len(), 3);
    }
}
