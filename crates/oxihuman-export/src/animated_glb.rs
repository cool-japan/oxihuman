// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use oxihuman_mesh::MeshBuffers;

// ── Data types ────────────────────────────────────────────────────────────────

/// A single joint in a skeletal hierarchy.
pub struct SkeletonJoint {
    pub name: String,
    pub parent: Option<usize>,
    pub bind_translation: [f32; 3],
    pub bind_rotation: [f32; 4], // xyzw quaternion
    pub bind_scale: [f32; 3],
}

/// Per-joint animation keyframe data.
pub struct JointKeyframes {
    pub joint_idx: usize,
    pub times: Vec<f32>,
    pub translations: Option<Vec<[f32; 3]>>,
    pub rotations: Option<Vec<[f32; 4]>>,
    pub scales: Option<Vec<[f32; 3]>>,
}

/// Options controlling animated GLB output.
pub struct AnimatedGlbOptions {
    pub include_skeleton: bool,
    pub include_morph_weights: bool,
    pub fps: f32,
    pub duration: f32,
    pub morph_target_names: Vec<String>,
}

/// Summary of a build_animated_glb_json call.
pub struct AnimatedGlbResult {
    pub json_size: usize,
    pub bin_size: usize,
    pub joint_count: usize,
    pub morph_target_count: usize,
    pub keyframe_count: usize,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Produce a GLTF JSON string with skins + animations sections.
#[allow(clippy::too_many_arguments)]
pub fn build_animated_glb_json(
    mesh: &MeshBuffers,
    skeleton: &[SkeletonJoint],
    joint_anims: &[JointKeyframes],
    morph_times: &[f32],
    morph_weight_frames: &[Vec<f32>],
    opts: &AnimatedGlbOptions,
) -> String {
    let vertex_count = mesh.positions.len();
    let face_count = mesh.indices.len() / 3;

    let mut sections: Vec<String> = Vec::new();

    // Mesh primitive
    sections.push(r#""meshes": [{ "name": "Body", "primitives": [{ "attributes": { "POSITION": 0 }, "indices": 1 }] }]"#.to_string());

    // Asset block
    sections
        .push(r#""asset": { "version": "2.0", "generator": "OxiHuman animated_glb" }"#.to_string());

    // Mesh statistics comment (embedded in extras)
    sections.push(format!(
        r#""extras": {{ "vertexCount": {}, "faceCount": {} }}"#,
        vertex_count, face_count
    ));

    // Skins section
    if opts.include_skeleton && !skeleton.is_empty() {
        let skin_json = build_skeleton_json(skeleton);
        sections.push(format!(r#""skins": [{}]"#, skin_json));
    }

    // Animations section
    let mut anim_parts: Vec<String> = Vec::new();

    if !joint_anims.is_empty() {
        let samplers = build_joint_anim_samplers_json(joint_anims, 10);
        anim_parts.push(format!(
            r#"{{ "name": "JointAnimation", "samplers": [{}], "channels": [] }}"#,
            samplers
        ));
    }

    if opts.include_morph_weights && !morph_times.is_empty() && !morph_weight_frames.is_empty() {
        let morph_samplers = build_morph_anim_samplers_json(morph_times, morph_weight_frames, 100);
        anim_parts.push(format!(
            r#"{{ "name": "MorphAnimation", "samplers": [{}], "channels": [] }}"#,
            morph_samplers
        ));
    }

    if !anim_parts.is_empty() {
        sections.push(format!(r#""animations": [{}]"#, anim_parts.join(", ")));
    }

    // Morph target names
    if !opts.morph_target_names.is_empty() {
        let names: Vec<String> = opts
            .morph_target_names
            .iter()
            .map(|n| format!(r#""{}""#, n))
            .collect();
        sections.push(format!(r#""morphTargetNames": [{}]"#, names.join(", ")));
    }

    format!("{{\n  {}\n}}", sections.join(",\n  "))
}

/// Produce the "skins" array entry JSON for the given joints.
pub fn build_skeleton_json(joints: &[SkeletonJoint]) -> String {
    let joint_indices: Vec<String> = (0..joints.len()).map(|i| i.to_string()).collect();

    let joint_names: Vec<String> = joints.iter().map(|j| format!(r#""{}""#, j.name)).collect();

    format!(
        r#"{{ "name": "Armature", "joints": [{}], "jointNames": [{}], "skeleton": 0 }}"#,
        joint_indices.join(", "),
        joint_names.join(", ")
    )
}

/// Produce GLTF animation sampler JSON entries for joint keyframes.
pub fn build_joint_anim_samplers_json(anims: &[JointKeyframes], base_accessor: u32) -> String {
    let mut samplers: Vec<String> = Vec::new();
    let mut acc = base_accessor;

    for anim in anims {
        if anim.translations.is_some() {
            samplers.push(format!(
                r#"{{ "input": {}, "output": {}, "interpolation": "LINEAR", "target": "translation", "joint": {} }}"#,
                acc, acc + 1, anim.joint_idx
            ));
            acc += 2;
        }
        if anim.rotations.is_some() {
            samplers.push(format!(
                r#"{{ "input": {}, "output": {}, "interpolation": "LINEAR", "target": "rotation", "joint": {} }}"#,
                acc, acc + 1, anim.joint_idx
            ));
            acc += 2;
        }
        if anim.scales.is_some() {
            samplers.push(format!(
                r#"{{ "input": {}, "output": {}, "interpolation": "LINEAR", "target": "scale", "joint": {} }}"#,
                acc, acc + 1, anim.joint_idx
            ));
            acc += 2;
        }
    }

    samplers.join(", ")
}

/// Produce GLTF morph weight animation sampler JSON.
pub fn build_morph_anim_samplers_json(
    times: &[f32],
    weight_frames: &[Vec<f32>],
    accessor_base: u32,
) -> String {
    let time_count = times.len();
    let morph_count = weight_frames.first().map(|f| f.len()).unwrap_or(0);

    let times_str: Vec<String> = times.iter().map(|t| format!("{:.4}", t)).collect();
    let time_min = times.iter().cloned().fold(f32::INFINITY, f32::min);
    let time_max = times.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

    format!(
        r#"{{ "input": {}, "output": {}, "interpolation": "LINEAR", "timesAccessor": {{ "count": {}, "min": [{:.4}], "max": [{:.4}], "times": [{}] }}, "morphCount": {}, "frameCount": {} }}"#,
        accessor_base,
        accessor_base + 1,
        time_count,
        time_min,
        time_max,
        times_str.join(", "),
        morph_count,
        weight_frames.len()
    )
}

/// Return a 17-joint T-pose biped skeleton.
///
/// Joints: pelvis, spine_lower, spine_mid, spine_upper, head,
/// shoulder_l, shoulder_r, elbow_l, elbow_r, wrist_l, wrist_r,
/// hip_l, hip_r, knee_l, knee_r, ankle_l, ankle_r
pub fn default_t_pose_skeleton() -> Vec<SkeletonJoint> {
    vec![
        // 0: pelvis (root)
        SkeletonJoint {
            name: "pelvis".to_string(),
            parent: None,
            bind_translation: [0.0, 0.98, 0.0],
            bind_rotation: [0.0, 0.0, 0.0, 1.0],
            bind_scale: [1.0, 1.0, 1.0],
        },
        // 1: spine_lower
        SkeletonJoint {
            name: "spine_lower".to_string(),
            parent: Some(0),
            bind_translation: [0.0, 0.12, 0.0],
            bind_rotation: [0.0, 0.0, 0.0, 1.0],
            bind_scale: [1.0, 1.0, 1.0],
        },
        // 2: spine_mid
        SkeletonJoint {
            name: "spine_mid".to_string(),
            parent: Some(1),
            bind_translation: [0.0, 0.12, 0.0],
            bind_rotation: [0.0, 0.0, 0.0, 1.0],
            bind_scale: [1.0, 1.0, 1.0],
        },
        // 3: spine_upper
        SkeletonJoint {
            name: "spine_upper".to_string(),
            parent: Some(2),
            bind_translation: [0.0, 0.12, 0.0],
            bind_rotation: [0.0, 0.0, 0.0, 1.0],
            bind_scale: [1.0, 1.0, 1.0],
        },
        // 4: head
        SkeletonJoint {
            name: "head".to_string(),
            parent: Some(3),
            bind_translation: [0.0, 0.25, 0.0],
            bind_rotation: [0.0, 0.0, 0.0, 1.0],
            bind_scale: [1.0, 1.0, 1.0],
        },
        // 5: shoulder_l
        SkeletonJoint {
            name: "shoulder_l".to_string(),
            parent: Some(3),
            bind_translation: [0.18, 0.0, 0.0],
            bind_rotation: [0.0, 0.0, 0.0, 1.0],
            bind_scale: [1.0, 1.0, 1.0],
        },
        // 6: shoulder_r
        SkeletonJoint {
            name: "shoulder_r".to_string(),
            parent: Some(3),
            bind_translation: [-0.18, 0.0, 0.0],
            bind_rotation: [0.0, 0.0, 0.0, 1.0],
            bind_scale: [1.0, 1.0, 1.0],
        },
        // 7: elbow_l
        SkeletonJoint {
            name: "elbow_l".to_string(),
            parent: Some(5),
            bind_translation: [0.28, 0.0, 0.0],
            bind_rotation: [0.0, 0.0, 0.0, 1.0],
            bind_scale: [1.0, 1.0, 1.0],
        },
        // 8: elbow_r
        SkeletonJoint {
            name: "elbow_r".to_string(),
            parent: Some(6),
            bind_translation: [-0.28, 0.0, 0.0],
            bind_rotation: [0.0, 0.0, 0.0, 1.0],
            bind_scale: [1.0, 1.0, 1.0],
        },
        // 9: wrist_l
        SkeletonJoint {
            name: "wrist_l".to_string(),
            parent: Some(7),
            bind_translation: [0.26, 0.0, 0.0],
            bind_rotation: [0.0, 0.0, 0.0, 1.0],
            bind_scale: [1.0, 1.0, 1.0],
        },
        // 10: wrist_r
        SkeletonJoint {
            name: "wrist_r".to_string(),
            parent: Some(8),
            bind_translation: [-0.26, 0.0, 0.0],
            bind_rotation: [0.0, 0.0, 0.0, 1.0],
            bind_scale: [1.0, 1.0, 1.0],
        },
        // 11: hip_l
        SkeletonJoint {
            name: "hip_l".to_string(),
            parent: Some(0),
            bind_translation: [0.10, -0.08, 0.0],
            bind_rotation: [0.0, 0.0, 0.0, 1.0],
            bind_scale: [1.0, 1.0, 1.0],
        },
        // 12: hip_r
        SkeletonJoint {
            name: "hip_r".to_string(),
            parent: Some(0),
            bind_translation: [-0.10, -0.08, 0.0],
            bind_rotation: [0.0, 0.0, 0.0, 1.0],
            bind_scale: [1.0, 1.0, 1.0],
        },
        // 13: knee_l
        SkeletonJoint {
            name: "knee_l".to_string(),
            parent: Some(11),
            bind_translation: [0.0, -0.45, 0.0],
            bind_rotation: [0.0, 0.0, 0.0, 1.0],
            bind_scale: [1.0, 1.0, 1.0],
        },
        // 14: knee_r
        SkeletonJoint {
            name: "knee_r".to_string(),
            parent: Some(12),
            bind_translation: [0.0, -0.45, 0.0],
            bind_rotation: [0.0, 0.0, 0.0, 1.0],
            bind_scale: [1.0, 1.0, 1.0],
        },
        // 15: ankle_l
        SkeletonJoint {
            name: "ankle_l".to_string(),
            parent: Some(13),
            bind_translation: [0.0, -0.45, 0.0],
            bind_rotation: [0.0, 0.0, 0.0, 1.0],
            bind_scale: [1.0, 1.0, 1.0],
        },
        // 16: ankle_r
        SkeletonJoint {
            name: "ankle_r".to_string(),
            parent: Some(14),
            bind_translation: [0.0, -0.45, 0.0],
            bind_rotation: [0.0, 0.0, 0.0, 1.0],
            bind_scale: [1.0, 1.0, 1.0],
        },
    ]
}

/// Generate a subtle idle (breathing) animation as sinusoidal rotation
/// keyframes on the three spine joints (indices 1, 2, 3).
pub fn generate_idle_animation(
    skeleton: &[SkeletonJoint],
    fps: f32,
    duration: f32,
) -> Vec<JointKeyframes> {
    let frame_count = ((fps * duration) as usize).max(2);
    let times: Vec<f32> = (0..frame_count).map(|i| i as f32 / fps).collect();

    // Spine joints: look for joints named spine_* or with parent chain through index 0
    let spine_indices: Vec<usize> = skeleton
        .iter()
        .enumerate()
        .filter(|(_, j)| j.name.starts_with("spine"))
        .map(|(i, _)| i)
        .collect();

    let mut result: Vec<JointKeyframes> = Vec::new();

    for &joint_idx in &spine_indices {
        // Breathing: gentle sinusoidal rotation around X axis
        let amplitude = 0.01f32; // ~0.57 degrees
        let freq = 0.25f32; // 0.25 Hz breathing cycle

        let rotations: Vec<[f32; 4]> = times
            .iter()
            .map(|&t| {
                let angle = amplitude * (2.0 * std::f32::consts::PI * freq * t).sin();
                let half = angle * 0.5;
                // quaternion for rotation around X
                [half.sin(), 0.0, 0.0, half.cos()]
            })
            .collect();

        result.push(JointKeyframes {
            joint_idx,
            times: times.clone(),
            translations: None,
            rotations: Some(rotations),
            scales: None,
        });
    }

    result
}

/// Return a human-readable summary string for AnimatedGlbResult.
pub fn animated_glb_stats(result: &AnimatedGlbResult) -> String {
    format!(
        "AnimatedGlb: json={} bytes, bin={} bytes, joints={}, morphTargets={}, keyframes={}",
        result.json_size,
        result.bin_size,
        result.joint_count,
        result.morph_target_count,
        result.keyframe_count,
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn stub_mesh() -> MeshBuffers {
        MeshBuffers {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            colors: None,
            has_suit: false,
        }
    }

    #[test]
    fn t_pose_has_17_joints() {
        let skel = default_t_pose_skeleton();
        assert_eq!(skel.len(), 17);
    }

    #[test]
    fn t_pose_root_has_no_parent() {
        let skel = default_t_pose_skeleton();
        assert!(
            skel[0].parent.is_none(),
            "pelvis must be root with no parent"
        );
    }

    #[test]
    fn t_pose_all_non_root_have_parent() {
        let skel = default_t_pose_skeleton();
        for (i, j) in skel.iter().enumerate().skip(1) {
            assert!(
                j.parent.is_some(),
                "joint {} ({}) should have a parent",
                i,
                j.name
            );
        }
    }

    #[test]
    fn t_pose_parent_indices_in_range() {
        let skel = default_t_pose_skeleton();
        let n = skel.len();
        for j in &skel {
            if let Some(p) = j.parent {
                assert!(p < n, "parent index {} out of range", p);
            }
        }
    }

    #[test]
    fn build_skeleton_json_contains_joints_key() {
        let skel = default_t_pose_skeleton();
        let json = build_skeleton_json(&skel);
        assert!(
            json.contains("\"joints\""),
            "skeleton JSON must contain 'joints'"
        );
    }

    #[test]
    fn build_skeleton_json_contains_armature() {
        let skel = default_t_pose_skeleton();
        let json = build_skeleton_json(&skel);
        assert!(json.contains("Armature"));
    }

    #[test]
    fn build_animated_glb_json_contains_animations() {
        let mesh = stub_mesh();
        let skel = default_t_pose_skeleton();
        let idle = generate_idle_animation(&skel, 24.0, 2.0);
        let opts = AnimatedGlbOptions {
            include_skeleton: true,
            include_morph_weights: false,
            fps: 24.0,
            duration: 2.0,
            morph_target_names: vec![],
        };
        let json = build_animated_glb_json(&mesh, &skel, &idle, &[], &[], &opts);
        assert!(
            json.contains("\"animations\""),
            "JSON must contain animations"
        );
    }

    #[test]
    fn build_animated_glb_json_contains_skins_when_skeleton_enabled() {
        let mesh = stub_mesh();
        let skel = default_t_pose_skeleton();
        let opts = AnimatedGlbOptions {
            include_skeleton: true,
            include_morph_weights: false,
            fps: 24.0,
            duration: 1.0,
            morph_target_names: vec![],
        };
        let json = build_animated_glb_json(&mesh, &skel, &[], &[], &[], &opts);
        assert!(json.contains("\"skins\""));
    }

    #[test]
    fn build_animated_glb_json_no_skins_when_skeleton_disabled() {
        let mesh = stub_mesh();
        let skel = default_t_pose_skeleton();
        let opts = AnimatedGlbOptions {
            include_skeleton: false,
            include_morph_weights: false,
            fps: 24.0,
            duration: 1.0,
            morph_target_names: vec![],
        };
        let json = build_animated_glb_json(&mesh, &skel, &[], &[], &[], &opts);
        assert!(!json.contains("\"skins\""));
    }

    #[test]
    fn generate_idle_animation_has_spine_keyframes() {
        let skel = default_t_pose_skeleton();
        let anims = generate_idle_animation(&skel, 24.0, 2.0);
        assert!(!anims.is_empty(), "idle animation must have keyframe sets");
        // All should have rotations (breathing)
        for anim in &anims {
            assert!(
                anim.rotations.is_some(),
                "spine joints must have rotation keyframes"
            );
        }
    }

    #[test]
    fn generate_idle_animation_keyframe_count() {
        let skel = default_t_pose_skeleton();
        let fps = 30.0f32;
        let duration = 2.0f32;
        let anims = generate_idle_animation(&skel, fps, duration);
        let expected_frames = (fps * duration) as usize;
        for anim in &anims {
            assert_eq!(anim.times.len(), expected_frames);
        }
    }

    #[test]
    fn build_morph_anim_samplers_json_contains_structure() {
        let times = vec![0.0f32, 0.5, 1.0];
        let frames = vec![vec![0.0f32, 0.1], vec![0.5, 0.2], vec![1.0, 0.0]];
        let json = build_morph_anim_samplers_json(&times, &frames, 100);
        assert!(json.contains("\"input\""));
        assert!(json.contains("\"output\""));
        assert!(json.contains("\"interpolation\""));
        assert!(json.contains("frameCount"));
        assert!(json.contains("morphCount"));
    }

    #[test]
    fn animated_glb_stats_non_empty() {
        let result = AnimatedGlbResult {
            json_size: 1024,
            bin_size: 4096,
            joint_count: 17,
            morph_target_count: 5,
            keyframe_count: 120,
        };
        let s = animated_glb_stats(&result);
        assert!(!s.is_empty());
        assert!(s.contains("17"));
        assert!(s.contains("120"));
    }

    #[test]
    fn animated_glb_stats_contains_all_fields() {
        let result = AnimatedGlbResult {
            json_size: 500,
            bin_size: 2000,
            joint_count: 17,
            morph_target_count: 3,
            keyframe_count: 48,
        };
        let s = animated_glb_stats(&result);
        assert!(s.contains("500"));
        assert!(s.contains("2000"));
        assert!(s.contains("3"));
        assert!(s.contains("48"));
    }

    #[test]
    fn keyframe_count_from_joint_keyframes() {
        let kf = JointKeyframes {
            joint_idx: 2,
            times: vec![0.0, 0.5, 1.0],
            translations: None,
            rotations: Some(vec![[0.0, 0.0, 0.0, 1.0]; 3]),
            scales: None,
        };
        assert_eq!(kf.times.len(), 3);
        assert_eq!(kf.rotations.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn options_morph_weights_flag() {
        let mesh = stub_mesh();
        let skel = default_t_pose_skeleton();
        let times = vec![0.0f32, 1.0];
        let frames = vec![vec![0.0f32], vec![1.0f32]];
        let opts = AnimatedGlbOptions {
            include_skeleton: false,
            include_morph_weights: true,
            fps: 24.0,
            duration: 1.0,
            morph_target_names: vec!["blink".to_string()],
        };
        let json = build_animated_glb_json(&mesh, &skel, &[], &times, &frames, &opts);
        assert!(json.contains("MorphAnimation"));
    }

    #[test]
    fn build_joint_anim_samplers_json_with_rotations() {
        let anims = vec![JointKeyframes {
            joint_idx: 1,
            times: vec![0.0, 1.0],
            translations: None,
            rotations: Some(vec![[0.0, 0.0, 0.0, 1.0]; 2]),
            scales: None,
        }];
        let json = build_joint_anim_samplers_json(&anims, 10);
        assert!(json.contains("rotation"));
        assert!(json.contains("\"joint\": 1"));
    }
}
