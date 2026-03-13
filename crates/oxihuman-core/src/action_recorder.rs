//! Record a sequence of named actions with timestamps for replay and audit.
//!
//! Provides an append-only action log that can be replayed or filtered
//! by time range, useful for audit trails and undo/redo scaffolding.

#![allow(dead_code)]

/// Configuration for an `ActionRecorder`.
#[derive(Debug, Clone)]
pub struct ActionRecorderConfig {
    /// Maximum number of actions retained (older ones are dropped when exceeded).
    pub max_actions: usize,
    /// Whether recording is enabled by default.
    pub recording_enabled: bool,
}

/// A single recorded action entry.
#[derive(Debug, Clone)]
pub struct ActionRecord {
    /// Monotonic timestamp (arbitrary units, e.g. frame counter or ms).
    pub timestamp: u64,
    /// Name / description of the action.
    pub name: String,
    /// Optional payload string (JSON, tag, etc.).
    pub payload: String,
}

/// Records a sequence of named actions with timestamps.
#[derive(Debug, Clone)]
pub struct ActionRecorder {
    config: ActionRecorderConfig,
    records: Vec<ActionRecord>,
    is_recording: bool,
    current_time: u64,
}

/// Build a default `ActionRecorderConfig`.
#[allow(dead_code)]
pub fn default_action_recorder_config() -> ActionRecorderConfig {
    ActionRecorderConfig {
        max_actions: 1024,
        recording_enabled: true,
    }
}

/// Create a new `ActionRecorder`.
#[allow(dead_code)]
pub fn new_action_recorder(config: ActionRecorderConfig) -> ActionRecorder {
    let enabled = config.recording_enabled;
    ActionRecorder {
        config,
        records: Vec::new(),
        is_recording: enabled,
        current_time: 0,
    }
}

/// Push a named action with an optional payload at the current time.
#[allow(dead_code)]
pub fn recorder_push_action(recorder: &mut ActionRecorder, name: &str, payload: &str) {
    if !recorder.is_recording {
        return;
    }
    if recorder.records.len() >= recorder.config.max_actions {
        recorder.records.remove(0);
    }
    recorder.current_time += 1;
    recorder.records.push(ActionRecord {
        timestamp: recorder.current_time,
        name: name.to_string(),
        payload: payload.to_string(),
    });
}

/// Replay all actions by calling `callback` for each in order.
#[allow(dead_code)]
pub fn recorder_replay<F: FnMut(&ActionRecord)>(recorder: &ActionRecorder, mut callback: F) {
    for record in &recorder.records {
        callback(record);
    }
}

/// Return the total number of recorded actions.
#[allow(dead_code)]
pub fn recorder_action_count(recorder: &ActionRecorder) -> usize {
    recorder.records.len()
}

/// Return the last recorded action, if any.
#[allow(dead_code)]
pub fn recorder_last_action(recorder: &ActionRecorder) -> Option<&ActionRecord> {
    recorder.records.last()
}

/// Return all actions recorded at or after `since_time`.
#[allow(dead_code)]
pub fn recorder_since_time(recorder: &ActionRecorder, since_time: u64) -> Vec<&ActionRecord> {
    recorder
        .records
        .iter()
        .filter(|r| r.timestamp >= since_time)
        .collect()
}

/// Serialize the recorder state to a JSON string.
#[allow(dead_code)]
pub fn recorder_to_json(recorder: &ActionRecorder) -> String {
    format!(
        "{{\"action_count\":{},\"is_recording\":{},\"current_time\":{}}}",
        recorder.records.len(),
        recorder.is_recording,
        recorder.current_time
    )
}

/// Clear all recorded actions and reset the internal clock.
#[allow(dead_code)]
pub fn recorder_clear(recorder: &mut ActionRecorder) {
    recorder.records.clear();
    recorder.current_time = 0;
}

/// Return whether the recorder is currently recording.
#[allow(dead_code)]
pub fn recorder_is_recording(recorder: &ActionRecorder) -> bool {
    recorder.is_recording
}

/// Enable or disable recording.
#[allow(dead_code)]
pub fn recorder_set_recording(recorder: &mut ActionRecorder, enabled: bool) {
    recorder.is_recording = enabled;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_recorder() -> ActionRecorder {
        new_action_recorder(default_action_recorder_config())
    }

    #[test]
    fn test_initial_count_zero() {
        let r = make_recorder();
        assert_eq!(recorder_action_count(&r), 0);
    }

    #[test]
    fn test_push_increments_count() {
        let mut r = make_recorder();
        recorder_push_action(&mut r, "move", "");
        recorder_push_action(&mut r, "rotate", "");
        assert_eq!(recorder_action_count(&r), 2);
    }

    #[test]
    fn test_last_action_name() {
        let mut r = make_recorder();
        recorder_push_action(&mut r, "first", "");
        recorder_push_action(&mut r, "second", "");
        assert_eq!(recorder_last_action(&r).expect("should succeed").name, "second");
    }

    #[test]
    fn test_replay_calls_callback() {
        let mut r = make_recorder();
        recorder_push_action(&mut r, "a", "1");
        recorder_push_action(&mut r, "b", "2");
        let mut names = Vec::new();
        recorder_replay(&r, |rec| names.push(rec.name.clone()));
        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn test_since_time_filter() {
        let mut r = make_recorder();
        recorder_push_action(&mut r, "t1", ""); // ts=1
        recorder_push_action(&mut r, "t2", ""); // ts=2
        recorder_push_action(&mut r, "t3", ""); // ts=3
        let recent = recorder_since_time(&r, 2);
        assert_eq!(recent.len(), 2);
    }

    #[test]
    fn test_clear_resets() {
        let mut r = make_recorder();
        recorder_push_action(&mut r, "x", "");
        recorder_clear(&mut r);
        assert_eq!(recorder_action_count(&r), 0);
    }

    #[test]
    fn test_is_recording_default_true() {
        let r = make_recorder();
        assert!(recorder_is_recording(&r));
    }

    #[test]
    fn test_disabled_recording_no_push() {
        let mut r = make_recorder();
        recorder_set_recording(&mut r, false);
        recorder_push_action(&mut r, "ignored", "");
        assert_eq!(recorder_action_count(&r), 0);
    }

    #[test]
    fn test_to_json_contains_is_recording() {
        let r = make_recorder();
        let json = recorder_to_json(&r);
        assert!(json.contains("is_recording"));
    }

    #[test]
    fn test_max_actions_drops_oldest() {
        let cfg = ActionRecorderConfig {
            max_actions: 3,
            recording_enabled: true,
        };
        let mut r = new_action_recorder(cfg);
        recorder_push_action(&mut r, "a", "");
        recorder_push_action(&mut r, "b", "");
        recorder_push_action(&mut r, "c", "");
        recorder_push_action(&mut r, "d", ""); // drops "a"
        assert_eq!(recorder_action_count(&r), 3);
        assert_eq!(recorder_replay(&r, |_| {}), ());
        // First action should now be "b"
        let mut names = Vec::new();
        recorder_replay(&r, |rec| names.push(rec.name.clone()));
        assert_eq!(names[0], "b");
    }
}
