// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Emotional prosody analysis and generation for speech parameters.
//! Maps pitch, rate, emphasis and emotion to morph parameters.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProsodyFeatures {
    pub pitch_hz: f32,
    pub pitch_range: f32,
    pub speech_rate: f32,
    pub loudness: f32,
    pub energy: f32,
    pub jitter: f32,
    pub shimmer: f32,
    pub pause_ratio: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ProsodyEmotion {
    Neutral,
    Happy,
    Sad,
    Angry,
    Fearful,
    Disgusted,
    Surprised,
    Calm,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProsodyProfile {
    pub emotion: ProsodyEmotion,
    pub intensity: f32,
    pub features: ProsodyFeatures,
}

/// Rule-based classifier: maps prosody features to an emotion profile.
#[allow(dead_code)]
pub fn classify_prosody_emotion(features: &ProsodyFeatures) -> ProsodyProfile {
    // Simple rule-based scoring for each emotion
    let mut scores = [
        (ProsodyEmotion::Neutral, 0.0_f32),
        (ProsodyEmotion::Happy, 0.0_f32),
        (ProsodyEmotion::Sad, 0.0_f32),
        (ProsodyEmotion::Angry, 0.0_f32),
        (ProsodyEmotion::Fearful, 0.0_f32),
        (ProsodyEmotion::Disgusted, 0.0_f32),
        (ProsodyEmotion::Surprised, 0.0_f32),
        (ProsodyEmotion::Calm, 0.0_f32),
    ];

    // Happy: high pitch, very high rate (>5), high loudness, low jitter, low pause
    scores[1].1 += if features.pitch_hz > 200.0 { 1.0 } else { 0.0 };
    scores[1].1 += if features.speech_rate > 5.0 {
        1.5
    } else if features.speech_rate > 4.0 {
        0.5
    } else {
        0.0
    };
    scores[1].1 += if features.loudness > 0.6 { 1.0 } else { 0.0 };
    scores[1].1 += if features.pause_ratio < 0.15 {
        0.5
    } else {
        0.0
    };
    scores[1].1 += if features.jitter < 0.03 { 0.5 } else { 0.0 }; // happy has low irregularity

    // Sad: low pitch, low rate, low loudness, high pause_ratio, high jitter
    scores[2].1 += if features.pitch_hz < 150.0 { 1.0 } else { 0.0 };
    scores[2].1 += if features.speech_rate < 2.5 { 1.0 } else { 0.0 };
    scores[2].1 += if features.loudness < 0.4 { 1.0 } else { 0.0 };
    scores[2].1 += if features.pause_ratio > 0.4 { 1.0 } else { 0.0 };
    scores[2].1 += if features.jitter > 0.05 { 0.5 } else { 0.0 };

    // Angry: high pitch_range, high energy, very low pause_ratio (<0.1), high loudness, moderate speech rate
    scores[3].1 += if features.pitch_range > 80.0 {
        1.0
    } else {
        0.0
    };
    scores[3].1 += if features.energy > 0.7 { 1.0 } else { 0.0 };
    scores[3].1 += if features.loudness > 0.7 { 1.0 } else { 0.0 };
    scores[3].1 += if features.pause_ratio < 0.1 { 1.0 } else { 0.0 }; // angry pauses less
    scores[3].1 += if features.speech_rate < 5.2 && features.speech_rate > 3.5 {
        0.5
    } else {
        0.0
    };

    // Fearful: high jitter, high shimmer, moderate pitch, high rate
    scores[4].1 += if features.jitter > 0.06 { 1.0 } else { 0.0 };
    scores[4].1 += if features.shimmer > 0.06 { 1.0 } else { 0.0 };
    scores[4].1 += if features.speech_rate > 4.5 { 0.5 } else { 0.0 };
    scores[4].1 += if features.pause_ratio > 0.3 { 0.5 } else { 0.0 };

    // Disgusted: low pitch, low rate, moderate jitter
    scores[5].1 += if features.pitch_hz < 160.0 { 0.5 } else { 0.0 };
    scores[5].1 += if features.speech_rate < 3.0 { 0.5 } else { 0.0 };
    scores[5].1 += if features.jitter > 0.04 { 0.5 } else { 0.0 };
    scores[5].1 += if features.shimmer > 0.04 { 0.5 } else { 0.0 };

    // Surprised: high pitch, wide pitch_range, high rate
    scores[6].1 += if features.pitch_hz > 220.0 { 1.0 } else { 0.0 };
    scores[6].1 += if features.pitch_range > 100.0 {
        1.0
    } else {
        0.0
    };
    scores[6].1 += if features.speech_rate > 5.0 { 0.5 } else { 0.0 };

    // Calm: low jitter, low shimmer, low energy, moderate pause ratio, NOT too fast
    scores[7].1 += if features.jitter < 0.02 { 1.0 } else { 0.0 };
    scores[7].1 += if features.shimmer < 0.02 { 1.0 } else { 0.0 };
    scores[7].1 += if features.energy < 0.45 {
        1.0
    } else if features.energy < 0.55 {
        0.3
    } else {
        0.0
    };
    scores[7].1 += if features.pause_ratio > 0.2 && features.pause_ratio < 0.4 {
        0.5
    } else {
        0.0
    };

    // Neutral: moderate values across the board — boosted by proximity to midpoints
    let pitch_neutral = if (features.pitch_hz - 160.0).abs() < 20.0 {
        1.2
    } else {
        0.0
    };
    let rate_neutral = if (features.speech_rate - 3.5).abs() < 0.5 {
        1.2
    } else {
        0.0
    };
    let loudness_neutral = if (features.loudness - 0.5).abs() < 0.1 {
        0.8
    } else {
        0.0
    };
    let energy_neutral = if (features.energy - 0.5).abs() < 0.1 {
        0.5
    } else {
        0.0
    };
    scores[0].1 = pitch_neutral + rate_neutral + loudness_neutral + energy_neutral;

    let best = scores
        .iter()
        .enumerate()
        .max_by(|a, b| {
            a.1 .1
                .partial_cmp(&b.1 .1)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
        .unwrap_or(0);

    let total: f32 = scores.iter().map(|s| s.1).sum();
    let intensity = if total > 0.0 {
        (scores[best].1 / total).clamp(0.0, 1.0)
    } else {
        0.5
    };

    ProsodyProfile {
        emotion: scores[best].0.clone(),
        intensity,
        features: features.clone(),
    }
}

/// Generate canonical prosody features for a given emotion and intensity.
#[allow(dead_code)]
pub fn generate_prosody_for_emotion(emotion: &ProsodyEmotion, intensity: f32) -> ProsodyFeatures {
    let t = intensity.clamp(0.0, 1.0);
    let lerp = |a: f32, b: f32| a + (b - a) * t;

    match emotion {
        ProsodyEmotion::Neutral => ProsodyFeatures {
            pitch_hz: 160.0,
            pitch_range: 40.0,
            speech_rate: 3.5,
            loudness: 0.5,
            energy: 0.5,
            jitter: 0.01,
            shimmer: 0.01,
            pause_ratio: 0.25,
        },
        ProsodyEmotion::Happy => ProsodyFeatures {
            pitch_hz: lerp(160.0, 230.0),
            pitch_range: lerp(40.0, 110.0),
            speech_rate: lerp(3.5, 5.5),
            loudness: lerp(0.5, 0.85),
            energy: lerp(0.5, 0.8),
            jitter: lerp(0.01, 0.02),
            shimmer: lerp(0.01, 0.02),
            pause_ratio: lerp(0.25, 0.1),
        },
        ProsodyEmotion::Sad => ProsodyFeatures {
            pitch_hz: lerp(160.0, 120.0),
            pitch_range: lerp(40.0, 20.0),
            speech_rate: lerp(3.5, 1.8),
            loudness: lerp(0.5, 0.25),
            energy: lerp(0.5, 0.2),
            jitter: lerp(0.01, 0.08),
            shimmer: lerp(0.01, 0.07),
            pause_ratio: lerp(0.25, 0.55),
        },
        ProsodyEmotion::Angry => ProsodyFeatures {
            pitch_hz: lerp(160.0, 200.0),
            pitch_range: lerp(40.0, 120.0),
            speech_rate: lerp(3.5, 5.0),
            loudness: lerp(0.5, 0.95),
            energy: lerp(0.5, 0.9),
            jitter: lerp(0.01, 0.04),
            shimmer: lerp(0.01, 0.05),
            pause_ratio: lerp(0.25, 0.08),
        },
        ProsodyEmotion::Fearful => ProsodyFeatures {
            pitch_hz: lerp(160.0, 210.0),
            pitch_range: lerp(40.0, 90.0),
            speech_rate: lerp(3.5, 5.5),
            loudness: lerp(0.5, 0.6),
            energy: lerp(0.5, 0.55),
            jitter: lerp(0.01, 0.09),
            shimmer: lerp(0.01, 0.08),
            pause_ratio: lerp(0.25, 0.4),
        },
        ProsodyEmotion::Disgusted => ProsodyFeatures {
            pitch_hz: lerp(160.0, 140.0),
            pitch_range: lerp(40.0, 30.0),
            speech_rate: lerp(3.5, 2.5),
            loudness: lerp(0.5, 0.45),
            energy: lerp(0.5, 0.4),
            jitter: lerp(0.01, 0.06),
            shimmer: lerp(0.01, 0.06),
            pause_ratio: lerp(0.25, 0.35),
        },
        ProsodyEmotion::Surprised => ProsodyFeatures {
            pitch_hz: lerp(160.0, 250.0),
            pitch_range: lerp(40.0, 130.0),
            speech_rate: lerp(3.5, 5.8),
            loudness: lerp(0.5, 0.8),
            energy: lerp(0.5, 0.75),
            jitter: lerp(0.01, 0.03),
            shimmer: lerp(0.01, 0.03),
            pause_ratio: lerp(0.25, 0.12),
        },
        ProsodyEmotion::Calm => ProsodyFeatures {
            pitch_hz: lerp(160.0, 155.0),
            pitch_range: lerp(40.0, 20.0),
            speech_rate: lerp(3.5, 2.8),
            loudness: lerp(0.5, 0.35),
            energy: lerp(0.5, 0.3),
            jitter: lerp(0.01, 0.005),
            shimmer: lerp(0.01, 0.005),
            pause_ratio: lerp(0.25, 0.35),
        },
    }
}

/// Map prosody features to jaw/brow/lip morph parameters.
#[allow(dead_code)]
pub fn prosody_to_face_params(
    features: &ProsodyFeatures,
) -> std::collections::HashMap<String, f32> {
    let mut map = std::collections::HashMap::new();

    // Jaw open: driven by loudness and energy
    let jaw_open = (features.loudness * 0.6 + features.energy * 0.4).clamp(0.0, 1.0);
    map.insert("jaw_open".to_string(), jaw_open);

    // Lip corners up: high pitch and happy-like features
    let lip_corner_up = ((features.pitch_hz - 100.0) / 200.0).clamp(0.0, 1.0);
    map.insert("lip_corner_up".to_string(), lip_corner_up);

    // Brow raise: high pitch range and surprised features
    let brow_raise = (features.pitch_range / 150.0).clamp(0.0, 1.0);
    map.insert("brow_raise".to_string(), brow_raise);

    // Brow furrow: high jitter (distress) and low pause ratio
    let brow_furrow = (features.jitter * 5.0 + (1.0 - features.pause_ratio) * 0.2).clamp(0.0, 1.0);
    map.insert("brow_furrow".to_string(), brow_furrow);

    // Lip press: high energy maps to tighter lips
    let lip_press = (features.energy * 0.5).clamp(0.0, 1.0);
    map.insert("lip_press".to_string(), lip_press);

    // Lip stretch (wide mouth): high speech rate
    let lip_stretch = ((features.speech_rate - 2.0) / 5.0).clamp(0.0, 1.0);
    map.insert("lip_stretch".to_string(), lip_stretch);

    // Cheek raise: loudness
    let cheek_raise = (features.loudness * 0.7).clamp(0.0, 1.0);
    map.insert("cheek_raise".to_string(), cheek_raise);

    map
}

/// Linear interpolation between two prosody feature sets.
#[allow(dead_code)]
pub fn interpolate_prosody(a: &ProsodyFeatures, b: &ProsodyFeatures, t: f32) -> ProsodyFeatures {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    ProsodyFeatures {
        pitch_hz: lerp(a.pitch_hz, b.pitch_hz),
        pitch_range: lerp(a.pitch_range, b.pitch_range),
        speech_rate: lerp(a.speech_rate, b.speech_rate),
        loudness: lerp(a.loudness, b.loudness),
        energy: lerp(a.energy, b.energy),
        jitter: lerp(a.jitter, b.jitter),
        shimmer: lerp(a.shimmer, b.shimmer),
        pause_ratio: lerp(a.pause_ratio, b.pause_ratio),
    }
}

/// Weighted blend of multiple emotion features.
#[allow(dead_code)]
pub fn blend_prosody_emotions(emotions: &[(ProsodyEmotion, f32)]) -> ProsodyFeatures {
    if emotions.is_empty() {
        return generate_prosody_for_emotion(&ProsodyEmotion::Neutral, 0.5);
    }

    let total_weight: f32 = emotions.iter().map(|(_, w)| w.max(0.0)).sum();
    if total_weight <= 0.0 {
        return generate_prosody_for_emotion(&ProsodyEmotion::Neutral, 0.5);
    }

    let mut result = ProsodyFeatures {
        pitch_hz: 0.0,
        pitch_range: 0.0,
        speech_rate: 0.0,
        loudness: 0.0,
        energy: 0.0,
        jitter: 0.0,
        shimmer: 0.0,
        pause_ratio: 0.0,
    };

    for (emotion, weight) in emotions {
        let w = weight.max(0.0) / total_weight;
        let f = generate_prosody_for_emotion(emotion, 0.7);
        result.pitch_hz += f.pitch_hz * w;
        result.pitch_range += f.pitch_range * w;
        result.speech_rate += f.speech_rate * w;
        result.loudness += f.loudness * w;
        result.energy += f.energy * w;
        result.jitter += f.jitter * w;
        result.shimmer += f.shimmer * w;
        result.pause_ratio += f.pause_ratio * w;
    }

    result
}

/// Cosine-like similarity between two prosody feature vectors, normalized to 0..1.
#[allow(dead_code)]
pub fn prosody_similarity(a: &ProsodyFeatures, b: &ProsodyFeatures) -> f32 {
    // Normalize features to comparable scales then compute dot product similarity
    let normalize = |f: &ProsodyFeatures| {
        [
            f.pitch_hz / 300.0,
            f.pitch_range / 200.0,
            f.speech_rate / 8.0,
            f.loudness,
            f.energy,
            f.jitter * 10.0,
            f.shimmer * 10.0,
            f.pause_ratio,
        ]
    };

    let na = normalize(a);
    let nb = normalize(b);

    let dot: f32 = na.iter().zip(nb.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = na.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = nb.iter().map(|x| x * x).sum::<f32>().sqrt();

    if mag_a < 1e-6 || mag_b < 1e-6 {
        return 0.0;
    }

    (dot / (mag_a * mag_b)).clamp(0.0, 1.0)
}

/// Clamp all prosody fields to valid physical ranges.
#[allow(dead_code)]
pub fn normalize_prosody(features: &mut ProsodyFeatures) {
    features.pitch_hz = features.pitch_hz.clamp(50.0, 600.0);
    features.pitch_range = features.pitch_range.clamp(0.0, 300.0);
    features.speech_rate = features.speech_rate.clamp(0.1, 10.0);
    features.loudness = features.loudness.clamp(0.0, 1.0);
    features.energy = features.energy.clamp(0.0, 1.0);
    features.jitter = features.jitter.clamp(0.0, 1.0);
    features.shimmer = features.shimmer.clamp(0.0, 1.0);
    features.pause_ratio = features.pause_ratio.clamp(0.0, 1.0);
}

/// Serialize prosody features to a JSON string.
#[allow(dead_code)]
pub fn prosody_to_json(features: &ProsodyFeatures) -> String {
    format!(
        r#"{{"pitch_hz":{:.4},"pitch_range":{:.4},"speech_rate":{:.4},"loudness":{:.4},"energy":{:.4},"jitter":{:.4},"shimmer":{:.4},"pause_ratio":{:.4}}}"#,
        features.pitch_hz,
        features.pitch_range,
        features.speech_rate,
        features.loudness,
        features.energy,
        features.jitter,
        features.shimmer,
        features.pause_ratio,
    )
}

/// Return the profile with highest intensity from a slice.
#[allow(dead_code)]
pub fn dominant_prosody_emotion(profiles: &[ProsodyProfile]) -> Option<&ProsodyProfile> {
    profiles.iter().max_by(|a, b| {
        a.intensity
            .partial_cmp(&b.intensity)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

/// Categorize speech rate into descriptive labels.
#[allow(dead_code)]
pub fn speech_rate_category(rate: f32) -> &'static str {
    if rate < 2.0 {
        "slow"
    } else if rate < 4.0 {
        "normal"
    } else if rate < 6.0 {
        "fast"
    } else {
        "very_fast"
    }
}

/// Estimate (arousal, valence) in the 2D emotion circumplex model.
/// Both values are in -1..1 range.
#[allow(dead_code)]
pub fn estimate_arousal_valence(features: &ProsodyFeatures) -> (f32, f32) {
    // Arousal: driven by energy, speech_rate, loudness
    let arousal =
        (features.energy * 0.4 + features.speech_rate / 10.0 * 0.3 + features.loudness * 0.3) * 2.0
            - 1.0;

    // Valence: driven by pitch (higher = more positive), low jitter = more positive
    let valence = ((features.pitch_hz - 100.0) / 300.0 * 0.5
        + (1.0 - features.jitter * 10.0).clamp(0.0, 1.0) * 0.3
        + (1.0 - features.pause_ratio) * 0.2)
        * 2.0
        - 1.0;

    (arousal.clamp(-1.0, 1.0), valence.clamp(-1.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn neutral_features() -> ProsodyFeatures {
        ProsodyFeatures {
            pitch_hz: 160.0,
            pitch_range: 40.0,
            speech_rate: 3.5,
            loudness: 0.5,
            energy: 0.5,
            jitter: 0.01,
            shimmer: 0.01,
            pause_ratio: 0.25,
        }
    }

    fn happy_features() -> ProsodyFeatures {
        ProsodyFeatures {
            pitch_hz: 230.0,
            pitch_range: 110.0,
            speech_rate: 5.5,
            loudness: 0.85,
            energy: 0.8,
            jitter: 0.02,
            shimmer: 0.02,
            pause_ratio: 0.1,
        }
    }

    fn sad_features() -> ProsodyFeatures {
        ProsodyFeatures {
            pitch_hz: 120.0,
            pitch_range: 20.0,
            speech_rate: 1.8,
            loudness: 0.25,
            energy: 0.2,
            jitter: 0.08,
            shimmer: 0.07,
            pause_ratio: 0.55,
        }
    }

    #[test]
    fn test_classify_happy() {
        let profile = classify_prosody_emotion(&happy_features());
        assert_eq!(profile.emotion, ProsodyEmotion::Happy);
    }

    #[test]
    fn test_classify_sad() {
        let profile = classify_prosody_emotion(&sad_features());
        assert_eq!(profile.emotion, ProsodyEmotion::Sad);
    }

    #[test]
    fn test_classify_neutral() {
        let profile = classify_prosody_emotion(&neutral_features());
        assert_eq!(profile.emotion, ProsodyEmotion::Neutral);
    }

    #[test]
    fn test_generate_happy_pitch_increases() {
        let f = generate_prosody_for_emotion(&ProsodyEmotion::Happy, 1.0);
        assert!(f.pitch_hz > 160.0);
    }

    #[test]
    fn test_generate_sad_pitch_decreases() {
        let f = generate_prosody_for_emotion(&ProsodyEmotion::Sad, 1.0);
        assert!(f.pitch_hz < 160.0);
    }

    #[test]
    fn test_prosody_to_face_params_keys() {
        let map = prosody_to_face_params(&neutral_features());
        assert!(map.contains_key("jaw_open"));
        assert!(map.contains_key("brow_raise"));
        assert!(map.contains_key("lip_corner_up"));
    }

    #[test]
    fn test_prosody_to_face_params_range() {
        let map = prosody_to_face_params(&happy_features());
        for v in map.values() {
            assert!(*v >= 0.0 && *v <= 1.0, "param out of range: {v}");
        }
    }

    #[test]
    fn test_interpolate_midpoint() {
        let mid = interpolate_prosody(&neutral_features(), &happy_features(), 0.5);
        assert!(mid.pitch_hz > 160.0 && mid.pitch_hz < 230.0);
    }

    #[test]
    fn test_interpolate_t0_equals_a() {
        let a = neutral_features();
        let result = interpolate_prosody(&a, &happy_features(), 0.0);
        assert!((result.pitch_hz - a.pitch_hz).abs() < 1e-4);
    }

    #[test]
    fn test_blend_single_emotion() {
        let blended = blend_prosody_emotions(&[(ProsodyEmotion::Happy, 1.0)]);
        let expected = generate_prosody_for_emotion(&ProsodyEmotion::Happy, 0.7);
        assert!((blended.pitch_hz - expected.pitch_hz).abs() < 1e-3);
    }

    #[test]
    fn test_blend_empty_returns_neutral() {
        let blended = blend_prosody_emotions(&[]);
        assert!((blended.speech_rate - 3.5).abs() < 0.5);
    }

    #[test]
    fn test_prosody_similarity_self() {
        let f = neutral_features();
        let sim = prosody_similarity(&f, &f);
        assert!(
            (sim - 1.0).abs() < 1e-4,
            "self-similarity should be 1.0, got {sim}"
        );
    }

    #[test]
    fn test_prosody_similarity_different() {
        let sim = prosody_similarity(&happy_features(), &sad_features());
        assert!(sim < 1.0);
    }

    #[test]
    fn test_normalize_prosody_clamps() {
        let mut f = ProsodyFeatures {
            pitch_hz: -100.0,
            pitch_range: 9999.0,
            speech_rate: -5.0,
            loudness: 2.0,
            energy: -0.5,
            jitter: 5.0,
            shimmer: 5.0,
            pause_ratio: 3.0,
        };
        normalize_prosody(&mut f);
        assert!(f.pitch_hz >= 50.0);
        assert!(f.loudness <= 1.0);
        assert!(f.jitter <= 1.0);
    }

    #[test]
    fn test_prosody_to_json_contains_fields() {
        let json = prosody_to_json(&neutral_features());
        assert!(json.contains("pitch_hz"));
        assert!(json.contains("speech_rate"));
    }

    #[test]
    fn test_dominant_prosody_emotion() {
        let profiles = vec![
            ProsodyProfile {
                emotion: ProsodyEmotion::Happy,
                intensity: 0.3,
                features: happy_features(),
            },
            ProsodyProfile {
                emotion: ProsodyEmotion::Sad,
                intensity: 0.8,
                features: sad_features(),
            },
        ];
        let dom = dominant_prosody_emotion(&profiles).expect("should succeed");
        assert_eq!(dom.emotion, ProsodyEmotion::Sad);
    }

    #[test]
    fn test_speech_rate_category() {
        assert_eq!(speech_rate_category(1.0), "slow");
        assert_eq!(speech_rate_category(3.0), "normal");
        assert_eq!(speech_rate_category(5.0), "fast");
        assert_eq!(speech_rate_category(7.0), "very_fast");
    }

    #[test]
    fn test_estimate_arousal_valence_range() {
        let (arousal, valence) = estimate_arousal_valence(&neutral_features());
        assert!((-1.0..=1.0).contains(&arousal));
        assert!((-1.0..=1.0).contains(&valence));
    }

    #[test]
    fn test_arousal_higher_for_angry() {
        let angry = generate_prosody_for_emotion(&ProsodyEmotion::Angry, 1.0);
        let calm = generate_prosody_for_emotion(&ProsodyEmotion::Calm, 1.0);
        let (a_angry, _) = estimate_arousal_valence(&angry);
        let (a_calm, _) = estimate_arousal_valence(&calm);
        assert!(
            a_angry > a_calm,
            "angry arousal {a_angry} should exceed calm {a_calm}"
        );
    }
}
