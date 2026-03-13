// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Panel resize drag state.

#![allow(dead_code)]

/// State for an ongoing panel drag-resize operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PanelDrag {
    /// Whether a drag is in progress.
    pub active: bool,
    /// Identifier of the panel being resized.
    pub panel_id: u32,
    /// Screen position where the drag started.
    pub start_pos: [f32; 2],
    /// Current screen position of the pointer.
    pub current_pos: [f32; 2],
    /// Minimum allowed panel size.
    pub min_size: f32,
    /// Maximum allowed panel size.
    pub max_size: f32,
}

/// Create a default (inactive) panel drag state.
#[allow(dead_code)]
pub fn default_panel_drag() -> PanelDrag {
    PanelDrag {
        active: false,
        panel_id: 0,
        start_pos: [0.0, 0.0],
        current_pos: [0.0, 0.0],
        min_size: 50.0,
        max_size: 2000.0,
    }
}

/// Begin a panel drag operation.
#[allow(dead_code)]
pub fn start_drag(drag: &mut PanelDrag, panel_id: u32, pos: [f32; 2]) {
    drag.active = true;
    drag.panel_id = panel_id;
    drag.start_pos = pos;
    drag.current_pos = pos;
}

/// Update the current pointer position during a drag.
#[allow(dead_code)]
pub fn update_drag(drag: &mut PanelDrag, pos: [f32; 2]) {
    if drag.active {
        drag.current_pos = pos;
    }
}

/// End the drag operation.
#[allow(dead_code)]
pub fn end_drag(drag: &mut PanelDrag) {
    drag.active = false;
}

/// Return the delta displacement `[dx, dy]` from start to current position.
#[allow(dead_code)]
pub fn drag_delta(drag: &PanelDrag) -> [f32; 2] {
    [
        drag.current_pos[0] - drag.start_pos[0],
        drag.current_pos[1] - drag.start_pos[1],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_inactive() {
        let d = default_panel_drag();
        assert!(!d.active);
        assert_eq!(d.panel_id, 0);
    }

    #[test]
    fn test_start_drag() {
        let mut d = default_panel_drag();
        start_drag(&mut d, 5, [100.0, 200.0]);
        assert!(d.active);
        assert_eq!(d.panel_id, 5);
        assert_eq!(d.start_pos, [100.0, 200.0]);
    }

    #[test]
    fn test_update_drag() {
        let mut d = default_panel_drag();
        start_drag(&mut d, 1, [10.0, 10.0]);
        update_drag(&mut d, [50.0, 30.0]);
        assert_eq!(d.current_pos, [50.0, 30.0]);
    }

    #[test]
    fn test_update_no_effect_when_inactive() {
        let mut d = default_panel_drag();
        update_drag(&mut d, [99.0, 99.0]);
        assert_eq!(d.current_pos, [0.0, 0.0]);
    }

    #[test]
    fn test_end_drag() {
        let mut d = default_panel_drag();
        start_drag(&mut d, 2, [0.0, 0.0]);
        end_drag(&mut d);
        assert!(!d.active);
    }

    #[test]
    fn test_drag_delta_zero_at_start() {
        let mut d = default_panel_drag();
        start_drag(&mut d, 1, [20.0, 40.0]);
        let delta = drag_delta(&d);
        assert_eq!(delta, [0.0, 0.0]);
    }

    #[test]
    fn test_drag_delta_nonzero() {
        let mut d = default_panel_drag();
        start_drag(&mut d, 1, [10.0, 20.0]);
        update_drag(&mut d, [30.0, 50.0]);
        let delta = drag_delta(&d);
        assert!((delta[0] - 20.0).abs() < 1e-6);
        assert!((delta[1] - 30.0).abs() < 1e-6);
    }

    #[test]
    fn test_default_size_limits() {
        let d = default_panel_drag();
        assert_eq!(d.min_size, 50.0);
        assert_eq!(d.max_size, 2000.0);
    }

    #[test]
    fn test_negative_delta() {
        let mut d = default_panel_drag();
        start_drag(&mut d, 1, [100.0, 100.0]);
        update_drag(&mut d, [60.0, 40.0]);
        let delta = drag_delta(&d);
        assert!(delta[0] < 0.0);
        assert!(delta[1] < 0.0);
    }
}
