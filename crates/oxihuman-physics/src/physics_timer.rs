#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// A simple physics timer for measuring simulation steps.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PhysicsTimer {
    start_ms: f64,
    elapsed_ms: f64,
    running: bool,
    laps: Vec<f64>,
}

#[allow(dead_code)]
pub fn new_physics_timer() -> PhysicsTimer {
    PhysicsTimer {
        start_ms: 0.0,
        elapsed_ms: 0.0,
        running: false,
        laps: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn timer_start(timer: &mut PhysicsTimer, time_ms: f64) {
    timer.start_ms = time_ms;
    timer.running = true;
}

#[allow(dead_code)]
pub fn timer_stop(timer: &mut PhysicsTimer, time_ms: f64) {
    if timer.running {
        timer.elapsed_ms += time_ms - timer.start_ms;
        timer.running = false;
    }
}

#[allow(dead_code)]
pub fn timer_elapsed_ms_pt(timer: &PhysicsTimer) -> f64 {
    timer.elapsed_ms
}

#[allow(dead_code)]
pub fn timer_reset_pt(timer: &mut PhysicsTimer) {
    timer.start_ms = 0.0;
    timer.elapsed_ms = 0.0;
    timer.running = false;
    timer.laps.clear();
}

#[allow(dead_code)]
pub fn timer_is_running(timer: &PhysicsTimer) -> bool {
    timer.running
}

#[allow(dead_code)]
pub fn timer_lap(timer: &mut PhysicsTimer, time_ms: f64) {
    if timer.running {
        let lap = time_ms - timer.start_ms;
        timer.laps.push(lap);
        timer.start_ms = time_ms;
    }
}

#[allow(dead_code)]
pub fn timer_to_json(timer: &PhysicsTimer) -> String {
    format!(
        "{{\"elapsed_ms\":{:.6},\"running\":{},\"laps\":{}}}",
        timer.elapsed_ms,
        timer.running,
        timer.laps.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_physics_timer() {
        let t = new_physics_timer();
        assert!(!timer_is_running(&t));
    }

    #[test]
    fn test_timer_start() {
        let mut t = new_physics_timer();
        timer_start(&mut t, 0.0);
        assert!(timer_is_running(&t));
    }

    #[test]
    fn test_timer_stop() {
        let mut t = new_physics_timer();
        timer_start(&mut t, 0.0);
        timer_stop(&mut t, 10.0);
        assert!(!timer_is_running(&t));
        assert!((timer_elapsed_ms_pt(&t) - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_timer_elapsed_ms() {
        let mut t = new_physics_timer();
        timer_start(&mut t, 5.0);
        timer_stop(&mut t, 15.0);
        assert!((timer_elapsed_ms_pt(&t) - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_timer_reset() {
        let mut t = new_physics_timer();
        timer_start(&mut t, 0.0);
        timer_stop(&mut t, 10.0);
        timer_reset_pt(&mut t);
        assert!(timer_elapsed_ms_pt(&t).abs() < 1e-6);
    }

    #[test]
    fn test_timer_is_running() {
        let t = new_physics_timer();
        assert!(!timer_is_running(&t));
    }

    #[test]
    fn test_timer_lap() {
        let mut t = new_physics_timer();
        timer_start(&mut t, 0.0);
        timer_lap(&mut t, 5.0);
        timer_lap(&mut t, 10.0);
        assert_eq!(t.laps.len(), 2);
    }

    #[test]
    fn test_timer_to_json() {
        let t = new_physics_timer();
        let json = timer_to_json(&t);
        assert!(json.contains("\"running\":false"));
    }

    #[test]
    fn test_stop_when_not_running() {
        let mut t = new_physics_timer();
        timer_stop(&mut t, 10.0);
        assert!(timer_elapsed_ms_pt(&t).abs() < 1e-6);
    }

    #[test]
    fn test_lap_when_not_running() {
        let mut t = new_physics_timer();
        timer_lap(&mut t, 10.0);
        assert!(t.laps.is_empty());
    }
}
