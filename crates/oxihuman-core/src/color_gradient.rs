// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Multi-stop color gradient with sampling by t in [0, 1].

#![allow(dead_code)]

/// An RGBA color stop at position t in [0, 1].
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorStop {
    pub t: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorStop {
    pub fn new(t: f32, r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { t, r, g, b, a }
    }
    pub fn rgb(t: f32, r: f32, g: f32, b: f32) -> Self {
        Self { t, r, g, b, a: 1.0 }
    }
}

/// Multi-stop color gradient.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ColorGradient {
    stops: Vec<ColorStop>,
}

#[allow(dead_code)]
impl ColorGradient {
    pub fn new() -> Self {
        Self { stops: Vec::new() }
    }

    /// Add a color stop (will be kept sorted by t).
    pub fn add_stop(&mut self, stop: ColorStop) {
        let pos = self.stops.partition_point(|s| s.t <= stop.t);
        self.stops.insert(pos, stop);
    }

    /// Clear all stops.
    pub fn clear(&mut self) {
        self.stops.clear();
    }

    /// Number of stops.
    pub fn stop_count(&self) -> usize {
        self.stops.len()
    }

    /// Sample the gradient at t in [0, 1]. Returns (r, g, b, a).
    pub fn sample(&self, t: f32) -> (f32, f32, f32, f32) {
        if self.stops.is_empty() {
            return (0.0, 0.0, 0.0, 1.0);
        }
        if t <= self.stops[0].t {
            let s = &self.stops[0];
            return (s.r, s.g, s.b, s.a);
        }
        let last = &self.stops[self.stops.len() - 1];
        if t >= last.t {
            return (last.r, last.g, last.b, last.a);
        }
        // Find the two surrounding stops
        let hi = self.stops.partition_point(|s| s.t <= t);
        let lo = hi - 1;
        let s0 = &self.stops[lo];
        let s1 = &self.stops[hi];
        let dt = s1.t - s0.t;
        let f = if dt < 1e-10 { 0.0 } else { (t - s0.t) / dt };
        (
            lerp(s0.r, s1.r, f),
            lerp(s0.g, s1.g, f),
            lerp(s0.b, s1.b, f),
            lerp(s0.a, s1.a, f),
        )
    }

    /// Sample and return as [f32; 4] array.
    pub fn sample_arr(&self, t: f32) -> [f32; 4] {
        let (r, g, b, a) = self.sample(t);
        [r, g, b, a]
    }

    /// Build a simple two-stop gradient.
    pub fn two_stop(c0: ColorStop, c1: ColorStop) -> Self {
        let mut g = Self::new();
        g.add_stop(c0);
        g.add_stop(c1);
        g
    }
}

impl Default for ColorGradient {
    fn default() -> Self {
        Self::new()
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

/// Build a rainbow gradient (red→green→blue).
#[allow(dead_code)]
pub fn rainbow_gradient() -> ColorGradient {
    let mut g = ColorGradient::new();
    g.add_stop(ColorStop::rgb(0.0, 1.0, 0.0, 0.0));
    g.add_stop(ColorStop::rgb(0.5, 0.0, 1.0, 0.0));
    g.add_stop(ColorStop::rgb(1.0, 0.0, 0.0, 1.0));
    g
}

/// Build a grayscale gradient (black→white).
#[allow(dead_code)]
pub fn grayscale_gradient() -> ColorGradient {
    ColorGradient::two_stop(
        ColorStop::rgb(0.0, 0.0, 0.0, 0.0),
        ColorStop::rgb(1.0, 1.0, 1.0, 1.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_stop_at_zero_is_first_color() {
        let g = grayscale_gradient();
        let (r, g_, b, _) = g.sample(0.0);
        assert!(r.abs() < 1e-5 && g_.abs() < 1e-5 && b.abs() < 1e-5);
    }

    #[test]
    fn two_stop_at_one_is_last_color() {
        let g = grayscale_gradient();
        let (r, g_, b, _) = g.sample(1.0);
        assert!((r - 1.0).abs() < 1e-5 && (g_ - 1.0).abs() < 1e-5 && (b - 1.0).abs() < 1e-5);
    }

    #[test]
    fn two_stop_midpoint_is_gray() {
        let g = grayscale_gradient();
        let (r, _, _, _) = g.sample(0.5);
        assert!((r - 0.5).abs() < 1e-4);
    }

    #[test]
    fn empty_gradient_returns_opaque_black() {
        let g = ColorGradient::new();
        let (r, g_, b, a) = g.sample(0.5);
        assert!(r.abs() < 1e-5 && g_.abs() < 1e-5 && b.abs() < 1e-5 && (a - 1.0).abs() < 1e-5);
    }

    #[test]
    fn before_first_stop_clamps() {
        let g = grayscale_gradient();
        let (r, _, _, _) = g.sample(-0.5);
        assert!(r.abs() < 1e-5);
    }

    #[test]
    fn after_last_stop_clamps() {
        let g = grayscale_gradient();
        let (r, _, _, _) = g.sample(1.5);
        assert!((r - 1.0).abs() < 1e-5);
    }

    #[test]
    fn sample_arr_length() {
        let g = grayscale_gradient();
        let arr = g.sample_arr(0.5);
        assert_eq!(arr.len(), 4);
    }

    #[test]
    fn rainbow_gradient_red_at_zero() {
        let g = rainbow_gradient();
        let (r, g_, _, _) = g.sample(0.0);
        assert!(r > 0.5 && g_ < 0.1);
    }

    #[test]
    fn stop_count_correct() {
        let mut g = ColorGradient::new();
        g.add_stop(ColorStop::rgb(0.0, 1.0, 0.0, 0.0));
        g.add_stop(ColorStop::rgb(1.0, 0.0, 0.0, 1.0));
        assert_eq!(g.stop_count(), 2);
    }

    #[test]
    fn stops_sorted_after_add() {
        let mut g = ColorGradient::new();
        g.add_stop(ColorStop::rgb(1.0, 1.0, 0.0, 0.0));
        g.add_stop(ColorStop::rgb(0.0, 0.0, 1.0, 0.0));
        g.add_stop(ColorStop::rgb(0.5, 0.0, 0.0, 1.0));
        let ts: Vec<f32> = g.stops.iter().map(|s| s.t).collect();
        assert!(ts[0] <= ts[1] && ts[1] <= ts[2]);
    }
}
