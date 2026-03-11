#![allow(dead_code)]
//! Morph signal: a typed signal that can be emitted and read by morph systems.

/// The type of signal.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum SignalType {
    Trigger,
    Continuous,
    Pulse,
}

/// A morph signal with a value and type.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphSignal {
    name: String,
    signal_type: SignalType,
    value: f32,
    active: bool,
}

/// Create a new morph signal.
#[allow(dead_code)]
pub fn new_morph_signal(name: &str, signal_type: SignalType) -> MorphSignal {
    MorphSignal {
        name: name.to_string(),
        signal_type,
        value: 0.0,
        active: false,
    }
}

/// Emit the signal with a given value.
#[allow(dead_code)]
pub fn signal_emit(signal: &mut MorphSignal, value: f32) {
    signal.value = value;
    signal.active = true;
}

/// Return the current value.
#[allow(dead_code)]
pub fn signal_value(signal: &MorphSignal) -> f32 {
    signal.value
}

/// Return the signal type.
#[allow(dead_code)]
pub fn signal_type_ms(signal: &MorphSignal) -> &SignalType {
    &signal.signal_type
}

/// Return the signal name.
#[allow(dead_code)]
pub fn signal_name_ms(signal: &MorphSignal) -> &str {
    &signal.name
}

/// Check if the signal is currently active.
#[allow(dead_code)]
pub fn signal_is_active_ms(signal: &MorphSignal) -> bool {
    signal.active
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn signal_to_json(signal: &MorphSignal) -> String {
    let type_str = match signal.signal_type {
        SignalType::Trigger => "trigger",
        SignalType::Continuous => "continuous",
        SignalType::Pulse => "pulse",
    };
    format!(
        "{{\"name\":\"{}\",\"type\":\"{}\",\"value\":{},\"active\":{}}}",
        signal.name, type_str, signal.value, signal.active
    )
}

/// Reset the signal to inactive with value 0.
#[allow(dead_code)]
pub fn signal_reset_ms(signal: &mut MorphSignal) {
    signal.value = 0.0;
    signal.active = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_signal() {
        let s = new_morph_signal("test", SignalType::Trigger);
        assert_eq!(signal_name_ms(&s), "test");
        assert!(!signal_is_active_ms(&s));
    }

    #[test]
    fn test_emit() {
        let mut s = new_morph_signal("x", SignalType::Continuous);
        signal_emit(&mut s, 0.8);
        assert!(signal_is_active_ms(&s));
        assert!((signal_value(&s) - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_signal_type() {
        let s = new_morph_signal("p", SignalType::Pulse);
        assert_eq!(*signal_type_ms(&s), SignalType::Pulse);
    }

    #[test]
    fn test_reset() {
        let mut s = new_morph_signal("r", SignalType::Trigger);
        signal_emit(&mut s, 1.0);
        signal_reset_ms(&mut s);
        assert!(!signal_is_active_ms(&s));
        assert!((signal_value(&s) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let s = new_morph_signal("j", SignalType::Trigger);
        let json = signal_to_json(&s);
        assert!(json.contains("\"name\":\"j\""));
        assert!(json.contains("\"type\":\"trigger\""));
    }

    #[test]
    fn test_initial_value() {
        let s = new_morph_signal("v", SignalType::Continuous);
        assert!((signal_value(&s) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_multiple_emits() {
        let mut s = new_morph_signal("m", SignalType::Continuous);
        signal_emit(&mut s, 0.5);
        signal_emit(&mut s, 0.9);
        assert!((signal_value(&s) - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_name() {
        let s = new_morph_signal("hello", SignalType::Pulse);
        assert_eq!(signal_name_ms(&s), "hello");
    }

    #[test]
    fn test_json_continuous() {
        let s = new_morph_signal("c", SignalType::Continuous);
        let json = signal_to_json(&s);
        assert!(json.contains("\"type\":\"continuous\""));
    }

    #[test]
    fn test_json_pulse() {
        let s = new_morph_signal("p", SignalType::Pulse);
        let json = signal_to_json(&s);
        assert!(json.contains("\"type\":\"pulse\""));
    }
}
