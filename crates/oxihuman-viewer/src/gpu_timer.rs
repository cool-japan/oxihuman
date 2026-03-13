// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// CPU-side stub for a GPU timer query.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GpuTimer {
    pub label: String,
    pub start_ns: u64,
    pub end_ns: u64,
    pub active: bool,
}

/// A set of GPU timers.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct GpuTimerSet {
    pub timers: Vec<GpuTimer>,
}

/// Create a new empty GpuTimerSet.
#[allow(dead_code)]
pub fn new_gpu_timer_set() -> GpuTimerSet {
    GpuTimerSet { timers: Vec::new() }
}

/// Begin a named timer, returning its index.
#[allow(dead_code)]
pub fn begin_timer(ts: &mut GpuTimerSet, label: &str) -> usize {
    let idx = ts.timers.len();
    ts.timers.push(GpuTimer {
        label: label.to_string(),
        start_ns: 0,
        end_ns: 0,
        active: true,
    });
    idx
}

/// End a timer by recording the end timestamp.
#[allow(dead_code)]
pub fn end_timer(ts: &mut GpuTimerSet, idx: usize, end_ns: u64) {
    if let Some(t) = ts.timers.get_mut(idx) {
        t.end_ns = end_ns;
        t.active = false;
    }
}

/// Return duration in milliseconds for a timer.
#[allow(dead_code)]
pub fn timer_duration_ms(ts: &GpuTimerSet, idx: usize) -> f64 {
    ts.timers
        .get(idx)
        .map(|t| (t.end_ns.saturating_sub(t.start_ns)) as f64 / 1_000_000.0)
        .unwrap_or(0.0)
}

/// Return the sum of all timer durations in milliseconds.
#[allow(dead_code)]
pub fn total_gpu_time_ms(ts: &GpuTimerSet) -> f64 {
    ts.timers
        .iter()
        .map(|t| (t.end_ns.saturating_sub(t.start_ns)) as f64 / 1_000_000.0)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_timer_set_empty() {
        let ts = new_gpu_timer_set();
        assert!(ts.timers.is_empty());
    }

    #[test]
    fn begin_timer_returns_index() {
        let mut ts = new_gpu_timer_set();
        let idx = begin_timer(&mut ts, "shadow");
        assert_eq!(idx, 0);
    }

    #[test]
    fn begin_timer_active() {
        let mut ts = new_gpu_timer_set();
        let idx = begin_timer(&mut ts, "test");
        assert!(ts.timers[idx].active);
    }

    #[test]
    fn end_timer_sets_end_ns() {
        let mut ts = new_gpu_timer_set();
        let idx = begin_timer(&mut ts, "pass");
        end_timer(&mut ts, idx, 5_000_000);
        assert_eq!(ts.timers[idx].end_ns, 5_000_000);
    }

    #[test]
    fn end_timer_deactivates() {
        let mut ts = new_gpu_timer_set();
        let idx = begin_timer(&mut ts, "pass");
        end_timer(&mut ts, idx, 1_000_000);
        assert!(!ts.timers[idx].active);
    }

    #[test]
    fn timer_duration_ms_correct() {
        let mut ts = new_gpu_timer_set();
        let idx = begin_timer(&mut ts, "p");
        end_timer(&mut ts, idx, 2_000_000); // 2 ms
        assert!((timer_duration_ms(&ts, idx) - 2.0).abs() < 1e-9);
    }

    #[test]
    fn timer_duration_invalid_index() {
        let ts = new_gpu_timer_set();
        assert!((timer_duration_ms(&ts, 99)).abs() < 1e-9);
    }

    #[test]
    fn total_gpu_time_ms_sum() {
        let mut ts = new_gpu_timer_set();
        let i0 = begin_timer(&mut ts, "a");
        end_timer(&mut ts, i0, 1_000_000); // 1 ms
        let i1 = begin_timer(&mut ts, "b");
        end_timer(&mut ts, i1, 3_000_000); // 3 ms
        assert!((total_gpu_time_ms(&ts) - 4.0).abs() < 1e-9);
    }

    #[test]
    fn label_stored() {
        let mut ts = new_gpu_timer_set();
        let idx = begin_timer(&mut ts, "gbuffer");
        assert_eq!(ts.timers[idx].label, "gbuffer");
    }

    #[test]
    fn multiple_timers_sequential_indices() {
        let mut ts = new_gpu_timer_set();
        let i0 = begin_timer(&mut ts, "a");
        let i1 = begin_timer(&mut ts, "b");
        assert_eq!(i0, 0);
        assert_eq!(i1, 1);
    }
}
