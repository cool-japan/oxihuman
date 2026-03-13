// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Timer / alarm system for deferred and recurring events.

/// Whether a timer fires once or repeats.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimerKind {
    OneShot,
    Repeating,
}

/// A single timer instance.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Timer {
    pub id: u64,
    pub name: String,
    pub kind: TimerKind,
    /// Interval between firings (seconds).
    pub interval: f64,
    /// Seconds elapsed since the last reset/fire.
    pub elapsed: f64,
    pub active: bool,
    /// Whether this timer has been fired at least once (OneShot) or is still active.
    pub fired: bool,
}

/// System that owns and ticks a collection of timers.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TimerSystem {
    timers: Vec<Timer>,
    next_id: u64,
    /// Accumulated list of IDs that fired during the most recent [`tick_timers`] call.
    last_fired: Vec<u64>,
    /// Global time (sum of all dt values passed to tick_timers).
    global_time: f64,
}

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

/// Create a new, empty [`TimerSystem`].
#[allow(dead_code)]
pub fn new_timer_system() -> TimerSystem {
    TimerSystem {
        timers: Vec::new(),
        next_id: 1,
        last_fired: Vec::new(),
        global_time: 0.0,
    }
}

/// Add a timer with the given name, kind, and interval. Returns the assigned ID.
#[allow(dead_code)]
pub fn add_timer(sys: &mut TimerSystem, name: &str, kind: TimerKind, interval: f64) -> u64 {
    let id = sys.next_id;
    sys.next_id += 1;
    sys.timers.push(Timer {
        id,
        name: name.to_string(),
        kind,
        interval: interval.max(0.0),
        elapsed: 0.0,
        active: true,
        fired: false,
    });
    id
}

/// Remove the timer with the given ID. Returns `true` if found.
#[allow(dead_code)]
pub fn remove_timer(sys: &mut TimerSystem, id: u64) -> bool {
    let before = sys.timers.len();
    sys.timers.retain(|t| t.id != id);
    sys.timers.len() < before
}

/// Advance all active timers by `dt` seconds.
/// Returns a `Vec` of IDs that fired during this tick.
#[allow(dead_code)]
pub fn tick_timers(sys: &mut TimerSystem, dt: f64) -> Vec<u64> {
    sys.global_time += dt;
    let mut fired_ids = Vec::new();

    for timer in &mut sys.timers {
        if !timer.active {
            continue;
        }
        if timer.kind == TimerKind::OneShot && timer.fired {
            continue;
        }
        timer.elapsed += dt;
        if timer.elapsed >= timer.interval {
            fired_ids.push(timer.id);
            timer.fired = true;
            match timer.kind {
                TimerKind::OneShot => {
                    timer.active = false;
                }
                TimerKind::Repeating => {
                    // carry over remainder
                    timer.elapsed -= timer.interval;
                }
            }
        }
    }

    sys.last_fired = fired_ids.clone();
    fired_ids
}

/// Return the total number of timers (active and inactive).
#[allow(dead_code)]
pub fn timer_count(sys: &TimerSystem) -> usize {
    sys.timers.len()
}

/// Return the number of currently active timers.
#[allow(dead_code)]
pub fn active_timer_count(sys: &TimerSystem) -> usize {
    sys.timers.iter().filter(|t| t.active).count()
}

/// Return the list of timer IDs that fired during the last [`tick_timers`] call.
#[allow(dead_code)]
pub fn fired_timers_since(sys: &TimerSystem) -> &[u64] {
    &sys.last_fired
}

/// Reset a timer's elapsed counter to zero (keeping it active).
/// Returns `false` if the timer was not found.
#[allow(dead_code)]
pub fn reset_timer(sys: &mut TimerSystem, id: u64) -> bool {
    for timer in &mut sys.timers {
        if timer.id == id {
            timer.elapsed = 0.0;
            timer.fired = false;
            timer.active = true;
            return true;
        }
    }
    false
}

/// Pause a timer so it stops accumulating elapsed time.
/// Returns `false` if the timer was not found.
#[allow(dead_code)]
pub fn pause_timer(sys: &mut TimerSystem, id: u64) -> bool {
    for timer in &mut sys.timers {
        if timer.id == id {
            timer.active = false;
            return true;
        }
    }
    false
}

/// Resume a paused timer.
/// Returns `false` if the timer was not found.
#[allow(dead_code)]
pub fn resume_timer(sys: &mut TimerSystem, id: u64) -> bool {
    for timer in &mut sys.timers {
        if timer.id == id {
            timer.active = true;
            return true;
        }
    }
    false
}

/// Return the remaining seconds until the timer fires, or `None` if not found.
#[allow(dead_code)]
pub fn timer_remaining(sys: &TimerSystem, id: u64) -> Option<f64> {
    sys.timers
        .iter()
        .find(|t| t.id == id)
        .map(|t| (t.interval - t.elapsed).max(0.0))
}

/// Return the elapsed seconds for the given timer, or `None` if not found.
#[allow(dead_code)]
pub fn timer_elapsed(sys: &TimerSystem, id: u64) -> Option<f64> {
    sys.timers.iter().find(|t| t.id == id).map(|t| t.elapsed)
}

/// Remove all timers from the system.
#[allow(dead_code)]
pub fn clear_all_timers(sys: &mut TimerSystem) {
    sys.timers.clear();
    sys.last_fired.clear();
}

/// Serialise the timer system to a simple JSON string.
#[allow(dead_code)]
pub fn timer_system_to_json(sys: &TimerSystem) -> String {
    let items: Vec<String> = sys
        .timers
        .iter()
        .map(|t| {
            format!(
                "  {{\"id\": {}, \"name\": \"{}\", \"interval\": {:.4}, \"elapsed\": {:.4}, \"active\": {}}}",
                t.id, t.name, t.interval, t.elapsed, t.active
            )
        })
        .collect();
    format!("[\n{}\n]", items.join(",\n"))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sys() -> TimerSystem {
        new_timer_system()
    }

    #[test]
    fn test_new_system_empty() {
        let s = make_sys();
        assert_eq!(timer_count(&s), 0);
        assert_eq!(active_timer_count(&s), 0);
    }

    #[test]
    fn test_add_timer_increases_count() {
        let mut s = make_sys();
        add_timer(&mut s, "t1", TimerKind::OneShot, 1.0);
        assert_eq!(timer_count(&s), 1);
        assert_eq!(active_timer_count(&s), 1);
    }

    #[test]
    fn test_add_timer_returns_unique_ids() {
        let mut s = make_sys();
        let id1 = add_timer(&mut s, "a", TimerKind::OneShot, 1.0);
        let id2 = add_timer(&mut s, "b", TimerKind::OneShot, 1.0);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_remove_timer() {
        let mut s = make_sys();
        let id = add_timer(&mut s, "x", TimerKind::OneShot, 1.0);
        assert!(remove_timer(&mut s, id));
        assert_eq!(timer_count(&s), 0);
    }

    #[test]
    fn test_remove_timer_missing() {
        let mut s = make_sys();
        assert!(!remove_timer(&mut s, 999));
    }

    #[test]
    fn test_oneshot_fires_once() {
        let mut s = make_sys();
        let id = add_timer(&mut s, "shot", TimerKind::OneShot, 0.5);
        let fired = tick_timers(&mut s, 0.6);
        assert!(fired.contains(&id));
        // second tick should NOT fire again
        let fired2 = tick_timers(&mut s, 1.0);
        assert!(!fired2.contains(&id));
    }

    #[test]
    fn test_repeating_fires_multiple_times() {
        let mut s = make_sys();
        let id = add_timer(&mut s, "rep", TimerKind::Repeating, 0.5);
        let f1 = tick_timers(&mut s, 0.6);
        let f2 = tick_timers(&mut s, 0.5);
        assert!(f1.contains(&id));
        assert!(f2.contains(&id));
    }

    #[test]
    fn test_timer_not_fired_before_interval() {
        let mut s = make_sys();
        let id = add_timer(&mut s, "slow", TimerKind::OneShot, 10.0);
        let fired = tick_timers(&mut s, 0.1);
        assert!(!fired.contains(&id));
    }

    #[test]
    fn test_active_timer_count_after_oneshot_fires() {
        let mut s = make_sys();
        add_timer(&mut s, "t", TimerKind::OneShot, 0.1);
        tick_timers(&mut s, 0.2);
        assert_eq!(active_timer_count(&s), 0);
    }

    #[test]
    fn test_fired_timers_since() {
        let mut s = make_sys();
        let id = add_timer(&mut s, "q", TimerKind::OneShot, 0.2);
        tick_timers(&mut s, 0.3);
        let last = fired_timers_since(&s);
        assert!(last.contains(&id));
    }

    #[test]
    fn test_reset_timer() {
        let mut s = make_sys();
        let id = add_timer(&mut s, "r", TimerKind::OneShot, 1.0);
        tick_timers(&mut s, 0.8);
        reset_timer(&mut s, id);
        assert!((timer_elapsed(&s, id).expect("should succeed")).abs() < 1e-9);
    }

    #[test]
    fn test_reset_timer_missing_returns_false() {
        let mut s = make_sys();
        assert!(!reset_timer(&mut s, 42));
    }

    #[test]
    fn test_pause_and_resume() {
        let mut s = make_sys();
        let id = add_timer(&mut s, "p", TimerKind::OneShot, 1.0);
        pause_timer(&mut s, id);
        tick_timers(&mut s, 0.9);
        // elapsed should still be 0 because paused
        assert!((timer_elapsed(&s, id).expect("should succeed")).abs() < 1e-9);
        resume_timer(&mut s, id);
        tick_timers(&mut s, 0.5);
        assert!(timer_elapsed(&s, id).expect("should succeed") > 0.0);
    }

    #[test]
    fn test_timer_remaining() {
        let mut s = make_sys();
        let id = add_timer(&mut s, "rem", TimerKind::OneShot, 2.0);
        tick_timers(&mut s, 0.5);
        let rem = timer_remaining(&s, id).expect("should succeed");
        assert!((rem - 1.5).abs() < 1e-9);
    }

    #[test]
    fn test_timer_elapsed() {
        let mut s = make_sys();
        let id = add_timer(&mut s, "e", TimerKind::OneShot, 5.0);
        tick_timers(&mut s, 1.25);
        assert!((timer_elapsed(&s, id).expect("should succeed") - 1.25).abs() < 1e-9);
    }

    #[test]
    fn test_clear_all_timers() {
        let mut s = make_sys();
        add_timer(&mut s, "a", TimerKind::OneShot, 1.0);
        add_timer(&mut s, "b", TimerKind::Repeating, 2.0);
        clear_all_timers(&mut s);
        assert_eq!(timer_count(&s), 0);
    }

    #[test]
    fn test_timer_system_to_json() {
        let mut s = make_sys();
        add_timer(&mut s, "alpha", TimerKind::OneShot, 3.0);
        let json = timer_system_to_json(&s);
        assert!(json.contains("alpha"));
    }

    #[test]
    fn test_timer_kind_equality() {
        assert_eq!(TimerKind::OneShot, TimerKind::OneShot);
        assert_ne!(TimerKind::OneShot, TimerKind::Repeating);
    }
}
