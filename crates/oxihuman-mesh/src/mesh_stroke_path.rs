// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Generate a stroke path mesh (ribbon) from a sequence of 3-D points.

#[allow(unused_imports)]
use std::f32::consts::PI;

/// A single stroke sample.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StrokeSample {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub width: f32,
}

/// Result of a stroke path build.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StrokePathResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub uvs: Vec<[f32; 2]>,
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0].powi(2) + v[1].powi(2) + v[2].powi(2))
        .sqrt()
        .max(1e-9);
    [v[0] / l, v[1] / l, v[2] / l]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Build a ribbon mesh from a list of stroke samples.
#[allow(dead_code)]
pub fn build_stroke_path(samples: &[StrokeSample]) -> StrokePathResult {
    if samples.len() < 2 {
        return StrokePathResult {
            positions: vec![],
            indices: vec![],
            uvs: vec![],
        };
    }
    let n = samples.len();
    let mut positions = Vec::with_capacity(n * 2);
    let mut uvs = Vec::with_capacity(n * 2);
    let mut total_len = 0.0_f32;
    let mut lengths = vec![0.0_f32; n];
    for i in 1..n {
        let d = [
            samples[i].position[0] - samples[i - 1].position[0],
            samples[i].position[1] - samples[i - 1].position[1],
            samples[i].position[2] - samples[i - 1].position[2],
        ];
        total_len += (d[0].powi(2) + d[1].powi(2) + d[2].powi(2)).sqrt();
        lengths[i] = total_len;
    }
    let total_len = total_len.max(1e-9);
    for (i, s) in samples.iter().enumerate() {
        let fwd = if i + 1 < n {
            normalize3([
                samples[i + 1].position[0] - s.position[0],
                samples[i + 1].position[1] - s.position[1],
                samples[i + 1].position[2] - s.position[2],
            ])
        } else {
            normalize3([
                s.position[0] - samples[i - 1].position[0],
                s.position[1] - samples[i - 1].position[1],
                s.position[2] - samples[i - 1].position[2],
            ])
        };
        let right = normalize3(cross3(fwd, normalize3(s.normal)));
        let half = s.width * 0.5;
        let u = lengths[i] / total_len;
        positions.push([
            s.position[0] + right[0] * half,
            s.position[1] + right[1] * half,
            s.position[2] + right[2] * half,
        ]);
        positions.push([
            s.position[0] - right[0] * half,
            s.position[1] - right[1] * half,
            s.position[2] - right[2] * half,
        ]);
        uvs.push([u, 0.0]);
        uvs.push([u, 1.0]);
    }
    let mut indices = Vec::with_capacity((n - 1) * 6);
    for i in 0..(n as u32 - 1) {
        let base = i * 2;
        indices.extend_from_slice(&[base, base + 1, base + 2, base + 1, base + 3, base + 2]);
    }
    StrokePathResult {
        positions,
        indices,
        uvs,
    }
}

/// Compute the total arc length of a stroke path.
#[allow(dead_code)]
pub fn stroke_arc_length(samples: &[StrokeSample]) -> f32 {
    samples
        .windows(2)
        .map(|w| {
            let a = w[0].position;
            let b = w[1].position;
            ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
        })
        .sum()
}

/// Create a simple straight stroke.
#[allow(dead_code)]
pub fn straight_stroke(from: [f32; 3], to: [f32; 3], width: f32, n: usize) -> Vec<StrokeSample> {
    (0..n)
        .map(|i| {
            let t = i as f32 / (n - 1).max(1) as f32;
            let pos = [
                from[0] + (to[0] - from[0]) * t,
                from[1] + (to[1] - from[1]) * t,
                from[2] + (to[2] - from[2]) * t,
            ];
            StrokeSample {
                position: pos,
                normal: [0.0, 1.0, 0.0],
                width,
            }
        })
        .collect()
}

/// Count the number of faces in the stroke mesh.
#[allow(dead_code)]
pub fn stroke_face_count(res: &StrokePathResult) -> usize {
    res.indices.len() / 3
}

/// Return true if all UV coordinates are in [0, 1].
#[allow(dead_code)]
pub fn uvs_in_range(res: &StrokePathResult) -> bool {
    res.uvs
        .iter()
        .all(|uv| (0.0..=1.0).contains(&uv[0]) && (0.0..=1.0).contains(&uv[1]))
}

/// Apply a uniform scale to all widths.
#[allow(dead_code)]
pub fn scale_stroke_widths(samples: &mut [StrokeSample], scale: f32) {
    for s in samples.iter_mut() {
        s.width *= scale;
    }
}

/// Return the midpoint of the stroke.
#[allow(dead_code)]
pub fn stroke_midpoint(samples: &[StrokeSample]) -> Option<[f32; 3]> {
    if samples.is_empty() {
        return None;
    }
    let mid = samples.len() / 2;
    Some(samples[mid].position)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_stroke() -> Vec<StrokeSample> {
        straight_stroke([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.1, 5)
    }

    #[test]
    fn build_smoke() {
        let s = line_stroke();
        let res = build_stroke_path(&s);
        assert!(!res.positions.is_empty());
    }

    #[test]
    fn face_count_correct() {
        let s = line_stroke();
        let res = build_stroke_path(&s);
        assert_eq!(stroke_face_count(&res), (s.len() - 1) * 2);
    }

    #[test]
    fn uvs_in_range_test() {
        let s = line_stroke();
        let res = build_stroke_path(&s);
        assert!(uvs_in_range(&res));
    }

    #[test]
    fn arc_length_positive() {
        let s = line_stroke();
        assert!(stroke_arc_length(&s) > 0.0);
    }

    #[test]
    fn scale_widths() {
        let mut s = line_stroke();
        let orig = s[0].width;
        scale_stroke_widths(&mut s, 2.0);
        assert!((s[0].width - orig * 2.0).abs() < 1e-6);
    }

    #[test]
    fn midpoint_exists() {
        let s = line_stroke();
        assert!(stroke_midpoint(&s).is_some());
    }

    #[test]
    fn empty_stroke() {
        let res = build_stroke_path(&[]);
        assert!(res.positions.is_empty());
    }

    #[test]
    fn single_sample_stroke() {
        let s = vec![StrokeSample {
            position: [0.0; 3],
            normal: [0.0, 1.0, 0.0],
            width: 0.1,
        }];
        let res = build_stroke_path(&s);
        assert!(res.positions.is_empty());
    }

    #[test]
    fn pi_used() {
        assert!((PI - std::f32::consts::PI).abs() < 1e-6);
    }

    #[test]
    fn indices_in_bounds() {
        let s = line_stroke();
        let res = build_stroke_path(&s);
        let vcount = res.positions.len() as u32;
        assert!(res.indices.iter().all(|&i| i < vcount));
    }
}
