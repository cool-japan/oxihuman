// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export with animation retargeting metadata (GLTF-style JSON output).

#[allow(dead_code)]
pub struct RetargetBoneMap {
    pub source_bone: String,
    pub target_bone: String,
    pub rotation_offset: [f32; 4],
    pub scale_ratio: f32,
}

#[allow(dead_code)]
pub struct RetargetExportConfig {
    pub bone_maps: Vec<RetargetBoneMap>,
    pub preserve_foot_contact: bool,
    pub preserve_hand_contact: bool,
    pub scale_root: bool,
    pub source_height: f32,
    pub target_height: f32,
}

#[allow(dead_code)]
pub struct RetargetFrame {
    pub time: f32,
    /// (bone_name, rotation_quat, translation)
    pub bone_transforms: Vec<(String, [f32; 4], [f32; 3])>,
}

#[allow(dead_code)]
pub struct RetargetAnimation {
    pub name: String,
    pub frames: Vec<RetargetFrame>,
    pub fps: f32,
    pub config: RetargetExportConfig,
}

impl Default for RetargetExportConfig {
    fn default() -> Self {
        RetargetExportConfig {
            bone_maps: Vec::new(),
            preserve_foot_contact: false,
            preserve_hand_contact: false,
            scale_root: true,
            source_height: 1.8,
            target_height: 1.8,
        }
    }
}

#[allow(dead_code)]
pub fn new_retarget_animation(
    name: &str,
    fps: f32,
    config: RetargetExportConfig,
) -> RetargetAnimation {
    RetargetAnimation {
        name: name.to_string(),
        frames: Vec::new(),
        fps,
        config,
    }
}

#[allow(dead_code)]
pub fn add_retarget_frame(anim: &mut RetargetAnimation, frame: RetargetFrame) {
    anim.frames.push(frame);
}

/// Multiply two quaternions a * b.
fn quat_mul(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let [ax, ay, az, aw] = a;
    let [bx, by, bz, bw] = b;
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}

fn quat_normalize(q: [f32; 4]) -> [f32; 4] {
    let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if len < 1e-9 {
        [0.0, 0.0, 0.0, 1.0]
    } else {
        [q[0] / len, q[1] / len, q[2] / len, q[3] / len]
    }
}

fn quat_lerp(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    // Ensure shortest path
    let dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
    let b_adj = if dot < 0.0 {
        [-b[0], -b[1], -b[2], -b[3]]
    } else {
        b
    };
    let r = [
        a[0] + (b_adj[0] - a[0]) * t,
        a[1] + (b_adj[1] - a[1]) * t,
        a[2] + (b_adj[2] - a[2]) * t,
        a[3] + (b_adj[3] - a[3]) * t,
    ];
    quat_normalize(r)
}

#[allow(dead_code)]
pub fn apply_bone_map(frame: &RetargetFrame, maps: &[RetargetBoneMap]) -> RetargetFrame {
    let mut new_transforms = Vec::new();
    for (bone, rot, trans) in &frame.bone_transforms {
        // Find matching bone map entry by source_bone
        let mapped = maps.iter().find(|m| &m.source_bone == bone);
        if let Some(m) = mapped {
            let new_rot = quat_normalize(quat_mul(*rot, m.rotation_offset));
            let scale = m.scale_ratio;
            let new_trans = [trans[0] * scale, trans[1] * scale, trans[2] * scale];
            new_transforms.push((m.target_bone.clone(), new_rot, new_trans));
        } else {
            new_transforms.push((bone.clone(), *rot, *trans));
        }
    }
    RetargetFrame {
        time: frame.time,
        bone_transforms: new_transforms,
    }
}

#[allow(dead_code)]
pub fn scale_animation(anim: &mut RetargetAnimation, scale: f32) {
    for frame in &mut anim.frames {
        for (_, _, trans) in &mut frame.bone_transforms {
            trans[0] *= scale;
            trans[1] *= scale;
            trans[2] *= scale;
        }
    }
}

#[allow(dead_code)]
pub fn retarget_animation_to_json(anim: &RetargetAnimation) -> String {
    let scale_ratio = if anim.config.source_height.abs() > 1e-9 {
        anim.config.target_height / anim.config.source_height
    } else {
        1.0
    };

    let mut out = format!(
        r#"{{"name":"{}","fps":{},"frame_count":{},"duration":{},"scale_ratio":{},"preserve_foot":{},"preserve_hand":{},"frames":["#,
        escape_json(&anim.name),
        anim.fps,
        anim.frames.len(),
        duration(anim),
        scale_ratio,
        anim.config.preserve_foot_contact,
        anim.config.preserve_hand_contact,
    );

    for (fi, frame) in anim.frames.iter().enumerate() {
        if fi > 0 {
            out.push(',');
        }
        out.push_str(&format!(r#"{{"time":{:.4},"bones":{{"#, frame.time));
        for (bi, (bone, rot, trans)) in frame.bone_transforms.iter().enumerate() {
            if bi > 0 {
                out.push(',');
            }
            out.push_str(&format!(
                r#""{}": {{"r":[{},{},{},{}],"t":[{},{},{}]}}"#,
                escape_json(bone),
                rot[0],
                rot[1],
                rot[2],
                rot[3],
                trans[0],
                trans[1],
                trans[2],
            ));
        }
        out.push_str("}}");
    }
    out.push_str("]}");
    out
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[allow(dead_code)]
pub fn duration(anim: &RetargetAnimation) -> f32 {
    anim.frames.last().map(|f| f.time).unwrap_or(0.0)
}

#[allow(dead_code)]
pub fn frame_at_time(anim: &RetargetAnimation, t: f32) -> Option<&RetargetFrame> {
    if anim.frames.is_empty() {
        return None;
    }
    let best = anim.frames.iter().min_by(|a, b| {
        let da = (a.time - t).abs();
        let db = (b.time - t).abs();
        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
    });
    best
}

#[allow(dead_code)]
pub fn interpolate_frames(a: &RetargetFrame, b: &RetargetFrame, t: f32) -> RetargetFrame {
    let mut bone_transforms = Vec::new();

    for (bone_a, rot_a, trans_a) in &a.bone_transforms {
        if let Some((_, rot_b, trans_b)) = b.bone_transforms.iter().find(|(bn, _, _)| bn == bone_a)
        {
            let rot = quat_lerp(*rot_a, *rot_b, t);
            let tr = [
                trans_a[0] + (trans_b[0] - trans_a[0]) * t,
                trans_a[1] + (trans_b[1] - trans_a[1]) * t,
                trans_a[2] + (trans_b[2] - trans_a[2]) * t,
            ];
            bone_transforms.push((bone_a.clone(), rot, tr));
        } else {
            bone_transforms.push((bone_a.clone(), *rot_a, *trans_a));
        }
    }

    RetargetFrame {
        time: a.time + (b.time - a.time) * t,
        bone_transforms,
    }
}

#[allow(dead_code)]
pub fn standard_humanoid_bone_map(src_rig: &str, tgt_rig: &str) -> Vec<RetargetBoneMap> {
    let identity = [0.0f32, 0.0, 0.0, 1.0];

    // Common mapping between popular rig naming conventions
    let pairs: &[(&str, &str)] = match (src_rig, tgt_rig) {
        ("mixamo", "humanoid") => &[
            ("Hips", "pelvis"),
            ("Spine", "spine_01"),
            ("Spine1", "spine_02"),
            ("Spine2", "spine_03"),
            ("Neck", "neck_01"),
            ("Head", "head"),
            ("LeftArm", "upperarm_l"),
            ("LeftForeArm", "lowerarm_l"),
            ("LeftHand", "hand_l"),
            ("RightArm", "upperarm_r"),
            ("RightForeArm", "lowerarm_r"),
            ("RightHand", "hand_r"),
            ("LeftUpLeg", "thigh_l"),
            ("LeftLeg", "calf_l"),
            ("LeftFoot", "foot_l"),
            ("RightUpLeg", "thigh_r"),
            ("RightLeg", "calf_r"),
            ("RightFoot", "foot_r"),
        ],
        ("humanoid", "mixamo") => &[
            ("pelvis", "Hips"),
            ("spine_01", "Spine"),
            ("spine_02", "Spine1"),
            ("spine_03", "Spine2"),
            ("neck_01", "Neck"),
            ("head", "Head"),
            ("upperarm_l", "LeftArm"),
            ("lowerarm_l", "LeftForeArm"),
            ("hand_l", "LeftHand"),
            ("upperarm_r", "RightArm"),
            ("lowerarm_r", "RightForeArm"),
            ("hand_r", "RightHand"),
            ("thigh_l", "LeftUpLeg"),
            ("calf_l", "LeftLeg"),
            ("foot_l", "LeftFoot"),
            ("thigh_r", "RightUpLeg"),
            ("calf_r", "RightLeg"),
            ("foot_r", "RightFoot"),
        ],
        _ => &[],
    };

    pairs
        .iter()
        .map(|(src, tgt)| RetargetBoneMap {
            source_bone: src.to_string(),
            target_bone: tgt.to_string(),
            rotation_offset: identity,
            scale_ratio: 1.0,
        })
        .collect()
}

#[allow(dead_code)]
pub fn validate_bone_map(maps: &[RetargetBoneMap], available_bones: &[String]) -> Vec<String> {
    let mut errors = Vec::new();
    for m in maps {
        if !available_bones.contains(&m.source_bone) {
            errors.push(format!("source bone '{}' not found", m.source_bone));
        }
        if !available_bones.contains(&m.target_bone) {
            errors.push(format!("target bone '{}' not found", m.target_bone));
        }
        // Validate quaternion normalization
        let len = (m.rotation_offset[0].powi(2)
            + m.rotation_offset[1].powi(2)
            + m.rotation_offset[2].powi(2)
            + m.rotation_offset[3].powi(2))
        .sqrt();
        if (len - 1.0).abs() > 0.01 {
            errors.push(format!(
                "bone '{}' rotation_offset is not normalized (len={len:.4})",
                m.source_bone
            ));
        }
    }
    errors
}

#[allow(dead_code)]
pub fn strip_retarget_metadata(anim: &mut RetargetAnimation) {
    anim.config.bone_maps.clear();
    anim.config.preserve_foot_contact = false;
    anim.config.preserve_hand_contact = false;
    anim.config.scale_root = false;
    anim.config.source_height = 0.0;
    anim.config.target_height = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config() -> RetargetExportConfig {
        RetargetExportConfig {
            source_height: 1.8,
            target_height: 1.6,
            ..Default::default()
        }
    }

    fn make_frame(t: f32) -> RetargetFrame {
        RetargetFrame {
            time: t,
            bone_transforms: vec![
                ("Hips".to_string(), [0.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0]),
                ("Spine".to_string(), [0.0, 0.0, 0.0, 1.0], [0.0, 0.1, 0.0]),
            ],
        }
    }

    #[test]
    fn test_new_anim() {
        let anim = new_retarget_animation("walk", 30.0, make_config());
        assert_eq!(anim.name, "walk");
        assert!((anim.fps - 30.0).abs() < 1e-6);
        assert!(anim.frames.is_empty());
    }

    #[test]
    fn test_add_frame() {
        let mut anim = new_retarget_animation("run", 24.0, make_config());
        add_retarget_frame(&mut anim, make_frame(0.0));
        add_retarget_frame(&mut anim, make_frame(1.0 / 24.0));
        assert_eq!(anim.frames.len(), 2);
    }

    #[test]
    fn test_duration() {
        let mut anim = new_retarget_animation("t", 24.0, make_config());
        assert!((duration(&anim) - 0.0).abs() < 1e-6);
        add_retarget_frame(&mut anim, make_frame(0.0));
        add_retarget_frame(&mut anim, make_frame(0.5));
        assert!((duration(&anim) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_apply_bone_map() {
        let maps = standard_humanoid_bone_map("mixamo", "humanoid");
        let frame = make_frame(0.0);
        let mapped = apply_bone_map(&frame, &maps);
        let has_pelvis = mapped.bone_transforms.iter().any(|(b, _, _)| b == "pelvis");
        assert!(has_pelvis);
    }

    #[test]
    fn test_scale_animation() {
        let mut anim = new_retarget_animation("s", 24.0, make_config());
        add_retarget_frame(&mut anim, make_frame(0.0));
        scale_animation(&mut anim, 2.0);
        let trans = &anim.frames[0].bone_transforms[0].2;
        assert!((trans[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let mut anim = new_retarget_animation("walk", 30.0, make_config());
        add_retarget_frame(&mut anim, make_frame(0.0));
        let json = retarget_animation_to_json(&anim);
        assert!(json.contains("\"name\":\"walk\""));
        assert!(json.contains("Hips") || json.contains("frames"));
    }

    #[test]
    fn test_frame_at_time() {
        let mut anim = new_retarget_animation("t", 24.0, make_config());
        add_retarget_frame(&mut anim, make_frame(0.0));
        add_retarget_frame(&mut anim, make_frame(1.0));
        let f = frame_at_time(&anim, 0.4);
        assert!(f.is_some());
        assert!((f.expect("should succeed").time - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_frame_at_time_empty() {
        let anim = new_retarget_animation("t", 24.0, make_config());
        assert!(frame_at_time(&anim, 0.0).is_none());
    }

    #[test]
    fn test_interpolate_frames() {
        let a = make_frame(0.0);
        let b = RetargetFrame {
            time: 1.0,
            bone_transforms: vec![
                ("Hips".to_string(), [0.0, 0.0, 0.0, 1.0], [0.0, 2.0, 0.0]),
                ("Spine".to_string(), [0.0, 0.0, 0.0, 1.0], [0.0, 0.2, 0.0]),
            ],
        };
        let mid = interpolate_frames(&a, &b, 0.5);
        let hips_trans = mid
            .bone_transforms
            .iter()
            .find(|(n, _, _)| n == "Hips")
            .expect("should succeed");
        assert!((hips_trans.2[1] - 1.5).abs() < 1e-5);
    }

    #[test]
    fn test_standard_bone_map_mixamo_to_humanoid() {
        let maps = standard_humanoid_bone_map("mixamo", "humanoid");
        assert!(!maps.is_empty());
        assert!(maps.iter().any(|m| m.source_bone == "Hips"));
    }

    #[test]
    fn test_standard_bone_map_humanoid_to_mixamo() {
        let maps = standard_humanoid_bone_map("humanoid", "mixamo");
        assert!(!maps.is_empty());
        assert!(maps.iter().any(|m| m.source_bone == "pelvis"));
    }

    #[test]
    fn test_standard_bone_map_unknown() {
        let maps = standard_humanoid_bone_map("foo", "bar");
        assert!(maps.is_empty());
    }

    #[test]
    fn test_validate_bone_map_missing() {
        let maps = standard_humanoid_bone_map("mixamo", "humanoid");
        let available: Vec<String> = vec!["Hips".to_string()];
        let errors = validate_bone_map(&maps, &available);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_validate_bone_map_ok() {
        let available: Vec<String> = vec!["Hips".to_string(), "pelvis".to_string()];
        let maps = vec![RetargetBoneMap {
            source_bone: "Hips".to_string(),
            target_bone: "pelvis".to_string(),
            rotation_offset: [0.0, 0.0, 0.0, 1.0],
            scale_ratio: 1.0,
        }];
        let errors = validate_bone_map(&maps, &available);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_strip_metadata() {
        let mut anim = new_retarget_animation("walk", 30.0, make_config());
        anim.config.bone_maps.push(RetargetBoneMap {
            source_bone: "Hips".to_string(),
            target_bone: "pelvis".to_string(),
            rotation_offset: [0.0, 0.0, 0.0, 1.0],
            scale_ratio: 1.0,
        });
        strip_retarget_metadata(&mut anim);
        assert!(anim.config.bone_maps.is_empty());
        assert!(!anim.config.preserve_foot_contact);
    }
}
