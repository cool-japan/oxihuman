// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Color ramp (gradient stop) export.

/// A single color stop.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ColorStop {
    pub position: f32,
    pub color: [f32; 4],
}

/// Color ramp export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ColorRampExport {
    pub stops: Vec<ColorStop>,
}

#[allow(dead_code)]
pub fn new_color_ramp() -> ColorRampExport {
    ColorRampExport { stops: Vec::new() }
}

#[allow(dead_code)]
pub fn ramp_add_stop(r: &mut ColorRampExport, pos: f32, color: [f32; 4]) {
    r.stops.push(ColorStop {
        position: pos.clamp(0.0, 1.0),
        color,
    });
    r.stops.sort_by(|a, b| {
        a.position
            .partial_cmp(&b.position)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

#[allow(dead_code)]
pub fn ramp_stop_count(r: &ColorRampExport) -> usize {
    r.stops.len()
}

#[allow(dead_code)]
pub fn ramp_evaluate(r: &ColorRampExport, t: f32) -> [f32; 4] {
    if r.stops.is_empty() {
        return [0.0; 4];
    }
    if r.stops.len() == 1 || t <= r.stops[0].position {
        return r.stops[0].color;
    }
    if let Some(last) = r.stops.last() {
        if t >= last.position {
            return last.color;
        }
    }
    for w in r.stops.windows(2) {
        if t >= w[0].position && t <= w[1].position {
            let f = (t - w[0].position) / (w[1].position - w[0].position);
            let mut c = [0.0f32; 4];
            for (i, ci) in c.iter_mut().enumerate() {
                *ci = w[0].color[i] * (1.0 - f) + w[1].color[i] * f;
            }
            return c;
        }
    }
    r.stops.last().map_or([0.0; 4], |s| s.color)
}

#[allow(dead_code)]
pub fn ramp_clear(r: &mut ColorRampExport) {
    r.stops.clear();
}

#[allow(dead_code)]
pub fn ramp_validate(r: &ColorRampExport) -> bool {
    r.stops.windows(2).all(|w| w[0].position <= w[1].position)
}

#[allow(dead_code)]
pub fn color_ramp_to_json(r: &ColorRampExport) -> String {
    format!("{{\"stop_count\":{}}}", r.stops.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        assert_eq!(ramp_stop_count(&new_color_ramp()), 0);
    }

    #[test]
    fn test_add_stop() {
        let mut r = new_color_ramp();
        ramp_add_stop(&mut r, 0.0, [0.0; 4]);
        assert_eq!(ramp_stop_count(&r), 1);
    }

    #[test]
    fn test_sorted() {
        let mut r = new_color_ramp();
        ramp_add_stop(&mut r, 0.8, [1.0; 4]);
        ramp_add_stop(&mut r, 0.2, [0.0; 4]);
        assert!(r.stops[0].position < r.stops[1].position);
    }

    #[test]
    fn test_evaluate_single() {
        let mut r = new_color_ramp();
        ramp_add_stop(&mut r, 0.5, [1.0, 0.0, 0.0, 1.0]);
        let c = ramp_evaluate(&r, 0.5);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_lerp() {
        let mut r = new_color_ramp();
        ramp_add_stop(&mut r, 0.0, [0.0; 4]);
        ramp_add_stop(&mut r, 1.0, [1.0; 4]);
        let c = ramp_evaluate(&r, 0.5);
        assert!((c[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_empty() {
        let c = ramp_evaluate(&new_color_ramp(), 0.5);
        assert!((c[0]).abs() < 1e-6);
    }

    #[test]
    fn test_clear() {
        let mut r = new_color_ramp();
        ramp_add_stop(&mut r, 0.0, [0.0; 4]);
        ramp_clear(&mut r);
        assert_eq!(ramp_stop_count(&r), 0);
    }

    #[test]
    fn test_validate() {
        assert!(ramp_validate(&new_color_ramp()));
    }

    #[test]
    fn test_to_json() {
        let r = new_color_ramp();
        assert!(color_ramp_to_json(&r).contains("\"stop_count\":0"));
    }

    #[test]
    fn test_clamp_position() {
        let mut r = new_color_ramp();
        ramp_add_stop(&mut r, 1.5, [1.0; 4]);
        assert!((r.stops[0].position - 1.0).abs() < 1e-6);
    }
}
