// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct RetopoStroke {
    pub points: Vec<[f32; 3]>,
    pub flow_direction: [f32; 3],
}

pub fn new_retopo_stroke(points: Vec<[f32; 3]>) -> RetopoStroke {
    let dir = stroke_direction_from_pts(&points);
    RetopoStroke {
        points,
        flow_direction: dir,
    }
}

fn stroke_direction_from_pts(pts: &[[f32; 3]]) -> [f32; 3] {
    if pts.len() < 2 {
        return [0.0, 0.0, 0.0];
    }
    let a = pts[0];
    let b = pts[pts.len() - 1];
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if len < 1e-8 {
        [0.0, 0.0, 0.0]
    } else {
        [d[0] / len, d[1] / len, d[2] / len]
    }
}

pub fn stroke_length(s: &RetopoStroke) -> f32 {
    if s.points.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0_f32;
    for i in 1..s.points.len() {
        let a = s.points[i - 1];
        let b = s.points[i];
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let dz = b[2] - a[2];
        total += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    total
}

pub fn stroke_direction(s: &RetopoStroke) -> [f32; 3] {
    stroke_direction_from_pts(&s.points)
}

pub fn stroke_point_count(s: &RetopoStroke) -> usize {
    s.points.len()
}

pub fn stroke_resample(s: &RetopoStroke, n: usize) -> Vec<[f32; 3]> {
    if n == 0 || s.points.is_empty() {
        return vec![];
    }
    if n == 1 {
        return vec![s.points[0]];
    }
    let total = stroke_length(s);
    if total < 1e-8 {
        return vec![s.points[0]; n];
    }
    let step = total / (n - 1) as f32;
    let mut result = Vec::with_capacity(n);
    result.push(s.points[0]);
    let mut dist_walked = 0.0_f32;
    let mut seg_idx = 0_usize;
    let mut seg_walked = 0.0_f32;
    for k in 1..n {
        let target = k as f32 * step;
        while seg_idx + 1 < s.points.len() {
            let a = s.points[seg_idx];
            let b = s.points[seg_idx + 1];
            let dx = b[0] - a[0];
            let dy = b[1] - a[1];
            let dz = b[2] - a[2];
            let seg_len = (dx * dx + dy * dy + dz * dz).sqrt();
            if dist_walked + seg_len - seg_walked >= target - (dist_walked - seg_walked) {
                break;
            }
            dist_walked += seg_len - seg_walked;
            seg_walked = 0.0;
            seg_idx += 1;
        }
        if seg_idx + 1 >= s.points.len() {
            result.push(s.points[s.points.len() - 1]);
        } else {
            let a = s.points[seg_idx];
            let b = s.points[seg_idx + 1];
            let dx = b[0] - a[0];
            let dy = b[1] - a[1];
            let dz = b[2] - a[2];
            let seg_len = (dx * dx + dy * dy + dz * dz).sqrt();
            let t = if seg_len > 1e-8 {
                ((target - dist_walked) / seg_len).clamp(0.0, 1.0)
            } else {
                0.0
            };
            result.push([a[0] + t * dx, a[1] + t * dy, a[2] + t * dz]);
        }
    }
    result
}

pub fn stroke_snap_to_surface(
    p: [f32; 3],
    surface_pos: [f32; 3],
    surface_normal: [f32; 3],
) -> [f32; 3] {
    /* project p onto the tangent plane at surface_pos */
    let d = [
        p[0] - surface_pos[0],
        p[1] - surface_pos[1],
        p[2] - surface_pos[2],
    ];
    let dot = d[0] * surface_normal[0] + d[1] * surface_normal[1] + d[2] * surface_normal[2];
    [
        p[0] - dot * surface_normal[0],
        p[1] - dot * surface_normal[1],
        p[2] - dot * surface_normal[2],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stroke() {
        /* direction from first to last */
        let s = new_retopo_stroke(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        assert!((s.flow_direction[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_stroke_length() {
        /* two points 1 apart */
        let s = new_retopo_stroke(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        assert!((stroke_length(&s) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_stroke_point_count() {
        /* count */
        let s = new_retopo_stroke(vec![[0.0; 3]; 5]);
        assert_eq!(stroke_point_count(&s), 5);
    }

    #[test]
    fn test_stroke_resample() {
        /* resample to 3 gives 3 points */
        let s = new_retopo_stroke(vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        let r = stroke_resample(&s, 3);
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn test_snap_to_surface_on_plane() {
        /* point exactly on plane => unchanged */
        let sp = [0.0, 0.0, 0.0];
        let sn = [0.0, 1.0, 0.0];
        let p = [1.0, 0.0, 2.0];
        let s = stroke_snap_to_surface(p, sp, sn);
        assert!((s[1]).abs() < 1e-6);
    }

    #[test]
    fn test_snap_to_surface_above_plane() {
        /* point above plane gets snapped down */
        let sp = [0.0, 0.0, 0.0];
        let sn = [0.0, 1.0, 0.0];
        let p = [0.0, 2.0, 0.0];
        let s = stroke_snap_to_surface(p, sp, sn);
        assert!(s[1].abs() < 1e-6);
    }
}
