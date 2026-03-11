// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use crate::mesh::MeshBuffers;

/// RGBA color [r, g, b, a] in 0..1
pub type Color4 = [f32; 4];
/// RGB color [r, g, b] in 0..1
pub type Color3 = [f32; 3];

/// Color ramp variants for mapping scalar values to colors.
pub enum ColorRamp {
    /// Black → white
    Grayscale,
    /// Blue → cyan → green → yellow → red
    Rainbow,
    /// Viridis-like: purple → blue → green → yellow
    Viridis,
    /// Cool: cyan → magenta
    Cool,
    /// Hot: black → red → yellow → white
    Hot,
    /// Diverging: blue (negative) → white (zero) → red (positive)
    Diverging,
    /// Custom: list of control colors at t=0..1
    Custom(Vec<(f32, Color3)>),
}

/// Per-vertex scalar heat map with associated color ramp and range.
pub struct HeatMap {
    pub scalars: Vec<f32>,
    pub ramp: ColorRamp,
    pub min_val: f32,
    pub max_val: f32,
}

impl HeatMap {
    /// Create a new HeatMap, automatically computing min/max from scalars.
    pub fn new(scalars: Vec<f32>, ramp: ColorRamp) -> Self {
        let min_val = scalars.iter().cloned().fold(f32::INFINITY, f32::min);
        let max_val = scalars.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let (min_val, max_val) = if min_val > max_val {
            (0.0, 1.0)
        } else if (min_val - max_val).abs() < f32::EPSILON {
            (min_val - 0.5, max_val + 0.5)
        } else {
            (min_val, max_val)
        };
        HeatMap {
            scalars,
            ramp,
            min_val,
            max_val,
        }
    }

    /// Create a HeatMap with an explicit scalar range.
    pub fn with_range(scalars: Vec<f32>, ramp: ColorRamp, min: f32, max: f32) -> Self {
        HeatMap {
            scalars,
            ramp,
            min_val: min,
            max_val: max,
        }
    }

    /// Normalize a scalar value to [0, 1] using the stored range.
    pub fn normalize(&self, val: f32) -> f32 {
        let range = self.max_val - self.min_val;
        if range.abs() < f32::EPSILON {
            return 0.5;
        }
        ((val - self.min_val) / range).clamp(0.0, 1.0)
    }

    /// Sample the color ramp at t in [0, 1].
    pub fn sample_ramp(&self, t: f32) -> Color3 {
        sample_ramp(&self.ramp, t)
    }

    /// Get color for vertex at index i.
    pub fn color_at(&self, i: usize) -> Color3 {
        let t = self.normalize(self.scalars[i]);
        self.sample_ramp(t)
    }

    /// Get all vertex colors.
    pub fn all_colors(&self) -> Vec<Color3> {
        self.scalars
            .iter()
            .map(|&v| {
                let t = self.normalize(v);
                self.sample_ramp(t)
            })
            .collect()
    }

    /// Get all vertex colors as Color4 with alpha=1.0.
    pub fn all_colors_rgba(&self) -> Vec<Color4> {
        self.all_colors()
            .into_iter()
            .map(|[r, g, b]| [r, g, b, 1.0])
            .collect()
    }

    /// Apply heat map colors to mesh vertex colors field.
    pub fn apply_to_mesh(&self, mesh: &mut MeshBuffers) {
        mesh.colors = Some(self.all_colors_rgba());
    }

    /// Number of vertices (scalars) in this heat map.
    pub fn vertex_count(&self) -> usize {
        self.scalars.len()
    }
}

/// Sample a named color ramp at t in [0, 1].
pub fn sample_ramp(ramp: &ColorRamp, t: f32) -> Color3 {
    let t = t.clamp(0.0, 1.0);
    match ramp {
        ColorRamp::Grayscale => [t, t, t],
        ColorRamp::Rainbow => sample_rainbow(t),
        ColorRamp::Viridis => sample_viridis(t),
        ColorRamp::Cool => lerp_color([0.0, 1.0, 1.0], [1.0, 0.0, 1.0], t),
        ColorRamp::Hot => sample_hot(t),
        ColorRamp::Diverging => sample_diverging(t),
        ColorRamp::Custom(stops) => sample_custom(stops, t),
    }
}

fn sample_rainbow(t: f32) -> Color3 {
    // Segments: blue→cyan→green→yellow→red at t=0,0.25,0.5,0.75,1.0
    let stops: &[(f32, Color3)] = &[
        (0.00, [0.0, 0.0, 1.0]),
        (0.25, [0.0, 1.0, 1.0]),
        (0.50, [0.0, 1.0, 0.0]),
        (0.75, [1.0, 1.0, 0.0]),
        (1.00, [1.0, 0.0, 0.0]),
    ];
    sample_stops(stops, t)
}

fn sample_viridis(t: f32) -> Color3 {
    let stops: &[(f32, Color3)] = &[
        (0.00, [0.267, 0.005, 0.329]),
        (0.33, [0.128, 0.567, 0.551]),
        (0.67, [0.369, 0.788, 0.383]),
        (1.00, [0.993, 0.906, 0.144]),
    ];
    sample_stops(stops, t)
}

fn sample_hot(t: f32) -> Color3 {
    // black→red (0..0.33), red→yellow (0.33..0.67), yellow→white (0.67..1)
    let stops: &[(f32, Color3)] = &[
        (0.00, [0.0, 0.0, 0.0]),
        (0.33, [1.0, 0.0, 0.0]),
        (0.67, [1.0, 1.0, 0.0]),
        (1.00, [1.0, 1.0, 1.0]),
    ];
    sample_stops(stops, t)
}

fn sample_diverging(t: f32) -> Color3 {
    let blue: Color3 = [0.0, 0.0, 1.0];
    let white: Color3 = [1.0, 1.0, 1.0];
    let red: Color3 = [1.0, 0.0, 0.0];
    if t < 0.5 {
        let local_t = t / 0.5;
        lerp_color(blue, white, local_t)
    } else {
        let local_t = (t - 0.5) / 0.5;
        lerp_color(white, red, local_t)
    }
}

fn sample_custom(stops: &[(f32, Color3)], t: f32) -> Color3 {
    if stops.is_empty() {
        return [t, t, t];
    }
    // Sort by t and interpolate
    let mut sorted: Vec<(f32, Color3)> = stops.to_vec();
    sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    sample_stops_slice(&sorted, t)
}

fn sample_stops(stops: &[(f32, Color3)], t: f32) -> Color3 {
    if stops.is_empty() {
        return [t, t, t];
    }
    if t <= stops[0].0 {
        return stops[0].1;
    }
    if t >= stops[stops.len() - 1].0 {
        return stops[stops.len() - 1].1;
    }
    for i in 0..stops.len() - 1 {
        let (t0, c0) = stops[i];
        let (t1, c1) = stops[i + 1];
        if t >= t0 && t <= t1 {
            let seg_t = if (t1 - t0).abs() < f32::EPSILON {
                0.0
            } else {
                (t - t0) / (t1 - t0)
            };
            return lerp_color(c0, c1, seg_t);
        }
    }
    stops[stops.len() - 1].1
}

fn sample_stops_slice(stops: &[(f32, Color3)], t: f32) -> Color3 {
    sample_stops(stops, t)
}

/// Convert scalar slice to colors using auto min/max range.
pub fn scalars_to_colors(scalars: &[f32], ramp: &ColorRamp) -> Vec<Color3> {
    if scalars.is_empty() {
        return Vec::new();
    }
    let min_val = scalars.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_val = scalars.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    scalars_to_colors_range(scalars, ramp, min_val, max_val)
}

/// Convert scalar slice to colors with explicit range.
pub fn scalars_to_colors_range(
    scalars: &[f32],
    ramp: &ColorRamp,
    min: f32,
    max: f32,
) -> Vec<Color3> {
    let range = max - min;
    scalars
        .iter()
        .map(|&v| {
            let t = if range.abs() < f32::EPSILON {
                0.5
            } else {
                ((v - min) / range).clamp(0.0, 1.0)
            };
            sample_ramp(ramp, t)
        })
        .collect()
}

/// Linearly interpolate between two Color3 values.
pub fn lerp_color(a: Color3, b: Color3, t: f32) -> Color3 {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Convert Color3 (0..1 floats) to [u8; 3] (0..255).
pub fn color3_to_u8(c: Color3) -> [u8; 3] {
    [
        (c[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (c[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (c[2].clamp(0.0, 1.0) * 255.0).round() as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn make_mesh(n: usize) -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0f32, 0.0, 0.0]; n],
            normals: vec![[0.0f32, 0.0, 1.0]; n],
            uvs: vec![[0.0f32, 0.0]; n],
            indices: vec![],
            has_suit: false,
        })
    }

    #[test]
    fn test_grayscale_ramp() {
        let c = sample_ramp(&ColorRamp::Grayscale, 0.0);
        assert_eq!(c, [0.0, 0.0, 0.0]);
        let c = sample_ramp(&ColorRamp::Grayscale, 1.0);
        assert_eq!(c, [1.0, 1.0, 1.0]);
        let c = sample_ramp(&ColorRamp::Grayscale, 0.5);
        assert!((c[0] - 0.5).abs() < 1e-5);
        assert_eq!(c[0], c[1]);
        assert_eq!(c[1], c[2]);
    }

    #[test]
    fn test_rainbow_ramp_endpoints() {
        let blue = sample_ramp(&ColorRamp::Rainbow, 0.0);
        assert!((blue[0]).abs() < 1e-5);
        assert!((blue[1]).abs() < 1e-5);
        assert!((blue[2] - 1.0).abs() < 1e-5);

        let red = sample_ramp(&ColorRamp::Rainbow, 1.0);
        assert!((red[0] - 1.0).abs() < 1e-5);
        assert!((red[1]).abs() < 1e-5);
        assert!((red[2]).abs() < 1e-5);

        // midpoint should be green
        let green = sample_ramp(&ColorRamp::Rainbow, 0.5);
        assert!((green[0]).abs() < 1e-5);
        assert!((green[1] - 1.0).abs() < 1e-5);
        assert!((green[2]).abs() < 1e-5);
    }

    #[test]
    fn test_viridis_ramp() {
        let start = sample_ramp(&ColorRamp::Viridis, 0.0);
        assert!((start[0] - 0.267).abs() < 1e-3);
        assert!((start[1] - 0.005).abs() < 1e-3);
        assert!((start[2] - 0.329).abs() < 1e-3);

        let end = sample_ramp(&ColorRamp::Viridis, 1.0);
        assert!((end[0] - 0.993).abs() < 1e-3);
        assert!((end[1] - 0.906).abs() < 1e-3);
        assert!((end[2] - 0.144).abs() < 1e-3);
    }

    #[test]
    fn test_cool_ramp() {
        let start = sample_ramp(&ColorRamp::Cool, 0.0);
        assert!((start[0]).abs() < 1e-5);
        assert!((start[1] - 1.0).abs() < 1e-5);
        assert!((start[2] - 1.0).abs() < 1e-5);

        let end = sample_ramp(&ColorRamp::Cool, 1.0);
        assert!((end[0] - 1.0).abs() < 1e-5);
        assert!((end[1]).abs() < 1e-5);
        assert!((end[2] - 1.0).abs() < 1e-5);

        // midpoint
        let mid = sample_ramp(&ColorRamp::Cool, 0.5);
        assert!((mid[0] - 0.5).abs() < 1e-5);
        assert!((mid[1] - 0.5).abs() < 1e-5);
        assert!((mid[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_hot_ramp() {
        let black = sample_ramp(&ColorRamp::Hot, 0.0);
        assert!((black[0]).abs() < 1e-5);
        assert!((black[1]).abs() < 1e-5);
        assert!((black[2]).abs() < 1e-5);

        let white = sample_ramp(&ColorRamp::Hot, 1.0);
        assert!((white[0] - 1.0).abs() < 1e-5);
        assert!((white[1] - 1.0).abs() < 1e-5);
        assert!((white[2] - 1.0).abs() < 1e-5);

        // near t=0.33 should be reddish
        let red_area = sample_ramp(&ColorRamp::Hot, 0.33);
        assert!(red_area[0] > 0.9);
    }

    #[test]
    fn test_diverging_ramp_midpoint() {
        let mid = sample_ramp(&ColorRamp::Diverging, 0.5);
        assert!((mid[0] - 1.0).abs() < 1e-5);
        assert!((mid[1] - 1.0).abs() < 1e-5);
        assert!((mid[2] - 1.0).abs() < 1e-5);

        let start = sample_ramp(&ColorRamp::Diverging, 0.0);
        assert!((start[2] - 1.0).abs() < 1e-5); // blue end

        let end = sample_ramp(&ColorRamp::Diverging, 1.0);
        assert!((end[0] - 1.0).abs() < 1e-5); // red end
    }

    #[test]
    fn test_heat_map_new_auto_range() {
        let scalars = vec![0.0f32, 1.0, 2.0, 3.0, 4.0];
        let hm = HeatMap::new(scalars, ColorRamp::Grayscale);
        assert!((hm.min_val - 0.0).abs() < 1e-5);
        assert!((hm.max_val - 4.0).abs() < 1e-5);
        assert_eq!(hm.vertex_count(), 5);
    }

    #[test]
    fn test_heat_map_normalize() {
        let hm = HeatMap::with_range(vec![0.0, 5.0, 10.0], ColorRamp::Grayscale, 0.0, 10.0);
        assert!((hm.normalize(0.0) - 0.0).abs() < 1e-5);
        assert!((hm.normalize(5.0) - 0.5).abs() < 1e-5);
        assert!((hm.normalize(10.0) - 1.0).abs() < 1e-5);
        // clamp below min
        assert!((hm.normalize(-5.0) - 0.0).abs() < 1e-5);
        // clamp above max
        assert!((hm.normalize(20.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_heat_map_color_at() {
        let scalars = vec![0.0f32, 10.0];
        let hm = HeatMap::with_range(scalars, ColorRamp::Grayscale, 0.0, 10.0);
        let c0 = hm.color_at(0);
        assert!((c0[0]).abs() < 1e-5); // black
        let c1 = hm.color_at(1);
        assert!((c1[0] - 1.0).abs() < 1e-5); // white
    }

    #[test]
    fn test_scalars_to_colors() {
        let scalars = vec![0.0f32, 5.0, 10.0];
        let colors = scalars_to_colors(&scalars, &ColorRamp::Grayscale);
        assert_eq!(colors.len(), 3);
        assert!((colors[0][0]).abs() < 1e-5); // black
        assert!((colors[1][0] - 0.5).abs() < 1e-5); // mid gray
        assert!((colors[2][0] - 1.0).abs() < 1e-5); // white
    }

    #[test]
    fn test_lerp_color() {
        let black: Color3 = [0.0, 0.0, 0.0];
        let white: Color3 = [1.0, 1.0, 1.0];
        let mid = lerp_color(black, white, 0.5);
        assert!((mid[0] - 0.5).abs() < 1e-5);
        assert!((mid[1] - 0.5).abs() < 1e-5);
        assert!((mid[2] - 0.5).abs() < 1e-5);

        let start = lerp_color(black, white, 0.0);
        assert_eq!(start, black);

        let end = lerp_color(black, white, 1.0);
        assert_eq!(end, white);
    }

    #[test]
    fn test_color3_to_u8() {
        let black: Color3 = [0.0, 0.0, 0.0];
        assert_eq!(color3_to_u8(black), [0, 0, 0]);

        let white: Color3 = [1.0, 1.0, 1.0];
        assert_eq!(color3_to_u8(white), [255, 255, 255]);

        let mid: Color3 = [0.5, 0.5, 0.5];
        let u8_mid = color3_to_u8(mid);
        assert!((u8_mid[0] as i32 - 128).abs() <= 1);

        // clamp out-of-range
        let over: Color3 = [2.0, -1.0, 0.5];
        let u8_over = color3_to_u8(over);
        assert_eq!(u8_over[0], 255);
        assert_eq!(u8_over[1], 0);
    }

    #[test]
    fn test_custom_ramp() {
        let stops = vec![(0.0f32, [1.0f32, 0.0, 0.0]), (1.0f32, [0.0f32, 0.0, 1.0])];
        let ramp = ColorRamp::Custom(stops);
        let start = sample_ramp(&ramp, 0.0);
        assert!((start[0] - 1.0).abs() < 1e-5);
        assert!((start[2]).abs() < 1e-5);

        let end = sample_ramp(&ramp, 1.0);
        assert!((end[0]).abs() < 1e-5);
        assert!((end[2] - 1.0).abs() < 1e-5);

        let mid = sample_ramp(&ramp, 0.5);
        assert!((mid[0] - 0.5).abs() < 1e-5);
        assert!((mid[2] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_all_colors_rgba() {
        let scalars = vec![0.0f32, 5.0, 10.0];
        let hm = HeatMap::with_range(scalars, ColorRamp::Grayscale, 0.0, 10.0);
        let rgba = hm.all_colors_rgba();
        assert_eq!(rgba.len(), 3);
        for c in &rgba {
            assert!((c[3] - 1.0).abs() < 1e-5, "alpha should be 1.0");
        }
        assert!((rgba[0][0]).abs() < 1e-5);
        assert!((rgba[2][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_apply_to_mesh() {
        let scalars = vec![0.0f32, 5.0, 10.0];
        let hm = HeatMap::with_range(scalars, ColorRamp::Grayscale, 0.0, 10.0);
        let mut mesh = make_mesh(3);
        assert!(mesh.colors.is_none());
        hm.apply_to_mesh(&mut mesh);
        let colors = mesh.colors.as_ref().unwrap();
        assert_eq!(colors.len(), 3);
        assert!((colors[0][3] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_scalars_to_colors_range_explicit() {
        let scalars = vec![2.0f32, 6.0, 10.0];
        let colors = scalars_to_colors_range(&scalars, &ColorRamp::Grayscale, 2.0, 10.0);
        assert_eq!(colors.len(), 3);
        assert!((colors[0][0]).abs() < 1e-5);
        assert!((colors[1][0] - 0.5).abs() < 1e-5);
        assert!((colors[2][0] - 1.0).abs() < 1e-5);
    }
}
