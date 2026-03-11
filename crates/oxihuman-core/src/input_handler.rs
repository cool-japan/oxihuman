// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Input event handling system (keyboard, mouse, gamepad stubs).
//!
//! Provides a unified input event queue, state tracking, and key action bindings
//! for keyboard, mouse, scroll, and gamepad axis inputs.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum InputKind {
    KeyPress,
    KeyRelease,
    MouseMove,
    MouseButton,
    Scroll,
    GamepadAxis,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InputEvent {
    pub kind: InputKind,
    /// Key code or button index (0 if not applicable).
    pub key_code: u32,
    /// Axis values: [x, y] for mouse/gamepad, [delta] for scroll.
    pub axis: [f32; 2],
    /// Extra flag (e.g., button pressed = true).
    pub pressed: bool,
    /// Monotonic timestamp in seconds.
    pub timestamp: f64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InputState {
    pub keys_down: HashMap<u32, bool>,
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub mouse_prev_x: f32,
    pub mouse_prev_y: f32,
    pub scroll_x: f32,
    pub scroll_y: f32,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct InputHandler {
    pub events: Vec<InputEvent>,
    pub state: InputState,
    /// Map from key_code → action name.
    pub bindings: HashMap<u32, String>,
    pub consumed: Vec<bool>,
}

/// Creates a new `InputHandler` with empty state.
#[allow(dead_code)]
pub fn new_input_handler() -> InputHandler {
    InputHandler {
        events: Vec::new(),
        state: InputState {
            keys_down: HashMap::new(),
            mouse_x: 0.0,
            mouse_y: 0.0,
            mouse_prev_x: 0.0,
            mouse_prev_y: 0.0,
            scroll_x: 0.0,
            scroll_y: 0.0,
        },
        bindings: HashMap::new(),
        consumed: Vec::new(),
    }
}

/// Pushes a new event into the handler's queue, updating state accordingly.
#[allow(dead_code)]
pub fn push_event(handler: &mut InputHandler, event: InputEvent) {
    match event.kind {
        InputKind::KeyPress => {
            handler.state.keys_down.insert(event.key_code, true);
        }
        InputKind::KeyRelease => {
            handler.state.keys_down.insert(event.key_code, false);
        }
        InputKind::MouseMove => {
            handler.state.mouse_prev_x = handler.state.mouse_x;
            handler.state.mouse_prev_y = handler.state.mouse_y;
            handler.state.mouse_x = event.axis[0];
            handler.state.mouse_y = event.axis[1];
        }
        InputKind::Scroll => {
            handler.state.scroll_x += event.axis[0];
            handler.state.scroll_y += event.axis[1];
        }
        _ => {}
    }
    handler.consumed.push(false);
    handler.events.push(event);
}

/// Returns a snapshot of all unprocessed events (clones).
#[allow(dead_code)]
pub fn poll_events(handler: &InputHandler) -> Vec<InputEvent> {
    handler
        .events
        .iter()
        .enumerate()
        .filter(|(i, _)| !handler.consumed[*i])
        .map(|(_, e)| e.clone())
        .collect()
}

/// Returns true if the given key code is currently pressed.
#[allow(dead_code)]
pub fn key_is_down(handler: &InputHandler, key_code: u32) -> bool {
    *handler.state.keys_down.get(&key_code).unwrap_or(&false)
}

/// Returns the current mouse position as `(x, y)`.
#[allow(dead_code)]
pub fn mouse_position(handler: &InputHandler) -> (f32, f32) {
    (handler.state.mouse_x, handler.state.mouse_y)
}

/// Returns the mouse delta (movement) since the last mouse move event.
#[allow(dead_code)]
pub fn mouse_delta(handler: &InputHandler) -> (f32, f32) {
    (
        handler.state.mouse_x - handler.state.mouse_prev_x,
        handler.state.mouse_y - handler.state.mouse_prev_y,
    )
}

/// Returns the accumulated scroll delta as `(x, y)`.
#[allow(dead_code)]
pub fn scroll_delta(handler: &InputHandler) -> (f32, f32) {
    (handler.state.scroll_x, handler.state.scroll_y)
}

/// Clears all events and resets scroll accumulation.
#[allow(dead_code)]
pub fn clear_events(handler: &mut InputHandler) {
    handler.events.clear();
    handler.consumed.clear();
    handler.state.scroll_x = 0.0;
    handler.state.scroll_y = 0.0;
}

/// Returns the total number of events in the queue (including consumed).
#[allow(dead_code)]
pub fn event_count(handler: &InputHandler) -> usize {
    handler.events.len()
}

/// Binds a key code to an action name string.
#[allow(dead_code)]
pub fn bind_key_action(handler: &mut InputHandler, key_code: u32, action: impl Into<String>) {
    handler.bindings.insert(key_code, action.into());
}

/// Removes the binding for a given key code.
#[allow(dead_code)]
pub fn unbind_key_action(handler: &mut InputHandler, key_code: u32) {
    handler.bindings.remove(&key_code);
}

/// Serializes handler state to a minimal JSON string.
#[allow(dead_code)]
pub fn input_handler_to_json(handler: &InputHandler) -> String {
    let keys_down_count = handler.state.keys_down.values().filter(|&&v| v).count();
    format!(
        r#"{{"event_count":{},"keys_down":{},"mouse_x":{:.2},"mouse_y":{:.2},"bindings":{}}}"#,
        handler.events.len(),
        keys_down_count,
        handler.state.mouse_x,
        handler.state.mouse_y,
        handler.bindings.len(),
    )
}

/// Marks a specific event by index as consumed.
#[allow(dead_code)]
pub fn consume_event(handler: &mut InputHandler, index: usize) {
    if index < handler.consumed.len() {
        handler.consumed[index] = true;
    }
}

/// Returns true if any key is currently held down.
#[allow(dead_code)]
pub fn any_key_down(handler: &InputHandler) -> bool {
    handler.state.keys_down.values().any(|&v| v)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key_press(key_code: u32) -> InputEvent {
        InputEvent {
            kind: InputKind::KeyPress,
            key_code,
            axis: [0.0, 0.0],
            pressed: true,
            timestamp: 0.0,
        }
    }

    fn make_key_release(key_code: u32) -> InputEvent {
        InputEvent {
            kind: InputKind::KeyRelease,
            key_code,
            axis: [0.0, 0.0],
            pressed: false,
            timestamp: 0.0,
        }
    }

    fn make_mouse_move(x: f32, y: f32) -> InputEvent {
        InputEvent {
            kind: InputKind::MouseMove,
            key_code: 0,
            axis: [x, y],
            pressed: false,
            timestamp: 0.0,
        }
    }

    fn make_scroll(dx: f32, dy: f32) -> InputEvent {
        InputEvent {
            kind: InputKind::Scroll,
            key_code: 0,
            axis: [dx, dy],
            pressed: false,
            timestamp: 0.0,
        }
    }

    #[test]
    fn test_new_input_handler() {
        let h = new_input_handler();
        assert_eq!(event_count(&h), 0);
        assert!(!any_key_down(&h));
    }

    #[test]
    fn test_push_key_press() {
        let mut h = new_input_handler();
        push_event(&mut h, make_key_press(65));
        assert!(key_is_down(&h, 65));
        assert_eq!(event_count(&h), 1);
    }

    #[test]
    fn test_push_key_release() {
        let mut h = new_input_handler();
        push_event(&mut h, make_key_press(65));
        push_event(&mut h, make_key_release(65));
        assert!(!key_is_down(&h, 65));
    }

    #[test]
    fn test_key_is_down_unknown() {
        let h = new_input_handler();
        assert!(!key_is_down(&h, 999));
    }

    #[test]
    fn test_any_key_down_true() {
        let mut h = new_input_handler();
        push_event(&mut h, make_key_press(32));
        assert!(any_key_down(&h));
    }

    #[test]
    fn test_any_key_down_false() {
        let h = new_input_handler();
        assert!(!any_key_down(&h));
    }

    #[test]
    fn test_mouse_move() {
        let mut h = new_input_handler();
        push_event(&mut h, make_mouse_move(100.0, 200.0));
        let (x, y) = mouse_position(&h);
        assert!((x - 100.0).abs() < 1e-5);
        assert!((y - 200.0).abs() < 1e-5);
    }

    #[test]
    fn test_mouse_delta() {
        let mut h = new_input_handler();
        push_event(&mut h, make_mouse_move(10.0, 20.0));
        push_event(&mut h, make_mouse_move(15.0, 25.0));
        let (dx, dy) = mouse_delta(&h);
        assert!((dx - 5.0).abs() < 1e-5);
        assert!((dy - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_scroll_delta() {
        let mut h = new_input_handler();
        push_event(&mut h, make_scroll(0.0, 3.0));
        push_event(&mut h, make_scroll(0.0, -1.0));
        let (_, sy) = scroll_delta(&h);
        assert!((sy - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_clear_events() {
        let mut h = new_input_handler();
        push_event(&mut h, make_key_press(65));
        push_event(&mut h, make_scroll(1.0, 2.0));
        clear_events(&mut h);
        assert_eq!(event_count(&h), 0);
        let (_, sy) = scroll_delta(&h);
        assert_eq!(sy, 0.0);
    }

    #[test]
    fn test_poll_events() {
        let mut h = new_input_handler();
        push_event(&mut h, make_key_press(65));
        push_event(&mut h, make_key_press(66));
        let polled = poll_events(&h);
        assert_eq!(polled.len(), 2);
    }

    #[test]
    fn test_consume_event() {
        let mut h = new_input_handler();
        push_event(&mut h, make_key_press(65));
        push_event(&mut h, make_key_press(66));
        consume_event(&mut h, 0);
        let polled = poll_events(&h);
        assert_eq!(polled.len(), 1);
        assert_eq!(polled[0].key_code, 66);
    }

    #[test]
    fn test_bind_key_action() {
        let mut h = new_input_handler();
        bind_key_action(&mut h, 65, "jump");
        assert_eq!(h.bindings.get(&65).map(String::as_str), Some("jump"));
    }

    #[test]
    fn test_unbind_key_action() {
        let mut h = new_input_handler();
        bind_key_action(&mut h, 65, "jump");
        unbind_key_action(&mut h, 65);
        assert!(!h.bindings.contains_key(&65));
    }

    #[test]
    fn test_input_handler_to_json() {
        let mut h = new_input_handler();
        push_event(&mut h, make_key_press(65));
        let json = input_handler_to_json(&h);
        assert!(json.contains("event_count"));
        assert!(json.contains("keys_down"));
    }

    #[test]
    fn test_event_count_multiple() {
        let mut h = new_input_handler();
        for i in 0..5u32 {
            push_event(&mut h, make_key_press(i));
        }
        assert_eq!(event_count(&h), 5);
    }

    #[test]
    fn test_gamepad_axis_event() {
        let mut h = new_input_handler();
        let ev = InputEvent {
            kind: InputKind::GamepadAxis,
            key_code: 0,
            axis: [0.5, -0.3],
            pressed: false,
            timestamp: 1.0,
        };
        push_event(&mut h, ev);
        assert_eq!(event_count(&h), 1);
    }

    #[test]
    fn test_mouse_button_event() {
        let mut h = new_input_handler();
        let ev = InputEvent {
            kind: InputKind::MouseButton,
            key_code: 0,
            axis: [0.0, 0.0],
            pressed: true,
            timestamp: 0.5,
        };
        push_event(&mut h, ev);
        assert_eq!(event_count(&h), 1);
    }
}
