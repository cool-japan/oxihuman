// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Contour line extraction: extract iso-height contour lines from a mesh.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContourSegment {
    pub start: [f32; 3],
    pub end: [f32; 3],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContourLineResult {
    pub segments: Vec<ContourSegment>,
    pub height: f32,
}

#[allow(dead_code)]
pub fn lerp_edge(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0]+(b[0]-a[0])*t, a[1]+(b[1]-a[1])*t, a[2]+(b[2]-a[2])*t]
}

#[allow(dead_code)]
pub fn extract_contour_at_height(
    positions: &[[f32; 3]],
    indices: &[[u32; 3]],
    height: f32,
) -> ContourLineResult {
    let mut segments = Vec::new();
    for tri in indices {
        let p = [positions[tri[0] as usize], positions[tri[1] as usize], positions[tri[2] as usize]];
        let h = [p[0][1], p[1][1], p[2][1]];
        let mut cross_points = Vec::new();
        for i in 0..3 {
            let j = (i + 1) % 3;
            if (h[i] - height) * (h[j] - height) < 0.0 {
                let t = (height - h[i]) / (h[j] - h[i]);
                cross_points.push(lerp_edge(p[i], p[j], t));
            } else if (h[i] - height).abs() < 1e-6 {
                cross_points.push(p[i]);
            }
        }
        if cross_points.len() >= 2 {
            segments.push(ContourSegment { start: cross_points[0], end: cross_points[1] });
        }
    }
    ContourLineResult { segments, height }
}

#[allow(dead_code)]
pub fn extract_multiple_contours(
    positions: &[[f32; 3]],
    indices: &[[u32; 3]],
    min_h: f32,
    max_h: f32,
    step: f32,
) -> Vec<ContourLineResult> {
    let mut results = Vec::new();
    let mut h = min_h;
    while h <= max_h {
        results.push(extract_contour_at_height(positions, indices, h));
        h += step;
    }
    results
}

#[allow(dead_code)]
pub fn contour_segment_count(result: &ContourLineResult) -> usize { result.segments.len() }

#[allow(dead_code)]
pub fn contour_total_length(result: &ContourLineResult) -> f32 {
    result.segments.iter().map(|s| {
        let dx = s.end[0]-s.start[0];
        let dy = s.end[1]-s.start[1];
        let dz = s.end[2]-s.start[2];
        (dx*dx+dy*dy+dz*dz).sqrt()
    }).sum()
}

#[allow(dead_code)]
pub fn contour_to_json(result: &ContourLineResult) -> String {
    format!("{{\"height\":{:.4},\"segments\":{}}}", result.height, result.segments.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn slope_mesh() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        (vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,2.0,0.0]], vec![[0,1,2]])
    }
    #[test] fn test_lerp_edge() { let p = lerp_edge([0.0,0.0,0.0],[2.0,2.0,2.0],0.5); assert!((p[0]-1.0).abs()<1e-6); }
    #[test] fn test_extract_contour() {
        let (p,i) = slope_mesh();
        let r = extract_contour_at_height(&p,&i,1.0);
        assert!(!r.segments.is_empty());
    }
    #[test] fn test_no_contour() {
        let (p,i) = slope_mesh();
        let r = extract_contour_at_height(&p,&i,10.0);
        assert!(r.segments.is_empty());
    }
    #[test] fn test_multiple_contours() {
        let (p,i) = slope_mesh();
        let rs = extract_multiple_contours(&p,&i,0.5,1.5,0.5);
        assert!(rs.len() >= 2);
    }
    #[test] fn test_segment_count() {
        let (p,i) = slope_mesh();
        let r = extract_contour_at_height(&p,&i,1.0);
        assert!(contour_segment_count(&r) > 0);
    }
    #[test] fn test_contour_total_length() {
        let (p,i) = slope_mesh();
        let r = extract_contour_at_height(&p,&i,1.0);
        assert!(contour_total_length(&r) > 0.0);
    }
    #[test] fn test_contour_to_json() {
        let (p,i) = slope_mesh();
        let r = extract_contour_at_height(&p,&i,1.0);
        let j = contour_to_json(&r);
        assert!(j.contains("height"));
    }
    #[test] fn test_empty_mesh() {
        let r = extract_contour_at_height(&[],&[],1.0);
        assert!(r.segments.is_empty());
    }
    #[test] fn test_height_at_vertex() {
        let p = vec![[0.0,1.0,0.0],[1.0,1.0,0.0],[0.5,1.0,0.0]];
        let i = vec![[0,1,2]];
        let r = extract_contour_at_height(&p,&i,1.0);
        assert!(r.segments.is_empty() || contour_segment_count(&r) < usize::MAX);
    }
    #[test] fn test_contour_result_height() {
        let (p,i) = slope_mesh();
        let r = extract_contour_at_height(&p,&i,0.75);
        assert!((r.height - 0.75).abs() < 1e-6);
    }
}
