// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Feather geometry generation.

/// Parameters controlling feather shape.
#[derive(Debug, Clone)]
pub struct FeatherParams {
    pub shaft_length: f32,
    pub max_barb_length: f32,
    pub barb_count: usize,
    pub taper: f32,
}

impl Default for FeatherParams {
    fn default() -> Self {
        Self {
            shaft_length: 1.0,
            max_barb_length: 0.4,
            barb_count: 10,
            taper: 0.8,
        }
    }
}

/// A single barb of a feather.
#[derive(Debug, Clone)]
pub struct Barb {
    pub root: [f32; 3],
    pub tip: [f32; 3],
}

impl Barb {
    /// Barb length.
    pub fn length(&self) -> f32 {
        let dx = self.tip[0] - self.root[0];
        let dy = self.tip[1] - self.root[1];
        (dx * dx + dy * dy).sqrt()
    }
}

/// Generated feather geometry.
#[derive(Debug, Clone)]
pub struct Feather {
    pub shaft_start: [f32; 3],
    pub shaft_end: [f32; 3],
    pub barbs_left: Vec<Barb>,
    pub barbs_right: Vec<Barb>,
}

impl Feather {
    /// Total barb count (both sides).
    pub fn total_barb_count(&self) -> usize {
        self.barbs_left.len() + self.barbs_right.len()
    }

    /// Average barb length across both sides.
    pub fn average_barb_length(&self) -> f32 {
        let all: Vec<&Barb> = self
            .barbs_left
            .iter()
            .chain(self.barbs_right.iter())
            .collect();
        if all.is_empty() {
            return 0.0;
        }
        let sum: f32 = all.iter().map(|b| b.length()).sum();
        sum / all.len() as f32
    }
}

/// Generate a feather mesh.
pub fn generate_feather(params: &FeatherParams, origin: [f32; 3]) -> Feather {
    let shaft_end = [origin[0], origin[1] + params.shaft_length, origin[2]];
    let mut barbs_left = Vec::new();
    let mut barbs_right = Vec::new();

    let n = params.barb_count;
    for i in 0..n {
        let t = if n <= 1 {
            0.5
        } else {
            i as f32 / (n - 1) as f32
        };
        let taper_factor = 1.0 - t * (1.0 - params.taper);
        let barb_len = params.max_barb_length * taper_factor;
        let y = origin[1] + params.shaft_length * t;
        let root = [origin[0], y, origin[2]];
        let tip_l = [origin[0] - barb_len, y, origin[2]];
        let tip_r = [origin[0] + barb_len, y, origin[2]];
        barbs_left.push(Barb { root, tip: tip_l });
        barbs_right.push(Barb { root, tip: tip_r });
    }

    Feather {
        shaft_start: origin,
        shaft_end,
        barbs_left,
        barbs_right,
    }
}

/// Count total line segments in feather (shaft + barbs).
pub fn total_segment_count(feather: &Feather) -> usize {
    1 + feather.total_barb_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_feather() -> Feather {
        generate_feather(&FeatherParams::default(), [0.0, 0.0, 0.0])
    }

    #[test]
    fn test_total_barb_count() {
        /* total barb count is 2x barb_count */
        let f = default_feather();
        assert_eq!(f.total_barb_count(), 20);
    }

    #[test]
    fn test_average_barb_length_positive() {
        /* average barb length is positive */
        let f = default_feather();
        assert!(f.average_barb_length() > 0.0);
    }

    #[test]
    fn test_total_segment_count() {
        /* segment count includes shaft */
        let f = default_feather();
        assert_eq!(total_segment_count(&f), 21);
    }

    #[test]
    fn test_shaft_length() {
        /* shaft length matches params */
        let p = FeatherParams::default();
        let f = generate_feather(&p, [0.0, 0.0, 0.0]);
        let dy = f.shaft_end[1] - f.shaft_start[1];
        assert!((dy - p.shaft_length).abs() < 1e-5);
    }

    #[test]
    fn test_barbs_symmetric() {
        /* barbs on both sides are equal in count */
        let f = default_feather();
        assert_eq!(f.barbs_left.len(), f.barbs_right.len());
    }

    #[test]
    fn test_barb_length_positive() {
        /* individual barbs have positive length */
        let f = default_feather();
        for b in &f.barbs_left {
            assert!(b.length() > 0.0);
        }
    }

    #[test]
    fn test_zero_barb_count() {
        /* feather with zero barbs has no barbs */
        let p = FeatherParams {
            barb_count: 0,
            ..Default::default()
        };
        let f = generate_feather(&p, [0.0, 0.0, 0.0]);
        assert_eq!(f.total_barb_count(), 0);
    }

    #[test]
    fn test_custom_origin() {
        /* feather starts at specified origin */
        let p = FeatherParams::default();
        let f = generate_feather(&p, [1.0, 2.0, 3.0]);
        assert!((f.shaft_start[0] - 1.0).abs() < 1e-5);
        assert!((f.shaft_start[1] - 2.0).abs() < 1e-5);
    }
}
