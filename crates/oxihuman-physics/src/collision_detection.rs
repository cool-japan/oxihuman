// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Broad + narrow phase collision detection pipeline.

/// Axis-aligned bounding box (3D).
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub struct Aabb3 {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Aabb3 {
    #[allow(dead_code)]
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }

    /// Test AABB overlap.
    #[allow(dead_code)]
    pub fn overlaps(&self, other: &Aabb3) -> bool {
        self.min[0] <= other.max[0]
            && self.max[0] >= other.min[0]
            && self.min[1] <= other.max[1]
            && self.max[1] >= other.min[1]
            && self.min[2] <= other.max[2]
            && self.max[2] >= other.min[2]
    }

    /// Expand AABB by margin.
    #[allow(dead_code)]
    pub fn expanded(&self, margin: f32) -> Self {
        Self {
            min: [
                self.min[0] - margin,
                self.min[1] - margin,
                self.min[2] - margin,
            ],
            max: [
                self.max[0] + margin,
                self.max[1] + margin,
                self.max[2] + margin,
            ],
        }
    }

    /// Surface area (used in BVH cost).
    #[allow(dead_code)]
    pub fn surface_area(&self) -> f32 {
        let d = [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ];
        2.0 * (d[0] * d[1] + d[1] * d[2] + d[2] * d[0])
    }

    /// Center of the AABB.
    #[allow(dead_code)]
    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }

    /// Merge two AABBs.
    #[allow(dead_code)]
    pub fn merge(&self, other: &Aabb3) -> Self {
        Self {
            min: [
                self.min[0].min(other.min[0]),
                self.min[1].min(other.min[1]),
                self.min[2].min(other.min[2]),
            ],
            max: [
                self.max[0].max(other.max[0]),
                self.max[1].max(other.max[1]),
                self.max[2].max(other.max[2]),
            ],
        }
    }

    /// Contains point.
    #[allow(dead_code)]
    pub fn contains_point(&self, p: [f32; 3]) -> bool {
        p[0] >= self.min[0]
            && p[0] <= self.max[0]
            && p[1] >= self.min[1]
            && p[1] <= self.max[1]
            && p[2] >= self.min[2]
            && p[2] <= self.max[2]
    }
}

/// A collidable object descriptor for broad phase.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BroadPhaseObject {
    pub id: usize,
    pub aabb: Aabb3,
    pub group: u32,
    pub mask: u32,
}

impl BroadPhaseObject {
    #[allow(dead_code)]
    pub fn new(id: usize, aabb: Aabb3) -> Self {
        Self {
            id,
            aabb,
            group: 1,
            mask: !0,
        }
    }

    /// Check if this object can collide with another based on group/mask.
    #[allow(dead_code)]
    pub fn can_collide_with(&self, other: &BroadPhaseObject) -> bool {
        (self.mask & other.group) != 0 && (other.mask & self.group) != 0
    }
}

/// Candidate overlap pair from broad phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub struct OverlapPair {
    pub id_a: usize,
    pub id_b: usize,
}

impl OverlapPair {
    #[allow(dead_code)]
    pub fn new(a: usize, b: usize) -> Self {
        if a <= b {
            Self { id_a: a, id_b: b }
        } else {
            Self { id_a: b, id_b: a }
        }
    }
}

/// Broad phase: O(n^2) AABB overlap test (brute force).
#[allow(dead_code)]
pub fn broad_phase_brute(objects: &[BroadPhaseObject]) -> Vec<OverlapPair> {
    let mut pairs = Vec::new();
    for i in 0..objects.len() {
        for j in (i + 1)..objects.len() {
            let a = &objects[i];
            let b = &objects[j];
            if a.id == b.id {
                continue;
            }
            if !a.can_collide_with(b) {
                continue;
            }
            if a.aabb.overlaps(&b.aabb) {
                pairs.push(OverlapPair::new(a.id, b.id));
            }
        }
    }
    pairs
}

/// Broad phase: Sort-and-sweep on X axis.
#[allow(dead_code)]
pub fn broad_phase_sap_x(objects: &[BroadPhaseObject]) -> Vec<OverlapPair> {
    let mut sorted: Vec<usize> = (0..objects.len()).collect();
    sorted.sort_by(|&a, &b| {
        objects[a].aabb.min[0]
            .partial_cmp(&objects[b].aabb.min[0])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut pairs = Vec::new();
    for i in 0..sorted.len() {
        let ai = sorted[i];
        for bi in sorted.iter().skip(i + 1).copied() {
            let a = &objects[ai];
            let b = &objects[bi];
            if b.aabb.min[0] > a.aabb.max[0] {
                break;
            }
            if !a.can_collide_with(b) {
                continue;
            }
            if a.aabb.overlaps(&b.aabb) {
                pairs.push(OverlapPair::new(a.id, b.id));
            }
        }
    }
    pairs
}

/// Contact point from narrow phase.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ContactPoint {
    pub point: [f32; 3],
    pub normal: [f32; 3],
    pub depth: f32,
    pub id_a: usize,
    pub id_b: usize,
}

impl ContactPoint {
    #[allow(dead_code)]
    pub fn new(point: [f32; 3], normal: [f32; 3], depth: f32, id_a: usize, id_b: usize) -> Self {
        Self {
            point,
            normal,
            depth,
            id_a,
            id_b,
        }
    }
}

/// Sphere collider.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Sphere {
    pub center: [f32; 3],
    pub radius: f32,
}

impl Sphere {
    #[allow(dead_code)]
    pub fn new(center: [f32; 3], radius: f32) -> Self {
        Self { center, radius }
    }

    #[allow(dead_code)]
    pub fn aabb(&self) -> Aabb3 {
        Aabb3::new(
            [
                self.center[0] - self.radius,
                self.center[1] - self.radius,
                self.center[2] - self.radius,
            ],
            [
                self.center[0] + self.radius,
                self.center[1] + self.radius,
                self.center[2] + self.radius,
            ],
        )
    }
}

/// Narrow phase: sphere–sphere contact.
#[allow(dead_code)]
pub fn narrow_sphere_sphere(
    a: &Sphere,
    b: &Sphere,
    id_a: usize,
    id_b: usize,
) -> Option<ContactPoint> {
    let dx = b.center[0] - a.center[0];
    let dy = b.center[1] - a.center[1];
    let dz = b.center[2] - a.center[2];
    let dist_sq = dx * dx + dy * dy + dz * dz;
    let radii_sum = a.radius + b.radius;
    if dist_sq >= radii_sum * radii_sum {
        return None;
    }
    let dist = dist_sq.sqrt();
    let depth = radii_sum - dist;
    let (nx, ny, nz) = if dist < 1e-8 {
        (0.0f32, 1.0f32, 0.0f32)
    } else {
        (dx / dist, dy / dist, dz / dist)
    };
    let point = [
        a.center[0] + nx * a.radius,
        a.center[1] + ny * a.radius,
        a.center[2] + nz * a.radius,
    ];
    Some(ContactPoint::new(point, [nx, ny, nz], depth, id_a, id_b))
}

/// Narrow phase: sphere–plane contact.
/// Plane defined by normal and offset d (n·x = d).
#[allow(dead_code)]
pub fn narrow_sphere_plane(
    sphere: &Sphere,
    plane_normal: [f32; 3],
    plane_d: f32,
    id_sphere: usize,
    id_plane: usize,
) -> Option<ContactPoint> {
    let n = plane_normal;
    let dist =
        n[0] * sphere.center[0] + n[1] * sphere.center[1] + n[2] * sphere.center[2] - plane_d;
    let depth = sphere.radius - dist;
    if depth <= 0.0 {
        return None;
    }
    let point = [
        sphere.center[0] - n[0] * sphere.radius,
        sphere.center[1] - n[1] * sphere.radius,
        sphere.center[2] - n[2] * sphere.radius,
    ];
    Some(ContactPoint::new(point, n, depth, id_sphere, id_plane))
}

/// Full pipeline: broad + narrow for sphere–sphere pairs.
#[allow(dead_code)]
pub fn detect_sphere_contacts(spheres: &[(usize, Sphere)]) -> Vec<ContactPoint> {
    // Broad phase
    let objects: Vec<BroadPhaseObject> = spheres
        .iter()
        .map(|(id, s)| BroadPhaseObject::new(*id, s.aabb()))
        .collect();
    let pairs = broad_phase_sap_x(&objects);

    // Narrow phase
    let mut contacts = Vec::new();
    for pair in &pairs {
        let sa = spheres.iter().find(|(id, _)| *id == pair.id_a);
        let sb = spheres.iter().find(|(id, _)| *id == pair.id_b);
        if let (Some((_, sphere_a)), Some((_, sphere_b))) = (sa, sb) {
            if let Some(c) = narrow_sphere_sphere(sphere_a, sphere_b, pair.id_a, pair.id_b) {
                contacts.push(c);
            }
        }
    }
    contacts
}

/// Count overlap pairs from broad phase.
#[allow(dead_code)]
pub fn overlap_count(objects: &[BroadPhaseObject]) -> usize {
    broad_phase_brute(objects).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_aabb(cx: f32, cy: f32, cz: f32) -> Aabb3 {
        Aabb3::new(
            [cx - 0.5, cy - 0.5, cz - 0.5],
            [cx + 0.5, cy + 0.5, cz + 0.5],
        )
    }

    #[test]
    fn aabb_overlap_touching() {
        let a = unit_aabb(0.0, 0.0, 0.0);
        let b = unit_aabb(1.0, 0.0, 0.0);
        assert!(a.overlaps(&b));
    }

    #[test]
    fn aabb_no_overlap_separated() {
        let a = unit_aabb(0.0, 0.0, 0.0);
        let b = unit_aabb(2.0, 0.0, 0.0);
        assert!(!a.overlaps(&b));
    }

    #[test]
    fn aabb_contains_point() {
        let a = unit_aabb(0.0, 0.0, 0.0);
        assert!(a.contains_point([0.0, 0.0, 0.0]));
        assert!(!a.contains_point([1.0, 0.0, 0.0]));
    }

    #[test]
    fn aabb_merge() {
        let a = Aabb3::new([0.0; 3], [1.0; 3]);
        let b = Aabb3::new([0.5; 3], [2.0; 3]);
        let m = a.merge(&b);
        assert!((m.max[0] - 2.0).abs() < 1e-6);
        assert!((m.min[0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn broad_brute_finds_overlap() {
        let mut objs = vec![
            BroadPhaseObject::new(0, unit_aabb(0.0, 0.0, 0.0)),
            BroadPhaseObject::new(1, unit_aabb(0.5, 0.0, 0.0)),
        ];
        let pairs = broad_phase_brute(&objs);
        assert_eq!(pairs.len(), 1);
        // No overlap when separated
        objs[1].aabb = unit_aabb(5.0, 0.0, 0.0);
        let pairs2 = broad_phase_brute(&objs);
        assert_eq!(pairs2.len(), 0);
    }

    #[test]
    fn broad_sap_matches_brute() {
        let objs = vec![
            BroadPhaseObject::new(0, unit_aabb(0.0, 0.0, 0.0)),
            BroadPhaseObject::new(1, unit_aabb(0.3, 0.0, 0.0)),
            BroadPhaseObject::new(2, unit_aabb(5.0, 0.0, 0.0)),
        ];
        let brute = broad_phase_brute(&objs);
        let sap = broad_phase_sap_x(&objs);
        assert_eq!(brute.len(), sap.len());
    }

    #[test]
    fn sphere_sphere_contact_detected() {
        let a = Sphere::new([0.0; 3], 1.0);
        let b = Sphere::new([1.5, 0.0, 0.0], 1.0);
        let c = narrow_sphere_sphere(&a, &b, 0, 1);
        assert!(c.is_some());
        let contact = c.unwrap();
        assert!(contact.depth > 0.0);
    }

    #[test]
    fn sphere_sphere_no_contact_far() {
        let a = Sphere::new([0.0; 3], 0.5);
        let b = Sphere::new([5.0, 0.0, 0.0], 0.5);
        let c = narrow_sphere_sphere(&a, &b, 0, 1);
        assert!(c.is_none());
    }

    #[test]
    fn sphere_plane_contact_detected() {
        let s = Sphere::new([0.0, 0.4, 0.0], 0.5);
        let c = narrow_sphere_plane(&s, [0.0, 1.0, 0.0], 0.0, 0, 99);
        assert!(c.is_some());
    }

    #[test]
    fn sphere_plane_no_contact_above() {
        let s = Sphere::new([0.0, 2.0, 0.0], 0.5);
        let c = narrow_sphere_plane(&s, [0.0, 1.0, 0.0], 0.0, 0, 99);
        assert!(c.is_none());
    }

    #[test]
    fn detect_sphere_contacts_pipeline() {
        let spheres = vec![
            (0, Sphere::new([0.0; 3], 1.0)),
            (1, Sphere::new([1.5, 0.0, 0.0], 1.0)),
            (2, Sphere::new([10.0, 0.0, 0.0], 1.0)),
        ];
        let contacts = detect_sphere_contacts(&spheres);
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].id_a, 0);
        assert_eq!(contacts[0].id_b, 1);
    }
}
