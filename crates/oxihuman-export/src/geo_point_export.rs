// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export geometry points (scatter / point cloud metadata).

/// A single geometry point with attributes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GeoPoint {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub scale: f32,
    pub rotation_deg: f32,
    pub label: u32,
}

/// A geometry point export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GeoPointExport {
    pub points: Vec<GeoPoint>,
}

/// Create a new geo point export.
#[allow(dead_code)]
pub fn new_geo_point_export() -> GeoPointExport {
    GeoPointExport { points: Vec::new() }
}

/// Add a point.
#[allow(dead_code)]
pub fn add_geo_point(export: &mut GeoPointExport, point: GeoPoint) {
    export.points.push(point);
}

/// Count points.
#[allow(dead_code)]
pub fn geo_point_count(export: &GeoPointExport) -> usize {
    export.points.len()
}

/// Compute the bounding box.
#[allow(dead_code)]
pub fn geo_point_bounds(export: &GeoPointExport) -> Option<([f32; 3], [f32; 3])> {
    if export.points.is_empty() {
        return None;
    }
    let mut mn = export.points[0].position;
    let mut mx = export.points[0].position;
    for p in &export.points {
        for k in 0..3 {
            if p.position[k] < mn[k] {
                mn[k] = p.position[k];
            }
            if p.position[k] > mx[k] {
                mx[k] = p.position[k];
            }
        }
    }
    Some((mn, mx))
}

/// Average scale.
#[allow(dead_code)]
pub fn avg_geo_point_scale(export: &GeoPointExport) -> f32 {
    let n = export.points.len();
    if n == 0 {
        return 0.0;
    }
    export.points.iter().map(|p| p.scale).sum::<f32>() / n as f32
}

/// Count points with a given label.
#[allow(dead_code)]
pub fn points_with_label_gp(export: &GeoPointExport, label: u32) -> usize {
    export.points.iter().filter(|p| p.label == label).count()
}

/// Validate scale is positive.
#[allow(dead_code)]
pub fn validate_geo_points(export: &GeoPointExport) -> bool {
    export.points.iter().all(|p| p.scale > 0.0)
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn geo_point_to_json(export: &GeoPointExport) -> String {
    format!("{{\"point_count\":{}}}", export.points.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_point(x: f32) -> GeoPoint {
        GeoPoint {
            position: [x, 0.0, 0.0],
            normal: [0.0, 1.0, 0.0],
            scale: 1.0,
            rotation_deg: 0.0,
            label: 0,
        }
    }

    #[test]
    fn test_add_and_count() {
        let mut e = new_geo_point_export();
        add_geo_point(&mut e, sample_point(0.0));
        assert_eq!(geo_point_count(&e), 1);
    }

    #[test]
    fn test_bounds_none_empty() {
        let e = new_geo_point_export();
        assert!(geo_point_bounds(&e).is_none());
    }

    #[test]
    fn test_bounds_some() {
        let mut e = new_geo_point_export();
        add_geo_point(&mut e, sample_point(0.0));
        add_geo_point(&mut e, sample_point(5.0));
        let (mn, mx) = geo_point_bounds(&e).expect("should succeed");
        assert!(mn[0].abs() < 1e-5);
        assert!((mx[0] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_avg_scale() {
        let mut e = new_geo_point_export();
        add_geo_point(&mut e, sample_point(0.0));
        add_geo_point(&mut e, sample_point(1.0));
        assert!((avg_geo_point_scale(&e) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_points_with_label() {
        let mut e = new_geo_point_export();
        add_geo_point(&mut e, sample_point(0.0));
        assert_eq!(points_with_label_gp(&e, 0), 1);
        assert_eq!(points_with_label_gp(&e, 1), 0);
    }

    #[test]
    fn test_validate_valid() {
        let mut e = new_geo_point_export();
        add_geo_point(&mut e, sample_point(0.0));
        assert!(validate_geo_points(&e));
    }

    #[test]
    fn test_validate_invalid_zero_scale() {
        let mut e = new_geo_point_export();
        e.points.push(GeoPoint {
            position: [0.0; 3],
            normal: [0.0, 1.0, 0.0],
            scale: 0.0,
            rotation_deg: 0.0,
            label: 0,
        });
        assert!(!validate_geo_points(&e));
    }

    #[test]
    fn test_geo_point_to_json() {
        let e = new_geo_point_export();
        let j = geo_point_to_json(&e);
        assert!(j.contains("point_count"));
    }

    #[test]
    fn test_avg_scale_empty() {
        let e = new_geo_point_export();
        assert!(avg_geo_point_scale(&e).abs() < 1e-6);
    }

    #[test]
    fn test_validate_empty() {
        let e = new_geo_point_export();
        assert!(validate_geo_points(&e));
    }
}
