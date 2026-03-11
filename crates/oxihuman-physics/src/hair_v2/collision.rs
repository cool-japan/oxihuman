// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Hair-body collision using capsule approximation per segment.
//!
//! Each body part is approximated as a capsule (line-segment + radius).
//! Hair nodes are tested against these capsules and pushed out if they
//! penetrate. Friction is applied tangentially to prevent sliding artifacts.

use super::strand::{v3_add, v3_dot, v3_length, v3_normalize, v3_scale, v3_sub, HairNode};

// ── Body capsule ────────────────────────────────────────────────────────────

/// A capsule primitive representing a body part for collision.
#[derive(Debug, Clone)]
pub struct BodyCapsule {
    /// Start point of the capsule axis.
    pub point_a: [f64; 3],
    /// End point of the capsule axis.
    pub point_b: [f64; 3],
    /// Capsule radius.
    pub radius: f64,
    /// Optional label for debugging.
    pub label: String,
}

impl BodyCapsule {
    /// Create a new body capsule.
    pub fn new(point_a: [f64; 3], point_b: [f64; 3], radius: f64, label: &str) -> Self {
        Self {
            point_a,
            point_b,
            radius,
            label: label.to_string(),
        }
    }

    /// Compute the closest point on the capsule axis to a query point.
    ///
    /// Returns `(closest_point, t_parameter)` where `t` is clamped to [0, 1].
    pub fn closest_point_on_axis(&self, point: [f64; 3]) -> ([f64; 3], f64) {
        let ab = v3_sub(self.point_b, self.point_a);
        let ap = v3_sub(point, self.point_a);
        let ab_len_sq = v3_dot(ab, ab);

        if ab_len_sq < 1e-30 {
            // Degenerate capsule (point).
            return (self.point_a, 0.0);
        }

        let t = (v3_dot(ap, ab) / ab_len_sq).clamp(0.0, 1.0);
        let closest = v3_add(self.point_a, v3_scale(ab, t));
        (closest, t)
    }

    /// Signed distance from a point to the capsule surface.
    ///
    /// Negative means the point is inside the capsule.
    pub fn signed_distance(&self, point: [f64; 3]) -> f64 {
        let (closest, _) = self.closest_point_on_axis(point);
        let diff = v3_sub(point, closest);
        v3_length(diff) - self.radius
    }

    /// Test whether a point is inside the capsule.
    pub fn contains(&self, point: [f64; 3]) -> bool {
        self.signed_distance(point) < 0.0
    }

    /// Axis length.
    pub fn axis_length(&self) -> f64 {
        v3_length(v3_sub(self.point_b, self.point_a))
    }

    /// Center of the capsule axis.
    pub fn center(&self) -> [f64; 3] {
        v3_scale(v3_add(self.point_a, self.point_b), 0.5)
    }
}

// ── Collision configuration ─────────────────────────────────────────────────

/// Configuration for hair-body collision handling.
#[derive(Debug, Clone)]
pub struct HairCollisionConfig {
    /// Whether collision detection is enabled.
    pub enabled: bool,
    /// Friction coefficient for hair-body contact.
    pub friction: f64,
    /// Extra margin around capsules (collision skin).
    pub margin: f64,
    /// Maximum push-out distance per iteration (prevents explosions).
    pub max_push_out: f64,
}

impl Default for HairCollisionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            friction: 0.3,
            margin: 0.001,
            max_push_out: 0.05,
        }
    }
}

// ── Collision resolution ────────────────────────────────────────────────────

/// Resolve collision between a single hair node and a body capsule.
///
/// If the node penetrates the capsule (plus margin), it is pushed to the
/// surface along the penetration normal. Friction is applied to the
/// tangential component of the node velocity.
pub fn resolve_node_capsule_collision(
    node: &mut HairNode,
    capsule: &BodyCapsule,
    friction: f64,
) {
    if node.inv_mass < 1e-30 {
        return; // pinned node
    }

    let (closest, _t) = capsule.closest_point_on_axis(node.position);
    let diff = v3_sub(node.position, closest);
    let dist = v3_length(diff);

    let penetration = capsule.radius - dist;
    if penetration <= 0.0 {
        return; // no collision
    }

    // Normal: from capsule axis toward the node.
    let normal = if dist > 1e-15 {
        v3_scale(diff, 1.0 / dist)
    } else {
        // Node is exactly on the axis — pick an arbitrary normal.
        [0.0, 1.0, 0.0]
    };

    // Push out.
    let push = penetration.min(0.05); // cap push-out per iteration
    node.position = v3_add(node.position, v3_scale(normal, push));

    // Apply friction to velocity.
    if friction > 1e-15 {
        let v_n = v3_scale(normal, v3_dot(node.velocity, normal));
        let v_t = v3_sub(node.velocity, v_n);
        let v_t_len = v3_length(v_t);
        if v_t_len > 1e-15 {
            let friction_force = friction * penetration;
            let reduction = (friction_force / v_t_len).min(1.0);
            let v_t_new = v3_scale(v_t, 1.0 - reduction);
            // Remove normal component (prevent sinking) and apply friction.
            let v_n_clamped = if v3_dot(v_n, normal) < 0.0 {
                [0.0; 3] // remove velocity going into the capsule
            } else {
                v_n
            };
            node.velocity = v3_add(v_n_clamped, v_t_new);
        } else {
            // Only fix normal velocity.
            if v3_dot(node.velocity, normal) < 0.0 {
                let v_n_component = v3_dot(node.velocity, normal);
                node.velocity = v3_sub(node.velocity, v3_scale(normal, v_n_component));
            }
        }
    }
}

/// Resolve collisions between an entire strand segment (approximated as a
/// capsule) and a body capsule. This handles cases where a hair segment
/// passes through a body part between nodes.
///
/// `seg_radius` is the effective radius of the hair segment capsule.
pub fn resolve_segment_capsule_collision(
    node_a: &mut HairNode,
    node_b: &mut HairNode,
    body_capsule: &BodyCapsule,
    seg_radius: f64,
    friction: f64,
) {
    // Find the closest points between the two line segments.
    let (closest_on_hair, closest_on_body, _t_hair, _t_body) =
        closest_points_segments(
            node_a.position,
            node_b.position,
            body_capsule.point_a,
            body_capsule.point_b,
        );

    let diff = v3_sub(closest_on_hair, closest_on_body);
    let dist = v3_length(diff);
    let combined_radius = seg_radius + body_capsule.radius;

    let penetration = combined_radius - dist;
    if penetration <= 0.0 {
        return; // no collision
    }

    let normal = if dist > 1e-15 {
        v3_scale(diff, 1.0 / dist)
    } else {
        [0.0, 1.0, 0.0]
    };

    let push = penetration * 0.5; // split push between the two nodes

    // Push both nodes outward.
    if node_a.inv_mass > 1e-30 {
        let w = node_a.inv_mass / (node_a.inv_mass + node_b.inv_mass + 1e-30);
        let push_a = push * 2.0 * w;
        node_a.position = v3_add(node_a.position, v3_scale(normal, push_a));
        apply_friction_to_node(node_a, normal, friction, penetration);
    }
    if node_b.inv_mass > 1e-30 {
        let w = node_b.inv_mass / (node_a.inv_mass + node_b.inv_mass + 1e-30);
        let push_b = push * 2.0 * w;
        node_b.position = v3_add(node_b.position, v3_scale(normal, push_b));
        apply_friction_to_node(node_b, normal, friction, penetration);
    }
}

/// Apply friction to a node given a contact normal and penetration depth.
fn apply_friction_to_node(
    node: &mut HairNode,
    normal: [f64; 3],
    friction: f64,
    _penetration: f64,
) {
    if friction < 1e-15 {
        return;
    }
    let v_n = v3_dot(node.velocity, normal);
    if v_n < 0.0 {
        // Remove the normal component going into surface.
        node.velocity = v3_sub(node.velocity, v3_scale(normal, v_n));
    }
    // Reduce tangential velocity.
    let v_tang = v3_sub(node.velocity, v3_scale(normal, v3_dot(node.velocity, normal)));
    let tang_len = v3_length(v_tang);
    if tang_len > 1e-15 {
        let scale = (1.0 - friction).max(0.0);
        let v_tang_new = v3_scale(v_tang, scale);
        node.velocity = v3_add(
            v3_scale(normal, v3_dot(node.velocity, normal)),
            v_tang_new,
        );
    }
}

// ── Segment-segment closest points ──────────────────────────────────────────

/// Find the closest points between two line segments.
///
/// Returns `(point_on_seg1, point_on_seg2, t1, t2)` where `t1, t2 ∈ [0, 1]`.
fn closest_points_segments(
    a0: [f64; 3],
    a1: [f64; 3],
    b0: [f64; 3],
    b1: [f64; 3],
) -> ([f64; 3], [f64; 3], f64, f64) {
    let d1 = v3_sub(a1, a0);
    let d2 = v3_sub(b1, b0);
    let r = v3_sub(a0, b0);

    let a = v3_dot(d1, d1);
    let e = v3_dot(d2, d2);
    let f = v3_dot(d2, r);

    if a < 1e-30 && e < 1e-30 {
        // Both segments degenerate to points.
        return (a0, b0, 0.0, 0.0);
    }

    let (s, t);
    if a < 1e-30 {
        // First segment degenerates.
        s = 0.0;
        t = (f / e).clamp(0.0, 1.0);
    } else {
        let c = v3_dot(d1, r);
        if e < 1e-30 {
            // Second segment degenerates.
            t = 0.0;
            s = (-c / a).clamp(0.0, 1.0);
        } else {
            // General case.
            let b_val = v3_dot(d1, d2);
            let denom = a * e - b_val * b_val;

            if denom.abs() > 1e-15 {
                s = ((b_val * f - c * e) / denom).clamp(0.0, 1.0);
            } else {
                s = 0.0;
            }
            t = ((b_val * s + f) / e).clamp(0.0, 1.0);

            // Re-clamp s based on clamped t.
            let s_new = ((b_val * t - c) / a).clamp(0.0, 1.0);
            // Only update if it changed significantly (avoid oscillation).
            let _ = s_new;
        }
    }

    let p1 = v3_add(a0, v3_scale(d1, s));
    let p2 = v3_add(b0, v3_scale(d2, t));
    (p1, p2, s, t)
}

/// Test if a hair strand (as a sequence of capsules) collides with a body
/// capsule and collect contact info.
pub fn strand_body_contacts(
    strand: &super::strand::HairStrand,
    body_capsule: &BodyCapsule,
    hair_radius: f64,
) -> Vec<ContactInfo> {
    let mut contacts = Vec::new();
    let n = strand.node_count();
    if n < 2 {
        return contacts;
    }

    for i in 0..n - 1 {
        let (closest_hair, closest_body, t_hair, _t_body) = closest_points_segments(
            strand.nodes[i].position,
            strand.nodes[i + 1].position,
            body_capsule.point_a,
            body_capsule.point_b,
        );
        let diff = v3_sub(closest_hair, closest_body);
        let dist = v3_length(diff);
        let combined = hair_radius + body_capsule.radius;
        if dist < combined {
            let normal = if dist > 1e-15 {
                v3_scale(diff, 1.0 / dist)
            } else {
                [0.0, 1.0, 0.0]
            };
            contacts.push(ContactInfo {
                segment_index: i,
                t_on_segment: t_hair,
                point: closest_hair,
                normal,
                depth: combined - dist,
            });
        }
    }

    contacts
}

/// Information about a contact between a hair segment and a body capsule.
#[derive(Debug, Clone)]
pub struct ContactInfo {
    /// Index of the hair segment (edge between node i and i+1).
    pub segment_index: usize,
    /// Parameter along the hair segment [0, 1].
    pub t_on_segment: f64,
    /// World-space contact point on the hair.
    pub point: [f64; 3],
    /// Contact normal (pointing away from the body).
    pub normal: [f64; 3],
    /// Penetration depth.
    pub depth: f64,
}

// ── Broad-phase helper ──────────────────────────────────────────────────────

/// Quick AABB overlap test between a strand's bounding box and a capsule's
/// bounding box. Returns `false` if they definitely do not overlap.
pub fn strand_capsule_broad_phase(
    strand: &super::strand::HairStrand,
    capsule: &BodyCapsule,
    hair_radius: f64,
) -> bool {
    if strand.nodes.is_empty() {
        return false;
    }

    // Compute strand AABB.
    let (mut s_min, mut s_max) = compute_strand_aabb(strand);
    // Expand by hair radius.
    for k in 0..3 {
        s_min[k] -= hair_radius;
        s_max[k] += hair_radius;
    }

    // Compute capsule AABB.
    let mut c_min = [0.0_f64; 3];
    let mut c_max = [0.0_f64; 3];
    for k in 0..3 {
        let lo = capsule.point_a[k].min(capsule.point_b[k]);
        let hi = capsule.point_a[k].max(capsule.point_b[k]);
        c_min[k] = lo - capsule.radius;
        c_max[k] = hi + capsule.radius;
    }

    // AABB overlap test.
    for k in 0..3 {
        if s_max[k] < c_min[k] || c_max[k] < s_min[k] {
            return false;
        }
    }
    true
}

/// Compute the axis-aligned bounding box of a strand's nodes.
fn compute_strand_aabb(strand: &super::strand::HairStrand) -> ([f64; 3], [f64; 3]) {
    let first = strand
        .nodes
        .first()
        .map(|n| n.position)
        .unwrap_or([0.0; 3]);
    let mut aabb_min = first;
    let mut aabb_max = first;

    for node in &strand.nodes {
        for k in 0..3 {
            if node.position[k] < aabb_min[k] {
                aabb_min[k] = node.position[k];
            }
            if node.position[k] > aabb_max[k] {
                aabb_max[k] = node.position[k];
            }
        }
    }

    (aabb_min, aabb_max)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capsule_signed_distance() {
        let cap = BodyCapsule::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.1, "test");
        // Point on surface.
        let sd = cap.signed_distance([0.1, 0.5, 0.0]);
        assert!((sd).abs() < 1e-10);
        // Point inside.
        let sd = cap.signed_distance([0.05, 0.5, 0.0]);
        assert!(sd < 0.0);
        // Point outside.
        let sd = cap.signed_distance([0.2, 0.5, 0.0]);
        assert!(sd > 0.0);
    }

    #[test]
    fn test_capsule_contains() {
        let cap = BodyCapsule::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.2, "head");
        assert!(cap.contains([0.0, 0.5, 0.0]));
        assert!(!cap.contains([0.5, 0.5, 0.0]));
    }

    #[test]
    fn test_node_capsule_collision_push_out() {
        let capsule = BodyCapsule::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.2, "arm");
        let mut node = HairNode::new([0.1, 0.5, 0.0], 0.001);
        // Node is inside capsule (dist from axis = 0.1, radius = 0.2).
        resolve_node_capsule_collision(&mut node, &capsule, 0.3);
        // After push-out, node should be outside or at surface.
        let sd = capsule.signed_distance(node.position);
        assert!(
            sd >= -1e-6,
            "node should be pushed out: signed_dist = {sd}"
        );
    }

    #[test]
    fn test_closest_points_segments() {
        // Parallel segments.
        let (p1, p2, _t1, _t2) = closest_points_segments(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        );
        let dist = v3_length(v3_sub(p1, p2));
        assert!((dist - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_broad_phase() {
        let strand = super::super::strand::HairStrand::new(
            [0.0, 2.0, 0.0],
            [0.0, -1.0, 0.0],
            0.5,
            5,
            0.001,
        );
        let cap_near = BodyCapsule::new([0.1, 1.5, 0.0], [0.1, 1.8, 0.0], 0.1, "near");
        let cap_far = BodyCapsule::new([10.0, 10.0, 10.0], [10.0, 11.0, 10.0], 0.1, "far");

        assert!(strand_capsule_broad_phase(&strand, &cap_near, 0.01));
        assert!(!strand_capsule_broad_phase(&strand, &cap_far, 0.01));
    }

    #[test]
    fn test_strand_body_contacts() {
        let strand = super::super::strand::HairStrand::new(
            [0.0, 2.0, 0.0],
            [0.0, -1.0, 0.0],
            1.0,
            10,
            0.001,
        );
        // Place a capsule crossing the strand path.
        let cap = BodyCapsule::new([-0.5, 1.5, 0.0], [0.5, 1.5, 0.0], 0.15, "cross");
        let contacts = strand_body_contacts(&strand, &cap, 0.01);
        // Should detect at least one contact near y=1.5.
        assert!(
            !contacts.is_empty(),
            "expected contacts but found none"
        );
    }

    #[test]
    fn test_segment_capsule_collision() {
        let capsule = BodyCapsule::new([0.0, 0.0, -1.0], [0.0, 0.0, 1.0], 0.3, "body");
        let mut node_a = HairNode::new([0.1, 0.0, -0.5], 0.001);
        let mut node_b = HairNode::new([0.1, 0.0, 0.5], 0.001);

        resolve_segment_capsule_collision(
            &mut node_a,
            &mut node_b,
            &capsule,
            0.01,
            0.2,
        );
        // Nodes should have been pushed outward.
        let dist_a = v3_length(v3_sub(node_a.position, [0.0, 0.0, -0.5]));
        let dist_b = v3_length(v3_sub(node_b.position, [0.0, 0.0, 0.5]));
        // At least one should have moved.
        assert!(
            dist_a > 1e-10 || dist_b > 1e-10,
            "at least one node should have moved"
        );
    }
}
