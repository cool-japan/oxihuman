#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Triangle strip construction from indexed triangle meshes.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceStrip {
    pub indices: Vec<u32>,
    pub restart_index: Option<u32>,
}

#[allow(dead_code)]
pub fn build_triangle_strip(tri_indices: &[u32]) -> FaceStrip {
    // Simple greedy strip: just output triangles in order with restart markers
    let mut strip = Vec::new();
    for tri in tri_indices.chunks(3) {
        if tri.len() == 3 {
            if !strip.is_empty() {
                strip.push(u32::MAX); // restart
            }
            strip.push(tri[0]);
            strip.push(tri[1]);
            strip.push(tri[2]);
        }
    }
    FaceStrip {
        indices: strip,
        restart_index: Some(u32::MAX),
    }
}

#[allow(dead_code)]
pub fn strip_length(strip: &FaceStrip) -> usize {
    strip.indices.len()
}

#[allow(dead_code)]
pub fn strip_to_indices(strip: &FaceStrip) -> Vec<u32> {
    let restart = strip.restart_index.unwrap_or(u32::MAX);
    let mut result = Vec::new();
    let mut run = Vec::new();
    for &idx in &strip.indices {
        if idx == restart {
            for tri in run.windows(3) {
                result.push(tri[0]);
                result.push(tri[1]);
                result.push(tri[2]);
            }
            run.clear();
        } else {
            run.push(idx);
        }
    }
    for tri in run.windows(3) {
        result.push(tri[0]);
        result.push(tri[1]);
        result.push(tri[2]);
    }
    result
}

#[allow(dead_code)]
pub fn strip_restart_count(strip: &FaceStrip) -> usize {
    let restart = strip.restart_index.unwrap_or(u32::MAX);
    strip.indices.iter().filter(|&&i| i == restart).count()
}

#[allow(dead_code)]
pub fn strips_from_mesh(tri_indices: &[u32]) -> Vec<FaceStrip> {
    if tri_indices.is_empty() {
        return Vec::new();
    }
    vec![build_triangle_strip(tri_indices)]
}

#[allow(dead_code)]
pub fn longest_strip(strips: &[FaceStrip]) -> usize {
    strips.iter().map(strip_length).max().unwrap_or(0)
}

#[allow(dead_code)]
pub fn strip_efficiency(strip: &FaceStrip) -> f32 {
    let restart = strip.restart_index.unwrap_or(u32::MAX);
    let total = strip.indices.len();
    let restarts = strip.indices.iter().filter(|&&i| i == restart).count();
    if total == 0 {
        return 0.0;
    }
    1.0 - (restarts as f32 / total as f32)
}

#[allow(dead_code)]
pub fn strip_to_json(strip: &FaceStrip) -> String {
    let idx_str: Vec<String> = strip.indices.iter().map(|i| i.to_string()).collect();
    format!(
        "{{\"length\":{},\"restarts\":{},\"indices\":[{}]}}",
        strip.indices.len(),
        strip_restart_count(strip),
        idx_str.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_strip() {
        let strip = build_triangle_strip(&[0, 1, 2, 3, 4, 5]);
        assert!(!strip.indices.is_empty());
    }

    #[test]
    fn test_strip_length() {
        let strip = build_triangle_strip(&[0, 1, 2]);
        assert_eq!(strip_length(&strip), 3);
    }

    #[test]
    fn test_strip_to_indices_roundtrip() {
        let tris = vec![0u32, 1, 2];
        let strip = build_triangle_strip(&tris);
        let back = strip_to_indices(&strip);
        assert_eq!(back, tris);
    }

    #[test]
    fn test_strip_restart_count_single() {
        let strip = build_triangle_strip(&[0, 1, 2]);
        assert_eq!(strip_restart_count(&strip), 0);
    }

    #[test]
    fn test_strip_restart_count_two() {
        let strip = build_triangle_strip(&[0, 1, 2, 3, 4, 5]);
        assert_eq!(strip_restart_count(&strip), 1);
    }

    #[test]
    fn test_strips_from_mesh() {
        let strips = strips_from_mesh(&[0, 1, 2]);
        assert_eq!(strips.len(), 1);
    }

    #[test]
    fn test_longest_strip() {
        let strips = strips_from_mesh(&[0, 1, 2, 3, 4, 5]);
        assert!(longest_strip(&strips) > 0);
    }

    #[test]
    fn test_strip_efficiency() {
        let strip = build_triangle_strip(&[0, 1, 2]);
        assert!((strip_efficiency(&strip) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_strip_to_json() {
        let strip = build_triangle_strip(&[0, 1, 2]);
        let json = strip_to_json(&strip);
        assert!(json.contains("\"length\":3"));
    }

    #[test]
    fn test_empty_strip() {
        let strip = build_triangle_strip(&[]);
        assert!(strip.indices.is_empty());
        assert!((strip_efficiency(&strip)).abs() < 1e-6);
    }
}
