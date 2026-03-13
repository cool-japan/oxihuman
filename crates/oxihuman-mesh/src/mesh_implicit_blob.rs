// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Blobby/implicit-surface node — a single weighted implicit primitive in a CSG tree.

/// Kind of implicit blob primitive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobKind {
    Ellipsoid,
    Cylinder,
    Torus,
}

/// A single implicit blob node.
#[derive(Debug, Clone)]
pub struct ImplicitBlob {
    pub kind: BlobKind,
    pub center: [f32; 3],
    /// Semi-axes or [radius, minor_radius, 0] depending on kind.
    pub params: [f32; 3],
    pub blend_weight: f32,
    pub label: String,
}

/// A collection of implicit blobs forming one compound surface.
#[derive(Debug, Default)]
pub struct ImplicitBlobSet {
    pub blobs: Vec<ImplicitBlob>,
    pub iso_level: f32,
}

/// Create a new implicit blob set.
pub fn new_blob_set(iso_level: f32) -> ImplicitBlobSet {
    ImplicitBlobSet {
        blobs: Vec::new(),
        iso_level: iso_level.max(0.0),
    }
}

/// Add an ellipsoid blob.
pub fn add_ellipsoid_blob(
    set: &mut ImplicitBlobSet,
    center: [f32; 3],
    semi_axes: [f32; 3],
    weight: f32,
    label: &str,
) {
    set.blobs.push(ImplicitBlob {
        kind: BlobKind::Ellipsoid,
        center,
        params: semi_axes,
        blend_weight: weight.clamp(0.0, 1.0),
        label: label.to_owned(),
    });
}

/// Add a cylinder blob.
pub fn add_cylinder_blob(
    set: &mut ImplicitBlobSet,
    center: [f32; 3],
    radius: f32,
    height: f32,
    weight: f32,
    label: &str,
) {
    set.blobs.push(ImplicitBlob {
        kind: BlobKind::Cylinder,
        center,
        params: [radius.max(1e-6), height.max(1e-6), 0.0],
        blend_weight: weight.clamp(0.0, 1.0),
        label: label.to_owned(),
    });
}

/// Number of blobs in the set.
pub fn blob_count(set: &ImplicitBlobSet) -> usize {
    set.blobs.len()
}

/// Evaluate a simplified Gaussian potential for an ellipsoid blob at query point.
fn ellipsoid_potential(blob: &ImplicitBlob, q: [f32; 3]) -> f32 {
    let ax = blob.params[0].max(1e-8);
    let ay = blob.params[1].max(1e-8);
    let az = blob.params[2].max(1e-8);
    let dx = (q[0] - blob.center[0]) / ax;
    let dy = (q[1] - blob.center[1]) / ay;
    let dz = (q[2] - blob.center[2]) / az;
    let r2 = dx * dx + dy * dy + dz * dz;
    blob.blend_weight * (-r2).exp()
}

/// Evaluate the combined implicit field at a query point.
pub fn evaluate_blob_field(set: &ImplicitBlobSet, query: [f32; 3]) -> f32 {
    set.blobs
        .iter()
        .map(|b| match b.kind {
            BlobKind::Ellipsoid => ellipsoid_potential(b, query),
            _ => ellipsoid_potential(b, query), /* simplified */
        })
        .sum()
}

/// Is the query point inside the iso surface?
pub fn is_inside_blob_surface(set: &ImplicitBlobSet, query: [f32; 3]) -> bool {
    evaluate_blob_field(set, query) >= set.iso_level
}

/// Average blend weight.
pub fn average_blend_weight(set: &ImplicitBlobSet) -> f32 {
    if set.blobs.is_empty() {
        return 0.0;
    }
    let sum: f32 = set.blobs.iter().map(|b| b.blend_weight).sum();
    sum / set.blobs.len() as f32
}

/// Serialize to JSON-style string.
pub fn blob_set_to_json(set: &ImplicitBlobSet) -> String {
    format!(
        r#"{{"iso_level":{:.4}, "blob_count":{}}}"#,
        set.iso_level,
        set.blobs.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_set_has_no_blobs() {
        /* fresh set has no blobs */
        let s = new_blob_set(0.5);
        assert_eq!(blob_count(&s), 0);
    }

    #[test]
    fn add_ellipsoid_increments_count() {
        /* adding an ellipsoid increases count by 1 */
        let mut s = new_blob_set(0.5);
        add_ellipsoid_blob(&mut s, [0.0; 3], [1.0; 3], 1.0, "e0");
        assert_eq!(blob_count(&s), 1);
    }

    #[test]
    fn add_cylinder_increments_count() {
        /* adding a cylinder increases count by 1 */
        let mut s = new_blob_set(0.5);
        add_cylinder_blob(&mut s, [0.0; 3], 1.0, 2.0, 1.0, "c0");
        assert_eq!(blob_count(&s), 1);
    }

    #[test]
    fn field_potential_at_center_is_positive() {
        /* potential at blob center should be positive */
        let mut s = new_blob_set(0.1);
        add_ellipsoid_blob(&mut s, [0.0; 3], [1.0; 3], 1.0, "e");
        assert!(evaluate_blob_field(&s, [0.0; 3]) > 0.0);
    }

    #[test]
    fn is_inside_at_center_with_low_iso() {
        /* with low iso level, center should be inside */
        let mut s = new_blob_set(0.01);
        add_ellipsoid_blob(&mut s, [0.0; 3], [1.0; 3], 1.0, "e");
        assert!(is_inside_blob_surface(&s, [0.0; 3]));
    }

    #[test]
    fn average_blend_weight_empty_is_zero() {
        /* empty set average is zero */
        let s = new_blob_set(0.5);
        assert_eq!(average_blend_weight(&s), 0.0);
    }

    #[test]
    fn average_blend_weight_correct() {
        /* average of 0.4 and 0.6 is 0.5 */
        let mut s = new_blob_set(0.5);
        add_ellipsoid_blob(&mut s, [0.0; 3], [1.0; 3], 0.4, "a");
        add_ellipsoid_blob(&mut s, [0.0; 3], [1.0; 3], 0.6, "b");
        assert!((average_blend_weight(&s) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_contains_blob_count() {
        /* JSON should contain blob_count */
        let s = new_blob_set(0.5);
        assert!(blob_set_to_json(&s).contains("blob_count"));
    }

    #[test]
    fn weight_clamped_to_one() {
        /* weight above 1 should be clamped */
        let mut s = new_blob_set(0.5);
        add_ellipsoid_blob(&mut s, [0.0; 3], [1.0; 3], 2.0, "e");
        assert!((s.blobs[0].blend_weight - 1.0).abs() < 1e-5);
    }
}
