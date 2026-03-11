// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! View frustum culling, backface culling, and visibility classification.
//!
//! Provides plane extraction (Gribb-Hartmann), frustum-AABB tests, sphere tests,
//! backface culling over mesh index buffers, and a `Visibility` enum for
//! inside/outside/intersecting classification.

use crate::bounds::Aabb;
use crate::mesh::MeshBuffers;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l < f32::EPSILON {
        [0.0, 0.0, 0.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Plane
// ─────────────────────────────────────────────────────────────────────────────

/// A half-space defined by a plane (normal + signed distance from origin).
///
/// Points satisfying `dot(normal, P) + d >= 0` are considered *inside*
/// (on the positive side of the plane).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    /// Unit normal pointing toward the inside of the half-space.
    pub normal: [f32; 3],
    /// Signed distance from the world origin along `normal`.
    pub d: f32,
}

impl Plane {
    /// Construct a plane from a normal and a signed distance.
    #[allow(dead_code)]
    pub fn new(normal: [f32; 3], d: f32) -> Self {
        Plane { normal, d }
    }

    /// Signed distance from `point` to the plane.
    ///
    /// Positive values mean the point is on the inside (same side as `normal`).
    #[allow(dead_code)]
    pub fn signed_distance(&self, point: [f32; 3]) -> f32 {
        dot3(self.normal, point) + self.d
    }

    /// Return a new `Plane` with a unit-length normal.
    #[allow(dead_code)]
    pub fn normalize(&self) -> Self {
        let l = len3(self.normal);
        if l < f32::EPSILON {
            return *self;
        }
        Plane {
            normal: [self.normal[0] / l, self.normal[1] / l, self.normal[2] / l],
            d: self.d / l,
        }
    }

    /// Construct a plane through three points with CCW winding → outward normal.
    #[allow(dead_code)]
    pub fn from_points(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> Self {
        let ab = sub3(b, a);
        let ac = sub3(c, a);
        let normal = normalize3(cross3(ab, ac));
        // d = -dot(normal, a)  so that dot(normal, a) + d = 0
        let d = -dot3(normal, a);
        Plane { normal, d }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Frustum
// ─────────────────────────────────────────────────────────────────────────────

/// A view frustum represented as six half-spaces.
///
/// Plane order: `[near, far, left, right, bottom, top]`.
/// Each plane's normal points **inward** (toward the interior of the frustum).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Frustum {
    /// `[near, far, left, right, bottom, top]`
    pub planes: [Plane; 6],
}

impl Frustum {
    /// Extract frustum planes from a row-major 4×4 view-projection matrix using
    /// the Gribb-Hartmann method.
    ///
    /// The matrix is stored in *row-major* order as `m[row * 4 + col]`, so:
    /// ```text
    /// row 0 = [m[0],  m[1],  m[2],  m[3] ]
    /// row 1 = [m[4],  m[5],  m[6],  m[7] ]
    /// row 2 = [m[8],  m[9],  m[10], m[11]]
    /// row 3 = [m[12], m[13], m[14], m[15]]
    /// ```
    ///
    /// Planes (all normalized):
    /// * Near:   row3 + row2
    /// * Far:    row3 − row2
    /// * Left:   row3 + row0
    /// * Right:  row3 − row0
    /// * Bottom: row3 + row1
    /// * Top:    row3 − row1
    #[allow(dead_code)]
    pub fn from_matrix(m: &[f32; 16]) -> Self {
        // Row accessors
        let r0 = [m[0], m[1], m[2], m[3]];
        let r1 = [m[4], m[5], m[6], m[7]];
        let r2 = [m[8], m[9], m[10], m[11]];
        let r3 = [m[12], m[13], m[14], m[15]];

        let make_plane = |a: [f32; 4], b: [f32; 4]| -> Plane {
            let raw = Plane {
                normal: [a[0] + b[0], a[1] + b[1], a[2] + b[2]],
                d: a[3] + b[3],
            };
            raw.normalize()
        };

        let make_plane_sub = |a: [f32; 4], b: [f32; 4]| -> Plane {
            let raw = Plane {
                normal: [a[0] - b[0], a[1] - b[1], a[2] - b[2]],
                d: a[3] - b[3],
            };
            raw.normalize()
        };

        let near = make_plane(r3, r2);
        let far = make_plane_sub(r3, r2);
        let left = make_plane(r3, r0);
        let right = make_plane_sub(r3, r0);
        let bottom = make_plane(r3, r1);
        let top = make_plane_sub(r3, r1);

        Frustum {
            planes: [near, far, left, right, bottom, top],
        }
    }

    /// Returns `true` if `point` is inside (or on the boundary of) every plane.
    #[allow(dead_code)]
    pub fn contains_point(&self, point: [f32; 3]) -> bool {
        self.planes.iter().all(|p| p.signed_distance(point) >= 0.0)
    }

    /// Returns `true` if the sphere (center + radius) overlaps the frustum.
    ///
    /// A sphere is *outside* if its signed distance to any plane is less than
    /// `-radius`.
    #[allow(dead_code)]
    pub fn intersects_sphere(&self, center: [f32; 3], radius: f32) -> bool {
        self.planes
            .iter()
            .all(|p| p.signed_distance(center) >= -radius)
    }

    /// Returns `true` if the AABB intersects (overlaps) the frustum.
    ///
    /// For each plane the *positive vertex* (the corner most in the direction of
    /// the plane normal) is tested; if it is on the negative side the AABB is
    /// fully outside.
    #[allow(dead_code)]
    pub fn intersects_aabb(&self, aabb: &Aabb) -> bool {
        for plane in &self.planes {
            // Positive vertex: choose max or min per axis based on normal sign.
            let px = if plane.normal[0] >= 0.0 {
                aabb.max[0]
            } else {
                aabb.min[0]
            };
            let py = if plane.normal[1] >= 0.0 {
                aabb.max[1]
            } else {
                aabb.min[1]
            };
            let pz = if plane.normal[2] >= 0.0 {
                aabb.max[2]
            } else {
                aabb.min[2]
            };
            if plane.signed_distance([px, py, pz]) < 0.0 {
                return false;
            }
        }
        true
    }

    /// Build an orthographic frustum directly from axis-aligned bounds.
    ///
    /// Plane normals all point inward (toward the center of the frustum).
    #[allow(dead_code)]
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        // near plane:   Z = near, normal = +Z (inward toward interior)
        let p_near = Plane::new([0.0, 0.0, 1.0], -near);
        // far plane:    Z = far,  normal = -Z
        let p_far = Plane::new([0.0, 0.0, -1.0], far);
        // left plane:   X = left, normal = +X
        let p_left = Plane::new([1.0, 0.0, 0.0], -left);
        // right plane:  X = right, normal = -X
        let p_right = Plane::new([-1.0, 0.0, 0.0], right);
        // bottom plane: Y = bottom, normal = +Y
        let p_bottom = Plane::new([0.0, 1.0, 0.0], -bottom);
        // top plane:    Y = top,   normal = -Y
        let p_top = Plane::new([0.0, -1.0, 0.0], top);

        Frustum {
            planes: [p_near, p_far, p_left, p_right, p_bottom, p_top],
        }
    }

    /// Build a perspective frustum from field-of-view, aspect ratio, and clip planes.
    ///
    /// `fov_y_rad` is the vertical field of view in radians.
    /// `aspect` is width / height.
    #[allow(dead_code)]
    pub fn perspective(fov_y_rad: f32, aspect: f32, near: f32, far: f32) -> Self {
        let half_v = (fov_y_rad * 0.5).tan();
        let half_h = half_v * aspect;

        // Near / Far planes (along Z axis, camera looks toward +Z in view space).
        let p_near = Plane::new([0.0, 0.0, 1.0], -near);
        let p_far = Plane::new([0.0, 0.0, -1.0], far);

        // Left / Right planes.
        // The right edge at near plane is at x = near * half_h.
        // Normal of right plane points inward (toward -x side), derived from
        // the plane passing through origin with slope.
        let right_normal = normalize3([-1.0, 0.0, half_h]);
        let p_right = Plane::new(right_normal, 0.0);

        let left_normal = normalize3([1.0, 0.0, half_h]);
        let p_left = Plane::new(left_normal, 0.0);

        // Top / Bottom planes.
        let top_normal = normalize3([0.0, -1.0, half_v]);
        let p_top = Plane::new(top_normal, 0.0);

        let bottom_normal = normalize3([0.0, 1.0, half_v]);
        let p_bottom = Plane::new(bottom_normal, 0.0);

        Frustum {
            planes: [p_near, p_far, p_left, p_right, p_bottom, p_top],
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Visibility
// ─────────────────────────────────────────────────────────────────────────────

/// Visibility classification of a bounding volume relative to a frustum.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    /// Completely inside the frustum.
    Inside,
    /// Completely outside the frustum (can be culled).
    Outside,
    /// Partially overlapping the frustum boundary.
    Intersecting,
}

/// Classify an AABB's visibility relative to a `Frustum`.
///
/// Returns:
/// * `Outside`      – AABB is completely outside at least one plane.
/// * `Inside`       – AABB is fully on the positive side of all planes.
/// * `Intersecting` – AABB straddles at least one plane.
#[allow(dead_code)]
pub fn classify_aabb(frustum: &Frustum, aabb: &Aabb) -> Visibility {
    let mut all_inside = true;

    for plane in &frustum.planes {
        // Positive vertex (most in the direction of the normal).
        let px = if plane.normal[0] >= 0.0 {
            aabb.max[0]
        } else {
            aabb.min[0]
        };
        let py = if plane.normal[1] >= 0.0 {
            aabb.max[1]
        } else {
            aabb.min[1]
        };
        let pz = if plane.normal[2] >= 0.0 {
            aabb.max[2]
        } else {
            aabb.min[2]
        };

        // Negative vertex (least in the direction of the normal).
        let nx = if plane.normal[0] >= 0.0 {
            aabb.min[0]
        } else {
            aabb.max[0]
        };
        let ny = if plane.normal[1] >= 0.0 {
            aabb.min[1]
        } else {
            aabb.max[1]
        };
        let nz = if plane.normal[2] >= 0.0 {
            aabb.min[2]
        } else {
            aabb.max[2]
        };

        if plane.signed_distance([px, py, pz]) < 0.0 {
            return Visibility::Outside;
        }
        if plane.signed_distance([nx, ny, nz]) < 0.0 {
            all_inside = false;
        }
    }

    if all_inside {
        Visibility::Inside
    } else {
        Visibility::Intersecting
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Backface culling
// ─────────────────────────────────────────────────────────────────────────────

/// Return `true` if the triangle `(p0, p1, p2)` is front-facing with respect to
/// `view_dir` (the direction **from the camera toward the triangle**, normalized).
///
/// A triangle is front-facing when the angle between its face normal and the
/// negated view direction is less than 90 degrees, i.e.
/// `dot(face_normal, -view_dir) > 0`.
#[allow(dead_code)]
pub fn is_front_facing(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], view_dir: [f32; 3]) -> bool {
    let ab = sub3(p1, p0);
    let ac = sub3(p2, p0);
    let face_normal = cross3(ab, ac);
    // Front-facing if face normal opposes the view direction.
    dot3(face_normal, view_dir) < 0.0
}

/// Cull backfaces from a mesh: return the **triangle indices** (into `mesh.indices`)
/// of front-facing triangles only.
///
/// Each entry `i` in the returned `Vec` corresponds to the triangle whose first
/// index lives at `mesh.indices[i * 3]`.
///
/// `camera_pos`: world-space camera position.
#[allow(dead_code)]
pub fn backface_cull(mesh: &MeshBuffers, camera_pos: [f32; 3]) -> Vec<usize> {
    let positions = &mesh.positions;
    let indices = &mesh.indices;
    let face_count = indices.len() / 3;

    let mut result = Vec::with_capacity(face_count);

    for tri in 0..face_count {
        let i0 = indices[tri * 3] as usize;
        let i1 = indices[tri * 3 + 1] as usize;
        let i2 = indices[tri * 3 + 2] as usize;

        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }

        let p0 = positions[i0];
        let p1 = positions[i1];
        let p2 = positions[i2];

        // Compute centroid for view direction.
        let centroid = [
            (p0[0] + p1[0] + p2[0]) / 3.0,
            (p0[1] + p1[1] + p2[1]) / 3.0,
            (p0[2] + p1[2] + p2[2]) / 3.0,
        ];
        let view_dir = normalize3(sub3(centroid, camera_pos));

        if is_front_facing(p0, p1, p2, view_dir) {
            result.push(tri);
        }
    }

    result
}

/// Count front-facing triangles from a given camera position.
#[allow(dead_code)]
pub fn count_front_facing(mesh: &MeshBuffers, camera_pos: [f32; 3]) -> usize {
    backface_cull(mesh, camera_pos).len()
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bounds::Aabb;
    use crate::mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    // ── helpers ──────────────────────────────────────────────────────────────

    fn ortho_frustum() -> Frustum {
        Frustum::orthographic(-1.0, 1.0, -1.0, 1.0, 0.0, 10.0)
    }

    fn small_aabb(cx: f32, cy: f32, cz: f32, half: f32) -> Aabb {
        Aabb {
            min: [cx - half, cy - half, cz - half],
            max: [cx + half, cy + half, cz + half],
        }
    }

    fn two_tri_mesh() -> MeshBuffers {
        // Two triangles facing +Z, arranged on the XY plane.
        // Triangle 0: CCW in XY → front face is +Z.
        // Triangle 1: CW in XY  → front face is -Z (backface from +Z camera).
        let mb = MB {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 6],
            uvs: vec![[0.0, 0.0]; 6],
            // Tri 0: CCW from +Z → front face.
            // Tri 1: CW from +Z  → back face.
            indices: vec![0, 1, 2, 3, 5, 4],
            has_suit: false,
        };
        MeshBuffers::from_morph(mb)
    }

    // ── Plane ─────────────────────────────────────────────────────────────────

    #[test]
    fn plane_signed_distance_positive_inside() {
        // Plane: normal = +X, d = 0 (the YZ plane). Points with x > 0 are inside.
        let plane = Plane::new([1.0, 0.0, 0.0], 0.0);
        assert!(plane.signed_distance([1.0, 0.0, 0.0]) > 0.0);
    }

    #[test]
    fn plane_signed_distance_negative_outside() {
        let plane = Plane::new([1.0, 0.0, 0.0], 0.0);
        assert!(plane.signed_distance([-1.0, 0.0, 0.0]) < 0.0);
    }

    #[test]
    fn plane_from_points_normal_correct() {
        // Triangle in XY plane (CCW) → normal should be +Z.
        let plane = Plane::from_points([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        // Normal should be approximately [0, 0, 1].
        assert!((plane.normal[2] - 1.0).abs() < 1e-5, "normal z should be 1");
        assert!(plane.normal[0].abs() < 1e-5, "normal x should be 0");
        assert!(plane.normal[1].abs() < 1e-5, "normal y should be 0");
    }

    // ── Frustum::orthographic ─────────────────────────────────────────────────

    #[test]
    fn frustum_orthographic_contains_center() {
        let f = ortho_frustum();
        // Center of the frustum volume.
        assert!(f.contains_point([0.0, 0.0, 5.0]));
    }

    #[test]
    fn frustum_orthographic_excludes_outside() {
        let f = ortho_frustum();
        // Clearly outside on the X axis.
        assert!(!f.contains_point([5.0, 0.0, 5.0]));
    }

    // ── Frustum::contains_point ───────────────────────────────────────────────

    #[test]
    fn frustum_contains_point_inside() {
        let f = ortho_frustum();
        assert!(f.contains_point([0.0, 0.0, 1.0]));
    }

    #[test]
    fn frustum_contains_point_outside() {
        let f = ortho_frustum();
        // Behind the near plane.
        assert!(!f.contains_point([0.0, 0.0, -1.0]));
    }

    // ── Frustum::intersects_sphere ────────────────────────────────────────────

    #[test]
    fn frustum_intersects_sphere_inside() {
        let f = ortho_frustum();
        // Small sphere fully inside.
        assert!(f.intersects_sphere([0.0, 0.0, 5.0], 0.1));
    }

    #[test]
    fn frustum_intersects_sphere_outside() {
        let f = ortho_frustum();
        // Sphere entirely beyond far plane.
        assert!(!f.intersects_sphere([0.0, 0.0, 20.0], 0.5));
    }

    // ── Frustum::intersects_aabb ──────────────────────────────────────────────

    #[test]
    fn frustum_intersects_aabb_inside() {
        let f = ortho_frustum();
        let aabb = small_aabb(0.0, 0.0, 5.0, 0.2);
        assert!(f.intersects_aabb(&aabb));
    }

    #[test]
    fn frustum_intersects_aabb_outside() {
        let f = ortho_frustum();
        // AABB way outside on Y.
        let aabb = small_aabb(0.0, 10.0, 5.0, 0.2);
        assert!(!f.intersects_aabb(&aabb));
    }

    // ── classify_aabb ─────────────────────────────────────────────────────────

    #[test]
    fn classify_aabb_inside() {
        let f = ortho_frustum();
        let aabb = small_aabb(0.0, 0.0, 5.0, 0.2);
        assert_eq!(classify_aabb(&f, &aabb), Visibility::Inside);
    }

    #[test]
    fn classify_aabb_outside() {
        let f = ortho_frustum();
        let aabb = small_aabb(0.0, 50.0, 5.0, 0.2);
        assert_eq!(classify_aabb(&f, &aabb), Visibility::Outside);
    }

    // ── is_front_facing ───────────────────────────────────────────────────────

    #[test]
    fn is_front_facing_forward_face() {
        // Triangle in XY plane, CCW winding → face normal = +Z.
        // Camera is at z=+5 looking toward -Z, so view_dir = -Z.
        // dot(face_normal=+Z, view_dir=-Z) = -1 < 0 → front-facing.
        let p0 = [0.0f32, 0.0, 0.0];
        let p1 = [1.0, 0.0, 0.0];
        let p2 = [0.0, 1.0, 0.0];
        assert!(is_front_facing(p0, p1, p2, [0.0, 0.0, -1.0]));
    }

    #[test]
    fn is_front_facing_back_face() {
        // Same triangle CW winding → face normal = -Z.
        // Camera still at z=+5, view_dir = -Z.
        // dot(face_normal=-Z, view_dir=-Z) = +1 > 0 → back-facing.
        let p0 = [0.0f32, 0.0, 0.0];
        let p1 = [0.0, 1.0, 0.0];
        let p2 = [1.0, 0.0, 0.0];
        assert!(!is_front_facing(p0, p1, p2, [0.0, 0.0, -1.0]));
    }

    // ── backface_cull / count_front_facing ────────────────────────────────────

    #[test]
    fn backface_cull_half_sphere_approx() {
        // two_tri_mesh: tri 0 indices [0,1,2] CCW → face normal +Z (front from +Z camera).
        //               tri 1 indices [3,5,4] CW  → face normal -Z (back from +Z camera).
        // Camera at [0.5, 0.5, 5.0] is on the +Z side → view_dir ≈ -Z.
        // Only tri 0 should survive the cull.
        let mesh = two_tri_mesh();
        let camera_pos = [0.5f32, 0.5, 5.0];
        let visible = backface_cull(&mesh, camera_pos);
        assert_eq!(visible.len(), 1, "only one front-facing triangle");
        assert_eq!(visible[0], 0, "triangle 0 should be front-facing");
    }

    #[test]
    fn count_front_facing_positive() {
        let mesh = two_tri_mesh();
        let camera_pos = [0.5f32, 0.5, 5.0];
        let count = count_front_facing(&mesh, camera_pos);
        assert!(count >= 1, "at least one front-facing triangle");
    }
}
