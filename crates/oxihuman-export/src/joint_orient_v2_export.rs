// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Export joint orientation data (v2 with Euler XYZ + quaternion).
#[allow(dead_code)]
pub struct JointOrientV2 {
    pub name: String,
    pub euler_xyz: [f32; 3],
    pub quaternion: [f32; 4],
    pub rotation_order: RotOrder,
}

#[allow(dead_code)]
pub enum RotOrder {
    Xyz,
    Xzy,
    Yxz,
    Yzx,
    Zxy,
    Zyx,
}

#[allow(dead_code)]
pub struct JointOrientV2Export {
    pub joints: Vec<JointOrientV2>,
}

fn euler_to_quat_xyz(e: [f32; 3]) -> [f32; 4] {
    let (cx, sx) = ((e[0] * 0.5).cos(), (e[0] * 0.5).sin());
    let (cy, sy) = ((e[1] * 0.5).cos(), (e[1] * 0.5).sin());
    let (cz, sz) = ((e[2] * 0.5).cos(), (e[2] * 0.5).sin());
    let w = cx * cy * cz + sx * sy * sz;
    let x = sx * cy * cz - cx * sy * sz;
    let y = cx * sy * cz + sx * cy * sz;
    let z = cx * cy * sz - sx * sy * cz;
    [x, y, z, w]
}

#[allow(dead_code)]
pub fn new_joint_orient_v2_export() -> JointOrientV2Export {
    JointOrientV2Export { joints: vec![] }
}

#[allow(dead_code)]
pub fn add_joint_orient(
    export: &mut JointOrientV2Export,
    name: &str,
    euler_deg: [f32; 3],
    order: RotOrder,
) {
    let euler_rad = [
        euler_deg[0] * PI / 180.0,
        euler_deg[1] * PI / 180.0,
        euler_deg[2] * PI / 180.0,
    ];
    let q = euler_to_quat_xyz(euler_rad);
    export.joints.push(JointOrientV2 {
        name: name.to_string(),
        euler_xyz: euler_deg,
        quaternion: q,
        rotation_order: order,
    });
}

#[allow(dead_code)]
pub fn joint_orient_count(export: &JointOrientV2Export) -> usize {
    export.joints.len()
}

#[allow(dead_code)]
pub fn find_joint_orient<'a>(
    export: &'a JointOrientV2Export,
    name: &str,
) -> Option<&'a JointOrientV2> {
    export.joints.iter().find(|j| j.name == name)
}

#[allow(dead_code)]
pub fn quaternion_is_unit(q: [f32; 4]) -> bool {
    let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    (len - 1.0).abs() < 0.01
}

#[allow(dead_code)]
pub fn joint_orient_to_json(j: &JointOrientV2) -> String {
    format!(
        "{{\"name\":\"{}\",\"euler\":[{},{},{}],\"quat\":[{},{},{},{}]}}",
        j.name,
        j.euler_xyz[0],
        j.euler_xyz[1],
        j.euler_xyz[2],
        j.quaternion[0],
        j.quaternion[1],
        j.quaternion[2],
        j.quaternion[3]
    )
}

#[allow(dead_code)]
pub fn joint_orient_export_to_json(export: &JointOrientV2Export) -> String {
    format!("{{\"joint_count\":{}}}", export.joints.len())
}

#[allow(dead_code)]
pub fn validate_joint_orients(export: &JointOrientV2Export) -> bool {
    export
        .joints
        .iter()
        .all(|j| !j.name.is_empty() && quaternion_is_unit(j.quaternion))
}

#[allow(dead_code)]
pub fn rot_order_name(order: &RotOrder) -> &'static str {
    match order {
        RotOrder::Xyz => "xyz",
        RotOrder::Xzy => "xzy",
        RotOrder::Yxz => "yxz",
        RotOrder::Yzx => "yzx",
        RotOrder::Zxy => "zxy",
        RotOrder::Zyx => "zyx",
    }
}

#[allow(dead_code)]
pub fn identity_joint_orient(name: &str) -> JointOrientV2 {
    JointOrientV2 {
        name: name.to_string(),
        euler_xyz: [0.0; 3],
        quaternion: [0.0, 0.0, 0.0, 1.0],
        rotation_order: RotOrder::Xyz,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_joint_orient() {
        let mut e = new_joint_orient_v2_export();
        add_joint_orient(&mut e, "spine", [0.0, 0.0, 0.0], RotOrder::Xyz);
        assert_eq!(joint_orient_count(&e), 1);
    }

    #[test]
    fn test_identity_quaternion_is_unit() {
        let j = identity_joint_orient("test");
        assert!(quaternion_is_unit(j.quaternion));
    }

    #[test]
    fn test_rotated_quaternion_is_unit() {
        let mut e = new_joint_orient_v2_export();
        add_joint_orient(&mut e, "hip", [45.0, 0.0, 0.0], RotOrder::Xyz);
        let j = find_joint_orient(&e, "hip").unwrap();
        assert!(quaternion_is_unit(j.quaternion));
    }

    #[test]
    fn test_find_joint_found() {
        let mut e = new_joint_orient_v2_export();
        add_joint_orient(&mut e, "shoulder", [0.0, 90.0, 0.0], RotOrder::Yxz);
        assert!(find_joint_orient(&e, "shoulder").is_some());
    }

    #[test]
    fn test_find_joint_missing() {
        let e = new_joint_orient_v2_export();
        assert!(find_joint_orient(&e, "knee").is_none());
    }

    #[test]
    fn test_validate_valid() {
        let mut e = new_joint_orient_v2_export();
        add_joint_orient(&mut e, "j1", [0.0, 0.0, 0.0], RotOrder::Xyz);
        assert!(validate_joint_orients(&e));
    }

    #[test]
    fn test_to_json() {
        let j = identity_joint_orient("test");
        let json = joint_orient_to_json(&j);
        assert!(json.contains("test"));
    }

    #[test]
    fn test_rot_order_name() {
        assert_eq!(rot_order_name(&RotOrder::Xyz), "xyz");
        assert_eq!(rot_order_name(&RotOrder::Zyx), "zyx");
    }

    #[test]
    fn test_export_to_json() {
        let mut e = new_joint_orient_v2_export();
        add_joint_orient(&mut e, "j1", [0.0; 3], RotOrder::Xyz);
        let j = joint_orient_export_to_json(&e);
        assert!(j.contains("joint_count"));
    }

    #[test]
    fn test_90deg_x_quat() {
        let q = euler_to_quat_xyz([PI / 2.0, 0.0, 0.0]);
        assert!(quaternion_is_unit(q));
        assert!((q[0] - (PI / 4.0).sin()).abs() < 0.01);
    }
}
