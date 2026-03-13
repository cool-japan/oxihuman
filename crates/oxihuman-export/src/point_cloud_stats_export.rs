// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Point cloud statistics export (density, bbox, centroid, etc.).

/// Statistics computed from a point cloud.
#[allow(dead_code)]
pub struct PointCloudStats {
    pub point_count: usize,
    pub centroid: [f32; 3],
    pub bbox_min: [f32; 3],
    pub bbox_max: [f32; 3],
    pub avg_nn_distance: f32,
    pub density_per_m3: f32,
}

/// Compute bounding box.
#[allow(dead_code)]
pub fn compute_bbox(points: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    if points.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = points[0];
    let mut mx = points[0];
    for &p in points.iter().skip(1) {
        for i in 0..3 {
            if p[i] < mn[i] { mn[i] = p[i]; }
            if p[i] > mx[i] { mx[i] = p[i]; }
        }
    }
    (mn, mx)
}

/// Compute centroid.
#[allow(dead_code)]
pub fn compute_centroid(points: &[[f32; 3]]) -> [f32; 3] {
    if points.is_empty() { return [0.0; 3]; }
    let mut sum = [0.0f32; 3];
    for &p in points {
        sum[0] += p[0]; sum[1] += p[1]; sum[2] += p[2];
    }
    let n = points.len() as f32;
    [sum[0]/n, sum[1]/n, sum[2]/n]
}

/// Compute bbox volume.
#[allow(dead_code)]
pub fn bbox_volume(mn: [f32; 3], mx: [f32; 3]) -> f32 {
    (mx[0]-mn[0]).max(0.0) * (mx[1]-mn[1]).max(0.0) * (mx[2]-mn[2]).max(0.0)
}

/// Compute average nearest-neighbour distance (simplified O(n^2)).
#[allow(dead_code)]
pub fn avg_nn_distance(points: &[[f32; 3]]) -> f32 {
    if points.len() < 2 { return 0.0; }
    let mut total = 0.0f32;
    for (i, &a) in points.iter().enumerate() {
        let mut min_d = f32::INFINITY;
        for (j, &b) in points.iter().enumerate() {
            if i == j { continue; }
            let dx = a[0]-b[0]; let dy = a[1]-b[1]; let dz = a[2]-b[2];
            let d = (dx*dx+dy*dy+dz*dz).sqrt();
            if d < min_d { min_d = d; }
        }
        total += min_d;
    }
    total / points.len() as f32
}

/// Compute full statistics.
#[allow(dead_code)]
pub fn compute_point_cloud_stats(points: &[[f32; 3]]) -> PointCloudStats {
    let (mn, mx) = compute_bbox(points);
    let centroid = compute_centroid(points);
    let vol = bbox_volume(mn, mx);
    let density = if vol > 1e-10 { points.len() as f32 / vol } else { 0.0 };
    let ann = avg_nn_distance(points);
    PointCloudStats {
        point_count: points.len(),
        centroid,
        bbox_min: mn,
        bbox_max: mx,
        avg_nn_distance: ann,
        density_per_m3: density,
    }
}

/// Export stats to JSON.
#[allow(dead_code)]
pub fn stats_to_json(stats: &PointCloudStats) -> String {
    format!(
        "{{\"point_count\":{},\"centroid\":[{},{},{}],\"avg_nn_distance\":{:.6},\"density\":{:.4}}}",
        stats.point_count,
        stats.centroid[0], stats.centroid[1], stats.centroid[2],
        stats.avg_nn_distance,
        stats.density_per_m3,
    )
}

/// Export stats to CSV.
#[allow(dead_code)]
pub fn stats_to_csv(stats: &PointCloudStats) -> String {
    format!(
        "point_count,{}\ncentroid_x,{}\ncentroid_y,{}\ncentroid_z,{}\navg_nn_dist,{:.6}\ndensity,{:.4}\n",
        stats.point_count,
        stats.centroid[0], stats.centroid[1], stats.centroid[2],
        stats.avg_nn_distance, stats.density_per_m3,
    )
}

/// Diameter of the point cloud (bbox diagonal).
#[allow(dead_code)]
pub fn point_cloud_diameter(stats: &PointCloudStats) -> f32 {
    let dx = stats.bbox_max[0] - stats.bbox_min[0];
    let dy = stats.bbox_max[1] - stats.bbox_min[1];
    let dz = stats.bbox_max[2] - stats.bbox_min[2];
    (dx*dx+dy*dy+dz*dz).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cloud() -> Vec<[f32; 3]> {
        vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0]]
    }

    #[test]
    fn bbox_correct() {
        let pts = cloud();
        let (mn, mx) = compute_bbox(&pts);
        assert!(mn[0].abs() < 1e-5);
        assert!((mx[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn centroid_inside() {
        let pts = cloud();
        let c = compute_centroid(&pts);
        let (mn, mx) = compute_bbox(&pts);
        for i in 0..3 {
            assert!(c[i] >= mn[i] && c[i] <= mx[i]);
        }
    }

    #[test]
    fn bbox_volume_correct() {
        let vol = bbox_volume([0.0,0.0,0.0],[2.0,3.0,4.0]);
        assert!((vol - 24.0).abs() < 1e-5);
    }

    #[test]
    fn avg_nn_positive() {
        let pts = cloud();
        let d = avg_nn_distance(&pts);
        assert!(d > 0.0);
    }

    #[test]
    fn stats_point_count() {
        let pts = cloud();
        let s = compute_point_cloud_stats(&pts);
        assert_eq!(s.point_count, 3);
    }

    #[test]
    fn stats_to_json_contains_count() {
        let pts = cloud();
        let s = compute_point_cloud_stats(&pts);
        let j = stats_to_json(&s);
        assert!(j.contains("3"));
    }

    #[test]
    fn stats_to_csv_has_header() {
        let pts = cloud();
        let s = compute_point_cloud_stats(&pts);
        let csv = stats_to_csv(&s);
        assert!(csv.contains("point_count"));
    }

    #[test]
    fn diameter_positive() {
        let pts = cloud();
        let s = compute_point_cloud_stats(&pts);
        assert!(point_cloud_diameter(&s) > 0.0);
    }

    #[test]
    fn empty_cloud_stats() {
        let s = compute_point_cloud_stats(&[]);
        assert_eq!(s.point_count, 0);
    }
}
