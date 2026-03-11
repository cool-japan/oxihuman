// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Time-ordered schedule queue for deferred task execution.

/// A scheduled task entry.
#[derive(Debug, Clone)]
pub struct ScheduleTask {
    pub id: u64,
    pub name: String,
    pub due_at: u64,
    pub repeat_interval: Option<u64>,
    pub enabled: bool,
}

/// Queue of tasks ordered by due time.
pub struct ScheduleQueue {
    tasks: Vec<ScheduleTask>,
    now: u64,
    next_id: u64,
    fired_count: u64,
}

#[allow(dead_code)]
impl ScheduleQueue {
    pub fn new() -> Self {
        ScheduleQueue {
            tasks: Vec::new(),
            now: 0,
            next_id: 0,
            fired_count: 0,
        }
    }

    pub fn schedule_once(&mut self, name: &str, due_at: u64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let pos = self.tasks.partition_point(|t| t.due_at <= due_at);
        self.tasks.insert(
            pos,
            ScheduleTask {
                id,
                name: name.to_string(),
                due_at,
                repeat_interval: None,
                enabled: true,
            },
        );
        id
    }

    pub fn schedule_repeating(&mut self, name: &str, due_at: u64, interval: u64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let pos = self.tasks.partition_point(|t| t.due_at <= due_at);
        self.tasks.insert(
            pos,
            ScheduleTask {
                id,
                name: name.to_string(),
                due_at,
                repeat_interval: Some(interval),
                enabled: true,
            },
        );
        id
    }

    pub fn advance(&mut self, dt: u64) -> Vec<ScheduleTask> {
        self.now += dt;
        let mut fired: Vec<ScheduleTask> = Vec::new();
        let mut to_reschedule: Vec<(String, u64, u64)> = Vec::new();

        self.tasks.retain(|t| {
            if t.enabled && t.due_at <= self.now {
                fired.push(t.clone());
                if let Some(interval) = t.repeat_interval {
                    to_reschedule.push((t.name.clone(), self.now + interval, interval));
                }
                false
            } else {
                true
            }
        });

        self.fired_count += fired.len() as u64;

        for (name, due, interval) in to_reschedule {
            self.schedule_repeating(&name, due, interval);
        }

        fired
    }

    pub fn cancel(&mut self, id: u64) -> bool {
        let before = self.tasks.len();
        self.tasks.retain(|t| t.id != id);
        self.tasks.len() < before
    }

    pub fn set_enabled(&mut self, id: u64, enabled: bool) -> bool {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == id) {
            t.enabled = enabled;
            true
        } else {
            false
        }
    }

    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn now(&self) -> u64 {
        self.now
    }

    pub fn fired_count(&self) -> u64 {
        self.fired_count
    }

    pub fn next_due(&self) -> Option<u64> {
        self.tasks
            .iter()
            .filter(|t| t.enabled)
            .map(|t| t.due_at)
            .min()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn clear(&mut self) {
        self.tasks.clear();
    }

    pub fn has_task(&self, id: u64) -> bool {
        self.tasks.iter().any(|t| t.id == id)
    }
}

impl Default for ScheduleQueue {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_schedule_queue() -> ScheduleQueue {
    ScheduleQueue::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedule_and_fire() {
        let mut q = new_schedule_queue();
        q.schedule_once("task1", 10);
        let fired = q.advance(10);
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].name, "task1");
    }

    #[test]
    fn not_due_yet() {
        let mut q = new_schedule_queue();
        q.schedule_once("t", 100);
        let fired = q.advance(50);
        assert!(fired.is_empty());
    }

    #[test]
    fn repeating_reschedules() {
        let mut q = new_schedule_queue();
        q.schedule_repeating("tick", 5, 5);
        q.advance(5);
        assert_eq!(q.task_count(), 1);
        q.advance(5);
        assert_eq!(q.fired_count(), 2);
    }

    #[test]
    fn cancel_task() {
        let mut q = new_schedule_queue();
        let id = q.schedule_once("x", 10);
        assert!(q.cancel(id));
        assert!(!q.has_task(id));
    }

    #[test]
    fn disabled_not_fired() {
        let mut q = new_schedule_queue();
        let id = q.schedule_once("t", 5);
        q.set_enabled(id, false);
        let fired = q.advance(10);
        assert!(fired.is_empty());
    }

    #[test]
    fn next_due_time() {
        let mut q = new_schedule_queue();
        q.schedule_once("a", 50);
        q.schedule_once("b", 20);
        assert_eq!(q.next_due(), Some(20));
    }

    #[test]
    fn fired_count_tracked() {
        let mut q = new_schedule_queue();
        q.schedule_once("a", 1);
        q.schedule_once("b", 2);
        q.advance(5);
        assert_eq!(q.fired_count(), 2);
    }

    #[test]
    fn clear_queue() {
        let mut q = new_schedule_queue();
        q.schedule_once("a", 10);
        q.clear();
        assert!(q.is_empty());
    }

    #[test]
    fn multiple_fire_same_tick() {
        let mut q = new_schedule_queue();
        q.schedule_once("a", 5);
        q.schedule_once("b", 5);
        let fired = q.advance(5);
        assert_eq!(fired.len(), 2);
    }
}
