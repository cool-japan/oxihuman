// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Keyframe animation curve with lerp/cubic interpolation.

#![allow(dead_code)]

/// A single keyframe: (time, value).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Keyframe {
    pub time: f32,
    pub value: f32,
    /// Optional in-tangent for Hermite interpolation.
    pub tan_in: f32,
    /// Optional out-tangent for Hermite interpolation.
    pub tan_out: f32,
}

impl Keyframe {
    pub fn new(time: f32, value: f32) -> Self {
        Self {
            time,
            value,
            tan_in: 0.0,
            tan_out: 0.0,
        }
    }
    pub fn with_tangents(time: f32, value: f32, tan_in: f32, tan_out: f32) -> Self {
        Self {
            time,
            value,
            tan_in,
            tan_out,
        }
    }
}

/// Interpolation mode between keyframes.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpMode {
    Linear,
    Cubic, // Hermite
    Step,
}

/// Animation curve: sorted list of keyframes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnimCurve {
    pub keyframes: Vec<Keyframe>,
    pub interp: InterpMode,
}

#[allow(dead_code)]
impl AnimCurve {
    pub fn new(interp: InterpMode) -> Self {
        Self {
            keyframes: Vec::new(),
            interp,
        }
    }

    /// Insert a keyframe (maintained sorted by time).
    pub fn insert(&mut self, kf: Keyframe) {
        let pos = self.keyframes.partition_point(|k| k.time <= kf.time);
        self.keyframes.insert(pos, kf);
    }

    /// Remove all keyframes.
    pub fn clear(&mut self) {
        self.keyframes.clear();
    }

    /// Evaluate the curve at time t.
    pub fn evaluate(&self, t: f32) -> f32 {
        let kfs = &self.keyframes;
        if kfs.is_empty() {
            return 0.0;
        }
        if t <= kfs[0].time {
            return kfs[0].value;
        }
        if t >= kfs[kfs.len() - 1].time {
            return kfs[kfs.len() - 1].value;
        }
        // Find the segment
        let i = kfs.partition_point(|k| k.time <= t) - 1;
        let k0 = &kfs[i];
        let k1 = &kfs[i + 1];
        let dt = k1.time - k0.time;
        if dt < 1e-10 {
            return k0.value;
        }
        let s = (t - k0.time) / dt;
        match self.interp {
            InterpMode::Step => k0.value,
            InterpMode::Linear => k0.value + s * (k1.value - k0.value),
            InterpMode::Cubic => {
                // Hermite spline with stored tangents
                let h00 = 2.0 * s * s * s - 3.0 * s * s + 1.0;
                let h10 = s * s * s - 2.0 * s * s + s;
                let h01 = -2.0 * s * s * s + 3.0 * s * s;
                let h11 = s * s * s - s * s;
                h00 * k0.value + h10 * dt * k0.tan_out + h01 * k1.value + h11 * dt * k1.tan_in
            }
        }
    }

    /// Duration of the curve (time of last minus first keyframe).
    pub fn duration(&self) -> f32 {
        if self.keyframes.len() < 2 {
            return 0.0;
        }
        self.keyframes[self.keyframes.len() - 1].time - self.keyframes[0].time
    }

    pub fn keyframe_count(&self) -> usize {
        self.keyframes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_linear() -> AnimCurve {
        let mut c = AnimCurve::new(InterpMode::Linear);
        c.insert(Keyframe::new(0.0, 0.0));
        c.insert(Keyframe::new(1.0, 10.0));
        c
    }

    #[test]
    fn evaluate_at_start() {
        let c = simple_linear();
        assert!((c.evaluate(0.0) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn evaluate_at_end() {
        let c = simple_linear();
        assert!((c.evaluate(1.0) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn evaluate_linear_midpoint() {
        let c = simple_linear();
        assert!((c.evaluate(0.5) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn evaluate_before_start_clamps() {
        let c = simple_linear();
        assert!((c.evaluate(-1.0) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn evaluate_after_end_clamps() {
        let c = simple_linear();
        assert!((c.evaluate(2.0) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn duration() {
        let c = simple_linear();
        assert!((c.duration() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn step_mode_holds_value() {
        let mut c = AnimCurve::new(InterpMode::Step);
        c.insert(Keyframe::new(0.0, 5.0));
        c.insert(Keyframe::new(1.0, 10.0));
        // At 0.5 should hold first value
        assert!((c.evaluate(0.5) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn cubic_mode_endpoints() {
        let mut c = AnimCurve::new(InterpMode::Cubic);
        c.insert(Keyframe::with_tangents(0.0, 0.0, 0.0, 0.0));
        c.insert(Keyframe::with_tangents(1.0, 1.0, 0.0, 0.0));
        assert!((c.evaluate(0.0) - 0.0).abs() < 1e-4);
        assert!((c.evaluate(1.0) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn insert_maintains_order() {
        let mut c = AnimCurve::new(InterpMode::Linear);
        c.insert(Keyframe::new(2.0, 2.0));
        c.insert(Keyframe::new(0.0, 0.0));
        c.insert(Keyframe::new(1.0, 1.0));
        assert!(c.keyframes[0].time <= c.keyframes[1].time);
        assert!(c.keyframes[1].time <= c.keyframes[2].time);
    }

    #[test]
    fn empty_curve_returns_zero() {
        let c = AnimCurve::new(InterpMode::Linear);
        assert!((c.evaluate(0.5) - 0.0).abs() < 1e-5);
    }
}
