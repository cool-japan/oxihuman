#![allow(dead_code)]
//! Export bounding box data.

/// Bounding box export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BoundingBoxExport {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// Export a bounding box from positions.
#[allow(dead_code)]
pub fn export_bounding_box(positions: &[[f32; 3]]) -> BoundingBoxExport {
    if positions.is_empty() {
        return BoundingBoxExport { min: [0.0; 3], max: [0.0; 3] };
    }
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for p in positions {
        for i in 0..3 {
            if p[i] < min[i] { min[i] = p[i]; }
            if p[i] > max[i] { max[i] = p[i]; }
        }
    }
    BoundingBoxExport { min, max }
}

/// Get the minimum corner.
#[allow(dead_code)]
pub fn bbox_min(export: &BoundingBoxExport) -> [f32; 3] {
    export.min
}

/// Get the maximum corner.
#[allow(dead_code)]
pub fn bbox_max(export: &BoundingBoxExport) -> [f32; 3] {
    export.max
}

/// Get the center of the bounding box.
#[allow(dead_code)]
pub fn bbox_center(export: &BoundingBoxExport) -> [f32; 3] {
    [
        (export.min[0] + export.max[0]) * 0.5,
        (export.min[1] + export.max[1]) * 0.5,
        (export.min[2] + export.max[2]) * 0.5,
    ]
}

/// Get the extents (half-sizes) of the bounding box.
#[allow(dead_code)]
pub fn bbox_extents(export: &BoundingBoxExport) -> [f32; 3] {
    [
        (export.max[0] - export.min[0]) * 0.5,
        (export.max[1] - export.min[1]) * 0.5,
        (export.max[2] - export.min[2]) * 0.5,
    ]
}

/// Convert bounding box to JSON.
#[allow(dead_code)]
pub fn bbox_to_json(export: &BoundingBoxExport) -> String {
    format!(
        "{{\"min\":[{:.4},{:.4},{:.4}],\"max\":[{:.4},{:.4},{:.4}]}}",
        export.min[0], export.min[1], export.min[2],
        export.max[0], export.max[1], export.max[2]
    )
}

/// Compute the volume of the bounding box.
#[allow(dead_code)]
pub fn bbox_volume(export: &BoundingBoxExport) -> f32 {
    let ext = bbox_extents(export);
    ext[0] * ext[1] * ext[2] * 8.0
}

/// Validate that the bounding box is valid (min <= max).
#[allow(dead_code)]
pub fn validate_bbox(export: &BoundingBoxExport) -> bool {
    (0..3).all(|i| export.min[i] <= export.max[i])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_bbox() -> BoundingBoxExport {
        export_bounding_box(&[[0.0, 0.0, 0.0], [1.0, 2.0, 3.0]])
    }

    #[test]
    fn test_export_bounding_box() {
        let bb = sample_bbox();
        assert_eq!(bb.min, [0.0, 0.0, 0.0]);
        assert_eq!(bb.max, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_export_bounding_box_empty() {
        let bb = export_bounding_box(&[]);
        assert_eq!(bb.min, [0.0; 3]);
    }

    #[test]
    fn test_bbox_min() {
        let bb = sample_bbox();
        assert_eq!(bbox_min(&bb), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_bbox_max() {
        let bb = sample_bbox();
        assert_eq!(bbox_max(&bb), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_bbox_center() {
        let bb = sample_bbox();
        let c = bbox_center(&bb);
        assert!((c[0] - 0.5).abs() < 1e-6);
        assert!((c[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_bbox_extents() {
        let bb = sample_bbox();
        let ext = bbox_extents(&bb);
        assert!((ext[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_bbox_to_json() {
        let bb = sample_bbox();
        let j = bbox_to_json(&bb);
        assert!(j.contains("min"));
        assert!(j.contains("max"));
    }

    #[test]
    fn test_bbox_volume() {
        let bb = sample_bbox();
        assert!((bbox_volume(&bb) - 6.0).abs() < 1e-4);
    }

    #[test]
    fn test_validate_bbox() {
        let bb = sample_bbox();
        assert!(validate_bbox(&bb));
    }

    #[test]
    fn test_validate_bbox_invalid() {
        let bb = BoundingBoxExport { min: [1.0; 3], max: [0.0; 3] };
        assert!(!validate_bbox(&bb));
    }
}
