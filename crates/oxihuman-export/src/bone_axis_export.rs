// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Export bone axis orientation data.
#[allow(dead_code)]
pub struct BoneAxis {
    pub name: String,
    pub head: [f32; 3],
    pub tail: [f32; 3],
    pub roll: f32,
    pub x_axis: [f32; 3],
    pub y_axis: [f32; 3],
    pub z_axis: [f32; 3],
}

#[allow(dead_code)]
pub struct BoneAxisExport {
    pub bones: Vec<BoneAxis>,
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        return [0.0, 1.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[allow(dead_code)]
pub fn new_bone_axis_export() -> BoneAxisExport {
    BoneAxisExport { bones: vec![] }
}

#[allow(dead_code)]
pub fn add_bone_axis(
    export: &mut BoneAxisExport,
    name: &str,
    head: [f32; 3],
    tail: [f32; 3],
    roll: f32,
) {
    let y = normalize3([tail[0] - head[0], tail[1] - head[1], tail[2] - head[2]]);
    let up = if y[1].abs() < 0.9 {
        [0.0, 1.0, 0.0]
    } else {
        [0.0, 0.0, 1.0]
    };
    let z0 = normalize3(cross3(y, up));
    let x0 = normalize3(cross3(z0, y));
    // Apply roll around y-axis
    let cos_r = roll.cos();
    let sin_r = roll.sin();
    let x = normalize3([
        x0[0] * cos_r + z0[0] * sin_r,
        x0[1] * cos_r + z0[1] * sin_r,
        x0[2] * cos_r + z0[2] * sin_r,
    ]);
    let z = normalize3(cross3(x, y));
    export.bones.push(BoneAxis {
        name: name.to_string(),
        head,
        tail,
        roll,
        x_axis: x,
        y_axis: y,
        z_axis: z,
    });
}

#[allow(dead_code)]
pub fn bone_axis_count(export: &BoneAxisExport) -> usize {
    export.bones.len()
}

#[allow(dead_code)]
pub fn find_bone_axis<'a>(export: &'a BoneAxisExport, name: &str) -> Option<&'a BoneAxis> {
    export.bones.iter().find(|b| b.name == name)
}

#[allow(dead_code)]
pub fn bone_length(b: &BoneAxis) -> f32 {
    let dx = b.tail[0] - b.head[0];
    let dy = b.tail[1] - b.head[1];
    let dz = b.tail[2] - b.head[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[allow(dead_code)]
pub fn bone_axis_to_json(b: &BoneAxis) -> String {
    format!(
        "{{\"name\":\"{}\",\"head\":[{},{},{}],\"tail\":[{},{},{}],\"roll\":{}}}",
        b.name, b.head[0], b.head[1], b.head[2], b.tail[0], b.tail[1], b.tail[2], b.roll
    )
}

#[allow(dead_code)]
pub fn bone_axis_export_to_json(export: &BoneAxisExport) -> String {
    format!("{{\"bone_count\":{}}}", export.bones.len())
}

#[allow(dead_code)]
pub fn validate_bone_axes(export: &BoneAxisExport) -> bool {
    export
        .bones
        .iter()
        .all(|b| !b.name.is_empty() && bone_length(b) > 0.0)
}

#[allow(dead_code)]
pub fn axes_are_orthonormal(b: &BoneAxis) -> bool {
    let dot_xy = b.x_axis[0] * b.y_axis[0] + b.x_axis[1] * b.y_axis[1] + b.x_axis[2] * b.y_axis[2];
    let dot_yz = b.y_axis[0] * b.z_axis[0] + b.y_axis[1] * b.z_axis[1] + b.y_axis[2] * b.z_axis[2];
    let lx =
        (b.x_axis[0] * b.x_axis[0] + b.x_axis[1] * b.x_axis[1] + b.x_axis[2] * b.x_axis[2]).sqrt();
    let ly =
        (b.y_axis[0] * b.y_axis[0] + b.y_axis[1] * b.y_axis[1] + b.y_axis[2] * b.y_axis[2]).sqrt();
    dot_xy.abs() < 0.01 && dot_yz.abs() < 0.01 && (lx - 1.0).abs() < 0.01 && (ly - 1.0).abs() < 0.01
}

#[allow(dead_code)]
pub fn total_bone_length(export: &BoneAxisExport) -> f32 {
    export.bones.iter().map(bone_length).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn arm_bone() -> BoneAxisExport {
        let mut e = new_bone_axis_export();
        add_bone_axis(&mut e, "upper_arm", [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.0);
        e
    }

    #[test]
    fn test_add_bone_axis() {
        let e = arm_bone();
        assert_eq!(bone_axis_count(&e), 1);
    }

    #[test]
    fn test_find_bone_axis_found() {
        let e = arm_bone();
        assert!(find_bone_axis(&e, "upper_arm").is_some());
    }

    #[test]
    fn test_find_bone_axis_missing() {
        let e = arm_bone();
        assert!(find_bone_axis(&e, "leg").is_none());
    }

    #[test]
    fn test_bone_length() {
        let e = arm_bone();
        let b = find_bone_axis(&e, "upper_arm").unwrap();
        assert!((bone_length(b) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_axes_orthonormal() {
        let e = arm_bone();
        let b = &e.bones[0];
        assert!(axes_are_orthonormal(b));
    }

    #[test]
    fn test_validate_bone_axes() {
        let e = arm_bone();
        assert!(validate_bone_axes(&e));
    }

    #[test]
    fn test_total_bone_length() {
        let mut e = arm_bone();
        add_bone_axis(&mut e, "forearm", [0.0, 1.0, 0.0], [0.0, 2.0, 0.0], 0.0);
        assert!((total_bone_length(&e) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_bone_axis_to_json() {
        let e = arm_bone();
        let b = &e.bones[0];
        let j = bone_axis_to_json(b);
        assert!(j.contains("upper_arm"));
    }

    #[test]
    fn test_roll_pi_over_4() {
        let mut e = new_bone_axis_export();
        add_bone_axis(&mut e, "rolled", [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], PI / 4.0);
        let b = &e.bones[0];
        assert!(axes_are_orthonormal(b));
    }

    #[test]
    fn test_export_to_json() {
        let e = arm_bone();
        let j = bone_axis_export_to_json(&e);
        assert!(j.contains("bone_count"));
    }
}
