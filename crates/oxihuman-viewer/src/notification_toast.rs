// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Notification toast messages.

#![allow(dead_code)]

/// Level constants for toasts.
pub const LEVEL_INFO: u8 = 0;
pub const LEVEL_WARNING: u8 = 1;
pub const LEVEL_ERROR: u8 = 2;

/// A single notification toast.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Toast {
    /// Toast message text.
    pub message: String,
    /// Severity level (0 = info, 1 = warning, 2 = error).
    pub level: u8,
    /// Total duration in ticks.
    pub duration_ticks: u32,
    /// Remaining ticks until the toast expires.
    pub remaining: u32,
}

/// A queue of pending and active toasts.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ToastQueue {
    /// All toasts (active while `remaining > 0`).
    pub toasts: Vec<Toast>,
}

/// Create an empty toast queue.
#[allow(dead_code)]
pub fn new_toast_queue() -> ToastQueue {
    ToastQueue { toasts: Vec::new() }
}

fn push_toast(q: &mut ToastQueue, msg: &str, level: u8, duration: u32) {
    q.toasts.push(Toast {
        message: msg.to_string(),
        level,
        duration_ticks: duration,
        remaining: duration,
    });
}

/// Push an informational toast.
#[allow(dead_code)]
pub fn push_info(q: &mut ToastQueue, msg: &str) {
    push_toast(q, msg, LEVEL_INFO, 120);
}

/// Push a warning toast.
#[allow(dead_code)]
pub fn push_warning(q: &mut ToastQueue, msg: &str) {
    push_toast(q, msg, LEVEL_WARNING, 180);
}

/// Push an error toast.
#[allow(dead_code)]
pub fn push_error(q: &mut ToastQueue, msg: &str) {
    push_toast(q, msg, LEVEL_ERROR, 240);
}

/// Advance all toast timers by one tick, removing expired ones.
#[allow(dead_code)]
pub fn tick_toasts(q: &mut ToastQueue) {
    for t in q.toasts.iter_mut() {
        t.remaining = t.remaining.saturating_sub(1);
    }
    q.toasts.retain(|t| t.remaining > 0);
}

/// Return references to all currently active toasts.
#[allow(dead_code)]
pub fn active_toasts(q: &ToastQueue) -> Vec<&Toast> {
    q.toasts.iter().filter(|t| t.remaining > 0).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_queue_empty() {
        let q = new_toast_queue();
        assert!(q.toasts.is_empty());
    }

    #[test]
    fn test_push_info() {
        let mut q = new_toast_queue();
        push_info(&mut q, "Saved");
        assert_eq!(q.toasts.len(), 1);
        assert_eq!(q.toasts[0].level, LEVEL_INFO);
        assert_eq!(q.toasts[0].message, "Saved");
    }

    #[test]
    fn test_push_warning() {
        let mut q = new_toast_queue();
        push_warning(&mut q, "Low memory");
        assert_eq!(q.toasts[0].level, LEVEL_WARNING);
    }

    #[test]
    fn test_push_error() {
        let mut q = new_toast_queue();
        push_error(&mut q, "Failed");
        assert_eq!(q.toasts[0].level, LEVEL_ERROR);
    }

    #[test]
    fn test_tick_reduces_remaining() {
        let mut q = new_toast_queue();
        push_info(&mut q, "Hi");
        let initial = q.toasts[0].remaining;
        tick_toasts(&mut q);
        assert_eq!(q.toasts[0].remaining, initial - 1);
    }

    #[test]
    fn test_tick_removes_expired() {
        let mut q = new_toast_queue();
        q.toasts.push(Toast {
            message: "Expire".to_string(),
            level: LEVEL_INFO,
            duration_ticks: 1,
            remaining: 1,
        });
        tick_toasts(&mut q);
        assert!(q.toasts.is_empty());
    }

    #[test]
    fn test_active_toasts() {
        let mut q = new_toast_queue();
        push_info(&mut q, "A");
        push_info(&mut q, "B");
        let active = active_toasts(&q);
        assert_eq!(active.len(), 2);
    }

    #[test]
    fn test_multiple_levels() {
        let mut q = new_toast_queue();
        push_info(&mut q, "Info");
        push_warning(&mut q, "Warn");
        push_error(&mut q, "Error");
        assert_eq!(q.toasts.len(), 3);
    }

    #[test]
    fn test_error_longer_than_info() {
        let mut q = new_toast_queue();
        push_info(&mut q, "i");
        push_error(&mut q, "e");
        assert!(q.toasts[1].duration_ticks > q.toasts[0].duration_ticks);
    }
}
