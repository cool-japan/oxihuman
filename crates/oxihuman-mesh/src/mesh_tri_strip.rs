// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Triangle-strip encoding and decoding for indexed triangle meshes.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TriStrip {
    pub indices: Vec<u32>,
    pub restart_index: u32,
    pub strip_count: usize,
}

#[allow(dead_code)]
pub fn new_tri_strip(restart_index: u32) -> TriStrip {
    TriStrip {
        indices: Vec::new(),
        restart_index,
        strip_count: 0,
    }
}

#[allow(dead_code)]
pub fn encode_tri_strip(triangles: &[u32]) -> TriStrip {
    let restart = u32::MAX;
    let mut out = TriStrip {
        indices: Vec::new(),
        restart_index: restart,
        strip_count: 0,
    };
    for chunk in triangles.chunks(3) {
        if chunk.len() < 3 {
            break;
        }
        if !out.indices.is_empty() {
            out.indices.push(restart);
        }
        out.indices.push(chunk[0]);
        out.indices.push(chunk[1]);
        out.indices.push(chunk[2]);
        out.strip_count += 1;
    }
    out
}

#[allow(dead_code)]
pub fn decode_tri_strip(strip: &TriStrip) -> Vec<u32> {
    let restart = strip.restart_index;
    let mut out = Vec::new();
    let mut run: Vec<u32> = Vec::new();
    for &idx in &strip.indices {
        if idx == restart {
            flush_run(&run, &mut out);
            run.clear();
        } else {
            run.push(idx);
        }
    }
    flush_run(&run, &mut out);
    out
}

fn flush_run(run: &[u32], out: &mut Vec<u32>) {
    if run.len() >= 3 {
        out.push(run[0]);
        out.push(run[1]);
        out.push(run[2]);
    }
}

#[allow(dead_code)]
pub fn strip_index_count(strip: &TriStrip) -> usize {
    strip.indices.len()
}

#[allow(dead_code)]
pub fn strip_restart_count(strip: &TriStrip) -> usize {
    let r = strip.restart_index;
    strip.indices.iter().filter(|&&i| i == r).count()
}

#[allow(dead_code)]
pub fn strip_triangle_count(strip: &TriStrip) -> usize {
    strip.strip_count
}

#[allow(dead_code)]
pub fn strip_efficiency_ratio(strip: &TriStrip) -> f32 {
    let total = strip.indices.len();
    if total == 0 {
        return 0.0;
    }
    let restarts = strip_restart_count(strip);
    1.0 - (restarts as f32 / total as f32)
}

#[allow(dead_code)]
pub fn strip_to_json(strip: &TriStrip) -> String {
    format!(
        "{{\"strip_count\":{},\"index_count\":{},\"restart\":{}}}",
        strip.strip_count,
        strip.indices.len(),
        strip.restart_index,
    )
}

/// Compute the half-angle used in strip angle analysis (uses PI constant).
#[allow(dead_code)]
pub fn strip_half_angle() -> f32 {
    PI / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_single_triangle() {
        let strip = encode_tri_strip(&[0, 1, 2]);
        assert_eq!(strip.strip_count, 1);
        assert!(!strip.indices.is_empty());
    }

    #[test]
    fn test_decode_roundtrip() {
        let tris = vec![0u32, 1, 2];
        let strip = encode_tri_strip(&tris);
        let decoded = decode_tri_strip(&strip);
        assert_eq!(decoded, tris);
    }

    #[test]
    fn test_restart_count_two_triangles() {
        let strip = encode_tri_strip(&[0, 1, 2, 3, 4, 5]);
        assert_eq!(strip_restart_count(&strip), 1);
    }

    #[test]
    fn test_strip_count_two() {
        let strip = encode_tri_strip(&[0, 1, 2, 3, 4, 5]);
        assert_eq!(strip_triangle_count(&strip), 2);
    }

    #[test]
    fn test_empty_strip() {
        let strip = encode_tri_strip(&[]);
        assert_eq!(strip.strip_count, 0);
        assert!(strip.indices.is_empty());
    }

    #[test]
    fn test_efficiency_single() {
        let strip = encode_tri_strip(&[0, 1, 2]);
        let e = strip_efficiency_ratio(&strip);
        assert!((0.0..=1.0).contains(&e));
    }

    #[test]
    fn test_index_count() {
        let strip = encode_tri_strip(&[0, 1, 2]);
        assert_eq!(strip_index_count(&strip), 3);
    }

    #[test]
    fn test_json_output() {
        let strip = encode_tri_strip(&[0, 1, 2]);
        let json = strip_to_json(&strip);
        assert!(json.contains("strip_count"));
    }

    #[test]
    fn test_half_angle() {
        let a = strip_half_angle();
        assert!((a - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
    }

    #[test]
    fn test_three_triangles() {
        let strip = encode_tri_strip(&[0, 1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(strip_triangle_count(&strip), 3);
    }
}
