#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Software watchdog timer.

/// Configuration for a watchdog.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WatchDogConfig {
    pub timeout_ms: u64,
}

/// A software watchdog timer (uses a monotonic tick counter).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WatchDog {
    pub config: WatchDogConfig,
    last_kick_tick: u64,
    current_tick: u64,
    kick_count: u64,
}

/// Create a default `WatchDogConfig` (5 second timeout).
#[allow(dead_code)]
pub fn default_watch_dog_config() -> WatchDogConfig {
    WatchDogConfig { timeout_ms: 5000 }
}

/// Create a new `WatchDog`.
#[allow(dead_code)]
pub fn new_watch_dog(config: WatchDogConfig) -> WatchDog {
    WatchDog { config, last_kick_tick: 0, current_tick: 0, kick_count: 0 }
}

/// Advance the internal tick by `delta_ms` milliseconds.
#[allow(dead_code)]
pub fn advance_tick(wd: &mut WatchDog, delta_ms: u64) {
    wd.current_tick += delta_ms;
}

/// Kick (reset) the watchdog timer.
#[allow(dead_code)]
pub fn kick_watch_dog(wd: &mut WatchDog) {
    wd.last_kick_tick = wd.current_tick;
    wd.kick_count += 1;
}

/// Return milliseconds since last kick.
#[allow(dead_code)]
pub fn watch_dog_elapsed_ms(wd: &WatchDog) -> u64 {
    wd.current_tick.saturating_sub(wd.last_kick_tick)
}

/// Return the configured timeout in milliseconds.
#[allow(dead_code)]
pub fn watch_dog_timeout_ms(wd: &WatchDog) -> u64 {
    wd.config.timeout_ms
}

/// Return true if the watchdog has expired (elapsed > timeout).
#[allow(dead_code)]
pub fn watch_dog_is_expired(wd: &WatchDog) -> bool {
    watch_dog_elapsed_ms(wd) > wd.config.timeout_ms
}

/// Reset both the tick and kick counter.
#[allow(dead_code)]
pub fn reset_watch_dog(wd: &mut WatchDog) {
    wd.current_tick = 0;
    wd.last_kick_tick = 0;
    wd.kick_count = 0;
}

/// Return the total number of kicks since creation or last reset.
#[allow(dead_code)]
pub fn watch_dog_kick_count(wd: &WatchDog) -> u64 {
    wd.kick_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_watch_dog() {
        let wd = new_watch_dog(default_watch_dog_config());
        assert_eq!(watch_dog_elapsed_ms(&wd), 0);
        assert!(!watch_dog_is_expired(&wd));
    }

    #[test]
    fn test_kick_resets_elapsed() {
        let mut wd = new_watch_dog(default_watch_dog_config());
        advance_tick(&mut wd, 1000);
        kick_watch_dog(&mut wd);
        assert_eq!(watch_dog_elapsed_ms(&wd), 0);
    }

    #[test]
    fn test_expiry() {
        let mut wd = new_watch_dog(WatchDogConfig { timeout_ms: 100 });
        advance_tick(&mut wd, 200);
        assert!(watch_dog_is_expired(&wd));
    }

    #[test]
    fn test_not_expired_before_timeout() {
        let mut wd = new_watch_dog(WatchDogConfig { timeout_ms: 1000 });
        advance_tick(&mut wd, 500);
        assert!(!watch_dog_is_expired(&wd));
    }

    #[test]
    fn test_kick_count() {
        let mut wd = new_watch_dog(default_watch_dog_config());
        kick_watch_dog(&mut wd);
        kick_watch_dog(&mut wd);
        assert_eq!(watch_dog_kick_count(&wd), 2);
    }

    #[test]
    fn test_reset_watch_dog() {
        let mut wd = new_watch_dog(default_watch_dog_config());
        advance_tick(&mut wd, 500);
        kick_watch_dog(&mut wd);
        reset_watch_dog(&mut wd);
        assert_eq!(watch_dog_elapsed_ms(&wd), 0);
        assert_eq!(watch_dog_kick_count(&wd), 0);
    }

    #[test]
    fn test_timeout_ms() {
        let wd = new_watch_dog(WatchDogConfig { timeout_ms: 2000 });
        assert_eq!(watch_dog_timeout_ms(&wd), 2000);
    }

    #[test]
    fn test_elapsed_after_advance() {
        let mut wd = new_watch_dog(default_watch_dog_config());
        advance_tick(&mut wd, 300);
        assert_eq!(watch_dog_elapsed_ms(&wd), 300);
    }
}
