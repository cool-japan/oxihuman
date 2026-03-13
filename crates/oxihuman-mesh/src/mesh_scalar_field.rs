// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// A scalar field over mesh vertices.
#[allow(dead_code)]
pub struct ScalarFieldV {
    pub values: Vec<f32>,
}

/// Build a scalar field from a height function evaluated at positions.
#[allow(dead_code)]
pub fn build_scalar_field<F>(positions: &[[f32; 3]], f: F) -> ScalarFieldV
where
    F: Fn([f32; 3]) -> f32,
{
    ScalarFieldV {
        values: positions.iter().map(|&p| f(p)).collect(),
    }
}

/// Build a signed distance field (sphere).
#[allow(dead_code)]
pub fn sdf_sphere_field(positions: &[[f32; 3]], center: [f32; 3], radius: f32) -> ScalarFieldV {
    build_scalar_field(positions, |p| {
        let dx = p[0] - center[0];
        let dy = p[1] - center[1];
        let dz = p[2] - center[2];
        (dx * dx + dy * dy + dz * dz).sqrt() - radius
    })
}

/// Get min value of the field.
#[allow(dead_code)]
pub fn field_min(sf: &ScalarFieldV) -> f32 {
    sf.values.iter().cloned().fold(f32::INFINITY, f32::min)
}

/// Get max value of the field.
#[allow(dead_code)]
pub fn field_max(sf: &ScalarFieldV) -> f32 {
    sf.values.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
}

/// Normalize field values to [0, 1].
#[allow(dead_code)]
pub fn normalize_field(sf: &ScalarFieldV) -> ScalarFieldV {
    let mn = field_min(sf);
    let mx = field_max(sf);
    let range = mx - mn;
    if range < 1e-9 {
        return ScalarFieldV {
            values: vec![0.0; sf.values.len()],
        };
    }
    ScalarFieldV {
        values: sf.values.iter().map(|&v| (v - mn) / range).collect(),
    }
}

/// Count values above a threshold.
#[allow(dead_code)]
pub fn count_above(sf: &ScalarFieldV, threshold: f32) -> usize {
    sf.values.iter().filter(|&&v| v > threshold).count()
}

/// Count values below a threshold.
#[allow(dead_code)]
pub fn count_below(sf: &ScalarFieldV, threshold: f32) -> usize {
    sf.values.iter().filter(|&&v| v < threshold).count()
}

/// Average field value.
#[allow(dead_code)]
pub fn field_avg(sf: &ScalarFieldV) -> f32 {
    if sf.values.is_empty() {
        return 0.0;
    }
    sf.values.iter().sum::<f32>() / sf.values.len() as f32
}

/// Apply a sin-based wave field for testing.
#[allow(dead_code)]
pub fn sine_field(positions: &[[f32; 3]], freq: f32) -> ScalarFieldV {
    build_scalar_field(positions, |p| (p[0] * freq * PI * 2.0).sin())
}

/// Serialize to JSON summary.
#[allow(dead_code)]
pub fn scalar_field_to_json(sf: &ScalarFieldV) -> String {
    format!(
        r#"{{"count":{},"min":{:.4},"max":{:.4},"avg":{:.4}}}"#,
        sf.values.len(),
        field_min(sf),
        field_max(sf),
        field_avg(sf)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
        ]
    }

    #[test]
    fn build_field_length() {
        let pos = sample_positions();
        let sf = build_scalar_field(&pos, |p| p[0]);
        assert_eq!(sf.values.len(), pos.len());
    }

    #[test]
    fn sdf_sphere_center_negative() {
        let pos = vec![[0.0_f32, 0.0, 0.0]];
        let sf = sdf_sphere_field(&pos, [0.0, 0.0, 0.0], 1.0);
        assert!(sf.values[0] < 0.0);
    }

    #[test]
    fn field_min_correct() {
        let sf = ScalarFieldV {
            values: vec![1.0, 3.0, 2.0],
        };
        assert!((field_min(&sf) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn field_max_correct() {
        let sf = ScalarFieldV {
            values: vec![1.0, 3.0, 2.0],
        };
        assert!((field_max(&sf) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_range() {
        let sf = ScalarFieldV {
            values: vec![2.0, 4.0, 6.0],
        };
        let n = normalize_field(&sf);
        assert!((field_min(&n) - 0.0).abs() < 1e-6);
        assert!((field_max(&n) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn count_above_threshold() {
        let sf = ScalarFieldV {
            values: vec![1.0, 2.0, 3.0],
        };
        assert_eq!(count_above(&sf, 1.5), 2);
    }

    #[test]
    fn count_below_threshold() {
        let sf = ScalarFieldV {
            values: vec![1.0, 2.0, 3.0],
        };
        assert_eq!(count_below(&sf, 2.5), 2);
    }

    #[test]
    fn avg_value() {
        let sf = ScalarFieldV {
            values: vec![1.0, 2.0, 3.0],
        };
        assert!((field_avg(&sf) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn sine_field_range() {
        let pos = sample_positions();
        let sf = sine_field(&pos, 1.0);
        assert!(field_min(&sf) >= -1.01 && field_max(&sf) <= 1.01);
    }

    #[test]
    fn json_contains_count() {
        let sf = ScalarFieldV {
            values: vec![1.0, 2.0],
        };
        let j = scalar_field_to_json(&sf);
        assert!(j.contains("\"count\":2"));
    }
}
