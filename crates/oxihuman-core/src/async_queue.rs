// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A simple async-style task queue that stores pending and completed tasks.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AsyncQueue {
    pending: Vec<AsyncTask>,
    completed: Vec<AsyncTask>,
    max_concurrent: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum TaskState {
    Pending,
    Running,
    Completed,
    Failed,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AsyncTask {
    pub id: u64,
    pub label: String,
    pub state: TaskState,
    pub progress: f32,
}

#[allow(dead_code)]
impl AsyncQueue {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            pending: Vec::new(),
            completed: Vec::new(),
            max_concurrent: max_concurrent.max(1),
        }
    }

    pub fn enqueue(&mut self, id: u64, label: &str) -> bool {
        if self.pending.iter().any(|t| t.id == id) || self.completed.iter().any(|t| t.id == id) {
            return false;
        }
        self.pending.push(AsyncTask {
            id,
            label: label.to_string(),
            state: TaskState::Pending,
            progress: 0.0,
        });
        true
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn completed_count(&self) -> usize {
        self.completed.len()
    }

    pub fn total_count(&self) -> usize {
        self.pending.len() + self.completed.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty() && self.completed.is_empty()
    }

    pub fn running_count(&self) -> usize {
        self.pending
            .iter()
            .filter(|t| t.state == TaskState::Running)
            .count()
    }

    pub fn can_start_more(&self) -> bool {
        self.running_count() < self.max_concurrent
    }

    pub fn start_next(&mut self) -> Option<u64> {
        if !self.can_start_more() {
            return None;
        }
        if let Some(task) = self
            .pending
            .iter_mut()
            .find(|t| t.state == TaskState::Pending)
        {
            task.state = TaskState::Running;
            Some(task.id)
        } else {
            None
        }
    }

    pub fn complete_task(&mut self, id: u64) -> bool {
        if let Some(pos) = self.pending.iter().position(|t| t.id == id) {
            let mut task = self.pending.remove(pos);
            task.state = TaskState::Completed;
            task.progress = 1.0;
            self.completed.push(task);
            true
        } else {
            false
        }
    }

    pub fn fail_task(&mut self, id: u64) -> bool {
        if let Some(pos) = self.pending.iter().position(|t| t.id == id) {
            let mut task = self.pending.remove(pos);
            task.state = TaskState::Failed;
            self.completed.push(task);
            true
        } else {
            false
        }
    }

    pub fn set_progress(&mut self, id: u64, progress: f32) -> bool {
        if let Some(task) = self.pending.iter_mut().find(|t| t.id == id) {
            task.progress = progress.clamp(0.0, 1.0);
            true
        } else {
            false
        }
    }

    pub fn clear_completed(&mut self) {
        self.completed.clear();
    }

    pub fn get_task(&self, id: u64) -> Option<&AsyncTask> {
        self.pending
            .iter()
            .chain(self.completed.iter())
            .find(|t| t.id == id)
    }

    pub fn failed_count(&self) -> usize {
        self.completed
            .iter()
            .filter(|t| t.state == TaskState::Failed)
            .count()
    }

    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let q = AsyncQueue::new(4);
        assert!(q.is_empty());
        assert_eq!(q.max_concurrent(), 4);
    }

    #[test]
    fn test_enqueue() {
        let mut q = AsyncQueue::new(2);
        assert!(q.enqueue(1, "task1"));
        assert_eq!(q.pending_count(), 1);
        assert!(!q.enqueue(1, "dup"));
    }

    #[test]
    fn test_start_next() {
        let mut q = AsyncQueue::new(1);
        q.enqueue(1, "a");
        q.enqueue(2, "b");
        assert_eq!(q.start_next(), Some(1));
        assert!(q.start_next().is_none());
    }

    #[test]
    fn test_complete() {
        let mut q = AsyncQueue::new(2);
        q.enqueue(1, "a");
        q.start_next();
        assert!(q.complete_task(1));
        assert_eq!(q.completed_count(), 1);
        assert_eq!(q.pending_count(), 0);
    }

    #[test]
    fn test_fail() {
        let mut q = AsyncQueue::new(2);
        q.enqueue(1, "a");
        q.start_next();
        assert!(q.fail_task(1));
        assert_eq!(q.failed_count(), 1);
    }

    #[test]
    fn test_progress() {
        let mut q = AsyncQueue::new(2);
        q.enqueue(1, "a");
        q.start_next();
        assert!(q.set_progress(1, 0.5));
        let t = q.get_task(1).unwrap();
        assert!((t.progress - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_clear_completed() {
        let mut q = AsyncQueue::new(2);
        q.enqueue(1, "a");
        q.start_next();
        q.complete_task(1);
        q.clear_completed();
        assert_eq!(q.completed_count(), 0);
    }

    #[test]
    fn test_total_count() {
        let mut q = AsyncQueue::new(2);
        q.enqueue(1, "a");
        q.enqueue(2, "b");
        q.start_next();
        q.complete_task(1);
        assert_eq!(q.total_count(), 2);
    }

    #[test]
    fn test_running_count() {
        let mut q = AsyncQueue::new(3);
        q.enqueue(1, "a");
        q.enqueue(2, "b");
        q.start_next();
        q.start_next();
        assert_eq!(q.running_count(), 2);
    }

    #[test]
    fn test_can_start_more() {
        let mut q = AsyncQueue::new(1);
        assert!(q.can_start_more());
        q.enqueue(1, "a");
        q.start_next();
        assert!(!q.can_start_more());
    }
}
