// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Rigid body transform (position + quaternion rotation).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RigidTransform {
    pub position: [f32; 3],
    /// Quaternion stored as [x, y, z, w]
    pub rotation: [f32; 4],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TransformConfig {
    pub normalize_threshold: f32,
}

#[allow(dead_code)]
pub fn default_transform_config() -> TransformConfig {
    TransformConfig { normalize_threshold: 1e-6 }
}

#[allow(dead_code)]
pub fn identity_transform() -> RigidTransform {
    RigidTransform { position: [0.0; 3], rotation: [0.0, 0.0, 0.0, 1.0] }
}

#[allow(dead_code)]
pub fn transform_translate(t: &RigidTransform, delta: [f32; 3]) -> RigidTransform {
    RigidTransform {
        position: [
            t.position[0] + delta[0],
            t.position[1] + delta[1],
            t.position[2] + delta[2],
        ],
        rotation: t.rotation,
    }
}

#[allow(dead_code)]
pub fn transform_rotate_y(t: &RigidTransform, angle_rad: f32) -> RigidTransform {
    // Create quaternion for Y-axis rotation
    let half = angle_rad * 0.5;
    let (s, c) = (half.sin(), half.cos());
    let q_new = [0.0, s, 0.0, c]; // [x, y, z, w]

    // Compose with existing rotation: t.rotation * q_new
    let [ax, ay, az, aw] = t.rotation;
    let [bx, by, bz, bw] = q_new;
    let rotation = [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ];
    RigidTransform { position: t.position, rotation }
}

#[allow(dead_code)]
pub fn transform_apply_point(t: &RigidTransform, p: [f32; 3]) -> [f32; 3] {
    // Rotate p by quaternion then translate
    let [qx, qy, qz, qw] = t.rotation;
    let [px, py, pz] = p;
    // q * p * q^-1 using formula v' = v + 2*qw*(q_xyz × v) + 2*(q_xyz × (q_xyz × v))
    let tx = 2.0 * (qy * pz - qz * py);
    let ty = 2.0 * (qz * px - qx * pz);
    let tz = 2.0 * (qx * py - qy * px);
    [
        px + qw * tx + qy * tz - qz * ty + t.position[0],
        py + qw * ty + qz * tx - qx * tz + t.position[1],
        pz + qw * tz + qx * ty - qy * tx + t.position[2],
    ]
}

#[allow(dead_code)]
pub fn transform_inverse(t: &RigidTransform) -> RigidTransform {
    // Inverse rotation is conjugate: [-x, -y, -z, w]
    let [qx, qy, qz, qw] = t.rotation;
    let inv_rot = [-qx, -qy, -qz, qw];
    // Inverse position: rotate -position by inverse rotation
    let neg_pos = [-t.position[0], -t.position[1], -t.position[2]];
    let inv_t = RigidTransform { position: [0.0; 3], rotation: inv_rot };
    let new_pos = transform_apply_point(&RigidTransform { position: [0.0; 3], rotation: inv_rot }, neg_pos);
    RigidTransform { position: new_pos, rotation: inv_t.rotation }
}

#[allow(dead_code)]
pub fn transform_combine(a: &RigidTransform, b: &RigidTransform) -> RigidTransform {
    // Combined position: a.rotation * b.position + a.position
    let new_pos = transform_apply_point(a, b.position);
    // Combined rotation: a.rotation * b.rotation
    let [ax, ay, az, aw] = a.rotation;
    let [bx, by, bz, bw] = b.rotation;
    let rotation = [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ];
    RigidTransform { position: new_pos, rotation }
}

#[allow(dead_code)]
pub fn transform_to_json(t: &RigidTransform) -> String {
    let [px, py, pz] = t.position;
    let [rx, ry, rz, rw] = t.rotation;
    format!(
        "{{\"position\":[{},{},{}],\"rotation\":[{},{},{},{}]}}",
        px, py, pz, rx, ry, rz, rw
    )
}

#[allow(dead_code)]
pub fn transform_is_identity(t: &RigidTransform, tol: f32) -> bool {
    let pos_ok = t.position.iter().all(|&v| v.abs() < tol);
    let [rx, ry, rz, rw] = t.rotation;
    let rot_ok = rx.abs() < tol && ry.abs() < tol && rz.abs() < tol && (rw - 1.0).abs() < tol;
    pos_ok && rot_ok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let t = identity_transform();
        assert!(transform_is_identity(&t, 1e-6));
    }

    #[test]
    fn test_translate() {
        let t = identity_transform();
        let t2 = transform_translate(&t, [1.0, 2.0, 3.0]);
        assert!((t2.position[0] - 1.0).abs() < 1e-6);
        assert!((t2.position[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_apply_point_identity() {
        let t = identity_transform();
        let p = [1.0, 2.0, 3.0];
        let out = transform_apply_point(&t, p);
        assert!((out[0] - 1.0).abs() < 1e-5);
        assert!((out[1] - 2.0).abs() < 1e-5);
        assert!((out[2] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let t = identity_transform();
        let json = transform_to_json(&t);
        assert!(json.contains("\"position\""));
        assert!(json.contains("\"rotation\""));
    }

    #[test]
    fn test_default_config() {
        let cfg = default_transform_config();
        assert!(cfg.normalize_threshold > 0.0);
    }

    #[test]
    fn test_is_not_identity_after_translate() {
        let t = transform_translate(&identity_transform(), [1.0, 0.0, 0.0]);
        assert!(!transform_is_identity(&t, 1e-6));
    }

    #[test]
    fn test_rotate_y_and_back() {
        use std::f32::consts::PI;
        let t = identity_transform();
        let rotated = transform_rotate_y(&t, PI);
        // Point (1,0,0) rotated 180 around Y should be approximately (-1, 0, 0)
        let p = transform_apply_point(&rotated, [1.0, 0.0, 0.0]);
        assert!((p[0] + 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_combine_with_identity() {
        let t = transform_translate(&identity_transform(), [5.0, 0.0, 0.0]);
        let combined = transform_combine(&t, &identity_transform());
        assert!((combined.position[0] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_inverse_identity() {
        let t = identity_transform();
        let inv = transform_inverse(&t);
        assert!(transform_is_identity(&inv, 1e-5));
    }
}
