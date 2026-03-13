// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Pose sequence animation export for skeletal rigs.

// ── Structs ───────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoseSeqConfig {
    pub frame_rate: f32,
    pub interpolate: bool,
    pub bake_to_frames: bool,
}

/// A single pose keyframe with joint quaternions.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoseKeyframe {
    pub time: f32,
    /// List of (joint_name, quaternion [x,y,z,w]).
    pub joint_rotations: Vec<(String, [f32; 4])>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoseSequence {
    pub name: String,
    pub keyframes: Vec<PoseKeyframe>,
    pub duration: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoseSeqExportResult {
    pub sequences: Vec<String>,
    pub total_keyframes: usize,
    pub duration_sec: f32,
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_pose_seq_config() -> PoseSeqConfig {
    PoseSeqConfig {
        frame_rate: 30.0,
        interpolate: true,
        bake_to_frames: false,
    }
}

#[allow(dead_code)]
pub fn new_pose_sequence(name: &str, duration: f32) -> PoseSequence {
    PoseSequence {
        name: name.to_string(),
        keyframes: Vec::new(),
        duration,
    }
}

#[allow(dead_code)]
pub fn add_pose_keyframe(seq: &mut PoseSequence, kf: PoseKeyframe) {
    seq.keyframes.push(kf);
}

#[allow(dead_code)]
pub fn new_pose_keyframe(time: f32) -> PoseKeyframe {
    PoseKeyframe {
        time,
        joint_rotations: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_joint_rotation(kf: &mut PoseKeyframe, joint: &str, quat: [f32; 4]) {
    kf.joint_rotations.push((joint.to_string(), quat));
}

#[allow(dead_code)]
pub fn export_pose_sequence(seq: &PoseSequence, cfg: &PoseSeqConfig) -> String {
    let frame_rate = cfg.frame_rate;
    let kf_strings: Vec<String> = seq
        .keyframes
        .iter()
        .map(|kf| {
            let rot_strings: Vec<String> = kf
                .joint_rotations
                .iter()
                .map(|(name, q)| {
                    format!(
                        r#"{{"joint":"{}","quat":[{},{},{},{}]}}"#,
                        name, q[0], q[1], q[2], q[3]
                    )
                })
                .collect();
            format!(
                r#"{{"time":{},"rotations":[{}]}}"#,
                kf.time,
                rot_strings.join(",")
            )
        })
        .collect();
    format!(
        r#"{{"name":"{}","duration":{},"frame_rate":{},"interpolate":{},"keyframes":[{}]}}"#,
        seq.name,
        seq.duration,
        frame_rate,
        cfg.interpolate,
        kf_strings.join(",")
    )
}

#[allow(dead_code)]
pub fn export_pose_sequences(
    seqs: &[PoseSequence],
    _cfg: &PoseSeqConfig,
) -> PoseSeqExportResult {
    let mut total_keyframes = 0usize;
    let mut max_duration = 0.0f32;
    let mut names = Vec::new();
    for seq in seqs {
        names.push(seq.name.clone());
        total_keyframes += seq.keyframes.len();
        if seq.duration > max_duration {
            max_duration = seq.duration;
        }
    }
    PoseSeqExportResult {
        sequences: names,
        total_keyframes,
        duration_sec: max_duration,
    }
}

#[allow(dead_code)]
pub fn pose_sequence_frame_count(seq: &PoseSequence) -> usize {
    seq.keyframes.len()
}

#[allow(dead_code)]
pub fn pose_sequence_joint_count(seq: &PoseSequence) -> usize {
    seq.keyframes
        .iter()
        .map(|kf| kf.joint_rotations.len())
        .max()
        .unwrap_or(0)
}

#[allow(dead_code)]
pub fn pose_seq_result_to_json(r: &PoseSeqExportResult) -> String {
    let names: Vec<String> = r
        .sequences
        .iter()
        .map(|s| format!(r#""{}""#, s))
        .collect();
    format!(
        r#"{{"sequences":[{}],"total_keyframes":{},"duration_sec":{}}}"#,
        names.join(","),
        r.total_keyframes,
        r.duration_sec
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_30fps() {
        let cfg = default_pose_seq_config();
        assert!((cfg.frame_rate - 30.0).abs() < 1e-6);
        assert!(cfg.interpolate);
        assert!(!cfg.bake_to_frames);
    }

    #[test]
    fn new_pose_sequence_is_empty() {
        let seq = new_pose_sequence("walk", 2.0);
        assert_eq!(seq.name, "walk");
        assert!((seq.duration - 2.0).abs() < 1e-6);
        assert!(seq.keyframes.is_empty());
    }

    #[test]
    fn add_pose_keyframe_appends() {
        let mut seq = new_pose_sequence("run", 1.0);
        let kf = new_pose_keyframe(0.5);
        add_pose_keyframe(&mut seq, kf);
        assert_eq!(seq.keyframes.len(), 1);
        assert!((seq.keyframes[0].time - 0.5).abs() < 1e-6);
    }

    #[test]
    fn add_joint_rotation_appends() {
        let mut kf = new_pose_keyframe(0.0);
        add_joint_rotation(&mut kf, "spine", [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(kf.joint_rotations.len(), 1);
        assert_eq!(kf.joint_rotations[0].0, "spine");
    }

    #[test]
    fn pose_sequence_frame_count_correct() {
        let mut seq = new_pose_sequence("idle", 3.0);
        for i in 0..5 {
            add_pose_keyframe(&mut seq, new_pose_keyframe(i as f32 * 0.5));
        }
        assert_eq!(pose_sequence_frame_count(&seq), 5);
    }

    #[test]
    fn pose_sequence_joint_count_correct() {
        let mut seq = new_pose_sequence("wave", 1.0);
        let mut kf = new_pose_keyframe(0.0);
        add_joint_rotation(&mut kf, "shoulder", [0.0, 0.0, 0.0, 1.0]);
        add_joint_rotation(&mut kf, "elbow", [0.0, 0.1, 0.0, 1.0]);
        add_pose_keyframe(&mut seq, kf);
        assert_eq!(pose_sequence_joint_count(&seq), 2);
    }

    #[test]
    fn export_pose_sequence_contains_name() {
        let cfg = default_pose_seq_config();
        let seq = new_pose_sequence("sprint", 1.5);
        let out = export_pose_sequence(&seq, &cfg);
        assert!(out.contains("sprint"));
    }

    #[test]
    fn export_pose_sequences_total_keyframes() {
        let cfg = default_pose_seq_config();
        let mut seq1 = new_pose_sequence("a", 1.0);
        add_pose_keyframe(&mut seq1, new_pose_keyframe(0.0));
        add_pose_keyframe(&mut seq1, new_pose_keyframe(0.5));
        let mut seq2 = new_pose_sequence("b", 2.0);
        add_pose_keyframe(&mut seq2, new_pose_keyframe(0.0));
        let result = export_pose_sequences(&[seq1, seq2], &cfg);
        assert_eq!(result.total_keyframes, 3);
        assert!((result.duration_sec - 2.0).abs() < 1e-6);
    }

    #[test]
    fn pose_seq_result_to_json_valid() {
        let r = PoseSeqExportResult {
            sequences: vec!["walk".to_string(), "run".to_string()],
            total_keyframes: 10,
            duration_sec: 3.0,
        };
        let json = pose_seq_result_to_json(&r);
        assert!(json.contains("walk"));
        assert!(json.contains("total_keyframes"));
        assert!(json.contains("10"));
    }
}
