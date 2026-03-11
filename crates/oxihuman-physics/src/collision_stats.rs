// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Collision statistics tracking per frame.

#![allow(dead_code)]

/// Accumulated collision statistics across frames.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct CollisionStats {
    pub broad_phase_pairs: u64,
    pub narrow_phase_tests: u64,
    pub contacts_generated: u64,
    pub constraints_solved: u64,
    pub frame_count: u64,
}

/// Create a new zeroed CollisionStats.
#[allow(dead_code)]
pub fn new_collision_stats() -> CollisionStats {
    CollisionStats::default()
}

/// Record one frame's statistics.
#[allow(dead_code)]
pub fn record_frame(
    stats: &mut CollisionStats,
    broad: u32,
    narrow: u32,
    contacts: u32,
    solved: u32,
) {
    stats.broad_phase_pairs += broad as u64;
    stats.narrow_phase_tests += narrow as u64;
    stats.contacts_generated += contacts as u64;
    stats.constraints_solved += solved as u64;
    stats.frame_count += 1;
}

/// Compute average contacts generated per frame.
#[allow(dead_code)]
pub fn avg_contacts_per_frame(stats: &CollisionStats) -> f32 {
    if stats.frame_count == 0 {
        return 0.0;
    }
    stats.contacts_generated as f32 / stats.frame_count as f32
}

/// Reset all statistics to zero.
#[allow(dead_code)]
pub fn reset_stats(stats: &mut CollisionStats) {
    *stats = CollisionStats::default();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stats_zeroed() {
        let s = new_collision_stats();
        assert_eq!(s.frame_count, 0);
        assert_eq!(s.broad_phase_pairs, 0);
    }

    #[test]
    fn test_record_frame_increments_count() {
        let mut s = new_collision_stats();
        record_frame(&mut s, 10, 5, 3, 3);
        assert_eq!(s.frame_count, 1);
    }

    #[test]
    fn test_record_multiple_frames() {
        let mut s = new_collision_stats();
        record_frame(&mut s, 10, 5, 3, 3);
        record_frame(&mut s, 20, 8, 6, 6);
        assert_eq!(s.frame_count, 2);
        assert_eq!(s.broad_phase_pairs, 30);
    }

    #[test]
    fn test_avg_contacts_empty() {
        let s = new_collision_stats();
        assert!((avg_contacts_per_frame(&s) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_avg_contacts_per_frame() {
        let mut s = new_collision_stats();
        record_frame(&mut s, 0, 0, 4, 0);
        record_frame(&mut s, 0, 0, 6, 0);
        let avg = avg_contacts_per_frame(&s);
        assert!((avg - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_reset_stats() {
        let mut s = new_collision_stats();
        record_frame(&mut s, 10, 5, 3, 3);
        reset_stats(&mut s);
        assert_eq!(s.frame_count, 0);
        assert_eq!(s.contacts_generated, 0);
    }

    #[test]
    fn test_contacts_accumulate() {
        let mut s = new_collision_stats();
        for _ in 0..10 {
            record_frame(&mut s, 0, 0, 2, 0);
        }
        assert_eq!(s.contacts_generated, 20);
    }

    #[test]
    fn test_constraints_accumulate() {
        let mut s = new_collision_stats();
        record_frame(&mut s, 0, 0, 0, 7);
        assert_eq!(s.constraints_solved, 7);
    }

    #[test]
    fn test_narrow_phase_accumulate() {
        let mut s = new_collision_stats();
        record_frame(&mut s, 5, 3, 1, 1);
        assert_eq!(s.narrow_phase_tests, 3);
    }
}
