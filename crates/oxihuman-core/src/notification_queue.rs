// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Notification queue with priority.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Priority level for notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotifPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// A notification item.
#[derive(Debug, Clone)]
pub struct Notification {
    pub id: u64,
    pub priority: NotifPriority,
    pub title: String,
    pub body: String,
}

#[derive(Debug)]
struct PrioritizedNotif {
    notif: Notification,
}

impl PartialEq for PrioritizedNotif {
    fn eq(&self, other: &Self) -> bool {
        self.notif.priority as u8 == other.notif.priority as u8 && self.notif.id == other.notif.id
    }
}
impl Eq for PrioritizedNotif {}

impl PartialOrd for PrioritizedNotif {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedNotif {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.notif.priority as u8)
            .cmp(&(other.notif.priority as u8))
            .then(other.notif.id.cmp(&self.notif.id))
    }
}

/// Priority-ordered notification queue.
#[derive(Debug, Default)]
pub struct NotificationQueue {
    heap: BinaryHeap<PrioritizedNotif>,
    next_id: u64,
}

impl NotificationQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, priority: NotifPriority, title: &str, body: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.heap.push(PrioritizedNotif {
            notif: Notification {
                id,
                priority,
                title: title.to_string(),
                body: body.to_string(),
            },
        });
        id
    }

    pub fn pop(&mut self) -> Option<Notification> {
        self.heap.pop().map(|p| p.notif)
    }

    pub fn peek_priority(&self) -> Option<NotifPriority> {
        self.heap.peek().map(|p| p.notif.priority)
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}

pub fn new_notification_queue() -> NotificationQueue {
    NotificationQueue::new()
}

pub fn nq_push(q: &mut NotificationQueue, priority: NotifPriority, title: &str, body: &str) -> u64 {
    q.push(priority, title, body)
}

pub fn nq_pop(q: &mut NotificationQueue) -> Option<Notification> {
    q.pop()
}

pub fn nq_len(q: &NotificationQueue) -> usize {
    q.len()
}

pub fn nq_peek_priority(q: &NotificationQueue) -> Option<NotifPriority> {
    q.peek_priority()
}

pub fn nq_is_empty(q: &NotificationQueue) -> bool {
    q.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_len() {
        let mut q = new_notification_queue();
        nq_push(&mut q, NotifPriority::Normal, "Hello", "World");
        assert_eq!(nq_len(&q), 1);
    }

    #[test]
    fn test_pop_returns_item() {
        let mut q = new_notification_queue();
        nq_push(&mut q, NotifPriority::Normal, "T", "B");
        let n = nq_pop(&mut q).expect("should succeed");
        assert_eq!(n.title, "T");
    }

    #[test]
    fn test_priority_ordering() {
        /* critical should come out first */
        let mut q = new_notification_queue();
        nq_push(&mut q, NotifPriority::Low, "low", "");
        nq_push(&mut q, NotifPriority::Critical, "crit", "");
        nq_push(&mut q, NotifPriority::Normal, "norm", "");
        let first = nq_pop(&mut q).expect("should succeed");
        assert_eq!(first.priority, NotifPriority::Critical);
    }

    #[test]
    fn test_peek_priority() {
        let mut q = new_notification_queue();
        nq_push(&mut q, NotifPriority::High, "h", "");
        assert_eq!(nq_peek_priority(&q), Some(NotifPriority::High));
    }

    #[test]
    fn test_is_empty_initially() {
        let q = new_notification_queue();
        assert!(nq_is_empty(&q));
    }

    #[test]
    fn test_pop_empty_returns_none() {
        let mut q = new_notification_queue();
        assert!(nq_pop(&mut q).is_none());
    }

    #[test]
    fn test_id_increments() {
        let mut q = new_notification_queue();
        let id0 = nq_push(&mut q, NotifPriority::Low, "a", "");
        let id1 = nq_push(&mut q, NotifPriority::Low, "b", "");
        assert_ne!(id0, id1);
    }

    #[test]
    fn test_multiple_pops_drain_queue() {
        let mut q = new_notification_queue();
        nq_push(&mut q, NotifPriority::Low, "a", "");
        nq_push(&mut q, NotifPriority::Low, "b", "");
        nq_pop(&mut q);
        nq_pop(&mut q);
        assert!(nq_is_empty(&q));
    }

    #[test]
    fn test_high_before_normal() {
        let mut q = new_notification_queue();
        nq_push(&mut q, NotifPriority::Normal, "n", "");
        nq_push(&mut q, NotifPriority::High, "h", "");
        let first = nq_pop(&mut q).expect("should succeed");
        assert_eq!(first.priority, NotifPriority::High);
    }
}
