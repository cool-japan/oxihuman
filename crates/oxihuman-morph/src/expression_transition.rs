// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A transition between two expression states over a duration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionTransition {
    pub from: String,
    pub to: String,
    pub duration: f32,
    pub elapsed: f32,
}

/// Create a new expression transition.
#[allow(dead_code)]
pub fn new_expression_transition(from: &str, to: &str, duration: f32) -> ExpressionTransition {
    ExpressionTransition {
        from: from.to_string(),
        to: to.to_string(),
        duration: duration.max(0.001),
        elapsed: 0.0,
    }
}

/// Advance the transition by dt seconds, returning the current blend factor.
#[allow(dead_code)]
pub fn transition_update(trans: &mut ExpressionTransition, dt: f32) -> f32 {
    trans.elapsed = (trans.elapsed + dt).min(trans.duration);
    transition_progress(trans)
}

/// Return the current progress (0..=1).
#[allow(dead_code)]
pub fn transition_progress(trans: &ExpressionTransition) -> f32 {
    (trans.elapsed / trans.duration).clamp(0.0, 1.0)
}

/// Check if the transition is complete.
#[allow(dead_code)]
pub fn transition_is_complete(trans: &ExpressionTransition) -> bool {
    trans.elapsed >= trans.duration
}

/// Return the duration of the transition.
#[allow(dead_code)]
pub fn transition_duration(trans: &ExpressionTransition) -> f32 {
    trans.duration
}

/// Return the source expression name.
#[allow(dead_code)]
pub fn transition_from(trans: &ExpressionTransition) -> &str {
    &trans.from
}

/// Return the target expression name.
#[allow(dead_code)]
pub fn transition_to_expr(trans: &ExpressionTransition) -> &str {
    &trans.to
}

/// Reset the transition to the beginning.
#[allow(dead_code)]
pub fn transition_reset(trans: &mut ExpressionTransition) {
    trans.elapsed = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_transition() {
        let t = new_expression_transition("idle", "smile", 1.0);
        assert!(!transition_is_complete(&t));
    }

    #[test]
    fn progress_starts_zero() {
        let t = new_expression_transition("a", "b", 1.0);
        assert!(transition_progress(&t) < 1e-6);
    }

    #[test]
    fn update_advances() {
        let mut t = new_expression_transition("a", "b", 1.0);
        let p = transition_update(&mut t, 0.5);
        assert!((p - 0.5).abs() < 1e-6);
    }

    #[test]
    fn complete_at_end() {
        let mut t = new_expression_transition("a", "b", 1.0);
        transition_update(&mut t, 1.5);
        assert!(transition_is_complete(&t));
    }

    #[test]
    fn duration_accessor() {
        let t = new_expression_transition("a", "b", 2.0);
        assert!((transition_duration(&t) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn from_accessor() {
        let t = new_expression_transition("idle", "smile", 1.0);
        assert_eq!(transition_from(&t), "idle");
    }

    #[test]
    fn to_accessor() {
        let t = new_expression_transition("idle", "smile", 1.0);
        assert_eq!(transition_to_expr(&t), "smile");
    }

    #[test]
    fn reset_works() {
        let mut t = new_expression_transition("a", "b", 1.0);
        transition_update(&mut t, 0.5);
        transition_reset(&mut t);
        assert!(transition_progress(&t) < 1e-6);
    }

    #[test]
    fn clamped_progress() {
        let mut t = new_expression_transition("a", "b", 1.0);
        transition_update(&mut t, 5.0);
        assert!((transition_progress(&t) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn min_duration() {
        let t = new_expression_transition("a", "b", 0.0);
        assert!(transition_duration(&t) > 0.0);
    }
}
