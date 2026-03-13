// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Dual Quaternion Skinning (DQS) — avoids the "candy wrapper" artifact of LBS.
//!
//! Reference: Kavan et al. "Skinning with Dual Quaternions", I3D 2007.

// ─── Quat ────────────────────────────────────────────────────────────────────

/// A unit quaternion: `[x, y, z, w]` (Hamilton convention).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[allow(dead_code)]
impl Quat {
    /// Identity quaternion (no rotation).
    pub fn identity() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }

    /// Create from axis (must be unit length) and angle in radians.
    pub fn from_axis_angle(axis: [f32; 3], angle_rad: f32) -> Self {
        let half = angle_rad * 0.5;
        let s = half.sin();
        Self {
            x: axis[0] * s,
            y: axis[1] * s,
            z: axis[2] * s,
            w: half.cos(),
        }
    }

    /// Return the squared length of this quaternion.
    #[inline]
    fn length_sq(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    /// Normalize to unit length.
    pub fn normalize(&self) -> Self {
        let lsq = self.length_sq();
        if lsq < 1e-30 {
            return Self::identity();
        }
        let inv = 1.0 / lsq.sqrt();
        Self {
            x: self.x * inv,
            y: self.y * inv,
            z: self.z * inv,
            w: self.w * inv,
        }
    }

    /// Conjugate (inverse for unit quaternions).
    pub fn conjugate(&self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }

    /// Dot product (scalar).
    pub fn dot(&self, other: &Quat) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    /// Hamilton (quaternion) product: `self * other`.
    pub fn mul(&self, other: &Quat) -> Quat {
        Quat {
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
        }
    }

    /// Rotate a 3-D vector by this (assumed unit) quaternion:
    /// `v' = q * (0, v) * q†`.
    pub fn rotate_vec(&self, v: [f32; 3]) -> [f32; 3] {
        // Efficient Rodrigues formula: v' = v + 2w(q×v) + 2(q×(q×v))
        let qx = self.x;
        let qy = self.y;
        let qz = self.z;
        let qw = self.w;
        let tx = 2.0 * (qy * v[2] - qz * v[1]);
        let ty = 2.0 * (qz * v[0] - qx * v[2]);
        let tz = 2.0 * (qx * v[1] - qy * v[0]);
        [
            v[0] + qw * tx + qy * tz - qz * ty,
            v[1] + qw * ty + qz * tx - qx * tz,
            v[2] + qw * tz + qx * ty - qy * tx,
        ]
    }
}

// ─── DualQuat ─────────────────────────────────────────────────────────────────

/// A dual quaternion: `real` (rotation) + `dual` (translation) parts.
///
/// Represents a rigid-body transform (rotation + translation) without shear or scale.
/// `dual = 0.5 * t_quat * real` where `t_quat = (tx, ty, tz, 0)`.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct DualQuat {
    /// Rotation part (unit quaternion in the primal part).
    pub real: Quat,
    /// Translation encoding: `dual = 0.5 * t_quat * real`.
    pub dual: Quat,
}

#[allow(dead_code)]
impl DualQuat {
    /// Identity transform (no rotation, no translation).
    pub fn identity() -> Self {
        Self {
            real: Quat::identity(),
            dual: Quat {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 0.0,
            },
        }
    }

    /// Create from a rotation quaternion and a translation vector.
    ///
    /// `dual = 0.5 * (0, tx, ty, tz) * rot`
    pub fn from_rot_trans(rot: Quat, trans: [f32; 3]) -> Self {
        let rot = rot.normalize();
        // t_quat has w=0 and xyz=trans
        let t = Quat {
            x: trans[0],
            y: trans[1],
            z: trans[2],
            w: 0.0,
        };
        // dual = 0.5 * t * rot
        let half_t_rot = t.mul(&rot);
        let dual = Quat {
            x: 0.5 * half_t_rot.x,
            y: 0.5 * half_t_rot.y,
            z: 0.5 * half_t_rot.z,
            w: 0.5 * half_t_rot.w,
        };
        Self { real: rot, dual }
    }

    /// Create from a rotation-only transform (no translation).
    pub fn from_rotation(rot: Quat) -> Self {
        Self {
            real: rot.normalize(),
            dual: Quat {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 0.0,
            },
        }
    }

    /// Component-wise addition (used during blending).
    pub fn add(&self, other: &DualQuat) -> DualQuat {
        DualQuat {
            real: Quat {
                x: self.real.x + other.real.x,
                y: self.real.y + other.real.y,
                z: self.real.z + other.real.z,
                w: self.real.w + other.real.w,
            },
            dual: Quat {
                x: self.dual.x + other.dual.x,
                y: self.dual.y + other.dual.y,
                z: self.dual.z + other.dual.z,
                w: self.dual.w + other.dual.w,
            },
        }
    }

    /// Scale both parts by a scalar.
    pub fn scale(&self, s: f32) -> DualQuat {
        DualQuat {
            real: Quat {
                x: self.real.x * s,
                y: self.real.y * s,
                z: self.real.z * s,
                w: self.real.w * s,
            },
            dual: Quat {
                x: self.dual.x * s,
                y: self.dual.y * s,
                z: self.dual.z * s,
                w: self.dual.w * s,
            },
        }
    }

    /// Normalize: divide both parts by the L2 norm of the real quaternion.
    pub fn normalize(&self) -> DualQuat {
        let mag = self.real.length_sq().sqrt();
        if mag < 1e-30 {
            return DualQuat::identity();
        }
        let inv = 1.0 / mag;
        DualQuat {
            real: Quat {
                x: self.real.x * inv,
                y: self.real.y * inv,
                z: self.real.z * inv,
                w: self.real.w * inv,
            },
            dual: Quat {
                x: self.dual.x * inv,
                y: self.dual.y * inv,
                z: self.dual.z * inv,
                w: self.dual.w * inv,
            },
        }
    }

    /// Extract the translation vector from the dual quaternion.
    ///
    /// `t = 2 * dual * conj(real)` (only the xyz part, w should be ≈ 0).
    pub fn translation(&self) -> [f32; 3] {
        let r = self.real;
        let d = self.dual;
        // t_quat = 2 * d * conj(r)
        let rx = -r.x;
        let ry = -r.y;
        let rz = -r.z;
        let rw = r.w;
        // quaternion product d * conj(r)
        let tx = d.w * rx + d.x * rw + d.y * rz - d.z * ry;
        let ty = d.w * ry - d.x * rz + d.y * rw + d.z * rx;
        let tz = d.w * rz + d.x * ry - d.y * rx + d.z * rw;
        [2.0 * tx, 2.0 * ty, 2.0 * tz]
    }

    /// Extract the rotation quaternion (just the real part, normalized).
    pub fn rotation(&self) -> Quat {
        self.real.normalize()
    }

    /// Transform a point (applies rotation then translation).
    pub fn transform_point(&self, p: [f32; 3]) -> [f32; 3] {
        let dq = self.normalize();
        let rotated = dq.real.rotate_vec(p);
        let t = dq.translation();
        [rotated[0] + t[0], rotated[1] + t[1], rotated[2] + t[2]]
    }

    /// Transform a direction vector (rotation only, no translation).
    pub fn transform_dir(&self, d: [f32; 3]) -> [f32; 3] {
        let dq = self.normalize();
        dq.real.rotate_vec(d)
    }
}

// ─── DQS kernel ──────────────────────────────────────────────────────────────

/// Apply Dual Quaternion Skinning to a set of positions and normals.
///
/// # Arguments
/// * `positions`    — input vertex positions
/// * `normals`      — input vertex normals
/// * `skin_weights` — per-vertex list of `(joint_index, weight)` pairs
/// * `joint_dqs`    — one `DualQuat` per joint (joint transform in DQ form)
///
/// Returns `(skinned_positions, skinned_normals)`.
#[allow(dead_code)]
pub fn apply_dqs(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    skin_weights: &[Vec<(usize, f32)>],
    joint_dqs: &[DualQuat],
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
    let zero_dq = DualQuat {
        real: Quat {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        },
        dual: Quat {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        },
    };

    let n = positions.len();
    let mut out_positions = Vec::with_capacity(n);
    let mut out_normals = Vec::with_capacity(n);

    for v in 0..n {
        let influences = &skin_weights[v];
        let mut blended = zero_dq;
        let mut first_real: Option<Quat> = None;

        for &(joint_idx, weight) in influences.iter() {
            if weight == 0.0 {
                continue;
            }
            let mut dq = joint_dqs[joint_idx];

            // Sign-consistency: flip if dot product with the first influence is negative.
            match first_real {
                None => {
                    first_real = Some(dq.real);
                }
                Some(ref first) => {
                    if first.dot(&dq.real) < 0.0 {
                        dq = dq.scale(-1.0);
                    }
                }
            }

            blended = blended.add(&dq.scale(weight));
        }

        // If no influences (shouldn't happen with valid data), use identity.
        let blended = if blended.real.length_sq() < 1e-30 {
            DualQuat::identity()
        } else {
            blended.normalize()
        };

        out_positions.push(blended.transform_point(positions[v]));
        out_normals.push(blended.transform_dir(normals[v]));
    }

    (out_positions, out_normals)
}

// ─── Matrix helpers ───────────────────────────────────────────────────────────

/// Convert a 4×4 column-major transform matrix to a `DualQuat`.
///
/// The matrix must be a rigid transform (no scale/shear).
#[allow(dead_code)]
pub fn matrix_to_dual_quat(m: &[f32; 16]) -> DualQuat {
    // Extract rotation (upper-left 3×3) as a quaternion.
    // Column-major layout: m[col*4 + row]
    let m00 = m[0];
    let m10 = m[1];
    let m20 = m[2];
    let m01 = m[4];
    let m11 = m[5];
    let m21 = m[6];
    let m02 = m[8];
    let m12 = m[9];
    let m22 = m[10];

    let trace = m00 + m11 + m22;
    let rot = if trace > 0.0 {
        let s = 0.5 / (trace + 1.0).sqrt();
        Quat {
            w: 0.25 / s,
            x: (m21 - m12) * s,
            y: (m02 - m20) * s,
            z: (m10 - m01) * s,
        }
    } else if m00 > m11 && m00 > m22 {
        let s = 2.0 * (1.0 + m00 - m11 - m22).sqrt();
        Quat {
            x: 0.25 * s,
            w: (m21 - m12) / s,
            y: (m01 + m10) / s,
            z: (m02 + m20) / s,
        }
    } else if m11 > m22 {
        let s = 2.0 * (1.0 + m11 - m00 - m22).sqrt();
        Quat {
            y: 0.25 * s,
            w: (m02 - m20) / s,
            x: (m01 + m10) / s,
            z: (m12 + m21) / s,
        }
    } else {
        let s = 2.0 * (1.0 + m22 - m00 - m11).sqrt();
        Quat {
            z: 0.25 * s,
            w: (m10 - m01) / s,
            x: (m02 + m20) / s,
            y: (m12 + m21) / s,
        }
    };

    // Extract translation from column 3: m[12], m[13], m[14].
    let trans = [m[12], m[13], m[14]];
    DualQuat::from_rot_trans(rot, trans)
}

/// Create a DualQuat representing the delta from bind pose to current pose.
///
/// `delta = current * inverse(bind_pose)`
///
/// For unit dual quaternions, the inverse is the conjugate:
/// `inv(dq) = (conj(real), -conj(dual))`.
#[allow(dead_code)]
pub fn joint_delta_dq(bind_pose: &DualQuat, current_pose: &DualQuat) -> DualQuat {
    // Inverse of a unit dual quaternion: inv(dq).real = conj(real), inv(dq).dual = -conj(dual)
    let bp = bind_pose.normalize();
    let inv_real = bp.real.conjugate();
    let inv_dual = Quat {
        x: -bp.dual.x,
        y: -bp.dual.y,
        z: -bp.dual.z,
        w: bp.dual.w, // conjugate negates xyz; then we negate the whole dual → w stays positive
    };
    // Actually: conj(dual) = (-dx, -dy, -dz, dw), then negate → (dx, dy, dz, -dw)
    let inv_dual_correct = Quat {
        x: bp.dual.x,
        y: bp.dual.y,
        z: bp.dual.z,
        w: -bp.dual.w,
    };
    let _ = inv_dual; // suppress: we use inv_dual_correct

    // Dual quaternion multiplication: (r1, d1) * (r2, d2) = (r1*r2, r1*d2 + d1*r2)
    let cp = current_pose.normalize();
    let new_real = cp.real.mul(&inv_real);
    let new_dual = cp
        .real
        .mul(&inv_dual_correct)
        .add_quat(&cp.dual.mul(&inv_real));

    DualQuat {
        real: new_real,
        dual: new_dual,
    }
}

// Helper trait to add two Quat values.
trait QuatAdd {
    fn add_quat(&self, other: &Quat) -> Quat;
}

impl QuatAdd for Quat {
    fn add_quat(&self, other: &Quat) -> Quat {
        Quat {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq3(a: [f32; 3], b: [f32; 3], eps: f32) -> bool {
        (a[0] - b[0]).abs() < eps && (a[1] - b[1]).abs() < eps && (a[2] - b[2]).abs() < eps
    }

    #[test]
    fn quat_identity_rotates_identity() {
        let q = Quat::identity();
        let v = [1.0f32, 2.0, 3.0];
        let rotated = q.rotate_vec(v);
        assert!(
            approx_eq3(rotated, v, 1e-5),
            "identity rotation should not change vector"
        );
    }

    #[test]
    fn quat_from_axis_angle_90_deg() {
        // 90° around Z rotates (1,0,0) → (0,1,0)
        let q = Quat::from_axis_angle([0.0, 0.0, 1.0], std::f32::consts::FRAC_PI_2);
        let v = q.rotate_vec([1.0, 0.0, 0.0]);
        assert!(
            approx_eq3(v, [0.0, 1.0, 0.0], 1e-5),
            "90 deg Z rotation wrong: {v:?}"
        );
    }

    #[test]
    fn quat_mul_identity_unchanged() {
        let q = Quat::from_axis_angle([1.0, 0.0, 0.0], 0.7);
        let id = Quat::identity();
        let q_id = q.mul(&id);
        assert!((q_id.x - q.x).abs() < 1e-6, "x changed");
        assert!((q_id.y - q.y).abs() < 1e-6, "y changed");
        assert!((q_id.z - q.z).abs() < 1e-6, "z changed");
        assert!((q_id.w - q.w).abs() < 1e-6, "w changed");
    }

    #[test]
    fn quat_rotate_vec_x_axis() {
        // 180° around Y rotates (1,0,0) → (-1,0,0)
        let q = Quat::from_axis_angle([0.0, 1.0, 0.0], std::f32::consts::PI);
        let v = q.rotate_vec([1.0, 0.0, 0.0]);
        assert!(
            approx_eq3(v, [-1.0, 0.0, 0.0], 1e-5),
            "180 deg Y rotation wrong: {v:?}"
        );
    }

    #[test]
    fn dual_quat_identity_transform_identity() {
        let dq = DualQuat::identity();
        let p = [3.0f32, 4.0, 5.0];
        let out = dq.transform_point(p);
        assert!(
            approx_eq3(out, p, 1e-5),
            "identity DQ should not move point"
        );
    }

    #[test]
    fn dual_quat_from_rot_trans_translation_correct() {
        let rot = Quat::identity();
        let trans = [1.0f32, 2.0, 3.0];
        let dq = DualQuat::from_rot_trans(rot, trans);
        let t = dq.translation();
        assert!(
            approx_eq3(t, trans, 1e-5),
            "extracted translation wrong: {t:?}"
        );
    }

    #[test]
    fn dual_quat_transform_point_translation() {
        let dq = DualQuat::from_rot_trans(Quat::identity(), [5.0, 0.0, 0.0]);
        let out = dq.transform_point([1.0, 0.0, 0.0]);
        assert!(
            approx_eq3(out, [6.0, 0.0, 0.0], 1e-5),
            "translation wrong: {out:?}"
        );
    }

    #[test]
    fn dual_quat_transform_point_rotation() {
        // 90° around Z, no translation: (1,0,0) → (0,1,0)
        let rot = Quat::from_axis_angle([0.0, 0.0, 1.0], std::f32::consts::FRAC_PI_2);
        let dq = DualQuat::from_rotation(rot);
        let out = dq.transform_point([1.0, 0.0, 0.0]);
        assert!(
            approx_eq3(out, [0.0, 1.0, 0.0], 1e-5),
            "rotation wrong: {out:?}"
        );
    }

    #[test]
    fn dual_quat_normalize_real_unit_length() {
        // Build a non-unit DQ by scaling.
        let dq =
            DualQuat::from_rot_trans(Quat::from_axis_angle([1.0, 0.0, 0.0], 0.3), [1.0, 2.0, 3.0]);
        let scaled = dq.scale(3.7);
        let norm = scaled.normalize();
        let len =
            (norm.real.x.powi(2) + norm.real.y.powi(2) + norm.real.z.powi(2) + norm.real.w.powi(2))
                .sqrt();
        assert!(
            (len - 1.0).abs() < 1e-5,
            "normalized real length should be 1.0, got {len}"
        );
    }

    #[test]
    fn apply_dqs_identity_unchanged() {
        let positions = vec![[1.0f32, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let normals = vec![[0.0f32, 1.0, 0.0], [1.0, 0.0, 0.0]];
        let skin_weights = vec![vec![(0usize, 1.0f32)], vec![(0usize, 1.0f32)]];
        let joint_dqs = vec![DualQuat::identity()];

        let (out_pos, _) = apply_dqs(&positions, &normals, &skin_weights, &joint_dqs);
        for (orig, got) in positions.iter().zip(out_pos.iter()) {
            assert!(
                approx_eq3(*orig, *got, 1e-5),
                "identity DQS moved vertex: {got:?}"
            );
        }
    }

    #[test]
    fn apply_dqs_translation_applied() {
        let positions = vec![[0.0f32, 0.0, 0.0]];
        let normals = vec![[0.0f32, 1.0, 0.0]];
        let skin_weights = vec![vec![(0usize, 1.0f32)]];
        let joint_dqs = vec![DualQuat::from_rot_trans(Quat::identity(), [3.0, 0.0, 0.0])];

        let (out_pos, _) = apply_dqs(&positions, &normals, &skin_weights, &joint_dqs);
        assert!(
            approx_eq3(out_pos[0], [3.0, 0.0, 0.0], 1e-5),
            "translation not applied: {:?}",
            out_pos[0]
        );
    }

    #[test]
    fn apply_dqs_rotation_applied() {
        // 90° around Z: (1,0,0) → (0,1,0)
        let positions = vec![[1.0f32, 0.0, 0.0]];
        let normals = vec![[1.0f32, 0.0, 0.0]];
        let skin_weights = vec![vec![(0usize, 1.0f32)]];
        let rot = Quat::from_axis_angle([0.0, 0.0, 1.0], std::f32::consts::FRAC_PI_2);
        let joint_dqs = vec![DualQuat::from_rotation(rot)];

        let (out_pos, out_nrm) = apply_dqs(&positions, &normals, &skin_weights, &joint_dqs);
        assert!(
            approx_eq3(out_pos[0], [0.0, 1.0, 0.0], 1e-5),
            "rotation pos wrong: {:?}",
            out_pos[0]
        );
        assert!(
            approx_eq3(out_nrm[0], [0.0, 1.0, 0.0], 1e-5),
            "rotation norm wrong: {:?}",
            out_nrm[0]
        );
    }

    #[test]
    fn dual_quat_add_and_normalize() {
        // Two identity DQs blended 50/50 should still be identity.
        let dq_a = DualQuat::identity().scale(0.5);
        let dq_b = DualQuat::identity().scale(0.5);
        let blended = dq_a.add(&dq_b).normalize();
        let out = blended.transform_point([1.0, 2.0, 3.0]);
        assert!(
            approx_eq3(out, [1.0, 2.0, 3.0], 1e-5),
            "blended identity wrong: {out:?}"
        );
    }
}
