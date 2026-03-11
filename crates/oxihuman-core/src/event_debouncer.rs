//! Debounce rapid-fire events: only forward an event after a quiet period.
//!
//! After a trigger is received, the debouncer waits for `quiet_period` ticks
//! without a new trigger before marking the event as fired.

#![allow(dead_code)]

/// Configuration for an `EventDebouncer`.
#[derive(Debug, Clone)]
pub struct DebouncerConfig {
    /// Number of ticks without a new trigger before the event fires.
    pub quiet_period: u64,
}

/// A debouncer that delays event firing until input stops.
#[derive(Debug, Clone)]
pub struct EventDebouncer {
    config: DebouncerConfig,
    /// Time of the last trigger (in ticks).
    last_trigger_time: u64,
    /// Current time (in ticks).
    current_time: u64,
    /// Whether we have an un-fired pending trigger.
    pending: bool,
    /// Whether the event has just fired (single-tick pulse).
    fired: bool,
    /// Total number of times the debounced event has fired.
    fire_count: u64,
}

/// Build a default `DebouncerConfig` (quiet_period = 5 ticks).
#[allow(dead_code)]
pub fn default_debouncer_config() -> DebouncerConfig {
    DebouncerConfig { quiet_period: 5 }
}

/// Create a new `EventDebouncer`.
#[allow(dead_code)]
pub fn new_event_debouncer(config: DebouncerConfig) -> EventDebouncer {
    EventDebouncer {
        config,
        last_trigger_time: 0,
        current_time: 0,
        pending: false,
        fired: false,
        fire_count: 0,
    }
}

/// Record a new trigger event at the current time.
#[allow(dead_code)]
pub fn debouncer_trigger(deb: &mut EventDebouncer) {
    deb.last_trigger_time = deb.current_time;
    deb.pending = true;
    deb.fired = false;
}

/// Advance time by one tick. Returns `true` if the event fires this tick.
#[allow(dead_code)]
pub fn debouncer_tick(deb: &mut EventDebouncer) -> bool {
    deb.current_time += 1;
    deb.fired = false;
    if deb.pending && deb.current_time.saturating_sub(deb.last_trigger_time) >= deb.config.quiet_period {
        deb.pending = false;
        deb.fired = true;
        deb.fire_count += 1;
    }
    deb.fired
}

/// Return `true` if a trigger is pending but has not yet fired.
#[allow(dead_code)]
pub fn debouncer_is_pending(deb: &EventDebouncer) -> bool {
    deb.pending
}

/// Return `true` if the event fired on the most recent tick.
#[allow(dead_code)]
pub fn debouncer_is_fired(deb: &EventDebouncer) -> bool {
    deb.fired
}

/// Reset the debouncer to its initial state.
#[allow(dead_code)]
pub fn debouncer_reset(deb: &mut EventDebouncer) {
    deb.last_trigger_time = 0;
    deb.current_time = 0;
    deb.pending = false;
    deb.fired = false;
    deb.fire_count = 0;
}

/// Return the tick time of the last trigger.
#[allow(dead_code)]
pub fn debouncer_last_trigger_time(deb: &EventDebouncer) -> u64 {
    deb.last_trigger_time
}

/// Serialize the debouncer state to a JSON string.
#[allow(dead_code)]
pub fn debouncer_to_json(deb: &EventDebouncer) -> String {
    format!(
        "{{\"quiet_period\":{},\"current_time\":{},\"pending\":{},\"fired\":{},\"fire_count\":{}}}",
        deb.config.quiet_period,
        deb.current_time,
        deb.pending,
        deb.fired,
        deb.fire_count
    )
}

/// Return the total number of times the debounced event has fired.
#[allow(dead_code)]
pub fn debouncer_fire_count(deb: &EventDebouncer) -> u64 {
    deb.fire_count
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_deb(quiet: u64) -> EventDebouncer {
        new_event_debouncer(DebouncerConfig { quiet_period: quiet })
    }

    #[test]
    fn test_initial_not_pending() {
        let deb = make_deb(3);
        assert!(!debouncer_is_pending(&deb));
        assert!(!debouncer_is_fired(&deb));
    }

    #[test]
    fn test_trigger_sets_pending() {
        let mut deb = make_deb(3);
        debouncer_trigger(&mut deb);
        assert!(debouncer_is_pending(&deb));
    }

    #[test]
    fn test_fires_after_quiet_period() {
        let mut deb = make_deb(3);
        debouncer_trigger(&mut deb);
        debouncer_tick(&mut deb); // t=1
        debouncer_tick(&mut deb); // t=2
        assert!(!debouncer_is_fired(&deb));
        let fired = debouncer_tick(&mut deb); // t=3 → quiet=3 → fires
        assert!(fired);
        assert!(debouncer_is_fired(&deb));
    }

    #[test]
    fn test_retriggering_resets_quiet_period() {
        let mut deb = make_deb(3);
        debouncer_trigger(&mut deb); // t=0
        debouncer_tick(&mut deb);   // t=1
        debouncer_trigger(&mut deb); // reset at t=1
        debouncer_tick(&mut deb);   // t=2
        debouncer_tick(&mut deb);   // t=3 → only 2 ticks since retrigger
        assert!(!debouncer_is_fired(&deb));
        debouncer_tick(&mut deb);   // t=4 → 3 ticks since retrigger → fires
        assert!(debouncer_is_fired(&deb));
    }

    #[test]
    fn test_fire_count_increments() {
        let mut deb = make_deb(2);
        debouncer_trigger(&mut deb);
        debouncer_tick(&mut deb);
        debouncer_tick(&mut deb); // fires
        assert_eq!(debouncer_fire_count(&deb), 1);
        debouncer_trigger(&mut deb);
        debouncer_tick(&mut deb);
        debouncer_tick(&mut deb); // fires again
        assert_eq!(debouncer_fire_count(&deb), 2);
    }

    #[test]
    fn test_not_fired_without_trigger() {
        let mut deb = make_deb(1);
        debouncer_tick(&mut deb);
        debouncer_tick(&mut deb);
        assert!(!debouncer_is_fired(&deb));
        assert_eq!(debouncer_fire_count(&deb), 0);
    }

    #[test]
    fn test_reset() {
        let mut deb = make_deb(2);
        debouncer_trigger(&mut deb);
        debouncer_tick(&mut deb);
        debouncer_reset(&mut deb);
        assert!(!debouncer_is_pending(&deb));
        assert_eq!(debouncer_fire_count(&deb), 0);
        assert_eq!(deb.current_time, 0);
    }

    #[test]
    fn test_last_trigger_time() {
        let mut deb = make_deb(5);
        debouncer_tick(&mut deb); // t=1
        debouncer_tick(&mut deb); // t=2
        debouncer_trigger(&mut deb);
        assert_eq!(debouncer_last_trigger_time(&deb), 2);
    }

    #[test]
    fn test_to_json_contains_quiet_period() {
        let deb = make_deb(7);
        let json = debouncer_to_json(&deb);
        assert!(json.contains("\"quiet_period\":7"));
    }

    #[test]
    fn test_fired_clears_next_tick() {
        let mut deb = make_deb(1);
        debouncer_trigger(&mut deb);
        debouncer_tick(&mut deb); // fires at t=1
        assert!(debouncer_is_fired(&deb));
        debouncer_tick(&mut deb); // no trigger, fired resets
        assert!(!debouncer_is_fired(&deb));
    }
}
