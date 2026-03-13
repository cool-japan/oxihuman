// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// The type of morph event.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MorphEventType {
    WeightChange,
    Reset,
    GroupChange,
    LodSwitch,
}

/// A morph event with type, target name, and value.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphEvent {
    pub event_type: MorphEventType,
    pub target: String,
    pub value: f32,
}

/// Create a new morph event.
#[allow(dead_code)]
pub fn new_morph_event(event_type: MorphEventType, target: &str, value: f32) -> MorphEvent {
    MorphEvent {
        event_type,
        target: target.to_string(),
        value,
    }
}

/// Return the event type.
#[allow(dead_code)]
pub fn event_type(event: &MorphEvent) -> MorphEventType {
    event.event_type
}

/// Return the target name.
#[allow(dead_code)]
pub fn event_target(event: &MorphEvent) -> &str {
    &event.target
}

/// Return the event value.
#[allow(dead_code)]
pub fn event_value(event: &MorphEvent) -> f32 {
    event.value
}

/// Serialize a list of events to JSON.
#[allow(dead_code)]
pub fn events_to_json(events: &[MorphEvent]) -> String {
    let entries: Vec<String> = events
        .iter()
        .map(|e| {
            format!(
                "{{\"type\":\"{}\",\"target\":\"{}\",\"value\":{:.4}}}",
                morph_event_to_string(e),
                e.target,
                e.value
            )
        })
        .collect();
    format!("[{}]", entries.join(","))
}

/// Check if the event is a weight change.
#[allow(dead_code)]
pub fn event_is_weight_change(event: &MorphEvent) -> bool {
    event.event_type == MorphEventType::WeightChange
}

/// Check if the event is a reset.
#[allow(dead_code)]
pub fn event_is_reset(event: &MorphEvent) -> bool {
    event.event_type == MorphEventType::Reset
}

/// Return a string representation of the event type.
#[allow(dead_code)]
pub fn morph_event_to_string(event: &MorphEvent) -> &'static str {
    match event.event_type {
        MorphEventType::WeightChange => "weight_change",
        MorphEventType::Reset => "reset",
        MorphEventType::GroupChange => "group_change",
        MorphEventType::LodSwitch => "lod_switch",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_event() {
        let e = new_morph_event(MorphEventType::WeightChange, "smile", 0.5);
        assert_eq!(event_type(&e), MorphEventType::WeightChange);
    }

    #[test]
    fn target_accessor() {
        let e = new_morph_event(MorphEventType::Reset, "all", 0.0);
        assert_eq!(event_target(&e), "all");
    }

    #[test]
    fn value_accessor() {
        let e = new_morph_event(MorphEventType::WeightChange, "x", 0.75);
        assert!((event_value(&e) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn is_weight_change() {
        let e = new_morph_event(MorphEventType::WeightChange, "x", 0.5);
        assert!(event_is_weight_change(&e));
    }

    #[test]
    fn is_not_weight_change() {
        let e = new_morph_event(MorphEventType::Reset, "x", 0.0);
        assert!(!event_is_weight_change(&e));
    }

    #[test]
    fn is_reset() {
        let e = new_morph_event(MorphEventType::Reset, "all", 0.0);
        assert!(event_is_reset(&e));
    }

    #[test]
    fn is_not_reset() {
        let e = new_morph_event(MorphEventType::WeightChange, "x", 0.5);
        assert!(!event_is_reset(&e));
    }

    #[test]
    fn to_string_repr() {
        let e = new_morph_event(MorphEventType::GroupChange, "g", 1.0);
        assert_eq!(morph_event_to_string(&e), "group_change");
    }

    #[test]
    fn events_to_json_empty() {
        let j = events_to_json(&[]);
        assert_eq!(j, "[]");
    }

    #[test]
    fn events_to_json_one() {
        let e = new_morph_event(MorphEventType::WeightChange, "test", 0.5);
        let j = events_to_json(&[e]);
        assert!(j.contains("weight_change"));
    }
}
