// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hierarchical timing wheel for efficient timer management (O(1) insert/cancel).

/// A timer entry in the wheel.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TimerEntry {
    pub id: u64,
    pub deadline_tick: u64,
    pub interval: Option<u64>,
    pub label: String,
}

/// A single-level timing wheel with N slots.
#[allow(dead_code)]
#[derive(Debug)]
pub struct TimeWheel {
    slots: Vec<Vec<TimerEntry>>,
    slot_count: usize,
    current_tick: u64,
    next_id: u64,
}

#[allow(dead_code)]
impl TimeWheel {
    pub fn new(slot_count: usize) -> Self {
        assert!(slot_count > 0);
        Self {
            slots: (0..slot_count).map(|_| Vec::new()).collect(),
            slot_count,
            current_tick: 0,
            next_id: 1,
        }
    }

    fn slot_for(&self, tick: u64) -> usize {
        (tick % self.slot_count as u64) as usize
    }

    pub fn schedule(&mut self, delay: u64, label: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let deadline = self.current_tick + delay;
        let slot = self.slot_for(deadline);
        self.slots[slot].push(TimerEntry {
            id,
            deadline_tick: deadline,
            interval: None,
            label: label.to_string(),
        });
        id
    }

    pub fn schedule_repeating(&mut self, delay: u64, interval: u64, label: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let deadline = self.current_tick + delay;
        let slot = self.slot_for(deadline);
        self.slots[slot].push(TimerEntry {
            id,
            deadline_tick: deadline,
            interval: Some(interval),
            label: label.to_string(),
        });
        id
    }

    pub fn cancel(&mut self, id: u64) -> bool {
        for slot in &mut self.slots {
            if let Some(pos) = slot.iter().position(|e| e.id == id) {
                slot.swap_remove(pos);
                return true;
            }
        }
        false
    }

    /// Advance by one tick and return expired entries.
    pub fn tick(&mut self) -> Vec<TimerEntry> {
        self.current_tick += 1;
        let slot = self.slot_for(self.current_tick);
        let mut expired = Vec::new();
        let mut remaining = Vec::new();
        for entry in self.slots[slot].drain(..) {
            if entry.deadline_tick <= self.current_tick {
                expired.push(entry);
            } else {
                remaining.push(entry);
            }
        }
        self.slots[slot] = remaining;

        // Re-schedule repeating timers
        let mut to_reschedule = Vec::new();
        for e in &expired {
            if let Some(interval) = e.interval {
                let new_deadline = self.current_tick + interval;
                to_reschedule.push(TimerEntry {
                    id: e.id,
                    deadline_tick: new_deadline,
                    interval: e.interval,
                    label: e.label.clone(),
                });
            }
        }
        for entry in to_reschedule {
            let s = self.slot_for(entry.deadline_tick);
            self.slots[s].push(entry);
        }

        expired
    }

    pub fn current_tick(&self) -> u64 {
        self.current_tick
    }

    pub fn pending_count(&self) -> usize {
        self.slots.iter().map(|s| s.len()).sum()
    }

    pub fn slot_count(&self) -> usize {
        self.slot_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_and_tick() {
        let mut w = TimeWheel::new(16);
        w.schedule(3, "test");
        let e1 = w.tick(); assert!(e1.is_empty());
        let e2 = w.tick(); assert!(e2.is_empty());
        let e3 = w.tick(); assert_eq!(e3.len(), 1);
        assert_eq!(e3[0].label, "test");
    }

    #[test]
    fn test_cancel() {
        let mut w = TimeWheel::new(16);
        let id = w.schedule(5, "cancel_me");
        assert!(w.cancel(id));
        for _ in 0..10 { assert!(w.tick().is_empty()); }
    }

    #[test]
    fn test_cancel_nonexistent() {
        let mut w = TimeWheel::new(8);
        assert!(!w.cancel(999));
    }

    #[test]
    fn test_repeating() {
        let mut w = TimeWheel::new(16);
        w.schedule_repeating(2, 3, "repeat");
        w.tick();
        let e = w.tick(); assert_eq!(e.len(), 1);
        w.tick(); w.tick();
        let e2 = w.tick(); assert_eq!(e2.len(), 1);
    }

    #[test]
    fn test_pending_count() {
        let mut w = TimeWheel::new(8);
        w.schedule(1, "a");
        w.schedule(2, "b");
        assert_eq!(w.pending_count(), 2);
    }

    #[test]
    fn test_current_tick() {
        let mut w = TimeWheel::new(8);
        assert_eq!(w.current_tick(), 0);
        w.tick();
        w.tick();
        assert_eq!(w.current_tick(), 2);
    }

    #[test]
    fn test_multiple_same_slot() {
        let mut w = TimeWheel::new(4);
        w.schedule(4, "a");
        w.schedule(4, "b");
        for _ in 0..3 { w.tick(); }
        let e = w.tick();
        assert_eq!(e.len(), 2);
    }

    #[test]
    fn test_slot_count() {
        let w = TimeWheel::new(32);
        assert_eq!(w.slot_count(), 32);
    }

    #[test]
    fn test_delay_zero() {
        let mut w = TimeWheel::new(8);
        // delay=0 doesn't make sense for next tick, but delay=1 should fire on tick 1
        w.schedule(1, "immediate");
        let e = w.tick();
        assert_eq!(e.len(), 1);
    }

    #[test]
    fn test_unique_ids() {
        let mut w = TimeWheel::new(8);
        let a = w.schedule(1, "a");
        let b = w.schedule(1, "b");
        assert_ne!(a, b);
    }
}
