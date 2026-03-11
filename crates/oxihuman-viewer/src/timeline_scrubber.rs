//! Timeline scrubber UI helper — manages playback position, frame range, and
//! scrub state for the OxiHuman viewer.

// ──────────────────────────────────────────────────────────────────────────────
// Structs / Enums
// ──────────────────────────────────────────────────────────────────────────────

/// Configuration for the timeline scrubber.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TimelineConfig {
    /// First frame of the playback range.
    pub start_frame: u32,
    /// Last frame of the playback range (inclusive).
    pub end_frame: u32,
    /// Playback speed in frames per second.
    pub fps: f32,
    /// Whether to loop when the end frame is reached.
    pub loop_playback: bool,
}

/// Current playback state of the scrubber.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    /// Not playing; sitting at a fixed frame.
    Stopped,
    /// Actively advancing frames with time.
    Playing,
    /// Paused mid-play; can be resumed.
    Paused,
    /// User is dragging the scrub handle.
    Scrubbing,
}

/// The timeline scrubber state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TimelineScrubber {
    /// Active configuration.
    pub config: TimelineConfig,
    /// Current frame index (clamped to `[start_frame, end_frame]`).
    pub current_frame: u32,
    /// Fractional accumulated time since last whole-frame advance (in seconds).
    pub time_accumulator: f32,
    /// Current playback state.
    pub state: PlaybackState,
}

// ──────────────────────────────────────────────────────────────────────────────
// Functions
// ──────────────────────────────────────────────────────────────────────────────

/// Return a default [`TimelineConfig`] (frames 0–239 at 24 fps, looping).
#[allow(dead_code)]
pub fn default_timeline_config() -> TimelineConfig {
    TimelineConfig {
        start_frame: 0,
        end_frame: 239,
        fps: 24.0,
        loop_playback: true,
    }
}

/// Create a new [`TimelineScrubber`] from a config, starting at `start_frame`
/// in the [`PlaybackState::Stopped`] state.
#[allow(dead_code)]
pub fn new_timeline_scrubber(cfg: &TimelineConfig) -> TimelineScrubber {
    TimelineScrubber {
        current_frame: cfg.start_frame,
        config: cfg.clone(),
        time_accumulator: 0.0,
        state: PlaybackState::Stopped,
    }
}

/// Jump to an explicit frame, clamping to the valid range.
#[allow(dead_code)]
pub fn timeline_set_frame(scrubber: &mut TimelineScrubber, frame: u32) {
    scrubber.current_frame = frame.clamp(
        scrubber.config.start_frame,
        scrubber.config.end_frame,
    );
    scrubber.time_accumulator = 0.0;
}

/// Advance the scrubber by `dt` seconds. Only moves the frame when
/// `state == Playing`.
#[allow(dead_code)]
pub fn timeline_advance(scrubber: &mut TimelineScrubber, dt: f32) {
    if scrubber.state != PlaybackState::Playing {
        return;
    }
    if scrubber.config.fps <= 0.0 {
        return;
    }
    scrubber.time_accumulator += dt;
    let frame_duration = 1.0 / scrubber.config.fps;
    while scrubber.time_accumulator >= frame_duration {
        scrubber.time_accumulator -= frame_duration;
        if scrubber.current_frame < scrubber.config.end_frame {
            scrubber.current_frame += 1;
        } else if scrubber.config.loop_playback {
            scrubber.current_frame = scrubber.config.start_frame;
        } else {
            scrubber.state = PlaybackState::Stopped;
            break;
        }
    }
}

/// Start or resume playback.
#[allow(dead_code)]
pub fn timeline_play(scrubber: &mut TimelineScrubber) {
    scrubber.state = PlaybackState::Playing;
}

/// Pause playback (resumable with [`timeline_play`]).
#[allow(dead_code)]
pub fn timeline_pause(scrubber: &mut TimelineScrubber) {
    scrubber.state = PlaybackState::Paused;
}

/// Stop playback and reset to the start frame.
#[allow(dead_code)]
pub fn timeline_stop(scrubber: &mut TimelineScrubber) {
    scrubber.state = PlaybackState::Stopped;
    scrubber.current_frame = scrubber.config.start_frame;
    scrubber.time_accumulator = 0.0;
}

/// Return the current frame index.
#[allow(dead_code)]
pub fn timeline_current_frame(scrubber: &TimelineScrubber) -> u32 {
    scrubber.current_frame
}

/// Return the current [`PlaybackState`].
#[allow(dead_code)]
pub fn timeline_playback_state(scrubber: &TimelineScrubber) -> PlaybackState {
    scrubber.state
}

/// Return the current position as a normalised value in `[0.0, 1.0]`.
///
/// Returns `0.0` when the range has zero length.
#[allow(dead_code)]
pub fn timeline_normalized_position(scrubber: &TimelineScrubber) -> f32 {
    let start = scrubber.config.start_frame as f32;
    let end = scrubber.config.end_frame as f32;
    let range = end - start;
    if range <= 0.0 {
        return 0.0;
    }
    ((scrubber.current_frame as f32) - start) / range
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_timeline_config();
        assert_eq!(cfg.start_frame, 0);
        assert_eq!(cfg.end_frame, 239);
        assert!((cfg.fps - 24.0).abs() < 1e-5);
        assert!(cfg.loop_playback);
    }

    #[test]
    fn test_new_scrubber_initial_state() {
        let cfg = default_timeline_config();
        let scrubber = new_timeline_scrubber(&cfg);
        assert_eq!(timeline_current_frame(&scrubber), 0);
        assert_eq!(timeline_playback_state(&scrubber), PlaybackState::Stopped);
    }

    #[test]
    fn test_set_frame_clamped() {
        let cfg = default_timeline_config();
        let mut scrubber = new_timeline_scrubber(&cfg);
        timeline_set_frame(&mut scrubber, 9999);
        assert_eq!(timeline_current_frame(&scrubber), 239);
        timeline_set_frame(&mut scrubber, 0);
        assert_eq!(timeline_current_frame(&scrubber), 0);
    }

    #[test]
    fn test_play_pause_stop() {
        let cfg = default_timeline_config();
        let mut scrubber = new_timeline_scrubber(&cfg);
        timeline_play(&mut scrubber);
        assert_eq!(timeline_playback_state(&scrubber), PlaybackState::Playing);
        timeline_pause(&mut scrubber);
        assert_eq!(timeline_playback_state(&scrubber), PlaybackState::Paused);
        timeline_stop(&mut scrubber);
        assert_eq!(timeline_playback_state(&scrubber), PlaybackState::Stopped);
        assert_eq!(timeline_current_frame(&scrubber), 0);
    }

    #[test]
    fn test_advance_one_frame() {
        let cfg = default_timeline_config(); // 24 fps
        let mut scrubber = new_timeline_scrubber(&cfg);
        timeline_play(&mut scrubber);
        // Advance by exactly one frame duration
        timeline_advance(&mut scrubber, 1.0 / 24.0);
        assert_eq!(timeline_current_frame(&scrubber), 1);
    }

    #[test]
    fn test_advance_no_change_when_stopped() {
        let cfg = default_timeline_config();
        let mut scrubber = new_timeline_scrubber(&cfg);
        timeline_advance(&mut scrubber, 10.0);
        assert_eq!(timeline_current_frame(&scrubber), 0);
    }

    #[test]
    fn test_normalized_position_at_start() {
        let cfg = default_timeline_config();
        let scrubber = new_timeline_scrubber(&cfg);
        assert!((timeline_normalized_position(&scrubber) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalized_position_at_end() {
        let cfg = default_timeline_config();
        let mut scrubber = new_timeline_scrubber(&cfg);
        timeline_set_frame(&mut scrubber, 239);
        let pos = timeline_normalized_position(&scrubber);
        assert!((pos - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_loop_playback() {
        let mut cfg = default_timeline_config();
        cfg.start_frame = 0;
        cfg.end_frame = 2;
        cfg.fps = 1.0;
        cfg.loop_playback = true;
        let mut scrubber = new_timeline_scrubber(&cfg);
        timeline_play(&mut scrubber);
        // Advance 4 seconds at 1 fps — should wrap around
        timeline_advance(&mut scrubber, 4.0);
        // After 4 frames over range [0,2] with looping: 0→1→2→0→1
        assert_eq!(timeline_current_frame(&scrubber), 1);
    }

    #[test]
    fn test_stop_no_loop() {
        let mut cfg = default_timeline_config();
        cfg.start_frame = 0;
        cfg.end_frame = 1;
        cfg.fps = 1.0;
        cfg.loop_playback = false;
        let mut scrubber = new_timeline_scrubber(&cfg);
        timeline_play(&mut scrubber);
        timeline_advance(&mut scrubber, 5.0);
        assert_eq!(timeline_playback_state(&scrubber), PlaybackState::Stopped);
    }
}
