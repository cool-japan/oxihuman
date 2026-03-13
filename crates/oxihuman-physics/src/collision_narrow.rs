#![allow(dead_code)]

/// Result of a narrow-phase collision test.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NarrowPhaseResult {
    pub contacts: Vec<ContactPoint>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContactPoint {
    pub normal: [f32; 3],
    pub depth: f32,
    pub position: [f32; 3],
}

/// Sphere vs sphere collision test.
#[allow(dead_code)]
pub fn sphere_vs_sphere(
    pos_a: [f32; 3], radius_a: f32,
    pos_b: [f32; 3], radius_b: f32,
) -> NarrowPhaseResult {
    let dx = pos_b[0] - pos_a[0];
    let dy = pos_b[1] - pos_a[1];
    let dz = pos_b[2] - pos_a[2];
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    let overlap = radius_a + radius_b - dist;
    if overlap > 0.0 && dist > f32::EPSILON {
        let inv = 1.0 / dist;
        NarrowPhaseResult {
            contacts: vec![ContactPoint {
                normal: [dx * inv, dy * inv, dz * inv],
                depth: overlap,
                position: [
                    pos_a[0] + dx * 0.5,
                    pos_a[1] + dy * 0.5,
                    pos_a[2] + dz * 0.5,
                ],
            }],
        }
    } else {
        NarrowPhaseResult { contacts: vec![] }
    }
}

/// Sphere vs AABB box collision test.
#[allow(dead_code)]
pub fn sphere_vs_box(
    sphere_pos: [f32; 3], radius: f32,
    box_min: [f32; 3], box_max: [f32; 3],
) -> NarrowPhaseResult {
    let mut closest = [0.0f32; 3];
    for i in 0..3 {
        closest[i] = sphere_pos[i].clamp(box_min[i], box_max[i]);
    }
    let dx = sphere_pos[0] - closest[0];
    let dy = sphere_pos[1] - closest[1];
    let dz = sphere_pos[2] - closest[2];
    let dist_sq = dx * dx + dy * dy + dz * dz;
    if dist_sq < radius * radius && dist_sq > f32::EPSILON {
        let dist = dist_sq.sqrt();
        let inv = 1.0 / dist;
        NarrowPhaseResult {
            contacts: vec![ContactPoint {
                normal: [dx * inv, dy * inv, dz * inv],
                depth: radius - dist,
                position: closest,
            }],
        }
    } else {
        NarrowPhaseResult { contacts: vec![] }
    }
}

/// AABB vs AABB collision test.
#[allow(dead_code)]
pub fn box_vs_box(
    min_a: [f32; 3], max_a: [f32; 3],
    min_b: [f32; 3], max_b: [f32; 3],
) -> NarrowPhaseResult {
    for i in 0..3 {
        if max_a[i] < min_b[i] || max_b[i] < min_a[i] {
            return NarrowPhaseResult { contacts: vec![] };
        }
    }
    let mut min_overlap = f32::MAX;
    let mut axis = 0usize;
    for i in 0..3 {
        let overlap = (max_a[i].min(max_b[i])) - (min_a[i].max(min_b[i]));
        if overlap < min_overlap {
            min_overlap = overlap;
            axis = i;
        }
    }
    let mut normal = [0.0f32; 3];
    let center_a = (min_a[axis] + max_a[axis]) * 0.5;
    let center_b = (min_b[axis] + max_b[axis]) * 0.5;
    normal[axis] = if center_b > center_a { 1.0 } else { -1.0 };
    NarrowPhaseResult {
        contacts: vec![ContactPoint {
            normal,
            depth: min_overlap,
            position: [
                (min_a[0].max(min_b[0]) + max_a[0].min(max_b[0])) * 0.5,
                (min_a[1].max(min_b[1]) + max_a[1].min(max_b[1])) * 0.5,
                (min_a[2].max(min_b[2]) + max_a[2].min(max_b[2])) * 0.5,
            ],
        }],
    }
}

/// Capsule vs sphere collision test.
#[allow(dead_code)]
pub fn capsule_vs_sphere(
    cap_a: [f32; 3], cap_b: [f32; 3], cap_radius: f32,
    sphere_pos: [f32; 3], sphere_radius: f32,
) -> NarrowPhaseResult {
    let ab = [cap_b[0] - cap_a[0], cap_b[1] - cap_a[1], cap_b[2] - cap_a[2]];
    let ap = [sphere_pos[0] - cap_a[0], sphere_pos[1] - cap_a[1], sphere_pos[2] - cap_a[2]];
    let ab_dot = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
    let t = if ab_dot > f32::EPSILON {
        ((ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / ab_dot).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let closest = [cap_a[0] + ab[0] * t, cap_a[1] + ab[1] * t, cap_a[2] + ab[2] * t];
    sphere_vs_sphere(closest, cap_radius, sphere_pos, sphere_radius)
}

/// Capsule vs capsule collision test (simplified).
#[allow(dead_code)]
pub fn capsule_vs_capsule(
    a1: [f32; 3], a2: [f32; 3], ra: f32,
    b1: [f32; 3], b2: [f32; 3], rb: f32,
) -> NarrowPhaseResult {
    let mid_a = [(a1[0] + a2[0]) * 0.5, (a1[1] + a2[1]) * 0.5, (a1[2] + a2[2]) * 0.5];
    let mid_b = [(b1[0] + b2[0]) * 0.5, (b1[1] + b2[1]) * 0.5, (b1[2] + b2[2]) * 0.5];
    sphere_vs_sphere(mid_a, ra, mid_b, rb)
}

/// Returns the number of contacts.
#[allow(dead_code)]
pub fn narrow_contact_count(result: &NarrowPhaseResult) -> usize {
    result.contacts.len()
}

/// Returns the contact normal of the first contact.
#[allow(dead_code)]
pub fn narrow_contact_normal(result: &NarrowPhaseResult) -> Option<[f32; 3]> {
    result.contacts.first().map(|c| c.normal)
}

/// Returns the penetration depth of the first contact.
#[allow(dead_code)]
pub fn narrow_contact_depth(result: &NarrowPhaseResult) -> Option<f32> {
    result.contacts.first().map(|c| c.depth)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_vs_sphere_hit() {
        let r = sphere_vs_sphere([0.0; 3], 1.0, [1.5, 0.0, 0.0], 1.0);
        assert_eq!(narrow_contact_count(&r), 1);
    }

    #[test]
    fn test_sphere_vs_sphere_miss() {
        let r = sphere_vs_sphere([0.0; 3], 0.5, [3.0, 0.0, 0.0], 0.5);
        assert_eq!(narrow_contact_count(&r), 0);
    }

    #[test]
    fn test_sphere_vs_box_hit() {
        let r = sphere_vs_box([0.0, 0.0, 0.0], 1.0, [0.5, -1.0, -1.0], [2.0, 1.0, 1.0]);
        assert_eq!(narrow_contact_count(&r), 1);
    }

    #[test]
    fn test_sphere_vs_box_miss() {
        let r = sphere_vs_box([0.0; 3], 0.1, [5.0, 5.0, 5.0], [6.0, 6.0, 6.0]);
        assert_eq!(narrow_contact_count(&r), 0);
    }

    #[test]
    fn test_box_vs_box_hit() {
        let r = box_vs_box([0.0; 3], [2.0; 3], [1.0; 3], [3.0; 3]);
        assert_eq!(narrow_contact_count(&r), 1);
    }

    #[test]
    fn test_box_vs_box_miss() {
        let r = box_vs_box([0.0; 3], [1.0; 3], [5.0; 3], [6.0; 3]);
        assert_eq!(narrow_contact_count(&r), 0);
    }

    #[test]
    fn test_capsule_vs_sphere() {
        let r = capsule_vs_sphere(
            [0.0, 0.0, 0.0], [0.0, 2.0, 0.0], 0.5,
            [0.5, 1.0, 0.0], 0.5,
        );
        assert_eq!(narrow_contact_count(&r), 1);
    }

    #[test]
    fn test_contact_normal() {
        let r = sphere_vs_sphere([0.0; 3], 1.0, [1.5, 0.0, 0.0], 1.0);
        let n = narrow_contact_normal(&r).expect("should succeed");
        assert!(n[0] > 0.0);
    }

    #[test]
    fn test_contact_depth() {
        let r = sphere_vs_sphere([0.0; 3], 1.0, [1.5, 0.0, 0.0], 1.0);
        let d = narrow_contact_depth(&r).expect("should succeed");
        assert!(d > 0.0);
    }

    #[test]
    fn test_capsule_vs_capsule() {
        let r = capsule_vs_capsule(
            [0.0; 3], [0.0, 1.0, 0.0], 0.5,
            [0.5, 0.0, 0.0], [0.5, 1.0, 0.0], 0.5,
        );
        assert_eq!(narrow_contact_count(&r), 1);
    }
}
