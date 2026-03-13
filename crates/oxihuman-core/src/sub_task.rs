// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Sub-task tracking: decompose a parent task into named sub-tasks with progress.

use std::collections::HashMap;

/// Status of a sub-task.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum SubTaskStatus {
    Pending,
    Running,
    Done,
    Failed(String),
    Skipped,
}

/// A single sub-task entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SubTask {
    pub name: String,
    pub status: SubTaskStatus,
    pub progress: f32,
    pub weight: f32,
}

/// A collection of sub-tasks belonging to a parent task.
#[allow(dead_code)]
pub struct SubTaskSet {
    tasks: Vec<SubTask>,
    name_index: HashMap<String, usize>,
}

#[allow(dead_code)]
impl SubTaskSet {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            name_index: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: &str, weight: f32) {
        let idx = self.tasks.len();
        self.tasks.push(SubTask {
            name: name.to_string(),
            status: SubTaskStatus::Pending,
            progress: 0.0,
            weight: weight.max(0.0),
        });
        self.name_index.insert(name.to_string(), idx);
    }

    pub fn set_running(&mut self, name: &str) -> bool {
        if let Some(&idx) = self.name_index.get(name) {
            self.tasks[idx].status = SubTaskStatus::Running;
            true
        } else {
            false
        }
    }

    pub fn set_progress(&mut self, name: &str, progress: f32) -> bool {
        if let Some(&idx) = self.name_index.get(name) {
            self.tasks[idx].progress = progress.clamp(0.0, 1.0);
            true
        } else {
            false
        }
    }

    pub fn set_done(&mut self, name: &str) -> bool {
        if let Some(&idx) = self.name_index.get(name) {
            self.tasks[idx].status = SubTaskStatus::Done;
            self.tasks[idx].progress = 1.0;
            true
        } else {
            false
        }
    }

    pub fn set_failed(&mut self, name: &str, reason: &str) -> bool {
        if let Some(&idx) = self.name_index.get(name) {
            self.tasks[idx].status = SubTaskStatus::Failed(reason.to_string());
            true
        } else {
            false
        }
    }

    pub fn set_skipped(&mut self, name: &str) -> bool {
        if let Some(&idx) = self.name_index.get(name) {
            self.tasks[idx].status = SubTaskStatus::Skipped;
            true
        } else {
            false
        }
    }

    pub fn get(&self, name: &str) -> Option<&SubTask> {
        self.name_index.get(name).map(|&i| &self.tasks[i])
    }

    /// Weighted overall progress (0.0..=1.0).
    pub fn overall_progress(&self) -> f32 {
        let total_weight: f32 = self.tasks.iter().map(|t| t.weight).sum();
        if total_weight <= 0.0 {
            return 0.0;
        }
        let weighted: f32 = self.tasks.iter().map(|t| t.progress * t.weight).sum();
        (weighted / total_weight).clamp(0.0, 1.0)
    }

    pub fn done_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| t.status == SubTaskStatus::Done)
            .count()
    }

    pub fn failed_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| matches!(t.status, SubTaskStatus::Failed(_)))
            .count()
    }

    pub fn pending_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| t.status == SubTaskStatus::Pending)
            .count()
    }

    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn all_done(&self) -> bool {
        !self.tasks.is_empty()
            && self
                .tasks
                .iter()
                .all(|t| matches!(t.status, SubTaskStatus::Done | SubTaskStatus::Skipped))
    }

    pub fn has_failures(&self) -> bool {
        self.tasks
            .iter()
            .any(|t| matches!(t.status, SubTaskStatus::Failed(_)))
    }
}

impl Default for SubTaskSet {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_sub_task_set() -> SubTaskSet {
    SubTaskSet::new()
}

pub fn sts_add(set: &mut SubTaskSet, name: &str, weight: f32) {
    set.add(name, weight);
}

pub fn sts_done(set: &mut SubTaskSet, name: &str) -> bool {
    set.set_done(name)
}

pub fn sts_failed(set: &mut SubTaskSet, name: &str, reason: &str) -> bool {
    set.set_failed(name, reason)
}

pub fn sts_overall(set: &SubTaskSet) -> f32 {
    set.overall_progress()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_set() {
        let s = new_sub_task_set();
        assert_eq!(s.task_count(), 0);
    }

    #[test]
    fn add_and_get() {
        let mut s = new_sub_task_set();
        sts_add(&mut s, "load", 1.0);
        assert!(s.get("load").is_some());
        assert_eq!(
            s.get("load").expect("should succeed").status,
            SubTaskStatus::Pending
        );
    }

    #[test]
    fn set_running() {
        let mut s = new_sub_task_set();
        sts_add(&mut s, "process", 1.0);
        assert!(s.set_running("process"));
        assert_eq!(
            s.get("process").expect("should succeed").status,
            SubTaskStatus::Running
        );
    }

    #[test]
    fn done_count() {
        let mut s = new_sub_task_set();
        sts_add(&mut s, "a", 1.0);
        sts_add(&mut s, "b", 1.0);
        sts_done(&mut s, "a");
        assert_eq!(s.done_count(), 1);
    }

    #[test]
    fn failed_count() {
        let mut s = new_sub_task_set();
        sts_add(&mut s, "task", 1.0);
        sts_failed(&mut s, "task", "timeout");
        assert_eq!(s.failed_count(), 1);
        assert!(s.has_failures());
    }

    #[test]
    fn overall_progress_even_weights() {
        let mut s = new_sub_task_set();
        sts_add(&mut s, "a", 1.0);
        sts_add(&mut s, "b", 1.0);
        sts_done(&mut s, "a");
        let p = sts_overall(&s);
        assert!((p - 0.5).abs() < 1e-6);
    }

    #[test]
    fn all_done_when_all_completed() {
        let mut s = new_sub_task_set();
        sts_add(&mut s, "x", 1.0);
        sts_done(&mut s, "x");
        assert!(s.all_done());
    }

    #[test]
    fn skipped_counts_as_done_for_all_done() {
        let mut s = new_sub_task_set();
        sts_add(&mut s, "opt", 0.5);
        s.set_skipped("opt");
        assert!(s.all_done());
    }

    #[test]
    fn set_progress_clamps() {
        let mut s = new_sub_task_set();
        sts_add(&mut s, "t", 1.0);
        s.set_progress("t", 2.0);
        assert!((s.get("t").expect("should succeed").progress - 1.0).abs() < 1e-6);
    }

    #[test]
    fn missing_task_returns_false() {
        let mut s = new_sub_task_set();
        assert!(!sts_done(&mut s, "ghost"));
    }
}
