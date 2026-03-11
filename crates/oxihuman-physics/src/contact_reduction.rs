// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Contact manifold reduction (keep best N contacts).

#![allow(dead_code)]

/// A single contact point with position, normal, and penetration depth.
#[allow(dead_code)]
pub struct ContactPoint {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub depth: f32,
}

/// Reduce a contact manifold to at most `max_keep` contacts.
/// Keeps the deepest contacts first.
#[allow(dead_code)]
pub fn reduce_contacts(points: &[ContactPoint], max_keep: usize) -> Vec<ContactPoint> {
    if points.is_empty() || max_keep == 0 {
        return Vec::new();
    }
    let mut indices: Vec<usize> = (0..points.len()).collect();
    // Sort by depth descending (deepest first)
    indices.sort_by(|&a, &b| {
        points[b].depth.partial_cmp(&points[a].depth).unwrap_or(std::cmp::Ordering::Equal)
    });
    indices.truncate(max_keep);
    indices
        .into_iter()
        .map(|i| ContactPoint {
            position: points[i].position,
            normal: points[i].normal,
            depth: points[i].depth,
        })
        .collect()
}

/// Approximate the contact area as the bounding box area of contact positions projected onto
/// the plane perpendicular to the average normal.
#[allow(dead_code)]
pub fn contact_area_approx(points: &[ContactPoint]) -> f32 {
    if points.len() < 2 {
        return 0.0;
    }
    let mut x_min = f32::INFINITY;
    let mut x_max = f32::NEG_INFINITY;
    let mut y_min = f32::INFINITY;
    let mut y_max = f32::NEG_INFINITY;
    for p in points {
        x_min = x_min.min(p.position[0]);
        x_max = x_max.max(p.position[0]);
        y_min = y_min.min(p.position[1]);
        y_max = y_max.max(p.position[1]);
    }
    (x_max - x_min) * (y_max - y_min)
}

/// Return a reference to the deepest contact point, or None if empty.
#[allow(dead_code)]
pub fn deepest_contact(points: &[ContactPoint]) -> Option<&ContactPoint> {
    points.iter().max_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap_or(std::cmp::Ordering::Equal))
}

/// Compute the centroid (average position) of all contact points.
#[allow(dead_code)]
pub fn contact_centroid(points: &[ContactPoint]) -> [f32; 3] {
    if points.is_empty() {
        return [0.0; 3];
    }
    let n = points.len() as f32;
    let mut sum = [0.0f32; 3];
    for p in points {
        sum[0] += p.position[0];
        sum[1] += p.position[1];
        sum[2] += p.position[2];
    }
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_contact(pos: [f32; 3], depth: f32) -> ContactPoint {
        ContactPoint {
            position: pos,
            normal: [0.0, 1.0, 0.0],
            depth,
        }
    }

    #[test]
    fn reduce_keeps_deepest() {
        let pts = vec![
            make_contact([0.0; 3], 0.1),
            make_contact([1.0, 0.0, 0.0], 0.5),
            make_contact([2.0, 0.0, 0.0], 0.3),
        ];
        let reduced = reduce_contacts(&pts, 2);
        assert_eq!(reduced.len(), 2);
        assert!((reduced[0].depth - 0.5).abs() < 1e-5);
    }

    #[test]
    fn reduce_empty_input() {
        let reduced = reduce_contacts(&[], 4);
        assert!(reduced.is_empty());
    }

    #[test]
    fn reduce_max_keep_zero() {
        let pts = vec![make_contact([0.0; 3], 0.1)];
        assert!(reduce_contacts(&pts, 0).is_empty());
    }

    #[test]
    fn reduce_max_keep_larger_than_input() {
        let pts = vec![make_contact([0.0; 3], 0.2), make_contact([1.0, 0.0, 0.0], 0.1)];
        let reduced = reduce_contacts(&pts, 10);
        assert_eq!(reduced.len(), 2);
    }

    #[test]
    fn deepest_contact_correct() {
        let pts = vec![
            make_contact([0.0; 3], 0.1),
            make_contact([1.0, 0.0, 0.0], 0.8),
        ];
        let d = deepest_contact(&pts).unwrap();
        assert!((d.depth - 0.8).abs() < 1e-5);
    }

    #[test]
    fn deepest_contact_empty() {
        assert!(deepest_contact(&[]).is_none());
    }

    #[test]
    fn contact_centroid_single() {
        let pts = vec![make_contact([3.0, 2.0, 1.0], 0.1)];
        let c = contact_centroid(&pts);
        assert!((c[0] - 3.0).abs() < 1e-5);
        assert!((c[1] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn contact_centroid_two_points() {
        let pts = vec![
            make_contact([0.0, 0.0, 0.0], 0.1),
            make_contact([2.0, 0.0, 0.0], 0.1),
        ];
        let c = contact_centroid(&pts);
        assert!((c[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn contact_area_approx_two_points() {
        let pts = vec![
            make_contact([0.0, 0.0, 0.0], 0.1),
            make_contact([1.0, 1.0, 0.0], 0.1),
        ];
        let area = contact_area_approx(&pts);
        assert!((area - 1.0).abs() < 1e-5);
    }
}
