// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cron-style task scheduler stub.

use std::collections::HashMap;

/// Recurrence rule for scheduled tasks.
#[derive(Debug, Clone)]
pub enum RecurrenceRule {
    Once,
    EveryMs(u64),
}

/// A single scheduled task.
#[derive(Debug, Clone)]
pub struct SchedulerTask {
    pub id: u64,
    pub name: String,
    pub next_run_ms: u64,
    pub rule: RecurrenceRule,
    pub enabled: bool,
    pub run_count: u64,
}

/// Cron-style task scheduler.
#[derive(Debug, Default)]
pub struct TaskScheduler {
    tasks: HashMap<u64, SchedulerTask>,
    next_id: u64,
    current_time_ms: u64,
}

impl TaskScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn schedule(&mut self, name: &str, first_run_ms: u64, rule: RecurrenceRule) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.tasks.insert(
            id,
            SchedulerTask {
                id,
                name: name.to_string(),
                next_run_ms: first_run_ms,
                rule,
                enabled: true,
                run_count: 0,
            },
        );
        id
    }

    pub fn advance(&mut self, time_ms: u64) -> Vec<u64> {
        /* returns IDs of tasks that fired */
        self.current_time_ms = time_ms;
        let mut fired = Vec::new();
        for task in self.tasks.values_mut() {
            if !task.enabled || task.next_run_ms > time_ms {
                continue;
            }
            fired.push(task.id);
            task.run_count += 1;
            match &task.rule {
                RecurrenceRule::Once => {
                    task.enabled = false;
                }
                RecurrenceRule::EveryMs(interval) => {
                    task.next_run_ms = time_ms + interval;
                }
            }
        }
        fired
    }

    pub fn cancel(&mut self, id: u64) -> bool {
        if let Some(t) = self.tasks.get_mut(&id) {
            t.enabled = false;
            true
        } else {
            false
        }
    }

    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn enabled_count(&self) -> usize {
        self.tasks.values().filter(|t| t.enabled).count()
    }

    pub fn run_count(&self, id: u64) -> u64 {
        self.tasks.get(&id).map(|t| t.run_count).unwrap_or(0)
    }
}

pub fn new_task_scheduler() -> TaskScheduler {
    TaskScheduler::new()
}

pub fn ts_schedule(
    sched: &mut TaskScheduler,
    name: &str,
    first_ms: u64,
    rule: RecurrenceRule,
) -> u64 {
    sched.schedule(name, first_ms, rule)
}

pub fn ts_advance(sched: &mut TaskScheduler, time_ms: u64) -> Vec<u64> {
    sched.advance(time_ms)
}

pub fn ts_cancel(sched: &mut TaskScheduler, id: u64) -> bool {
    sched.cancel(id)
}

pub fn ts_task_count(sched: &TaskScheduler) -> usize {
    sched.task_count()
}

pub fn ts_run_count(sched: &TaskScheduler, id: u64) -> u64 {
    sched.run_count(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_once() {
        let mut s = new_task_scheduler();
        let id = ts_schedule(&mut s, "job", 100, RecurrenceRule::Once);
        let fired = ts_advance(&mut s, 200);
        assert!(fired.contains(&id));
    }

    #[test]
    fn test_once_fires_only_once() {
        let mut s = new_task_scheduler();
        let id = ts_schedule(&mut s, "job", 50, RecurrenceRule::Once);
        ts_advance(&mut s, 100);
        let fired2 = ts_advance(&mut s, 200);
        assert!(!fired2.contains(&id));
    }

    #[test]
    fn test_repeating_fires_multiple_times() {
        let mut s = new_task_scheduler();
        let id = ts_schedule(&mut s, "tick", 100, RecurrenceRule::EveryMs(100));
        ts_advance(&mut s, 100);
        ts_advance(&mut s, 200);
        assert_eq!(ts_run_count(&s, id), 2);
    }

    #[test]
    fn test_before_first_run_not_fired() {
        let mut s = new_task_scheduler();
        let id = ts_schedule(&mut s, "future", 1000, RecurrenceRule::Once);
        let fired = ts_advance(&mut s, 500);
        assert!(!fired.contains(&id));
    }

    #[test]
    fn test_cancel_prevents_firing() {
        let mut s = new_task_scheduler();
        let id = ts_schedule(&mut s, "j", 100, RecurrenceRule::Once);
        ts_cancel(&mut s, id);
        let fired = ts_advance(&mut s, 200);
        assert!(!fired.contains(&id));
    }

    #[test]
    fn test_task_count() {
        let mut s = new_task_scheduler();
        ts_schedule(&mut s, "a", 0, RecurrenceRule::Once);
        ts_schedule(&mut s, "b", 0, RecurrenceRule::Once);
        assert_eq!(ts_task_count(&s), 2);
    }

    #[test]
    fn test_enabled_count_decreases_after_once() {
        let mut s = new_task_scheduler();
        ts_schedule(&mut s, "x", 10, RecurrenceRule::Once);
        assert_eq!(s.enabled_count(), 1);
        ts_advance(&mut s, 100);
        assert_eq!(s.enabled_count(), 0);
    }

    #[test]
    fn test_run_count_starts_at_zero() {
        let mut s = new_task_scheduler();
        let id = ts_schedule(&mut s, "z", 1000, RecurrenceRule::Once);
        assert_eq!(ts_run_count(&s, id), 0);
    }

    #[test]
    fn test_unknown_id_run_count_zero() {
        let s = new_task_scheduler();
        assert_eq!(ts_run_count(&s, 999), 0);
    }
}
