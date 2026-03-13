#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Lightweight physics profiler for measuring solver timings.

/// A single profile entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProfileEntry {
    pub name: String,
    pub start_ms: f64,
    pub end_ms: f64,
}

/// Collects profile data for physics simulation steps.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PhysicsProfiler {
    entries: Vec<ProfileEntry>,
    current: Option<(String, f64)>,
}

#[allow(dead_code)]
pub fn new_physics_profiler() -> PhysicsProfiler {
    PhysicsProfiler {
        entries: Vec::new(),
        current: None,
    }
}

#[allow(dead_code)]
pub fn begin_profile(profiler: &mut PhysicsProfiler, name: &str, time_ms: f64) {
    profiler.current = Some((name.to_string(), time_ms));
}

#[allow(dead_code)]
pub fn end_profile(profiler: &mut PhysicsProfiler, time_ms: f64) {
    if let Some((name, start)) = profiler.current.take() {
        profiler.entries.push(ProfileEntry {
            name,
            start_ms: start,
            end_ms: time_ms,
        });
    }
}

#[allow(dead_code)]
pub fn profile_elapsed_ms(entry: &ProfileEntry) -> f64 {
    entry.end_ms - entry.start_ms
}

#[allow(dead_code)]
pub fn profile_count(profiler: &PhysicsProfiler) -> usize {
    profiler.entries.len()
}

#[allow(dead_code)]
pub fn profiler_reset(profiler: &mut PhysicsProfiler) {
    profiler.entries.clear();
    profiler.current = None;
}

#[allow(dead_code)]
pub fn profiler_total_ms(profiler: &PhysicsProfiler) -> f64 {
    profiler.entries.iter().map(|e| e.end_ms - e.start_ms).sum()
}

#[allow(dead_code)]
pub fn profiler_average_ms(profiler: &PhysicsProfiler) -> f64 {
    if profiler.entries.is_empty() {
        return 0.0;
    }
    profiler_total_ms(profiler) / profiler.entries.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_profiler() {
        let p = new_physics_profiler();
        assert_eq!(profile_count(&p), 0);
    }

    #[test]
    fn test_begin_end() {
        let mut p = new_physics_profiler();
        begin_profile(&mut p, "solve", 0.0);
        end_profile(&mut p, 5.0);
        assert_eq!(profile_count(&p), 1);
    }

    #[test]
    fn test_elapsed() {
        let e = ProfileEntry {
            name: "test".to_string(),
            start_ms: 10.0,
            end_ms: 15.0,
        };
        assert!((profile_elapsed_ms(&e) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_total() {
        let mut p = new_physics_profiler();
        begin_profile(&mut p, "a", 0.0);
        end_profile(&mut p, 3.0);
        begin_profile(&mut p, "b", 3.0);
        end_profile(&mut p, 8.0);
        assert!((profiler_total_ms(&p) - 8.0).abs() < 1e-10);
    }

    #[test]
    fn test_average() {
        let mut p = new_physics_profiler();
        begin_profile(&mut p, "a", 0.0);
        end_profile(&mut p, 4.0);
        begin_profile(&mut p, "b", 4.0);
        end_profile(&mut p, 10.0);
        assert!((profiler_average_ms(&p) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_average_empty() {
        let p = new_physics_profiler();
        assert_eq!(profiler_average_ms(&p), 0.0);
    }

    #[test]
    fn test_reset() {
        let mut p = new_physics_profiler();
        begin_profile(&mut p, "x", 0.0);
        end_profile(&mut p, 1.0);
        profiler_reset(&mut p);
        assert_eq!(profile_count(&p), 0);
    }

    #[test]
    fn test_end_without_begin() {
        let mut p = new_physics_profiler();
        end_profile(&mut p, 5.0);
        assert_eq!(profile_count(&p), 0);
    }

    #[test]
    fn test_entry_name() {
        let mut p = new_physics_profiler();
        begin_profile(&mut p, "broadphase", 0.0);
        end_profile(&mut p, 2.0);
        assert_eq!(p.entries[0].name, "broadphase");
    }

    #[test]
    fn test_multiple_entries() {
        let mut p = new_physics_profiler();
        for i in 0..5 {
            begin_profile(&mut p, "step", i as f64);
            end_profile(&mut p, (i + 1) as f64);
        }
        assert_eq!(profile_count(&p), 5);
    }
}
