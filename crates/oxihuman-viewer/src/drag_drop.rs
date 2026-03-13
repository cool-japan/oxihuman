// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Drag and drop state.

#![allow(dead_code)]

/// The payload carried by a drag operation.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct DragPayload {
    /// Type discriminator (application-defined).
    pub payload_type: u8,
    /// Identifier of the dragged item.
    pub id: u32,
    /// Display label shown under the cursor.
    pub label: String,
}

/// Global drag-and-drop state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DragDropState {
    /// Whether a drag is in progress.
    pub active: bool,
    /// The payload being dragged.
    pub payload: Option<DragPayload>,
    /// Screen position where the drag started.
    pub source_pos: [f32; 2],
    /// Current screen position of the pointer.
    pub current_pos: [f32; 2],
}

/// Create an idle drag-drop state.
#[allow(dead_code)]
pub fn default_drag_drop() -> DragDropState {
    DragDropState {
        active: false,
        payload: None,
        source_pos: [0.0, 0.0],
        current_pos: [0.0, 0.0],
    }
}

/// Begin a drag operation with the given payload.
#[allow(dead_code)]
pub fn begin_drag(
    state: &mut DragDropState,
    type_: u8,
    id: u32,
    label: &str,
    pos: [f32; 2],
) {
    state.active = true;
    state.source_pos = pos;
    state.current_pos = pos;
    state.payload = Some(DragPayload {
        payload_type: type_,
        id,
        label: label.to_string(),
    });
}

/// Update the current pointer position.
#[allow(dead_code)]
pub fn update_drag_pos(state: &mut DragDropState, pos: [f32; 2]) {
    if state.active {
        state.current_pos = pos;
    }
}

/// End the drag, returning the payload if there was one.
#[allow(dead_code)]
pub fn end_drag(state: &mut DragDropState) -> Option<DragPayload> {
    state.active = false;
    state.payload.take()
}

/// Return the displacement `[dx, dy]` from source to current.
#[allow(dead_code)]
pub fn drag_delta(state: &DragDropState) -> [f32; 2] {
    [
        state.current_pos[0] - state.source_pos[0],
        state.current_pos[1] - state.source_pos[1],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_idle() {
        let s = default_drag_drop();
        assert!(!s.active);
        assert!(s.payload.is_none());
    }

    #[test]
    fn test_begin_drag() {
        let mut s = default_drag_drop();
        begin_drag(&mut s, 1, 42, "Mesh", [10.0, 20.0]);
        assert!(s.active);
        assert!(s.payload.is_some());
        let p = s.payload.as_ref().expect("should succeed");
        assert_eq!(p.id, 42);
        assert_eq!(p.label, "Mesh");
    }

    #[test]
    fn test_update_drag_pos() {
        let mut s = default_drag_drop();
        begin_drag(&mut s, 0, 1, "X", [0.0, 0.0]);
        update_drag_pos(&mut s, [30.0, 40.0]);
        assert_eq!(s.current_pos, [30.0, 40.0]);
    }

    #[test]
    fn test_update_ignored_when_idle() {
        let mut s = default_drag_drop();
        update_drag_pos(&mut s, [99.0, 99.0]);
        assert_eq!(s.current_pos, [0.0, 0.0]);
    }

    #[test]
    fn test_end_drag_returns_payload() {
        let mut s = default_drag_drop();
        begin_drag(&mut s, 2, 7, "Bone", [5.0, 5.0]);
        let payload = end_drag(&mut s);
        assert!(payload.is_some());
        assert_eq!(payload.expect("should succeed").id, 7);
        assert!(!s.active);
    }

    #[test]
    fn test_end_drag_clears_payload() {
        let mut s = default_drag_drop();
        begin_drag(&mut s, 0, 1, "A", [0.0, 0.0]);
        end_drag(&mut s);
        assert!(s.payload.is_none());
    }

    #[test]
    fn test_drag_delta_zero() {
        let mut s = default_drag_drop();
        begin_drag(&mut s, 0, 1, "A", [50.0, 60.0]);
        let d = drag_delta(&s);
        assert_eq!(d, [0.0, 0.0]);
    }

    #[test]
    fn test_drag_delta_nonzero() {
        let mut s = default_drag_drop();
        begin_drag(&mut s, 0, 1, "A", [10.0, 20.0]);
        update_drag_pos(&mut s, [40.0, 60.0]);
        let d = drag_delta(&s);
        assert!((d[0] - 30.0).abs() < 1e-6);
        assert!((d[1] - 40.0).abs() < 1e-6);
    }

    #[test]
    fn test_payload_type_stored() {
        let mut s = default_drag_drop();
        begin_drag(&mut s, 5, 1, "T", [0.0, 0.0]);
        assert_eq!(s.payload.as_ref().expect("should succeed").payload_type, 5);
    }
}
