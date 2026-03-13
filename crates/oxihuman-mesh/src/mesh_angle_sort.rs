// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Sort vertices or faces by angle around a central axis.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AngleSortEntry {
    pub index: usize,
    pub angle: f32,
}

#[allow(dead_code)]
pub fn compute_angle_2d(cx: f32, cy: f32, px: f32, py: f32) -> f32 {
    (py - cy).atan2(px - cx)
}

#[allow(dead_code)]
pub fn sort_indices_by_angle(positions: &[[f32; 3]], center: [f32; 3], axis: u8) -> Vec<AngleSortEntry> {
    let (u, v) = match axis {
        0 => (1usize, 2usize),
        1 => (0, 2),
        _ => (0, 1),
    };
    let mut entries: Vec<AngleSortEntry> = positions
        .iter()
        .enumerate()
        .map(|(i, p)| AngleSortEntry {
            index: i,
            angle: compute_angle_2d(center[u], center[v], p[u], p[v]),
        })
        .collect();
    entries.sort_by(|a, b| a.angle.partial_cmp(&b.angle).unwrap_or(std::cmp::Ordering::Equal));
    entries
}

#[allow(dead_code)]
pub fn angle_sort_face_centroids(positions: &[[f32; 3]], faces: &[[u32; 3]], center: [f32; 3], axis: u8) -> Vec<usize> {
    let centroids: Vec<[f32; 3]> = faces.iter().map(|f| {
        let (a, b, c) = (positions[f[0] as usize], positions[f[1] as usize], positions[f[2] as usize]);
        [(a[0]+b[0]+c[0])/3.0, (a[1]+b[1]+c[1])/3.0, (a[2]+b[2]+c[2])/3.0]
    }).collect();
    let sorted = sort_indices_by_angle(&centroids, center, axis);
    sorted.iter().map(|e| e.index).collect()
}

#[allow(dead_code)]
pub fn angle_range(entries: &[AngleSortEntry]) -> f32 {
    if entries.len() < 2 { return 0.0; }
    entries[entries.len() - 1].angle - entries[0].angle
}

#[allow(dead_code)]
pub fn angle_sort_count(entries: &[AngleSortEntry]) -> usize {
    entries.len()
}

#[allow(dead_code)]
pub fn angle_sort_min(entries: &[AngleSortEntry]) -> f32 {
    entries.first().map(|e| e.angle).unwrap_or(0.0)
}

#[allow(dead_code)]
pub fn angle_sort_max(entries: &[AngleSortEntry]) -> f32 {
    entries.last().map(|e| e.angle).unwrap_or(0.0)
}

#[allow(dead_code)]
pub fn angle_sort_to_json(entries: &[AngleSortEntry]) -> String {
    let items: Vec<String> = entries.iter().map(|e| format!("{{\"i\":{},\"a\":{:.4}}}", e.index, e.angle)).collect();
    format!("[{}]", items.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn pts() -> Vec<[f32; 3]> {
        vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0], [0.0, -1.0, 0.0]]
    }

    #[test] fn test_compute_angle() { let a = compute_angle_2d(0.0, 0.0, 1.0, 0.0); assert!(a.abs() < 1e-5); }
    #[test] fn test_sort_basic() { let s = sort_indices_by_angle(&pts(), [0.0, 0.0, 0.0], 2); assert_eq!(s.len(), 4); }
    #[test] fn test_sorted_order() { let s = sort_indices_by_angle(&pts(), [0.0, 0.0, 0.0], 2); assert!(s[0].angle <= s[3].angle); }
    #[test] fn test_angle_range() { let s = sort_indices_by_angle(&pts(), [0.0, 0.0, 0.0], 2); assert!(angle_range(&s) > PI); }
    #[test] fn test_face_centroids() {
        let p = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],[0.0,0.0,1.0]];
        let f = vec![[0,1,2],[0,1,3]];
        let r = angle_sort_face_centroids(&p, &f, [0.0,0.0,0.0], 2);
        assert_eq!(r.len(), 2);
    }
    #[test] fn test_count() { let s = sort_indices_by_angle(&pts(), [0.0,0.0,0.0], 2); assert_eq!(angle_sort_count(&s), 4); }
    #[test] fn test_min_max() { let s = sort_indices_by_angle(&pts(), [0.0,0.0,0.0], 2); assert!(angle_sort_min(&s) <= angle_sort_max(&s)); }
    #[test] fn test_to_json() { let s = sort_indices_by_angle(&pts(), [0.0,0.0,0.0], 2); assert!(angle_sort_to_json(&s).contains("\"a\"")); }
    #[test] fn test_empty() { let s = sort_indices_by_angle(&[], [0.0,0.0,0.0], 2); assert_eq!(s.len(), 0); }
    #[test] fn test_empty_range() { assert!((angle_range(&[])).abs() < 1e-6); }
}
