//! Facial rigging with control bones and corrective shapes.

#[allow(dead_code)]
pub struct FacialBone {
    pub name: String,
    pub position: [f32; 3],
    pub rotation: [f32; 4], // quaternion
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub weight: f32,
    pub influence_radius: f32,
}

#[allow(dead_code)]
pub struct CorrectiveShape {
    pub name: String,
    pub trigger_bone: String,
    pub trigger_angle: f32, // radians
    pub deltas: Vec<[f32; 3]>,
    pub weight: f32,
}

#[allow(dead_code)]
pub struct FacialRig {
    pub bones: Vec<FacialBone>,
    pub correctives: Vec<CorrectiveShape>,
    pub version: u32,
}

#[allow(dead_code)]
pub struct FacialPose {
    pub bone_rotations: Vec<[f32; 4]>, // per-bone quaternions
    pub blend_weights: Vec<f32>,       // per-corrective weights
}

#[allow(dead_code)]
pub fn new_facial_rig() -> FacialRig {
    FacialRig {
        bones: Vec::new(),
        correctives: Vec::new(),
        version: 1,
    }
}

#[allow(dead_code)]
pub fn add_bone(rig: &mut FacialRig, name: &str, pos: [f32; 3], parent: Option<usize>) -> usize {
    let idx = rig.bones.len();
    if let Some(p) = parent {
        if p < rig.bones.len() {
            rig.bones[p].children.push(idx);
        }
    }
    rig.bones.push(FacialBone {
        name: name.to_string(),
        position: pos,
        rotation: [0.0, 0.0, 0.0, 1.0],
        parent,
        children: Vec::new(),
        weight: 1.0,
        influence_radius: 0.1,
    });
    idx
}

#[allow(dead_code)]
pub fn add_corrective(rig: &mut FacialRig, c: CorrectiveShape) {
    rig.correctives.push(c);
}

#[allow(dead_code)]
pub fn default_facial_rig() -> FacialRig {
    let mut rig = new_facial_rig();
    // Root jaw
    let jaw = add_bone(&mut rig, "jaw", [0.0, -0.05, 0.0], None);
    // Eyes
    let _eye_l = add_bone(&mut rig, "eye_l", [-0.03, 0.02, 0.04], None);
    let _eye_r = add_bone(&mut rig, "eye_r", [0.03, 0.02, 0.04], None);
    // Brows
    let _brow_l = add_bone(&mut rig, "brow_l", [-0.03, 0.05, 0.03], None);
    let _brow_l_inner = add_bone(&mut rig, "brow_l_inner", [-0.015, 0.05, 0.03], None);
    let _brow_r = add_bone(&mut rig, "brow_r", [0.03, 0.05, 0.03], None);
    let _brow_r_inner = add_bone(&mut rig, "brow_r_inner", [0.015, 0.05, 0.03], None);
    // Cheeks
    let _cheek_l = add_bone(&mut rig, "cheek_l", [-0.04, 0.0, 0.03], None);
    let _cheek_r = add_bone(&mut rig, "cheek_r", [0.04, 0.0, 0.03], None);
    // Lips (children of jaw)
    let _lip_upper = add_bone(&mut rig, "lip_upper", [0.0, -0.01, 0.05], Some(jaw));
    let _lip_lower = add_bone(&mut rig, "lip_lower", [0.0, -0.03, 0.05], Some(jaw));
    let _lip_corner_l = add_bone(&mut rig, "lip_corner_l", [-0.02, -0.02, 0.05], Some(jaw));
    let _lip_corner_r = add_bone(&mut rig, "lip_corner_r", [0.02, -0.02, 0.05], Some(jaw));
    // Nose
    let _nose = add_bone(&mut rig, "nose", [0.0, 0.01, 0.06], None);
    rig
}

#[allow(dead_code)]
pub fn get_bone<'a>(rig: &'a FacialRig, name: &str) -> Option<&'a FacialBone> {
    rig.bones.iter().find(|b| b.name == name)
}

#[allow(dead_code)]
pub fn set_bone_rotation(rig: &mut FacialRig, name: &str, rot: [f32; 4]) -> bool {
    if let Some(bone) = rig.bones.iter_mut().find(|b| b.name == name) {
        bone.rotation = rot;
        true
    } else {
        false
    }
}

/// Compute corrective weights from bone angles.
#[allow(dead_code)]
pub fn evaluate_correctives(rig: &FacialRig) -> Vec<f32> {
    rig.correctives
        .iter()
        .map(|c| {
            // Find trigger bone rotation angle
            if let Some(bone) = rig.bones.iter().find(|b| b.name == c.trigger_bone) {
                let angle = quat_angle_from_identity(bone.rotation);
                let diff = (angle - c.trigger_angle).abs();
                let w = (1.0 - diff / std::f32::consts::PI).max(0.0);
                w * c.weight
            } else {
                0.0
            }
        })
        .collect()
}

fn quat_angle_from_identity(q: [f32; 4]) -> f32 {
    // w component of normalized quaternion → angle = 2 * acos(w)
    let w = q[3].clamp(-1.0, 1.0);
    2.0 * w.acos()
}

#[allow(dead_code)]
pub fn apply_facial_pose(
    rig: &FacialRig,
    pose: &FacialPose,
    base_positions: &[[f32; 3]],
) -> Vec<[f32; 3]> {
    let mut result: Vec<[f32; 3]> = base_positions.to_vec();

    // Apply corrective shape deltas
    for (i, &w) in pose.blend_weights.iter().enumerate() {
        if i >= rig.correctives.len() {
            break;
        }
        if w.abs() < 1e-6 {
            continue;
        }
        let corr = &rig.correctives[i];
        for (j, pos) in result.iter_mut().enumerate() {
            if j < corr.deltas.len() {
                pos[0] += corr.deltas[j][0] * w;
                pos[1] += corr.deltas[j][1] * w;
                pos[2] += corr.deltas[j][2] * w;
            }
        }
    }
    result
}

#[allow(dead_code)]
pub fn bone_count(rig: &FacialRig) -> usize {
    rig.bones.len()
}

#[allow(dead_code)]
pub fn corrective_count(rig: &FacialRig) -> usize {
    rig.correctives.len()
}

#[allow(dead_code)]
pub fn facial_rig_to_json(rig: &FacialRig) -> String {
    let mut s = String::from("{");
    s.push_str(&format!(
        r#""version":{},"bone_count":{},"corrective_count":{}"#,
        rig.version,
        rig.bones.len(),
        rig.correctives.len()
    ));
    s.push('}');
    s
}

#[allow(dead_code)]
pub fn identity_pose(rig: &FacialRig) -> FacialPose {
    FacialPose {
        bone_rotations: vec![[0.0, 0.0, 0.0, 1.0]; rig.bones.len()],
        blend_weights: vec![0.0; rig.correctives.len()],
    }
}

#[allow(dead_code)]
pub fn blend_facial_poses(a: &FacialPose, b: &FacialPose, t: f32) -> FacialPose {
    let len = a.bone_rotations.len().min(b.bone_rotations.len());
    let bone_rotations = (0..len)
        .map(|i| {
            let qa = a.bone_rotations[i];
            let qb = b.bone_rotations[i];
            slerp_quat(qa, qb, t)
        })
        .collect();

    let wlen = a.blend_weights.len().min(b.blend_weights.len());
    let blend_weights = (0..wlen)
        .map(|i| a.blend_weights[i] * (1.0 - t) + b.blend_weights[i] * t)
        .collect();

    FacialPose {
        bone_rotations,
        blend_weights,
    }
}

fn slerp_quat(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let mut dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
    let bq = if dot < 0.0 {
        dot = -dot;
        [-b[0], -b[1], -b[2], -b[3]]
    } else {
        b
    };

    if dot > 0.9995 {
        let r = [
            a[0] + t * (bq[0] - a[0]),
            a[1] + t * (bq[1] - a[1]),
            a[2] + t * (bq[2] - a[2]),
            a[3] + t * (bq[3] - a[3]),
        ];
        normalize_q(r)
    } else {
        let theta_0 = dot.acos();
        let theta = theta_0 * t;
        let sin_theta = theta.sin();
        let sin_theta_0 = theta_0.sin();
        let s0 = (theta_0 * (1.0 - t)).sin() / sin_theta_0;
        let s1 = sin_theta / sin_theta_0;
        [
            s0 * a[0] + s1 * bq[0],
            s0 * a[1] + s1 * bq[1],
            s0 * a[2] + s1 * bq[2],
            s0 * a[3] + s1 * bq[3],
        ]
    }
}

fn normalize_q(q: [f32; 4]) -> [f32; 4] {
    let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if len < 1e-9 {
        [0.0, 0.0, 0.0, 1.0]
    } else {
        [q[0] / len, q[1] / len, q[2] / len, q[3] / len]
    }
}

#[allow(dead_code)]
pub fn quat_angle_between(q1: [f32; 4], q2: [f32; 4]) -> f32 {
    // angle between two quaternions: 2 * acos(|q1 . q2|)
    let dot = (q1[0] * q2[0] + q1[1] * q2[1] + q1[2] * q2[2] + q1[3] * q2[3])
        .abs()
        .clamp(0.0, 1.0);
    2.0 * dot.acos()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_new_facial_rig_empty() {
        let rig = new_facial_rig();
        assert_eq!(rig.bones.len(), 0);
        assert_eq!(rig.correctives.len(), 0);
        assert_eq!(rig.version, 1);
    }

    #[test]
    fn test_add_bone_returns_index() {
        let mut rig = new_facial_rig();
        let idx = add_bone(&mut rig, "jaw", [0.0, 0.0, 0.0], None);
        assert_eq!(idx, 0);
        assert_eq!(rig.bones.len(), 1);
        assert_eq!(rig.bones[0].name, "jaw");
    }

    #[test]
    fn test_add_bone_parent_child_link() {
        let mut rig = new_facial_rig();
        let p = add_bone(&mut rig, "root", [0.0, 0.0, 0.0], None);
        let c = add_bone(&mut rig, "child", [1.0, 0.0, 0.0], Some(p));
        assert!(rig.bones[p].children.contains(&c));
        assert_eq!(rig.bones[c].parent, Some(p));
    }

    #[test]
    fn test_add_corrective() {
        let mut rig = new_facial_rig();
        add_bone(&mut rig, "jaw", [0.0, 0.0, 0.0], None);
        let cs = CorrectiveShape {
            name: "jaw_open".to_string(),
            trigger_bone: "jaw".to_string(),
            trigger_angle: PI / 4.0,
            deltas: vec![[0.0, -0.01, 0.0]; 3],
            weight: 1.0,
        };
        add_corrective(&mut rig, cs);
        assert_eq!(rig.correctives.len(), 1);
    }

    #[test]
    fn test_default_facial_rig_non_empty() {
        let rig = default_facial_rig();
        assert!(rig.bones.len() >= 14);
    }

    #[test]
    fn test_get_bone_found() {
        let rig = default_facial_rig();
        let bone = get_bone(&rig, "jaw");
        assert!(bone.is_some());
        assert_eq!(bone.expect("should succeed").name, "jaw");
    }

    #[test]
    fn test_get_bone_not_found() {
        let rig = default_facial_rig();
        assert!(get_bone(&rig, "nonexistent").is_none());
    }

    #[test]
    fn test_bone_count() {
        let rig = default_facial_rig();
        assert_eq!(bone_count(&rig), rig.bones.len());
    }

    #[test]
    fn test_corrective_count_zero() {
        let rig = default_facial_rig();
        assert_eq!(corrective_count(&rig), 0);
    }

    #[test]
    fn test_set_bone_rotation_success() {
        let mut rig = default_facial_rig();
        let rot = [0.0, 0.0, 0.707, 0.707];
        let ok = set_bone_rotation(&mut rig, "jaw", rot);
        assert!(ok);
        let bone = get_bone(&rig, "jaw").expect("should succeed");
        assert_eq!(bone.rotation, rot);
    }

    #[test]
    fn test_set_bone_rotation_fail() {
        let mut rig = default_facial_rig();
        let ok = set_bone_rotation(&mut rig, "ghost_bone", [0.0, 0.0, 0.0, 1.0]);
        assert!(!ok);
    }

    #[test]
    fn test_identity_pose() {
        let rig = default_facial_rig();
        let pose = identity_pose(&rig);
        assert_eq!(pose.bone_rotations.len(), rig.bones.len());
        for rot in &pose.bone_rotations {
            assert_eq!(*rot, [0.0, 0.0, 0.0, 1.0]);
        }
        for w in &pose.blend_weights {
            assert_eq!(*w, 0.0);
        }
    }

    #[test]
    fn test_blend_facial_poses() {
        let rig = default_facial_rig();
        let a = identity_pose(&rig);
        let mut b = identity_pose(&rig);
        if !b.bone_rotations.is_empty() {
            b.bone_rotations[0] = [0.0, 0.0, 0.707, 0.707];
        }
        let blended = blend_facial_poses(&a, &b, 0.5);
        assert_eq!(blended.bone_rotations.len(), a.bone_rotations.len());
    }

    #[test]
    fn test_evaluate_correctives_empty() {
        let rig = default_facial_rig();
        let weights = evaluate_correctives(&rig);
        assert!(weights.is_empty());
    }

    #[test]
    fn test_apply_facial_pose_no_correctives() {
        let rig = default_facial_rig();
        let pose = identity_pose(&rig);
        let base = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let result = apply_facial_pose(&rig, &pose, &base);
        assert_eq!(result.len(), base.len());
        for (r, b) in result.iter().zip(base.iter()) {
            assert!((r[0] - b[0]).abs() < 1e-6);
        }
    }

    #[test]
    fn test_quat_angle_between_identity() {
        let q = [0.0, 0.0, 0.0, 1.0];
        let angle = quat_angle_between(q, q);
        assert!(angle < 1e-5);
    }

    #[test]
    fn test_facial_rig_to_json_contains_version() {
        let rig = default_facial_rig();
        let json = facial_rig_to_json(&rig);
        assert!(json.contains("version"));
        assert!(json.contains("bone_count"));
    }
}
