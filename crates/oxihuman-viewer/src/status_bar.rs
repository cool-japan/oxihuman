// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Status bar at the bottom of the viewport.

#![allow(dead_code)]

/// Status bar display state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StatusBarState {
    /// Text in the left section.
    pub left_text: String,
    /// Text in the center section.
    pub center_text: String,
    /// Text in the right section.
    pub right_text: String,
    /// Optional progress value in [0, 1].
    pub progress: Option<f32>,
}

/// Create a default empty status bar.
#[allow(dead_code)]
pub fn default_status_bar() -> StatusBarState {
    StatusBarState {
        left_text: String::new(),
        center_text: String::new(),
        right_text: String::new(),
        progress: None,
    }
}

/// Set the left section text.
#[allow(dead_code)]
pub fn set_status_left(bar: &mut StatusBarState, text: &str) {
    bar.left_text = text.to_string();
}

/// Set the right section text.
#[allow(dead_code)]
pub fn set_status_right(bar: &mut StatusBarState, text: &str) {
    bar.right_text = text.to_string();
}

/// Set a progress value clamped to [0, 1].
#[allow(dead_code)]
pub fn set_progress(bar: &mut StatusBarState, p: f32) {
    bar.progress = Some(p.clamp(0.0, 1.0));
}

/// Clear the progress indicator.
#[allow(dead_code)]
pub fn clear_progress(bar: &mut StatusBarState) {
    bar.progress = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_empty() {
        let bar = default_status_bar();
        assert!(bar.left_text.is_empty());
        assert!(bar.center_text.is_empty());
        assert!(bar.right_text.is_empty());
        assert!(bar.progress.is_none());
    }

    #[test]
    fn test_set_left() {
        let mut bar = default_status_bar();
        set_status_left(&mut bar, "Ready");
        assert_eq!(bar.left_text, "Ready");
    }

    #[test]
    fn test_set_right() {
        let mut bar = default_status_bar();
        set_status_right(&mut bar, "100 verts");
        assert_eq!(bar.right_text, "100 verts");
    }

    #[test]
    fn test_set_progress() {
        let mut bar = default_status_bar();
        set_progress(&mut bar, 0.75);
        assert_eq!(bar.progress, Some(0.75));
    }

    #[test]
    fn test_progress_clamp_high() {
        let mut bar = default_status_bar();
        set_progress(&mut bar, 2.0);
        assert_eq!(bar.progress, Some(1.0));
    }

    #[test]
    fn test_progress_clamp_low() {
        let mut bar = default_status_bar();
        set_progress(&mut bar, -1.0);
        assert_eq!(bar.progress, Some(0.0));
    }

    #[test]
    fn test_clear_progress() {
        let mut bar = default_status_bar();
        set_progress(&mut bar, 0.5);
        clear_progress(&mut bar);
        assert!(bar.progress.is_none());
    }

    #[test]
    fn test_overwrite_left() {
        let mut bar = default_status_bar();
        set_status_left(&mut bar, "First");
        set_status_left(&mut bar, "Second");
        assert_eq!(bar.left_text, "Second");
    }

    #[test]
    fn test_center_independent() {
        let mut bar = default_status_bar();
        bar.center_text = "Center".to_string();
        set_status_left(&mut bar, "Left");
        assert_eq!(bar.center_text, "Center");
    }
}
