// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Gradient export: linear/radial gradient data.

/// Gradient type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GradientType {
    Linear,
    Radial,
}

/// Gradient stop.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GradientStop {
    pub t: f32,
    pub color: [f32; 4],
}

/// Gradient export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GradientExport {
    pub gradient_type: GradientType,
    pub stops: Vec<GradientStop>,
}

#[allow(dead_code)]
pub fn new_gradient(gtype: GradientType) -> GradientExport {
    GradientExport {
        gradient_type: gtype,
        stops: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn grad_add_stop(g: &mut GradientExport, t: f32, color: [f32; 4]) {
    g.stops.push(GradientStop {
        t: t.clamp(0.0, 1.0),
        color,
    });
    g.stops
        .sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));
}

#[allow(dead_code)]
pub fn grad_stop_count(g: &GradientExport) -> usize {
    g.stops.len()
}

#[allow(dead_code)]
pub fn grad_sample(g: &GradientExport, t: f32) -> [f32; 4] {
    if g.stops.is_empty() {
        return [0.0; 4];
    }
    if g.stops.len() == 1 || t <= g.stops[0].t {
        return g.stops[0].color;
    }
    if let Some(last) = g.stops.last() {
        if t >= last.t {
            return last.color;
        }
    }
    for w in g.stops.windows(2) {
        if t >= w[0].t && t <= w[1].t {
            let f = (t - w[0].t) / (w[1].t - w[0].t);
            let mut c = [0.0f32; 4];
            for (i, ci) in c.iter_mut().enumerate() {
                *ci = w[0].color[i] * (1.0 - f) + w[1].color[i] * f;
            }
            return c;
        }
    }
    g.stops.last().map_or([0.0; 4], |s| s.color)
}

#[allow(dead_code)]
pub fn grad_type_name(g: &GradientExport) -> &str {
    match g.gradient_type {
        GradientType::Linear => "linear",
        GradientType::Radial => "radial",
    }
}

#[allow(dead_code)]
pub fn grad_clear(g: &mut GradientExport) {
    g.stops.clear();
}

#[allow(dead_code)]
pub fn gradient_to_json(g: &GradientExport) -> String {
    format!(
        "{{\"type\":\"{}\",\"stops\":{}}}",
        grad_type_name(g),
        g.stops.len()
    )
}

#[allow(dead_code)]
pub fn grad_validate(g: &GradientExport) -> bool {
    g.stops.windows(2).all(|w| w[0].t <= w[1].t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_linear() {
        assert_eq!(
            grad_type_name(&new_gradient(GradientType::Linear)),
            "linear"
        );
    }

    #[test]
    fn test_add_stop() {
        let mut g = new_gradient(GradientType::Linear);
        grad_add_stop(&mut g, 0.5, [1.0; 4]);
        assert_eq!(grad_stop_count(&g), 1);
    }

    #[test]
    fn test_sample_lerp() {
        let mut g = new_gradient(GradientType::Linear);
        grad_add_stop(&mut g, 0.0, [0.0; 4]);
        grad_add_stop(&mut g, 1.0, [1.0; 4]);
        let c = grad_sample(&g, 0.5);
        assert!((c[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_sample_empty() {
        assert!((grad_sample(&new_gradient(GradientType::Radial), 0.5)[0]).abs() < 1e-6);
    }

    #[test]
    fn test_clear() {
        let mut g = new_gradient(GradientType::Linear);
        grad_add_stop(&mut g, 0.0, [0.0; 4]);
        grad_clear(&mut g);
        assert_eq!(grad_stop_count(&g), 0);
    }

    #[test]
    fn test_validate() {
        let mut g = new_gradient(GradientType::Linear);
        grad_add_stop(&mut g, 0.0, [0.0; 4]);
        grad_add_stop(&mut g, 1.0, [1.0; 4]);
        assert!(grad_validate(&g));
    }

    #[test]
    fn test_to_json() {
        assert!(
            gradient_to_json(&new_gradient(GradientType::Radial)).contains("\"type\":\"radial\"")
        );
    }

    #[test]
    fn test_radial_type() {
        assert_eq!(
            grad_type_name(&new_gradient(GradientType::Radial)),
            "radial"
        );
    }

    #[test]
    fn test_sample_before_first() {
        let mut g = new_gradient(GradientType::Linear);
        grad_add_stop(&mut g, 0.5, [1.0; 4]);
        let c = grad_sample(&g, 0.0);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_sorted_insertion() {
        let mut g = new_gradient(GradientType::Linear);
        grad_add_stop(&mut g, 0.9, [1.0; 4]);
        grad_add_stop(&mut g, 0.1, [0.0; 4]);
        assert!(g.stops[0].t < g.stops[1].t);
    }
}
