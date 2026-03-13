// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! RenderFence — GPU fence synchronization primitives (stub).

#![allow(dead_code)]

/// State of a fence.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FenceState {
    Unsignaled,
    Signaled,
}

/// A GPU fence for synchronization.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderFence {
    state: FenceState,
    value: u64,
    label: String,
}

/// Create a new unsignaled fence with label.
#[allow(dead_code)]
pub fn new_render_fence(label: &str) -> RenderFence {
    RenderFence {
        state: FenceState::Unsignaled,
        value: 0,
        label: label.to_owned(),
    }
}

/// Signal the fence, incrementing its value.
#[allow(dead_code)]
pub fn signal_fence(fence: &mut RenderFence) {
    fence.state = FenceState::Signaled;
    fence.value += 1;
}

/// Check whether the fence is signaled.
#[allow(dead_code)]
pub fn is_signaled(fence: &RenderFence) -> bool {
    fence.state == FenceState::Signaled
}

/// Wait for the fence to be signaled (stub — returns immediately).
#[allow(dead_code)]
pub fn wait_fence(fence: &RenderFence) -> bool {
    fence.state == FenceState::Signaled
}

/// Reset the fence to unsignaled.
#[allow(dead_code)]
pub fn reset_fence(fence: &mut RenderFence) {
    fence.state = FenceState::Unsignaled;
}

/// Return the current fence value.
#[allow(dead_code)]
pub fn fence_value(fence: &RenderFence) -> u64 {
    fence.value
}

/// Return the fence state.
#[allow(dead_code)]
pub fn fence_state(fence: &RenderFence) -> FenceState {
    fence.state
}

/// Human-readable fence description.
#[allow(dead_code)]
pub fn fence_to_string(fence: &RenderFence) -> String {
    format!(
        "Fence(label={}, state={:?}, value={})",
        fence.label, fence.state, fence.value
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_render_fence() {
        let f = new_render_fence("test");
        assert!(!is_signaled(&f));
    }

    #[test]
    fn test_signal_fence() {
        let mut f = new_render_fence("test");
        signal_fence(&mut f);
        assert!(is_signaled(&f));
    }

    #[test]
    fn test_is_signaled_false() {
        let f = new_render_fence("test");
        assert!(!is_signaled(&f));
    }

    #[test]
    fn test_wait_fence_unsignaled() {
        let f = new_render_fence("test");
        assert!(!wait_fence(&f));
    }

    #[test]
    fn test_wait_fence_signaled() {
        let mut f = new_render_fence("test");
        signal_fence(&mut f);
        assert!(wait_fence(&f));
    }

    #[test]
    fn test_reset_fence() {
        let mut f = new_render_fence("test");
        signal_fence(&mut f);
        reset_fence(&mut f);
        assert!(!is_signaled(&f));
    }

    #[test]
    fn test_fence_value() {
        let mut f = new_render_fence("test");
        assert_eq!(fence_value(&f), 0);
        signal_fence(&mut f);
        assert_eq!(fence_value(&f), 1);
        signal_fence(&mut f);
        assert_eq!(fence_value(&f), 2);
    }

    #[test]
    fn test_fence_state() {
        let f = new_render_fence("test");
        assert_eq!(fence_state(&f), FenceState::Unsignaled);
    }

    #[test]
    fn test_fence_to_string() {
        let f = new_render_fence("main");
        let s = fence_to_string(&f);
        assert!(s.contains("main"));
        assert!(s.contains("Unsignaled"));
    }

    #[test]
    fn test_fence_to_string_signaled() {
        let mut f = new_render_fence("gpu");
        signal_fence(&mut f);
        let s = fence_to_string(&f);
        assert!(s.contains("Signaled"));
        assert!(s.contains("value=1"));
    }
}
