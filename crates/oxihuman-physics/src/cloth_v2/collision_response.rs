// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cloth-body collision response using signed distance fields.
//!
//! Supports collision against analytical shapes (spheres, capsules, planes)
//! and simple cloth self-collision detection via spatial hashing.

/// Configuration for collision handling.
#[derive(Debug, Clone)]
pub struct CollisionConfig {
    /// Collision margin / skin thickness.
    pub margin: f64,
    /// Friction coefficient for cloth-body contacts (0..1).
    pub friction: f64,
    /// Restitution coefficient (0 = fully inelastic, 1 = fully elastic).
    pub restitution: f64,
    /// Grid cell size for self-collision spatial hashing.
    pub self_collision_cell_size: f64,
    /// Enable self-collision detection.
    pub enable_self_collision: bool,
    /// Minimum distance for self-collision.
    pub self_collision_distance: f64,
}

impl Default for CollisionConfig {
    fn default() -> Self {
        Self {
            margin: 0.005,
            friction: 0.3,
            restitution: 0.0,
            self_collision_cell_size: 0.05,
            enable_self_collision: false,
            self_collision_distance: 0.01,
        }
    }
}

/// A collision body defined by an analytical signed distance function.
#[derive(Debug, Clone)]
pub enum CollisionBody {
    /// Infinite plane defined by normal and offset (n . x + d = 0).
    Plane {
        normal: [f64; 3],
        offset: f64,
    },
    /// Sphere defined by center and radius.
    Sphere {
        center: [f64; 3],
        radius: f64,
    },
    /// Capsule defined by two endpoints and radius.
    Capsule {
        point_a: [f64; 3],
        point_b: [f64; 3],
        radius: f64,
    },
    /// Axis-aligned box defined by min and max corners.
    Box {
        min: [f64; 3],
        max: [f64; 3],
    },
}

impl CollisionBody {
    /// Compute the signed distance from a point to this body.
    ///
    /// Negative values indicate penetration. Also returns the closest
    /// surface normal pointing outward from the body.
    pub fn signed_distance(&self, point: &[f64; 3]) -> (f64, [f64; 3]) {
        match self {
            CollisionBody::Plane { normal, offset } => {
                let dist = vec3_dot(point, normal) + offset;
                (dist, *normal)
            }
            CollisionBody::Sphere { center, radius } => {
                let diff = vec3_sub(point, center);
                let d = vec3_len(&diff);
                if d < 1e-30 {
                    // Point is at center, push along +Y by convention
                    return (*radius, [0.0, 1.0, 0.0]);
                }
                let normal = vec3_scale(&diff, 1.0 / d);
                (d - radius, normal)
            }
            CollisionBody::Capsule {
                point_a,
                point_b,
                radius,
            } => {
                let closest = closest_point_on_segment(point, point_a, point_b);
                let diff = vec3_sub(point, &closest);
                let d = vec3_len(&diff);
                if d < 1e-30 {
                    return (*radius, [0.0, 1.0, 0.0]);
                }
                let normal = vec3_scale(&diff, 1.0 / d);
                (d - radius, normal)
            }
            CollisionBody::Box { min, max } => {
                signed_distance_box(point, min, max)
            }
        }
    }
}

/// Resolve cloth-body collisions by projecting penetrating vertices out.
///
/// For each cloth vertex, tests against all collision bodies. If the signed
/// distance is less than the collision margin, the vertex is pushed to the
/// surface plus margin, and its velocity is adjusted for friction and
/// restitution.
///
/// Returns the number of collisions resolved.
pub fn resolve_cloth_body_collisions(
    positions: &mut [[f64; 3]],
    velocities: &mut [[f64; 3]],
    inv_masses: &[f64],
    bodies: &[CollisionBody],
    config: &CollisionConfig,
    dt: f64,
) -> usize {
    let n = positions.len().min(velocities.len()).min(inv_masses.len());
    let mut collision_count = 0;

    for i in 0..n {
        if inv_masses[i] <= 0.0 {
            continue;
        }

        for body in bodies {
            let (sd, normal) = body.signed_distance(&positions[i]);

            if sd < config.margin {
                // Push vertex out to surface + margin
                let penetration = config.margin - sd;
                positions[i][0] += normal[0] * penetration;
                positions[i][1] += normal[1] * penetration;
                positions[i][2] += normal[2] * penetration;

                // Velocity correction
                let v_n = vec3_dot(&velocities[i], &normal);

                if v_n < 0.0 {
                    // Remove normal component and apply restitution
                    let v_n_vec = vec3_scale(&normal, v_n);
                    let v_t = vec3_sub(&velocities[i], &v_n_vec);

                    // Normal velocity with restitution
                    let v_n_new = -config.restitution * v_n;
                    let v_n_new_vec = vec3_scale(&normal, v_n_new);

                    // Tangential velocity with friction
                    let v_t_len = vec3_len(&v_t);
                    let friction_decel = config.friction * v_n.abs();
                    let v_t_new = if v_t_len > 1e-30 && friction_decel < v_t_len {
                        let scale = 1.0 - friction_decel / v_t_len;
                        vec3_scale(&v_t, scale)
                    } else if v_t_len > 1e-30 {
                        [0.0, 0.0, 0.0]
                    } else {
                        v_t
                    };

                    velocities[i] = vec3_add(&v_n_new_vec, &v_t_new);
                }

                collision_count += 1;
            }
        }
    }

    let _ = dt; // reserved for continuous collision detection in future
    collision_count
}

/// Simple cloth self-collision detection and response using spatial hashing.
///
/// Detects vertex-vertex proximity and applies repulsion forces.
/// Returns the number of self-collision pairs resolved.
pub fn resolve_cloth_self_collisions(
    positions: &mut [[f64; 3]],
    velocities: &mut [[f64; 3]],
    inv_masses: &[f64],
    triangles: &[[usize; 3]],
    config: &CollisionConfig,
) -> usize {
    if !config.enable_self_collision {
        return 0;
    }

    let n = positions.len().min(velocities.len()).min(inv_masses.len());
    if n == 0 {
        return 0;
    }

    let cell_size = config.self_collision_cell_size;
    if cell_size < 1e-30 {
        return 0;
    }

    // Build adjacency set for excluding connected vertices
    let mut adjacency = std::collections::HashSet::new();
    for tri in triangles {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            adjacency.insert(key);
        }
    }

    // Spatial hash
    let inv_cell = 1.0 / cell_size;
    let mut grid: std::collections::HashMap<(i64, i64, i64), Vec<usize>> =
        std::collections::HashMap::new();

    for i in 0..n {
        let cx = (positions[i][0] * inv_cell).floor() as i64;
        let cy = (positions[i][1] * inv_cell).floor() as i64;
        let cz = (positions[i][2] * inv_cell).floor() as i64;
        grid.entry((cx, cy, cz)).or_default().push(i);
    }

    let mut collision_count = 0;
    let min_dist = config.self_collision_distance;
    let min_dist_sq = min_dist * min_dist;

    for i in 0..n {
        if inv_masses[i] <= 0.0 {
            continue;
        }

        let cx = (positions[i][0] * inv_cell).floor() as i64;
        let cy = (positions[i][1] * inv_cell).floor() as i64;
        let cz = (positions[i][2] * inv_cell).floor() as i64;

        // Check 3x3x3 neighborhood
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let cell_key = (cx + dx, cy + dy, cz + dz);
                    let candidates = match grid.get(&cell_key) {
                        Some(c) => c,
                        None => continue,
                    };

                    for &j in candidates {
                        if j <= i {
                            continue;
                        }
                        if inv_masses[j] <= 0.0 {
                            continue;
                        }

                        // Skip adjacent vertices
                        let edge_key = if i < j { (i, j) } else { (j, i) };
                        if adjacency.contains(&edge_key) {
                            continue;
                        }

                        let diff = vec3_sub(&positions[i], &positions[j]);
                        let dist_sq = vec3_dot(&diff, &diff);

                        if dist_sq < min_dist_sq && dist_sq > 1e-30 {
                            let dist = dist_sq.sqrt();
                            let normal = vec3_scale(&diff, 1.0 / dist);
                            let penetration = min_dist - dist;

                            let wi = inv_masses[i];
                            let wj = inv_masses[j];
                            let w_sum = wi + wj;
                            if w_sum < 1e-30 {
                                continue;
                            }

                            let correction_i = penetration * wi / w_sum;
                            let correction_j = penetration * wj / w_sum;

                            positions[i][0] += normal[0] * correction_i;
                            positions[i][1] += normal[1] * correction_i;
                            positions[i][2] += normal[2] * correction_i;

                            if let Some(pj) = positions.get_mut(j) {
                                pj[0] -= normal[0] * correction_j;
                                pj[1] -= normal[1] * correction_j;
                                pj[2] -= normal[2] * correction_j;
                            }

                            collision_count += 1;
                        }
                    }
                }
            }
        }
    }

    collision_count
}

// ─── Signed Distance for Box ──────────────────────────────────────────────

fn signed_distance_box(point: &[f64; 3], min: &[f64; 3], max: &[f64; 3]) -> (f64, [f64; 3]) {
    let center = [
        (min[0] + max[0]) * 0.5,
        (min[1] + max[1]) * 0.5,
        (min[2] + max[2]) * 0.5,
    ];
    let half = [
        (max[0] - min[0]) * 0.5,
        (max[1] - min[1]) * 0.5,
        (max[2] - min[2]) * 0.5,
    ];

    let local = vec3_sub(point, &center);

    // Distance to box surface
    let dx = local[0].abs() - half[0];
    let dy = local[1].abs() - half[1];
    let dz = local[2].abs() - half[2];

    let outside_dist = [dx.max(0.0), dy.max(0.0), dz.max(0.0)];
    let outside_len = vec3_len(&outside_dist);

    let inside_dist = dx.max(dy).max(dz).min(0.0);

    let sd = outside_len + inside_dist;

    // Compute normal
    let normal = if sd > 1e-10 {
        // Outside: normal points away from closest surface point
        if outside_len > 1e-30 {
            vec3_scale(&outside_dist, 1.0 / outside_len)
        } else {
            // On surface, use component with smallest absolute distance
            box_interior_normal(&local, &half)
        }
    } else {
        box_interior_normal(&local, &half)
    };

    // Adjust normal signs to match the quadrant
    let signed_normal = [
        normal[0].copysign(if local[0] >= 0.0 { 1.0 } else { -1.0 }),
        normal[1].copysign(if local[1] >= 0.0 { 1.0 } else { -1.0 }),
        normal[2].copysign(if local[2] >= 0.0 { 1.0 } else { -1.0 }),
    ];

    let normal_len = vec3_len(&signed_normal);
    if normal_len > 1e-30 {
        (sd, vec3_scale(&signed_normal, 1.0 / normal_len))
    } else {
        (sd, [0.0, 1.0, 0.0])
    }
}

fn box_interior_normal(local: &[f64; 3], half: &[f64; 3]) -> [f64; 3] {
    let dx = half[0] - local[0].abs();
    let dy = half[1] - local[1].abs();
    let dz = half[2] - local[2].abs();

    if dx <= dy && dx <= dz {
        [1.0, 0.0, 0.0]
    } else if dy <= dz {
        [0.0, 1.0, 0.0]
    } else {
        [0.0, 0.0, 1.0]
    }
}

// ─── Geometry Helpers ─────────────────────────────────────────────────────

fn closest_point_on_segment(point: &[f64; 3], a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    let ab = vec3_sub(b, a);
    let ap = vec3_sub(point, a);
    let ab_sq = vec3_dot(&ab, &ab);

    if ab_sq < 1e-30 {
        return *a;
    }

    let t = (vec3_dot(&ap, &ab) / ab_sq).clamp(0.0, 1.0);

    [
        a[0] + ab[0] * t,
        a[1] + ab[1] * t,
        a[2] + ab[2] * t,
    ]
}

// ─── Vec3 math ────────────────────────────────────────────────────────────

#[inline]
fn vec3_sub(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn vec3_add(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn vec3_scale(a: &[f64; 3], s: f64) -> [f64; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
fn vec3_dot(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn vec3_len(a: &[f64; 3]) -> f64 {
    vec3_dot(a, a).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plane_sdf() {
        let body = CollisionBody::Plane {
            normal: [0.0, 1.0, 0.0],
            offset: 0.0,
        };
        let (sd, normal) = body.signed_distance(&[0.0, 0.5, 0.0]);
        assert!((sd - 0.5).abs() < 1e-10);
        assert!((normal[1] - 1.0).abs() < 1e-10);

        let (sd_neg, _) = body.signed_distance(&[0.0, -0.3, 0.0]);
        assert!(sd_neg < 0.0);
    }

    #[test]
    fn test_sphere_sdf() {
        let body = CollisionBody::Sphere {
            center: [0.0, 0.0, 0.0],
            radius: 1.0,
        };
        let (sd_outside, _) = body.signed_distance(&[2.0, 0.0, 0.0]);
        assert!((sd_outside - 1.0).abs() < 1e-10);

        let (sd_inside, _) = body.signed_distance(&[0.5, 0.0, 0.0]);
        assert!(sd_inside < 0.0);
    }

    #[test]
    fn test_capsule_sdf() {
        let body = CollisionBody::Capsule {
            point_a: [0.0, 0.0, 0.0],
            point_b: [0.0, 2.0, 0.0],
            radius: 0.5,
        };
        let (sd, _) = body.signed_distance(&[1.0, 1.0, 0.0]);
        assert!((sd - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_resolve_plane_collision() {
        let mut positions = vec![[0.0, -0.1, 0.0]];
        let mut velocities = vec![[0.0, -5.0, 0.0]];
        let inv_masses = vec![1.0];
        let bodies = vec![CollisionBody::Plane {
            normal: [0.0, 1.0, 0.0],
            offset: 0.0,
        }];
        let config = CollisionConfig {
            margin: 0.01,
            friction: 0.0,
            restitution: 0.5,
            ..CollisionConfig::default()
        };

        let count = resolve_cloth_body_collisions(
            &mut positions,
            &mut velocities,
            &inv_masses,
            &bodies,
            &config,
            0.01,
        );

        assert_eq!(count, 1);
        assert!(positions[0][1] >= 0.0, "Vertex should be pushed above plane");
        assert!(velocities[0][1] > 0.0, "Velocity should be reflected");
    }

    #[test]
    fn test_pinned_vertex_no_collision() {
        let mut positions = vec![[0.0, -1.0, 0.0]];
        let mut velocities = vec![[0.0, -5.0, 0.0]];
        let inv_masses = vec![0.0]; // pinned
        let bodies = vec![CollisionBody::Plane {
            normal: [0.0, 1.0, 0.0],
            offset: 0.0,
        }];
        let config = CollisionConfig::default();

        let count = resolve_cloth_body_collisions(
            &mut positions,
            &mut velocities,
            &inv_masses,
            &bodies,
            &config,
            0.01,
        );

        assert_eq!(count, 0);
    }

    #[test]
    fn test_box_sdf_outside() {
        let body = CollisionBody::Box {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        };
        let (sd, _) = body.signed_distance(&[2.0, 0.0, 0.0]);
        assert!((sd - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_box_sdf_inside() {
        let body = CollisionBody::Box {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        };
        let (sd, _) = body.signed_distance(&[0.0, 0.0, 0.0]);
        assert!(sd < 0.0);
    }
}
