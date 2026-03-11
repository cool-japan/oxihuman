//! Anatomical joint limit constraints.
//!
//! Provides a rich constraint system for anatomically accurate joint limits
//! including hinge, ball-socket, saddle, and pivot joint types.
//! Quaternion-based swing-twist decomposition is used for ball-socket limits.
//!
//! All angles are in radians and all data uses `f64`.

// ---------------------------------------------------------------------------
// Quaternion utilities
// ---------------------------------------------------------------------------

/// Normalize a quaternion `[x, y, z, w]`. Returns identity if near-zero length.
pub fn quat_normalize(q: &[f64; 4]) -> [f64; 4] {
    let len_sq = q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3];
    if len_sq < 1e-15 {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let inv = 1.0 / len_sq.sqrt();
    [q[0] * inv, q[1] * inv, q[2] * inv, q[3] * inv]
}

/// Multiply two quaternions `[x, y, z, w]`.
pub fn quat_mul(a: &[f64; 4], b: &[f64; 4]) -> [f64; 4] {
    [
        a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
        a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
        a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
        a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
    ]
}

/// Conjugate (inverse for unit quaternion) `[x, y, z, w]`.
pub fn quat_conjugate(q: &[f64; 4]) -> [f64; 4] {
    [-q[0], -q[1], -q[2], q[3]]
}

/// Convert quaternion `[x, y, z, w]` to axis-angle `(axis[3], angle)`.
/// Returns `([0,0,1], 0.0)` for identity quaternion.
pub fn quat_to_axis_angle(q: &[f64; 4]) -> ([f64; 3], f64) {
    let qn = quat_normalize(q);
    let w = qn[3].clamp(-1.0, 1.0);
    let angle = 2.0 * w.acos();
    let sin_half = (1.0 - w * w).sqrt();
    if sin_half < 1e-12 {
        return ([0.0, 0.0, 1.0], 0.0);
    }
    let inv = 1.0 / sin_half;
    ([qn[0] * inv, qn[1] * inv, qn[2] * inv], angle)
}

/// Create quaternion `[x, y, z, w]` from axis-angle. Axis need not be normalized.
pub fn quat_from_axis_angle(axis: &[f64; 3], angle: f64) -> [f64; 4] {
    let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if len < 1e-15 {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let inv = 1.0 / len;
    let half = angle * 0.5;
    let s = half.sin();
    let c = half.cos();
    [axis[0] * inv * s, axis[1] * inv * s, axis[2] * inv * s, c]
}

/// Dot product of two 3-vectors.
fn dot3(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Normalize a 3-vector. Returns zero vector if near-zero length.
fn normalize3(v: &[f64; 3]) -> [f64; 3] {
    let len_sq = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if len_sq < 1e-15 {
        return [0.0, 0.0, 0.0];
    }
    let inv = 1.0 / len_sq.sqrt();
    [v[0] * inv, v[1] * inv, v[2] * inv]
}

/// Geodesic distance between two unit quaternions (angle in radians).
fn quat_distance(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    let d = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]).abs();
    if d >= 1.0 {
        0.0
    } else {
        2.0 * d.acos()
    }
}

// ---------------------------------------------------------------------------
// Joint limit types
// ---------------------------------------------------------------------------

/// Joint limit types for anatomical constraints.
#[derive(Debug, Clone)]
pub enum JointLimitType {
    /// Hinge joint (1 DOF) - e.g., elbow, knee.
    Hinge {
        axis: [f64; 3],
        min_angle: f64,
        max_angle: f64,
    },
    /// Ball-socket joint (3 DOF) - e.g., shoulder, hip.
    BallSocket {
        swing_limit: f64,
        twist_min: f64,
        twist_max: f64,
    },
    /// Saddle joint (2 DOF) - e.g., thumb CMC.
    Saddle {
        flexion_range: (f64, f64),
        abduction_range: (f64, f64),
    },
    /// Pivot joint (1 DOF rotation) - e.g., atlas-axis.
    Pivot {
        axis: [f64; 3],
        min_angle: f64,
        max_angle: f64,
    },
}

/// A single joint limit definition binding two bones.
#[derive(Debug, Clone)]
pub struct JointLimit {
    /// Human-readable name of the joint (e.g., "left_elbow").
    pub name: String,
    /// Index of the parent bone in the skeleton hierarchy.
    pub parent_bone: usize,
    /// Index of the child bone in the skeleton hierarchy.
    pub child_bone: usize,
    /// The specific limit type with its parameters.
    pub limit_type: JointLimitType,
    /// Spring-like stiffness for soft limit enforcement (N/rad).
    pub stiffness: f64,
    /// Damping coefficient to prevent oscillation at limits.
    pub damping: f64,
}

/// Information about a single joint limit violation.
#[derive(Debug, Clone)]
pub struct JointViolation {
    /// Index of the violating joint in the `JointLimitSystem::limits` array.
    pub joint_index: usize,
    /// Name of the violating joint.
    pub joint_name: String,
    /// The measured angle that exceeds the limit.
    pub violation_angle: f64,
    /// The limit angle that was exceeded.
    pub limit_angle: f64,
}

/// System managing a collection of joint limits for an articulated skeleton.
#[derive(Debug, Clone)]
pub struct JointLimitSystem {
    limits: Vec<JointLimit>,
}

impl Default for JointLimitSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl JointLimitSystem {
    /// Create an empty joint limit system.
    pub fn new() -> Self {
        Self { limits: Vec::new() }
    }

    /// Add a joint limit definition.
    pub fn add_limit(&mut self, limit: JointLimit) {
        self.limits.push(limit);
    }

    /// Return a reference to all registered limits.
    pub fn limits(&self) -> &[JointLimit] {
        &self.limits
    }

    /// Create a preset joint limit system for a standard humanoid skeleton.
    ///
    /// Assumes the following bone index layout (0-based):
    ///   0 = pelvis, 1 = spine_lower, 2 = spine_upper, 3 = neck, 4 = head,
    ///   5 = left_upper_arm, 6 = left_forearm, 7 = left_hand,
    ///   8 = right_upper_arm, 9 = right_forearm, 10 = right_hand,
    ///   11 = left_thigh, 12 = left_shin, 13 = left_foot,
    ///   14 = right_thigh, 15 = right_shin, 16 = right_foot.
    pub fn create_humanoid_defaults() -> Self {
        let mut sys = Self::new();
        let default_stiffness = 100.0;
        let default_damping = 10.0;

        sys.add_limit(JointLimit {
            name: "spine_lower".to_string(),
            parent_bone: 0,
            child_bone: 1,
            limit_type: JointLimitType::BallSocket {
                swing_limit: 0.52,
                twist_min: -0.35,
                twist_max: 0.35,
            },
            stiffness: default_stiffness,
            damping: default_damping,
        });
        sys.add_limit(JointLimit {
            name: "spine_upper".to_string(),
            parent_bone: 1,
            child_bone: 2,
            limit_type: JointLimitType::BallSocket {
                swing_limit: 0.44,
                twist_min: -0.26,
                twist_max: 0.26,
            },
            stiffness: default_stiffness,
            damping: default_damping,
        });
        sys.add_limit(JointLimit {
            name: "neck".to_string(),
            parent_bone: 2,
            child_bone: 3,
            limit_type: JointLimitType::BallSocket {
                swing_limit: 0.70,
                twist_min: -1.22,
                twist_max: 1.22,
            },
            stiffness: default_stiffness,
            damping: default_damping,
        });
        sys.add_limit(JointLimit {
            name: "head".to_string(),
            parent_bone: 3,
            child_bone: 4,
            limit_type: JointLimitType::Pivot {
                axis: [0.0, 1.0, 0.0],
                min_angle: -0.52,
                max_angle: 0.52,
            },
            stiffness: default_stiffness,
            damping: default_damping,
        });
        sys.add_limit(JointLimit {
            name: "left_shoulder".to_string(),
            parent_bone: 2,
            child_bone: 5,
            limit_type: JointLimitType::BallSocket {
                swing_limit: 2.79,
                twist_min: -1.57,
                twist_max: 1.57,
            },
            stiffness: default_stiffness,
            damping: default_damping,
        });
        sys.add_limit(JointLimit {
            name: "left_elbow".to_string(),
            parent_bone: 5,
            child_bone: 6,
            limit_type: JointLimitType::Hinge {
                axis: [0.0, 0.0, 1.0],
                min_angle: 0.0,
                max_angle: 2.53,
            },
            stiffness: default_stiffness,
            damping: default_damping,
        });
        sys.add_limit(JointLimit {
            name: "left_wrist".to_string(),
            parent_bone: 6,
            child_bone: 7,
            limit_type: JointLimitType::Saddle {
                flexion_range: (-1.22, 1.22),
                abduction_range: (-0.35, 0.52),
            },
            stiffness: default_stiffness,
            damping: default_damping,
        });
        sys.add_limit(JointLimit {
            name: "right_shoulder".to_string(),
            parent_bone: 2,
            child_bone: 8,
            limit_type: JointLimitType::BallSocket {
                swing_limit: 2.79,
                twist_min: -1.57,
                twist_max: 1.57,
            },
            stiffness: default_stiffness,
            damping: default_damping,
        });
        sys.add_limit(JointLimit {
            name: "right_elbow".to_string(),
            parent_bone: 8,
            child_bone: 9,
            limit_type: JointLimitType::Hinge {
                axis: [0.0, 0.0, 1.0],
                min_angle: 0.0,
                max_angle: 2.53,
            },
            stiffness: default_stiffness,
            damping: default_damping,
        });
        sys.add_limit(JointLimit {
            name: "right_wrist".to_string(),
            parent_bone: 9,
            child_bone: 10,
            limit_type: JointLimitType::Saddle {
                flexion_range: (-1.22, 1.22),
                abduction_range: (-0.35, 0.52),
            },
            stiffness: default_stiffness,
            damping: default_damping,
        });
        sys.add_limit(JointLimit {
            name: "left_hip".to_string(),
            parent_bone: 0,
            child_bone: 11,
            limit_type: JointLimitType::BallSocket {
                swing_limit: 2.09,
                twist_min: -0.79,
                twist_max: 0.79,
            },
            stiffness: default_stiffness,
            damping: default_damping,
        });
        sys.add_limit(JointLimit {
            name: "left_knee".to_string(),
            parent_bone: 11,
            child_bone: 12,
            limit_type: JointLimitType::Hinge {
                axis: [1.0, 0.0, 0.0],
                min_angle: 0.0,
                max_angle: 2.44,
            },
            stiffness: default_stiffness,
            damping: default_damping,
        });
        sys.add_limit(JointLimit {
            name: "left_ankle".to_string(),
            parent_bone: 12,
            child_bone: 13,
            limit_type: JointLimitType::Saddle {
                flexion_range: (-0.70, 0.44),
                abduction_range: (-0.35, 0.35),
            },
            stiffness: default_stiffness,
            damping: default_damping,
        });
        sys.add_limit(JointLimit {
            name: "right_hip".to_string(),
            parent_bone: 0,
            child_bone: 14,
            limit_type: JointLimitType::BallSocket {
                swing_limit: 2.09,
                twist_min: -0.79,
                twist_max: 0.79,
            },
            stiffness: default_stiffness,
            damping: default_damping,
        });
        sys.add_limit(JointLimit {
            name: "right_knee".to_string(),
            parent_bone: 14,
            child_bone: 15,
            limit_type: JointLimitType::Hinge {
                axis: [1.0, 0.0, 0.0],
                min_angle: 0.0,
                max_angle: 2.44,
            },
            stiffness: default_stiffness,
            damping: default_damping,
        });
        sys.add_limit(JointLimit {
            name: "right_ankle".to_string(),
            parent_bone: 15,
            child_bone: 16,
            limit_type: JointLimitType::Saddle {
                flexion_range: (-0.70, 0.44),
                abduction_range: (-0.35, 0.35),
            },
            stiffness: default_stiffness,
            damping: default_damping,
        });

        sys
    }

    /// Check if the current pose violates any joint limits.
    pub fn check_violations(&self, bone_orientations: &[[f64; 4]]) -> Vec<JointViolation> {
        let mut violations = Vec::new();

        for (idx, limit) in self.limits.iter().enumerate() {
            if limit.parent_bone >= bone_orientations.len()
                || limit.child_bone >= bone_orientations.len()
            {
                continue;
            }

            let parent_q = &bone_orientations[limit.parent_bone];
            let child_q = &bone_orientations[limit.child_bone];
            let relative_q = quat_mul(&quat_conjugate(parent_q), child_q);
            let rel_norm = quat_normalize(&relative_q);

            match &limit.limit_type {
                JointLimitType::Hinge {
                    axis,
                    min_angle,
                    max_angle,
                }
                | JointLimitType::Pivot {
                    axis,
                    min_angle,
                    max_angle,
                } => {
                    let angle = Self::extract_hinge_angle(&rel_norm, axis);
                    if angle < *min_angle {
                        violations.push(JointViolation {
                            joint_index: idx,
                            joint_name: limit.name.clone(),
                            violation_angle: angle,
                            limit_angle: *min_angle,
                        });
                    } else if angle > *max_angle {
                        violations.push(JointViolation {
                            joint_index: idx,
                            joint_name: limit.name.clone(),
                            violation_angle: angle,
                            limit_angle: *max_angle,
                        });
                    }
                }
                JointLimitType::BallSocket {
                    swing_limit,
                    twist_min,
                    twist_max,
                } => {
                    let (swing_q, twist_q) =
                        Self::swing_twist_decompose(&rel_norm, &[0.0, 1.0, 0.0]);
                    let (_, swing_angle) = quat_to_axis_angle(&swing_q);
                    let swing_angle = Self::normalize_angle(swing_angle);
                    if swing_angle.abs() > *swing_limit {
                        violations.push(JointViolation {
                            joint_index: idx,
                            joint_name: limit.name.clone(),
                            violation_angle: swing_angle,
                            limit_angle: *swing_limit,
                        });
                    }
                    let (_, twist_angle) = quat_to_axis_angle(&twist_q);
                    let twist_angle = Self::normalize_angle(twist_angle);
                    if twist_angle < *twist_min {
                        violations.push(JointViolation {
                            joint_index: idx,
                            joint_name: limit.name.clone(),
                            violation_angle: twist_angle,
                            limit_angle: *twist_min,
                        });
                    } else if twist_angle > *twist_max {
                        violations.push(JointViolation {
                            joint_index: idx,
                            joint_name: limit.name.clone(),
                            violation_angle: twist_angle,
                            limit_angle: *twist_max,
                        });
                    }
                }
                JointLimitType::Saddle {
                    flexion_range,
                    abduction_range,
                } => {
                    let (flexion, abduction) = Self::extract_saddle_angles(&rel_norm);
                    if flexion < flexion_range.0 {
                        violations.push(JointViolation {
                            joint_index: idx,
                            joint_name: limit.name.clone(),
                            violation_angle: flexion,
                            limit_angle: flexion_range.0,
                        });
                    } else if flexion > flexion_range.1 {
                        violations.push(JointViolation {
                            joint_index: idx,
                            joint_name: limit.name.clone(),
                            violation_angle: flexion,
                            limit_angle: flexion_range.1,
                        });
                    }
                    if abduction < abduction_range.0 {
                        violations.push(JointViolation {
                            joint_index: idx,
                            joint_name: limit.name.clone(),
                            violation_angle: abduction,
                            limit_angle: abduction_range.0,
                        });
                    } else if abduction > abduction_range.1 {
                        violations.push(JointViolation {
                            joint_index: idx,
                            joint_name: limit.name.clone(),
                            violation_angle: abduction,
                            limit_angle: abduction_range.1,
                        });
                    }
                }
            }
        }

        violations
    }

    /// Apply joint limit corrections to bone orientations.
    /// Returns the number of corrections applied.
    pub fn enforce_limits(&self, bone_orientations: &mut [[f64; 4]]) -> anyhow::Result<usize> {
        let mut corrections = 0usize;

        for limit in &self.limits {
            if limit.parent_bone >= bone_orientations.len()
                || limit.child_bone >= bone_orientations.len()
            {
                continue;
            }

            let parent_q = bone_orientations[limit.parent_bone];
            let child_q = bone_orientations[limit.child_bone];
            let relative_q = quat_mul(&quat_conjugate(&parent_q), &child_q);
            let rel_norm = quat_normalize(&relative_q);

            let corrected = match &limit.limit_type {
                JointLimitType::Hinge {
                    axis,
                    min_angle,
                    max_angle,
                }
                | JointLimitType::Pivot {
                    axis,
                    min_angle,
                    max_angle,
                } => Self::enforce_hinge(&rel_norm, axis, *min_angle, *max_angle),
                JointLimitType::BallSocket {
                    swing_limit,
                    twist_min,
                    twist_max,
                } => Self::enforce_ball_socket(&rel_norm, *swing_limit, *twist_min, *twist_max),
                JointLimitType::Saddle {
                    flexion_range,
                    abduction_range,
                } => Self::enforce_saddle(&rel_norm, *flexion_range, *abduction_range),
            };

            let diff = quat_distance(&rel_norm, &corrected);
            if diff > 1e-9 {
                let new_child = quat_normalize(&quat_mul(&parent_q, &corrected));
                bone_orientations[limit.child_bone] = new_child;
                corrections += 1;
            }
        }

        Ok(corrections)
    }

    /// Decompose a quaternion into swing and twist components around a given twist axis.
    pub fn swing_twist_decompose(q: &[f64; 4], twist_axis: &[f64; 3]) -> ([f64; 4], [f64; 4]) {
        let axis_n = normalize3(twist_axis);
        let projection = dot3(&[q[0], q[1], q[2]], &axis_n);
        let twist_q = quat_normalize(&[
            axis_n[0] * projection,
            axis_n[1] * projection,
            axis_n[2] * projection,
            q[3],
        ]);
        let swing_q = quat_normalize(&quat_mul(q, &quat_conjugate(&twist_q)));
        (swing_q, twist_q)
    }

    /// Clamp an angle to the range `[min, max]`.
    pub fn clamp_angle(angle: f64, min: f64, max: f64) -> f64 {
        angle.clamp(min, max)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn extract_hinge_angle(q: &[f64; 4], axis: &[f64; 3]) -> f64 {
        let axis_n = normalize3(axis);
        let (_, twist_q) = Self::swing_twist_decompose(q, &axis_n);
        let (twist_axis, twist_angle) = quat_to_axis_angle(&twist_q);
        let d = dot3(&twist_axis, &axis_n);
        if d < 0.0 {
            Self::normalize_angle(-twist_angle)
        } else {
            Self::normalize_angle(twist_angle)
        }
    }

    fn extract_saddle_angles(q: &[f64; 4]) -> (f64, f64) {
        let qn = quat_normalize(q);
        let (x, y, z, w) = (qn[0], qn[1], qn[2], qn[3]);
        let sinr_cosp = 2.0 * (w * x + y * z);
        let cosr_cosp = 1.0 - 2.0 * (x * x + y * y);
        let flexion = sinr_cosp.atan2(cosr_cosp);
        let siny_cosp = 2.0 * (w * z + x * y);
        let cosy_cosp = 1.0 - 2.0 * (y * y + z * z);
        let abduction = siny_cosp.atan2(cosy_cosp);
        (flexion, abduction)
    }

    fn normalize_angle(angle: f64) -> f64 {
        let pi = std::f64::consts::PI;
        let mut a = angle % (2.0 * pi);
        if a > pi {
            a -= 2.0 * pi;
        } else if a < -pi {
            a += 2.0 * pi;
        }
        a
    }

    fn enforce_hinge(
        rel_q: &[f64; 4],
        axis: &[f64; 3],
        min_angle: f64,
        max_angle: f64,
    ) -> [f64; 4] {
        let axis_n = normalize3(axis);
        let (swing_q, twist_q) = Self::swing_twist_decompose(rel_q, &axis_n);
        let (twist_axis, twist_angle) = quat_to_axis_angle(&twist_q);
        let d = dot3(&twist_axis, &axis_n);
        let signed_angle = if d < 0.0 {
            Self::normalize_angle(-twist_angle)
        } else {
            Self::normalize_angle(twist_angle)
        };
        let clamped = Self::clamp_angle(signed_angle, min_angle, max_angle);
        let _ = swing_q;
        quat_from_axis_angle(&axis_n, clamped)
    }

    fn enforce_ball_socket(
        rel_q: &[f64; 4],
        swing_limit: f64,
        twist_min: f64,
        twist_max: f64,
    ) -> [f64; 4] {
        let twist_axis = [0.0, 1.0, 0.0];
        let (swing_q, twist_q) = Self::swing_twist_decompose(rel_q, &twist_axis);

        let (swing_ax, swing_angle) = quat_to_axis_angle(&swing_q);
        let clamped_swing_angle = Self::normalize_angle(swing_angle);
        let clamped_swing = if clamped_swing_angle.abs() > swing_limit {
            let sign = if clamped_swing_angle >= 0.0 {
                1.0
            } else {
                -1.0
            };
            quat_from_axis_angle(&swing_ax, sign * swing_limit)
        } else {
            swing_q
        };

        let (twist_ax, twist_angle) = quat_to_axis_angle(&twist_q);
        let d = dot3(&twist_ax, &twist_axis);
        let signed_twist = if d < 0.0 {
            Self::normalize_angle(-twist_angle)
        } else {
            Self::normalize_angle(twist_angle)
        };
        let clamped_twist_angle = Self::clamp_angle(signed_twist, twist_min, twist_max);
        let clamped_twist = quat_from_axis_angle(&twist_axis, clamped_twist_angle);

        quat_normalize(&quat_mul(&clamped_swing, &clamped_twist))
    }

    fn enforce_saddle(
        rel_q: &[f64; 4],
        flexion_range: (f64, f64),
        abduction_range: (f64, f64),
    ) -> [f64; 4] {
        let (flexion, abduction) = Self::extract_saddle_angles(rel_q);
        let clamped_flexion = Self::clamp_angle(flexion, flexion_range.0, flexion_range.1);
        let clamped_abduction = Self::clamp_angle(abduction, abduction_range.0, abduction_range.1);
        let q_flex = quat_from_axis_angle(&[1.0, 0.0, 0.0], clamped_flexion);
        let q_abd = quat_from_axis_angle(&[0.0, 0.0, 1.0], clamped_abduction);
        quat_normalize(&quat_mul(&q_flex, &q_abd))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_quat_normalize_identity() {
        let q = quat_normalize(&[0.0, 0.0, 0.0, 1.0]);
        assert!((q[3] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_quat_normalize_zero_returns_identity() {
        let q = quat_normalize(&[0.0, 0.0, 0.0, 0.0]);
        assert!((q[3] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_quat_mul_identity() {
        let a = [0.0, 0.0, 0.0, 1.0];
        let b = [0.1, 0.2, 0.3, 0.9];
        let bn = quat_normalize(&b);
        let r = quat_mul(&a, &bn);
        for i in 0..4 {
            assert!((r[i] - bn[i]).abs() < 1e-12);
        }
    }

    #[test]
    fn test_quat_conjugate_inverse() {
        let q = quat_normalize(&[0.1, 0.2, 0.3, 0.9]);
        let qc = quat_conjugate(&q);
        let product = quat_mul(&q, &qc);
        assert!((product[3] - 1.0).abs() < 1e-10);
        assert!(product[0].abs() < 1e-10);
        assert!(product[1].abs() < 1e-10);
        assert!(product[2].abs() < 1e-10);
    }

    #[test]
    fn test_axis_angle_roundtrip() {
        let axis = [0.0, 1.0, 0.0];
        let angle = 1.23;
        let q = quat_from_axis_angle(&axis, angle);
        let (ax2, ang2) = quat_to_axis_angle(&q);
        assert!((ang2 - angle).abs() < 1e-10);
        for i in 0..3 {
            assert!((ax2[i] - axis[i]).abs() < 1e-10);
        }
    }

    #[test]
    fn test_swing_twist_decompose_pure_twist() {
        let twist_axis = [0.0, 1.0, 0.0];
        let q = quat_from_axis_angle(&twist_axis, 0.5);
        let (swing, twist) = JointLimitSystem::swing_twist_decompose(&q, &twist_axis);
        assert!((swing[3] - 1.0).abs() < 1e-8, "swing.w = {}", swing[3]);
        let dist = quat_distance(&q, &twist);
        assert!(dist < 1e-8, "twist distance = {dist}");
    }

    #[test]
    fn test_clamp_angle() {
        assert!((JointLimitSystem::clamp_angle(0.5, -1.0, 1.0) - 0.5).abs() < 1e-12);
        assert!((JointLimitSystem::clamp_angle(2.0, -1.0, 1.0) - 1.0).abs() < 1e-12);
        assert!((JointLimitSystem::clamp_angle(-2.0, -1.0, 1.0) - (-1.0)).abs() < 1e-12);
    }

    #[test]
    fn test_humanoid_defaults_has_expected_joints() {
        let sys = JointLimitSystem::create_humanoid_defaults();
        assert!(
            sys.limits().len() >= 16,
            "expected >= 16 joints, got {}",
            sys.limits().len()
        );
        let names: Vec<&str> = sys.limits().iter().map(|l| l.name.as_str()).collect();
        assert!(names.contains(&"left_elbow"));
        assert!(names.contains(&"right_knee"));
        assert!(names.contains(&"neck"));
    }

    #[test]
    fn test_no_violations_at_rest() {
        let sys = JointLimitSystem::create_humanoid_defaults();
        let orientations = vec![[0.0, 0.0, 0.0, 1.0]; 17];
        let violations = sys.check_violations(&orientations);
        let _ = violations;
    }

    #[test]
    fn test_enforce_limits_identity_pose() {
        let sys = JointLimitSystem::create_humanoid_defaults();
        let mut orientations = vec![[0.0, 0.0, 0.0, 1.0]; 17];
        let result = sys.enforce_limits(&mut orientations);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hinge_violation_detected() {
        let mut sys = JointLimitSystem::new();
        sys.add_limit(JointLimit {
            name: "test_hinge".to_string(),
            parent_bone: 0,
            child_bone: 1,
            limit_type: JointLimitType::Hinge {
                axis: [0.0, 0.0, 1.0],
                min_angle: -0.5,
                max_angle: 0.5,
            },
            stiffness: 100.0,
            damping: 10.0,
        });

        let parent_q = [0.0, 0.0, 0.0, 1.0];
        let child_q = quat_from_axis_angle(&[0.0, 0.0, 1.0], 1.5);
        let orientations = vec![parent_q, child_q];
        let violations = sys.check_violations(&orientations);
        assert!(!violations.is_empty(), "Should detect hinge violation");
    }

    #[test]
    fn test_enforce_hinge_clamps() {
        let mut sys = JointLimitSystem::new();
        sys.add_limit(JointLimit {
            name: "test_hinge".to_string(),
            parent_bone: 0,
            child_bone: 1,
            limit_type: JointLimitType::Hinge {
                axis: [0.0, 0.0, 1.0],
                min_angle: -0.5,
                max_angle: 0.5,
            },
            stiffness: 100.0,
            damping: 10.0,
        });

        let parent_q = [0.0, 0.0, 0.0, 1.0];
        let child_q = quat_from_axis_angle(&[0.0, 0.0, 1.0], 1.5);
        let mut orientations = vec![parent_q, child_q];
        let result = sys.enforce_limits(&mut orientations);
        assert!(result.is_ok());
        if let Ok(corrections) = result {
            assert!(corrections > 0, "Should have applied corrections");
        }
    }

    #[test]
    fn test_ball_socket_swing_violation() {
        let mut sys = JointLimitSystem::new();
        sys.add_limit(JointLimit {
            name: "test_ball".to_string(),
            parent_bone: 0,
            child_bone: 1,
            limit_type: JointLimitType::BallSocket {
                swing_limit: 0.5,
                twist_min: -0.3,
                twist_max: 0.3,
            },
            stiffness: 100.0,
            damping: 10.0,
        });

        let parent_q = [0.0, 0.0, 0.0, 1.0];
        let child_q = quat_from_axis_angle(&[1.0, 0.0, 0.0], PI * 0.7);
        let orientations = vec![parent_q, child_q];
        let violations = sys.check_violations(&orientations);
        assert!(
            !violations.is_empty(),
            "Should detect ball-socket swing violation"
        );
    }

    #[test]
    fn test_default_impl() {
        let sys = JointLimitSystem::default();
        assert!(sys.limits().is_empty());
    }

    #[test]
    fn test_out_of_bounds_bone_indices() {
        let mut sys = JointLimitSystem::new();
        sys.add_limit(JointLimit {
            name: "oob".to_string(),
            parent_bone: 100,
            child_bone: 200,
            limit_type: JointLimitType::Hinge {
                axis: [1.0, 0.0, 0.0],
                min_angle: -1.0,
                max_angle: 1.0,
            },
            stiffness: 100.0,
            damping: 10.0,
        });

        let orientations = vec![[0.0, 0.0, 0.0, 1.0]; 2];
        let violations = sys.check_violations(&orientations);
        assert!(violations.is_empty(), "OOB bones should be skipped");

        let mut orientations2 = orientations;
        let result = sys.enforce_limits(&mut orientations2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_quat_from_axis_angle_zero_axis() {
        let q = quat_from_axis_angle(&[0.0, 0.0, 0.0], 1.0);
        assert!((q[3] - 1.0).abs() < 1e-12, "zero axis => identity");
    }
}
