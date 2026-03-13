// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct ResourceTracker {
    pub allocated: u64,
    pub limit: u64,
    pub peak: u64,
}

impl ResourceTracker {
    pub fn new(limit: u64) -> Self {
        ResourceTracker {
            allocated: 0,
            limit,
            peak: 0,
        }
    }
}

pub fn new_resource_tracker(limit: u64) -> ResourceTracker {
    ResourceTracker::new(limit)
}

pub fn tracker_allocate(t: &mut ResourceTracker, amount: u64) -> bool {
    if t.allocated + amount > t.limit {
        return false;
    }
    t.allocated += amount;
    if t.allocated > t.peak {
        t.peak = t.allocated;
    }
    true
}

pub fn tracker_free(t: &mut ResourceTracker, amount: u64) {
    t.allocated = t.allocated.saturating_sub(amount);
}

pub fn tracker_available(t: &ResourceTracker) -> u64 {
    t.limit.saturating_sub(t.allocated)
}

pub fn tracker_peak(t: &ResourceTracker) -> u64 {
    t.peak
}

pub fn tracker_utilization(t: &ResourceTracker) -> f32 {
    if t.limit == 0 {
        return 0.0;
    }
    t.allocated as f32 / t.limit as f32
}

pub fn tracker_is_full(t: &ResourceTracker) -> bool {
    t.allocated >= t.limit
}

pub fn tracker_reset(t: &mut ResourceTracker) {
    t.allocated = 0;
    t.peak = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* new tracker starts empty */
        let t = new_resource_tracker(100);
        assert_eq!(tracker_available(&t), 100);
        assert_eq!(t.allocated, 0);
    }

    #[test]
    fn test_allocate_success() {
        /* allocate within limit succeeds */
        let mut t = new_resource_tracker(100);
        assert!(tracker_allocate(&mut t, 50));
        assert_eq!(t.allocated, 50);
    }

    #[test]
    fn test_allocate_over_limit() {
        /* allocate beyond limit fails */
        let mut t = new_resource_tracker(100);
        assert!(!tracker_allocate(&mut t, 101));
        assert_eq!(t.allocated, 0);
    }

    #[test]
    fn test_free() {
        /* free reduces allocation */
        let mut t = new_resource_tracker(100);
        tracker_allocate(&mut t, 60);
        tracker_free(&mut t, 20);
        assert_eq!(t.allocated, 40);
    }

    #[test]
    fn test_peak() {
        /* peak tracks highest allocation */
        let mut t = new_resource_tracker(200);
        tracker_allocate(&mut t, 100);
        tracker_free(&mut t, 50);
        assert_eq!(tracker_peak(&t), 100);
    }

    #[test]
    fn test_utilization() {
        /* utilization is ratio of allocated to limit */
        let mut t = new_resource_tracker(200);
        tracker_allocate(&mut t, 100);
        assert!((tracker_utilization(&t) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_is_full() {
        /* is_full when at limit */
        let mut t = new_resource_tracker(50);
        tracker_allocate(&mut t, 50);
        assert!(tracker_is_full(&t));
    }

    #[test]
    fn test_reset() {
        /* reset clears allocated and peak */
        let mut t = new_resource_tracker(100);
        tracker_allocate(&mut t, 80);
        tracker_reset(&mut t);
        assert_eq!(t.allocated, 0);
        assert_eq!(t.peak, 0);
    }
}
