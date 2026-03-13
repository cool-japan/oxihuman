#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Event throttle: limit how often events pass through.

/// Configuration for an event throttle.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ThrottleConfig {
    pub interval_ms: u64,
}

/// An event throttle backed by a tick counter.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EventThrottle {
    config: ThrottleConfig,
    last_pass_tick: u64,
    current_tick: u64,
    pass_count: u64,
    drop_count: u64,
    active: bool,
    first_event: bool,
}

/// Create a default `ThrottleConfig` (100 ms interval).
#[allow(dead_code)]
pub fn default_throttle_config() -> ThrottleConfig {
    ThrottleConfig { interval_ms: 100 }
}

/// Create a new `EventThrottle`.
#[allow(dead_code)]
pub fn new_event_throttle(config: ThrottleConfig) -> EventThrottle {
    EventThrottle {
        config,
        last_pass_tick: 0,
        current_tick: 0,
        pass_count: 0,
        drop_count: 0,
        active: true,
        first_event: true,
    }
}

/// Advance internal tick by `delta_ms`.
#[allow(dead_code)]
pub fn advance_throttle_tick(et: &mut EventThrottle, delta_ms: u64) {
    et.current_tick += delta_ms;
}

/// Attempt to pass an event through the throttle.
/// Returns `true` if the event passes, `false` if it is dropped.
/// The first event always passes.
#[allow(dead_code)]
pub fn throttle_event(et: &mut EventThrottle) -> bool {
    if !et.active {
        return false;
    }
    if et.first_event {
        et.first_event = false;
        et.last_pass_tick = et.current_tick;
        et.pass_count += 1;
        return true;
    }
    let elapsed = et.current_tick.saturating_sub(et.last_pass_tick);
    if elapsed >= et.config.interval_ms {
        et.last_pass_tick = et.current_tick;
        et.pass_count += 1;
        true
    } else {
        et.drop_count += 1;
        false
    }
}

/// Return the number of events that have passed.
#[allow(dead_code)]
pub fn throttle_count(et: &EventThrottle) -> u64 {
    et.pass_count
}

/// Reset the throttle state.
#[allow(dead_code)]
pub fn reset_throttle(et: &mut EventThrottle) {
    et.last_pass_tick = et.current_tick;
    et.pass_count = 0;
    et.drop_count = 0;
    et.first_event = true;
}

/// Return the configured throttle interval in milliseconds.
#[allow(dead_code)]
pub fn throttle_interval_ms(et: &EventThrottle) -> u64 {
    et.config.interval_ms
}

/// Return the number of dropped events.
#[allow(dead_code)]
pub fn events_dropped(et: &EventThrottle) -> u64 {
    et.drop_count
}

/// Return true if the throttle is active.
#[allow(dead_code)]
pub fn throttle_is_active(et: &EventThrottle) -> bool {
    et.active
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_event_throttle() {
        let et = new_event_throttle(default_throttle_config());
        assert_eq!(throttle_count(&et), 0);
        assert!(throttle_is_active(&et));
    }

    #[test]
    fn test_first_event_passes() {
        let mut et = new_event_throttle(ThrottleConfig { interval_ms: 100 });
        assert!(throttle_event(&mut et));
    }

    #[test]
    fn test_second_event_too_fast() {
        let mut et = new_event_throttle(ThrottleConfig { interval_ms: 100 });
        throttle_event(&mut et);
        assert!(!throttle_event(&mut et));
    }

    #[test]
    fn test_event_passes_after_interval() {
        let mut et = new_event_throttle(ThrottleConfig { interval_ms: 100 });
        throttle_event(&mut et);
        advance_throttle_tick(&mut et, 100);
        assert!(throttle_event(&mut et));
    }

    #[test]
    fn test_events_dropped_count() {
        let mut et = new_event_throttle(ThrottleConfig { interval_ms: 100 });
        throttle_event(&mut et);
        throttle_event(&mut et);
        throttle_event(&mut et);
        assert_eq!(events_dropped(&et), 2);
    }

    #[test]
    fn test_reset_throttle() {
        let mut et = new_event_throttle(ThrottleConfig { interval_ms: 100 });
        throttle_event(&mut et);
        reset_throttle(&mut et);
        assert_eq!(throttle_count(&et), 0);
        assert_eq!(events_dropped(&et), 0);
    }

    #[test]
    fn test_throttle_interval_ms() {
        let et = new_event_throttle(ThrottleConfig { interval_ms: 250 });
        assert_eq!(throttle_interval_ms(&et), 250);
    }

    #[test]
    fn test_inactive_throttle_drops_all() {
        let mut et = new_event_throttle(ThrottleConfig { interval_ms: 0 });
        et.active = false;
        assert!(!throttle_event(&mut et));
    }
}
