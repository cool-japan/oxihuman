// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Event log for collision events with filtering and statistics.

/// A recorded collision event.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionEvent {
    pub frame: u64,
    pub body_a: u32,
    pub body_b: u32,
    pub contact_point: [f32; 3],
    pub impulse: f32,
    pub is_trigger: bool,
}

/// A bounded ring-buffer log of collision events.
#[allow(dead_code)]
pub struct CollisionEventLog {
    events: Vec<CollisionEvent>,
    capacity: usize,
    total_recorded: u64,
    current_frame: u64,
}

#[allow(dead_code)]
impl CollisionEventLog {
    pub fn new(capacity: usize) -> Self {
        Self {
            events: Vec::with_capacity(capacity),
            capacity,
            total_recorded: 0,
            current_frame: 0,
        }
    }

    /// Advance to the next frame.
    pub fn next_frame(&mut self) {
        self.current_frame += 1;
    }

    /// Record a collision event.
    pub fn record(
        &mut self,
        body_a: u32,
        body_b: u32,
        contact_point: [f32; 3],
        impulse: f32,
        is_trigger: bool,
    ) {
        if self.events.len() >= self.capacity {
            self.events.remove(0);
        }
        self.events.push(CollisionEvent {
            frame: self.current_frame,
            body_a,
            body_b,
            contact_point,
            impulse,
            is_trigger,
        });
        self.total_recorded += 1;
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn total_recorded(&self) -> u64 {
        self.total_recorded
    }

    pub fn events_for_body(&self, id: u32) -> Vec<&CollisionEvent> {
        self.events
            .iter()
            .filter(|e| e.body_a == id || e.body_b == id)
            .collect()
    }

    pub fn events_in_frame(&self, frame: u64) -> Vec<&CollisionEvent> {
        self.events.iter().filter(|e| e.frame == frame).collect()
    }

    pub fn max_impulse(&self) -> f32 {
        self.events
            .iter()
            .map(|e| e.impulse)
            .fold(0.0_f32, f32::max)
    }

    pub fn trigger_event_count(&self) -> usize {
        self.events.iter().filter(|e| e.is_trigger).count()
    }

    pub fn physical_event_count(&self) -> usize {
        self.events.iter().filter(|e| !e.is_trigger).count()
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn last_event(&self) -> Option<&CollisionEvent> {
        self.events.last()
    }

    pub fn average_impulse(&self) -> f32 {
        if self.events.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.events.iter().map(|e| e.impulse).sum();
        sum / self.events.len() as f32
    }
}

pub fn new_collision_event_log(capacity: usize) -> CollisionEventLog {
    CollisionEventLog::new(capacity)
}

pub fn cel_record(
    log: &mut CollisionEventLog,
    body_a: u32,
    body_b: u32,
    contact: [f32; 3],
    impulse: f32,
    trigger: bool,
) {
    log.record(body_a, body_b, contact, impulse, trigger);
}

pub fn cel_event_count(log: &CollisionEventLog) -> usize {
    log.event_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_log_empty() {
        let log = new_collision_event_log(64);
        assert_eq!(cel_event_count(&log), 0);
    }

    #[test]
    fn record_adds_event() {
        let mut log = new_collision_event_log(64);
        cel_record(&mut log, 1, 2, [0.0; 3], 5.0, false);
        assert_eq!(cel_event_count(&log), 1);
    }

    #[test]
    fn capacity_ring_buffer() {
        let mut log = new_collision_event_log(2);
        cel_record(&mut log, 1, 2, [0.0; 3], 1.0, false);
        cel_record(&mut log, 2, 3, [0.0; 3], 2.0, false);
        cel_record(&mut log, 3, 4, [0.0; 3], 3.0, false); // evicts first
        assert_eq!(cel_event_count(&log), 2);
        assert_eq!(log.events[0].body_a, 2);
    }

    #[test]
    fn filter_by_body() {
        let mut log = new_collision_event_log(64);
        cel_record(&mut log, 1, 2, [0.0; 3], 1.0, false);
        cel_record(&mut log, 3, 4, [0.0; 3], 2.0, false);
        assert_eq!(log.events_for_body(1).len(), 1);
    }

    #[test]
    fn filter_by_frame() {
        let mut log = new_collision_event_log(64);
        cel_record(&mut log, 1, 2, [0.0; 3], 1.0, false);
        log.next_frame();
        cel_record(&mut log, 3, 4, [0.0; 3], 2.0, false);
        assert_eq!(log.events_in_frame(0).len(), 1);
        assert_eq!(log.events_in_frame(1).len(), 1);
    }

    #[test]
    fn max_impulse() {
        let mut log = new_collision_event_log(64);
        cel_record(&mut log, 1, 2, [0.0; 3], 3.0, false);
        cel_record(&mut log, 1, 3, [0.0; 3], 7.5, false);
        assert!((log.max_impulse() - 7.5).abs() < 1e-5);
    }

    #[test]
    fn trigger_count() {
        let mut log = new_collision_event_log(64);
        cel_record(&mut log, 1, 2, [0.0; 3], 0.0, true);
        cel_record(&mut log, 1, 3, [0.0; 3], 1.0, false);
        assert_eq!(log.trigger_event_count(), 1);
        assert_eq!(log.physical_event_count(), 1);
    }

    #[test]
    fn average_impulse() {
        let mut log = new_collision_event_log(64);
        cel_record(&mut log, 1, 2, [0.0; 3], 2.0, false);
        cel_record(&mut log, 1, 3, [0.0; 3], 4.0, false);
        assert!((log.average_impulse() - 3.0).abs() < 1e-5);
    }

    #[test]
    fn clear_empties_log() {
        let mut log = new_collision_event_log(64);
        cel_record(&mut log, 1, 2, [0.0; 3], 1.0, false);
        log.clear();
        assert_eq!(cel_event_count(&log), 0);
    }

    #[test]
    fn total_recorded_tracks_all() {
        let mut log = new_collision_event_log(2);
        cel_record(&mut log, 1, 2, [0.0; 3], 1.0, false);
        cel_record(&mut log, 2, 3, [0.0; 3], 1.0, false);
        cel_record(&mut log, 3, 4, [0.0; 3], 1.0, false);
        assert_eq!(log.total_recorded(), 3);
    }
}
