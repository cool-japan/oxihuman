// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Timed blend between two poses.

/// Configuration for a blend pose transition.
#[derive(Debug, Clone)]
pub struct BlendPoseTransition {
    pub duration: f32,
    pub elapsed: f32,
    pub from_weights: Vec<f32>,
    pub to_weights: Vec<f32>,
    pub curve: TransitionCurve,
}

/// The easing curve used during the transition.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransitionCurve {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl BlendPoseTransition {
    /// Create a new transition between two weight sets.
    pub fn new(from_weights: Vec<f32>, to_weights: Vec<f32>, duration: f32) -> Self {
        Self {
            duration: duration.max(0.0001),
            elapsed: 0.0,
            from_weights,
            to_weights,
            curve: TransitionCurve::EaseInOut,
        }
    }

    /// Advance the transition by `dt` seconds and return blended weights.
    pub fn step(&mut self, dt: f32) -> Vec<f32> {
        self.elapsed = (self.elapsed + dt).min(self.duration);
        let t = self.elapsed / self.duration;
        let t_curved = apply_curve(t, self.curve);
        blend_weights(&self.from_weights, &self.to_weights, t_curved)
    }

    /// Returns true when the transition has completed.
    pub fn is_done(&self) -> bool {
        self.elapsed >= self.duration
    }

    /// Reset the transition to the beginning.
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }
}

/// Apply a transition curve to a normalized time value.
pub fn apply_curve(t: f32, curve: TransitionCurve) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match curve {
        TransitionCurve::Linear => t,
        TransitionCurve::EaseIn => t * t,
        TransitionCurve::EaseOut => t * (2.0 - t),
        TransitionCurve::EaseInOut => t * t * (3.0 - 2.0 * t),
    }
}

/// Linear interpolation between two weight arrays.
pub fn blend_weights(from: &[f32], to: &[f32], t: f32) -> Vec<f32> {
    let len = from.len().min(to.len());
    (0..len).map(|i| from[i] + (to[i] - from[i]) * t).collect()
}

/// Build a transition with a custom curve.
pub fn new_transition_with_curve(
    from_weights: Vec<f32>,
    to_weights: Vec<f32>,
    duration: f32,
    curve: TransitionCurve,
) -> BlendPoseTransition {
    let mut tr = BlendPoseTransition::new(from_weights, to_weights, duration);
    tr.curve = curve;
    tr
}

/// Return the normalized progress (0.0–1.0) of the transition.
pub fn transition_progress(tr: &BlendPoseTransition) -> f32 {
    (tr.elapsed / tr.duration).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_transition() {
        /* verify initial state */
        let tr = BlendPoseTransition::new(vec![0.0, 0.0], vec![1.0, 1.0], 1.0);
        assert_eq!(tr.elapsed, 0.0);
        assert!(!tr.is_done());
    }

    #[test]
    fn test_step_advances() {
        /* step should advance elapsed time */
        let mut tr = BlendPoseTransition::new(vec![0.0], vec![1.0], 1.0);
        let w = tr.step(0.5);
        assert!((tr.elapsed - 0.5).abs() < 1e-6);
        assert!(w[0] > 0.0 && w[0] < 1.0);
    }

    #[test]
    fn test_is_done() {
        /* transition should finish after full duration */
        let mut tr = BlendPoseTransition::new(vec![0.0], vec![1.0], 0.1);
        tr.step(0.2);
        assert!(tr.is_done());
    }

    #[test]
    fn test_reset() {
        /* reset should bring elapsed back to zero */
        let mut tr = BlendPoseTransition::new(vec![0.0], vec![1.0], 1.0);
        tr.step(0.5);
        tr.reset();
        assert_eq!(tr.elapsed, 0.0);
    }

    #[test]
    fn test_blend_weights_midpoint() {
        /* midpoint blend should average the two arrays */
        let result = blend_weights(&[0.0, 0.0], &[1.0, 2.0], 0.5);
        assert!((result[0] - 0.5).abs() < 1e-6);
        assert!((result[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_apply_curve_linear() {
        /* linear curve returns t unchanged */
        assert!((apply_curve(0.5, TransitionCurve::Linear) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_apply_curve_ease_in_out_bounds() {
        /* ease-in-out should be 0 at 0 and 1 at 1 */
        assert!((apply_curve(0.0, TransitionCurve::EaseInOut)).abs() < 1e-6);
        assert!((apply_curve(1.0, TransitionCurve::EaseInOut) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_transition_progress() {
        /* progress should match elapsed/duration */
        let mut tr = BlendPoseTransition::new(vec![0.0], vec![1.0], 2.0);
        tr.step(1.0);
        let p = transition_progress(&tr);
        assert!((p - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_new_transition_with_curve() {
        /* custom curve should be stored */
        let tr = new_transition_with_curve(vec![0.0], vec![1.0], 1.0, TransitionCurve::EaseIn);
        assert_eq!(tr.curve, TransitionCurve::EaseIn);
    }
}
