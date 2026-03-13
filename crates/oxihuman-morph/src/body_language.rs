// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use std::collections::HashMap;

/// Pose feature vector extracted from a body pose.
#[derive(Debug, Clone)]
pub struct PoseFeatures {
    /// Forward lean angle in degrees (+ve = leaning forward).
    pub spine_lean: f32,
    /// Shoulder elevation: 0 = relaxed, 1 = maximally raised.
    pub shoulder_elevation: f32,
    /// Arm openness: 0 = arms crossed, 1 = wide open.
    pub arm_openness: f32,
    /// Head tilt in degrees (+ve = right tilt).
    pub head_tilt: f32,
    /// Head nod: -1 = looking down, +1 = looking up.
    pub head_nod: f32,
    /// Lateral hip sway (normalised).
    pub hip_sway: f32,
    /// Leg spread: 0 = together, 1 = wide.
    pub leg_spread: f32,
    /// Gesture height: 0 = low, 1 = high.
    pub gesture_height: f32,
}

/// High-level body emotion category.
#[derive(Debug, Clone, PartialEq)]
pub enum BodyEmotion {
    Neutral,
    Confident,
    Submissive,
    Aggressive,
    Joyful,
    Sad,
    Fearful,
    Curious,
    Relaxed,
    Tense,
}

/// Classification result.
#[derive(Debug, Clone)]
pub struct BodyLanguageProfile {
    pub emotion: BodyEmotion,
    /// How strongly the features match [0, 1].
    pub confidence: f32,
    pub features: PoseFeatures,
}

// ─── Reference poses for each emotion ────────────────────────────────────────

fn reference_pose(emotion: &BodyEmotion) -> PoseFeatures {
    match emotion {
        BodyEmotion::Neutral => PoseFeatures {
            spine_lean: 0.0,
            shoulder_elevation: 0.2,
            arm_openness: 0.5,
            head_tilt: 0.0,
            head_nod: 0.0,
            hip_sway: 0.0,
            leg_spread: 0.3,
            gesture_height: 0.3,
        },
        BodyEmotion::Confident => PoseFeatures {
            spine_lean: -5.0,
            shoulder_elevation: 0.3,
            arm_openness: 0.7,
            head_tilt: 0.0,
            head_nod: 0.2,
            hip_sway: 0.1,
            leg_spread: 0.6,
            gesture_height: 0.5,
        },
        BodyEmotion::Submissive => PoseFeatures {
            spine_lean: 10.0,
            shoulder_elevation: 0.0,
            arm_openness: 0.2,
            head_tilt: 5.0,
            head_nod: -0.3,
            hip_sway: 0.0,
            leg_spread: 0.1,
            gesture_height: 0.1,
        },
        BodyEmotion::Aggressive => PoseFeatures {
            spine_lean: -8.0,
            shoulder_elevation: 0.8,
            arm_openness: 0.3,
            head_tilt: 0.0,
            head_nod: 0.1,
            hip_sway: 0.0,
            leg_spread: 0.7,
            gesture_height: 0.6,
        },
        BodyEmotion::Joyful => PoseFeatures {
            spine_lean: 0.0,
            shoulder_elevation: 0.4,
            arm_openness: 0.9,
            head_tilt: 8.0,
            head_nod: 0.2,
            hip_sway: 0.3,
            leg_spread: 0.5,
            gesture_height: 0.8,
        },
        BodyEmotion::Sad => PoseFeatures {
            spine_lean: 15.0,
            shoulder_elevation: 0.0,
            arm_openness: 0.1,
            head_tilt: -3.0,
            head_nod: -0.5,
            hip_sway: 0.0,
            leg_spread: 0.2,
            gesture_height: 0.1,
        },
        BodyEmotion::Fearful => PoseFeatures {
            spine_lean: 5.0,
            shoulder_elevation: 0.9,
            arm_openness: 0.2,
            head_tilt: -5.0,
            head_nod: -0.2,
            hip_sway: 0.0,
            leg_spread: 0.2,
            gesture_height: 0.4,
        },
        BodyEmotion::Curious => PoseFeatures {
            spine_lean: -3.0,
            shoulder_elevation: 0.3,
            arm_openness: 0.6,
            head_tilt: 12.0,
            head_nod: 0.1,
            hip_sway: 0.1,
            leg_spread: 0.4,
            gesture_height: 0.5,
        },
        BodyEmotion::Relaxed => PoseFeatures {
            spine_lean: 8.0,
            shoulder_elevation: 0.1,
            arm_openness: 0.6,
            head_tilt: 0.0,
            head_nod: 0.0,
            hip_sway: 0.2,
            leg_spread: 0.5,
            gesture_height: 0.2,
        },
        BodyEmotion::Tense => PoseFeatures {
            spine_lean: 0.0,
            shoulder_elevation: 0.7,
            arm_openness: 0.1,
            head_tilt: 0.0,
            head_nod: 0.0,
            hip_sway: 0.0,
            leg_spread: 0.3,
            gesture_height: 0.3,
        },
    }
}

// ─── Feature vector helpers ───────────────────────────────────────────────────

fn pose_to_vec(f: &PoseFeatures) -> [f32; 8] {
    [
        f.spine_lean / 90.0,
        f.shoulder_elevation,
        f.arm_openness,
        f.head_tilt / 45.0,
        f.head_nod,
        f.hip_sway,
        f.leg_spread,
        f.gesture_height,
    ]
}

fn dot8(a: &[f32; 8], b: &[f32; 8]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn norm8(a: &[f32; 8]) -> f32 {
    dot8(a, a).sqrt()
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Rule-based classifier: finds the reference emotion with highest cosine similarity.
pub fn classify_body_language(features: &PoseFeatures) -> BodyLanguageProfile {
    let all_emotions = [
        BodyEmotion::Neutral,
        BodyEmotion::Confident,
        BodyEmotion::Submissive,
        BodyEmotion::Aggressive,
        BodyEmotion::Joyful,
        BodyEmotion::Sad,
        BodyEmotion::Fearful,
        BodyEmotion::Curious,
        BodyEmotion::Relaxed,
        BodyEmotion::Tense,
    ];

    let query = pose_to_vec(features);
    let qn = norm8(&query);

    let mut best_emotion = BodyEmotion::Neutral;
    let mut best_sim: f32 = -2.0;

    for emotion in &all_emotions {
        let ref_vec = pose_to_vec(&reference_pose(emotion));
        let rn = norm8(&ref_vec);
        let sim = if qn > 1e-6 && rn > 1e-6 {
            dot8(&query, &ref_vec) / (qn * rn)
        } else {
            0.0
        };
        if sim > best_sim {
            best_sim = sim;
            best_emotion = emotion.clone();
        }
    }

    BodyLanguageProfile {
        emotion: best_emotion,
        confidence: ((best_sim + 1.0) / 2.0).clamp(0.0, 1.0),
        features: features.clone(),
    }
}

/// Inverse mapping: generate pose features for a given emotion at given intensity.
pub fn generate_pose_for_emotion(emotion: &BodyEmotion, intensity: f32) -> PoseFeatures {
    let neutral = reference_pose(&BodyEmotion::Neutral);
    let target = reference_pose(emotion);
    let t = intensity.clamp(0.0, 1.0);
    interpolate_pose_features(&neutral, &target, t)
}

/// Linear interpolation between two pose feature sets.
pub fn interpolate_pose_features(a: &PoseFeatures, b: &PoseFeatures, t: f32) -> PoseFeatures {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    PoseFeatures {
        spine_lean: lerp(a.spine_lean, b.spine_lean),
        shoulder_elevation: lerp(a.shoulder_elevation, b.shoulder_elevation),
        arm_openness: lerp(a.arm_openness, b.arm_openness),
        head_tilt: lerp(a.head_tilt, b.head_tilt),
        head_nod: lerp(a.head_nod, b.head_nod),
        hip_sway: lerp(a.hip_sway, b.hip_sway),
        leg_spread: lerp(a.leg_spread, b.leg_spread),
        gesture_height: lerp(a.gesture_height, b.gesture_height),
    }
}

/// Cosine-like similarity in [0, 1] between two pose feature vectors.
pub fn pose_similarity(a: &PoseFeatures, b: &PoseFeatures) -> f32 {
    let va = pose_to_vec(a);
    let vb = pose_to_vec(b);
    let na = norm8(&va);
    let nb = norm8(&vb);
    if na < 1e-6 || nb < 1e-6 {
        return 0.0;
    }
    ((dot8(&va, &vb) / (na * nb) + 1.0) / 2.0).clamp(0.0, 1.0)
}

/// Mirror pose left-right (negates head_tilt and hip_sway).
pub fn mirror_pose(features: &PoseFeatures) -> PoseFeatures {
    PoseFeatures {
        spine_lean: features.spine_lean,
        shoulder_elevation: features.shoulder_elevation,
        arm_openness: features.arm_openness,
        head_tilt: -features.head_tilt,
        head_nod: features.head_nod,
        hip_sway: -features.hip_sway,
        leg_spread: features.leg_spread,
        gesture_height: features.gesture_height,
    }
}

/// Weighted blend of multiple emotion poses.
pub fn blend_body_emotions(emotions: &[(BodyEmotion, f32)]) -> PoseFeatures {
    let mut total_weight = 0.0_f32;
    let mut acc = PoseFeatures {
        spine_lean: 0.0,
        shoulder_elevation: 0.0,
        arm_openness: 0.0,
        head_tilt: 0.0,
        head_nod: 0.0,
        hip_sway: 0.0,
        leg_spread: 0.0,
        gesture_height: 0.0,
    };

    for (emotion, w) in emotions {
        let pose = reference_pose(emotion);
        let w = w.max(0.0);
        acc.spine_lean += pose.spine_lean * w;
        acc.shoulder_elevation += pose.shoulder_elevation * w;
        acc.arm_openness += pose.arm_openness * w;
        acc.head_tilt += pose.head_tilt * w;
        acc.head_nod += pose.head_nod * w;
        acc.hip_sway += pose.hip_sway * w;
        acc.leg_spread += pose.leg_spread * w;
        acc.gesture_height += pose.gesture_height * w;
        total_weight += w;
    }

    if total_weight > 1e-6 {
        let inv = 1.0 / total_weight;
        acc.spine_lean *= inv;
        acc.shoulder_elevation *= inv;
        acc.arm_openness *= inv;
        acc.head_tilt *= inv;
        acc.head_nod *= inv;
        acc.hip_sway *= inv;
        acc.leg_spread *= inv;
        acc.gesture_height *= inv;
    }

    acc
}

pub fn pose_to_json(features: &PoseFeatures) -> String {
    format!(
        "{{\"spine_lean\":{:.4},\"shoulder_elevation\":{:.4},\"arm_openness\":{:.4},\
         \"head_tilt\":{:.4},\"head_nod\":{:.4},\"hip_sway\":{:.4},\
         \"leg_spread\":{:.4},\"gesture_height\":{:.4}}}",
        features.spine_lean,
        features.shoulder_elevation,
        features.arm_openness,
        features.head_tilt,
        features.head_nod,
        features.hip_sway,
        features.leg_spread,
        features.gesture_height,
    )
}

/// Return the profile with the highest confidence, if any.
pub fn dominant_emotion(profiles: &[BodyLanguageProfile]) -> Option<&BodyLanguageProfile> {
    profiles.iter().max_by(|a, b| {
        a.confidence
            .partial_cmp(&b.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

/// Map an emotion at a given intensity to morph parameter values.
pub fn apply_emotion_to_params(emotion: &BodyEmotion, intensity: f32) -> HashMap<String, f32> {
    let pose = generate_pose_for_emotion(emotion, intensity);
    let mut map = HashMap::new();
    map.insert(
        "spine_lean".to_string(),
        (pose.spine_lean / 90.0).clamp(-1.0, 1.0),
    );
    map.insert(
        "shoulder_elevation".to_string(),
        pose.shoulder_elevation.clamp(0.0, 1.0),
    );
    map.insert(
        "arm_openness".to_string(),
        pose.arm_openness.clamp(0.0, 1.0),
    );
    map.insert(
        "head_tilt".to_string(),
        (pose.head_tilt / 45.0).clamp(-1.0, 1.0),
    );
    map.insert("head_nod".to_string(), pose.head_nod.clamp(-1.0, 1.0));
    map.insert("hip_sway".to_string(), pose.hip_sway.clamp(-1.0, 1.0));
    map.insert("leg_spread".to_string(), pose.leg_spread.clamp(0.0, 1.0));
    map.insert(
        "gesture_height".to_string(),
        pose.gesture_height.clamp(0.0, 1.0),
    );
    map
}

/// Clamp all PoseFeatures fields to their valid ranges.
pub fn normalize_pose_features(features: &mut PoseFeatures) {
    features.spine_lean = features.spine_lean.clamp(-90.0, 90.0);
    features.shoulder_elevation = features.shoulder_elevation.clamp(0.0, 1.0);
    features.arm_openness = features.arm_openness.clamp(0.0, 1.0);
    features.head_tilt = features.head_tilt.clamp(-45.0, 45.0);
    features.head_nod = features.head_nod.clamp(-1.0, 1.0);
    features.hip_sway = features.hip_sway.clamp(-1.0, 1.0);
    features.leg_spread = features.leg_spread.clamp(0.0, 1.0);
    features.gesture_height = features.gesture_height.clamp(0.0, 1.0);
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn neutral_pose() -> PoseFeatures {
        reference_pose(&BodyEmotion::Neutral)
    }

    #[test]
    fn test_classify_neutral() {
        let p = reference_pose(&BodyEmotion::Neutral);
        let profile = classify_body_language(&p);
        assert_eq!(profile.emotion, BodyEmotion::Neutral);
        assert!(profile.confidence > 0.5);
    }

    #[test]
    fn test_classify_confident() {
        let p = reference_pose(&BodyEmotion::Confident);
        let profile = classify_body_language(&p);
        assert_eq!(profile.emotion, BodyEmotion::Confident);
    }

    #[test]
    fn test_generate_pose_zero_intensity() {
        let p = generate_pose_for_emotion(&BodyEmotion::Sad, 0.0);
        let neutral = neutral_pose();
        assert!((p.spine_lean - neutral.spine_lean).abs() < 1e-4);
    }

    #[test]
    fn test_generate_pose_full_intensity() {
        let p = generate_pose_for_emotion(&BodyEmotion::Sad, 1.0);
        let sad_ref = reference_pose(&BodyEmotion::Sad);
        assert!((p.spine_lean - sad_ref.spine_lean).abs() < 1e-4);
    }

    #[test]
    fn test_interpolate_pose_midpoint() {
        let a = PoseFeatures {
            spine_lean: 0.0,
            shoulder_elevation: 0.0,
            arm_openness: 0.0,
            head_tilt: 0.0,
            head_nod: 0.0,
            hip_sway: 0.0,
            leg_spread: 0.0,
            gesture_height: 0.0,
        };
        let b = PoseFeatures {
            spine_lean: 10.0,
            shoulder_elevation: 1.0,
            arm_openness: 1.0,
            head_tilt: 0.0,
            head_nod: 0.0,
            hip_sway: 0.0,
            leg_spread: 0.0,
            gesture_height: 0.0,
        };
        let mid = interpolate_pose_features(&a, &b, 0.5);
        assert!((mid.spine_lean - 5.0).abs() < 1e-4);
        assert!((mid.arm_openness - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_pose_similarity_identical() {
        let p = neutral_pose();
        let sim = pose_similarity(&p, &p);
        assert!(sim > 0.99);
    }

    #[test]
    fn test_mirror_pose_negates_tilt() {
        let p = PoseFeatures {
            head_tilt: 10.0,
            hip_sway: 0.3,
            spine_lean: 0.0,
            shoulder_elevation: 0.0,
            arm_openness: 0.0,
            head_nod: 0.0,
            leg_spread: 0.0,
            gesture_height: 0.0,
        };
        let m = mirror_pose(&p);
        assert!((m.head_tilt + 10.0).abs() < 1e-5);
        assert!((m.hip_sway + 0.3).abs() < 1e-5);
        assert!((m.spine_lean - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_blend_body_emotions_single() {
        let result = blend_body_emotions(&[(BodyEmotion::Joyful, 1.0)]);
        let joy = reference_pose(&BodyEmotion::Joyful);
        assert!((result.arm_openness - joy.arm_openness).abs() < 1e-4);
    }

    #[test]
    fn test_blend_body_emotions_equal_weights() {
        // two equal-weight emotions - result should be mid of reference poses
        let poses = [(BodyEmotion::Neutral, 1.0), (BodyEmotion::Confident, 1.0)];
        let result = blend_body_emotions(&poses);
        let n = reference_pose(&BodyEmotion::Neutral);
        let c = reference_pose(&BodyEmotion::Confident);
        let expected_arm = (n.arm_openness + c.arm_openness) / 2.0;
        assert!((result.arm_openness - expected_arm).abs() < 1e-4);
    }

    #[test]
    fn test_pose_to_json() {
        let p = neutral_pose();
        let j = pose_to_json(&p);
        assert!(j.contains("spine_lean"));
        assert!(j.contains("arm_openness"));
    }

    #[test]
    fn test_dominant_emotion_empty() {
        let profiles: Vec<BodyLanguageProfile> = vec![];
        assert!(dominant_emotion(&profiles).is_none());
    }

    #[test]
    fn test_dominant_emotion_picks_highest_confidence() {
        let profiles = vec![
            BodyLanguageProfile {
                emotion: BodyEmotion::Sad,
                confidence: 0.3,
                features: neutral_pose(),
            },
            BodyLanguageProfile {
                emotion: BodyEmotion::Joyful,
                confidence: 0.9,
                features: neutral_pose(),
            },
        ];
        let dom = dominant_emotion(&profiles).expect("should succeed");
        assert_eq!(dom.emotion, BodyEmotion::Joyful);
    }

    #[test]
    fn test_apply_emotion_to_params_keys() {
        let params = apply_emotion_to_params(&BodyEmotion::Confident, 1.0);
        assert!(params.contains_key("spine_lean"));
        assert!(params.contains_key("arm_openness"));
    }

    #[test]
    fn test_normalize_pose_features_clamps() {
        let mut p = PoseFeatures {
            spine_lean: 200.0,
            shoulder_elevation: 2.0,
            arm_openness: -1.0,
            head_tilt: 100.0,
            head_nod: 5.0,
            hip_sway: -5.0,
            leg_spread: 3.0,
            gesture_height: -1.0,
        };
        normalize_pose_features(&mut p);
        assert!(p.spine_lean <= 90.0);
        assert!(p.shoulder_elevation <= 1.0);
        assert!(p.arm_openness >= 0.0);
        assert!(p.head_tilt <= 45.0);
        assert!(p.head_nod >= -1.0 && p.head_nod <= 1.0);
    }
}
