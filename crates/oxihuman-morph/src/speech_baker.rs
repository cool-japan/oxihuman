// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Bake a phoneme event sequence into a morph weight track for lip sync.

use std::collections::HashMap;

// ── Types ────────────────────────────────────────────────────────────────────

/// A single phoneme occurrence in time.
#[allow(dead_code)]
pub struct PhonemeEvent {
    /// Phoneme label, e.g. "AA", "B", "SIL".
    pub phoneme: String,
    /// Start time in seconds.
    pub start: f32,
    /// End time in seconds.
    pub end: f32,
}

/// Configuration for the lip-sync baker.
#[allow(dead_code)]
pub struct BakerConfig {
    /// Bake resolution in frames per second (default 30).
    pub fps: f32,
    /// Crossfade window in seconds (default 0.05).
    pub blend_window: f32,
    /// Overall weight multiplier (default 1.0).
    pub emphasis_scale: f32,
    /// Phoneme label used for silence (default "SIL").
    pub silence_phoneme: String,
}

impl Default for BakerConfig {
    fn default() -> Self {
        Self {
            fps: 30.0,
            blend_window: 0.05,
            emphasis_scale: 1.0,
            silence_phoneme: "SIL".to_string(),
        }
    }
}

/// The output of a bake pass.
#[allow(dead_code)]
pub struct BakedLipSync {
    /// Frames-per-second used during baking.
    pub fps: f32,
    /// Per-frame morph weight maps.
    pub frames: Vec<HashMap<String, f32>>,
    /// Total duration of the baked sequence in seconds.
    pub duration: f32,
}

// ── Free functions ────────────────────────────────────────────────────────────

/// Build a simple English phoneme → morph weight map.
/// Morph targets: `mouth_open`, `lip_round`, `lip_wide`, `teeth_show`, `jaw_drop`.
#[allow(dead_code)]
pub fn build_default_viseme_map() -> HashMap<String, HashMap<String, f32>> {
    let mut m: HashMap<String, HashMap<String, f32>> = HashMap::new();

    // Silence
    m.insert(
        "SIL".into(),
        [
            ("mouth_open".into(), 0.0),
            ("lip_round".into(), 0.0),
            ("lip_wide".into(), 0.0),
            ("teeth_show".into(), 0.0),
            ("jaw_drop".into(), 0.0),
        ]
        .into_iter()
        .collect(),
    );

    // /AA/ – open mouth, wide
    m.insert(
        "AA".into(),
        [
            ("mouth_open".into(), 0.9),
            ("lip_round".into(), 0.0),
            ("lip_wide".into(), 0.4),
            ("teeth_show".into(), 0.6),
            ("jaw_drop".into(), 0.8),
        ]
        .into_iter()
        .collect(),
    );

    // /AE/ – mid-open, slightly wide
    m.insert(
        "AE".into(),
        [
            ("mouth_open".into(), 0.6),
            ("lip_round".into(), 0.0),
            ("lip_wide".into(), 0.5),
            ("teeth_show".into(), 0.4),
            ("jaw_drop".into(), 0.5),
        ]
        .into_iter()
        .collect(),
    );

    // /IY/ – smile shape
    m.insert(
        "IY".into(),
        [
            ("mouth_open".into(), 0.2),
            ("lip_round".into(), 0.0),
            ("lip_wide".into(), 0.9),
            ("teeth_show".into(), 0.5),
            ("jaw_drop".into(), 0.1),
        ]
        .into_iter()
        .collect(),
    );

    // /UW/ – round lips
    m.insert(
        "UW".into(),
        [
            ("mouth_open".into(), 0.3),
            ("lip_round".into(), 0.9),
            ("lip_wide".into(), 0.0),
            ("teeth_show".into(), 0.0),
            ("jaw_drop".into(), 0.2),
        ]
        .into_iter()
        .collect(),
    );

    // /OW/ – round, mid-open
    m.insert(
        "OW".into(),
        [
            ("mouth_open".into(), 0.5),
            ("lip_round".into(), 0.7),
            ("lip_wide".into(), 0.0),
            ("teeth_show".into(), 0.1),
            ("jaw_drop".into(), 0.4),
        ]
        .into_iter()
        .collect(),
    );

    // /B/ /P/ /M/ – bilabial, closed
    for ph in &["B", "P", "M"] {
        m.insert(
            ph.to_string(),
            [
                ("mouth_open".into(), 0.0),
                ("lip_round".into(), 0.0),
                ("lip_wide".into(), 0.0),
                ("teeth_show".into(), 0.0),
                ("jaw_drop".into(), 0.0),
            ]
            .into_iter()
            .collect(),
        );
    }

    // /F/ /V/ – teeth on lower lip
    for ph in &["F", "V"] {
        m.insert(
            ph.to_string(),
            [
                ("mouth_open".into(), 0.1),
                ("lip_round".into(), 0.0),
                ("lip_wide".into(), 0.3),
                ("teeth_show".into(), 0.8),
                ("jaw_drop".into(), 0.1),
            ]
            .into_iter()
            .collect(),
        );
    }

    // /TH/ /DH/ – tongue between teeth
    for ph in &["TH", "DH"] {
        m.insert(
            ph.to_string(),
            [
                ("mouth_open".into(), 0.15),
                ("lip_round".into(), 0.0),
                ("lip_wide".into(), 0.2),
                ("teeth_show".into(), 0.7),
                ("jaw_drop".into(), 0.1),
            ]
            .into_iter()
            .collect(),
        );
    }

    // /S/ /Z/ – slight opening
    for ph in &["S", "Z"] {
        m.insert(
            ph.to_string(),
            [
                ("mouth_open".into(), 0.05),
                ("lip_round".into(), 0.0),
                ("lip_wide".into(), 0.4),
                ("teeth_show".into(), 0.6),
                ("jaw_drop".into(), 0.05),
            ]
            .into_iter()
            .collect(),
        );
    }

    // /CH/ /JH/ /SH/ /ZH/ – rounded slightly
    for ph in &["CH", "JH", "SH", "ZH"] {
        m.insert(
            ph.to_string(),
            [
                ("mouth_open".into(), 0.2),
                ("lip_round".into(), 0.4),
                ("lip_wide".into(), 0.1),
                ("teeth_show".into(), 0.3),
                ("jaw_drop".into(), 0.15),
            ]
            .into_iter()
            .collect(),
        );
    }

    // /R/ – slight rounding
    m.insert(
        "R".into(),
        [
            ("mouth_open".into(), 0.2),
            ("lip_round".into(), 0.3),
            ("lip_wide".into(), 0.0),
            ("teeth_show".into(), 0.1),
            ("jaw_drop".into(), 0.2),
        ]
        .into_iter()
        .collect(),
    );

    // /L/ /N/ /D/ /T/ – neutral-ish
    for ph in &["L", "N", "D", "T"] {
        m.insert(
            ph.to_string(),
            [
                ("mouth_open".into(), 0.3),
                ("lip_round".into(), 0.0),
                ("lip_wide".into(), 0.3),
                ("teeth_show".into(), 0.3),
                ("jaw_drop".into(), 0.2),
            ]
            .into_iter()
            .collect(),
        );
    }

    m
}

/// Return the active phonemes (and their crossfade blend weights) at time `t`.
/// Returns a `Vec<(phoneme, blend_weight)>`. Weights sum to ≤1.0.
#[allow(dead_code)]
pub fn active_phonemes_at(
    events: &[PhonemeEvent],
    t: f32,
    blend_window: f32,
) -> Vec<(String, f32)> {
    let mut contributions: Vec<(String, f32)> = Vec::new();

    for ev in events {
        if t < ev.start - blend_window || t > ev.end + blend_window {
            continue;
        }

        let weight = if t < ev.start {
            // Fade-in ramp before this event's start (overlap with previous).
            let d = ev.start - t;
            1.0 - (d / blend_window).clamp(0.0, 1.0)
        } else if t > ev.end {
            // Fade-out ramp after this event's end (overlap with next).
            let d = t - ev.end;
            1.0 - (d / blend_window).clamp(0.0, 1.0)
        } else {
            1.0
        };

        if weight > 0.0 {
            contributions.push((ev.phoneme.clone(), weight));
        }
    }

    // Normalise so weights sum to 1.0.
    let total: f32 = contributions.iter().map(|(_, w)| w).sum();
    if total > 1.0 {
        for (_, w) in &mut contributions {
            *w /= total;
        }
    }
    contributions
}

/// Weighted sum of viseme morph weights from multiple phoneme contributions.
#[allow(dead_code)]
pub fn blend_viseme_weights(
    contributions: &[(String, f32)],
    viseme_map: &HashMap<String, HashMap<String, f32>>,
) -> HashMap<String, f32> {
    let mut result: HashMap<String, f32> = HashMap::new();

    for (phoneme, weight) in contributions {
        if let Some(morphs) = viseme_map.get(phoneme) {
            for (morph, &v) in morphs {
                *result.entry(morph.clone()).or_insert(0.0) += v * weight;
            }
        }
    }
    result
}

/// Bake a phoneme sequence into a [`BakedLipSync`] track.
#[allow(dead_code)]
pub fn bake_phoneme_sequence(
    events: &[PhonemeEvent],
    viseme_map: &HashMap<String, HashMap<String, f32>>,
    cfg: &BakerConfig,
) -> BakedLipSync {
    // Compute duration from events.
    let duration = events.iter().map(|e| e.end).fold(0.0_f32, f32::max);
    let frame_count = (duration * cfg.fps).ceil() as usize + 1;

    let silence_map: HashMap<String, f32> = viseme_map
        .get(&cfg.silence_phoneme)
        .cloned()
        .unwrap_or_default();

    let frames: Vec<HashMap<String, f32>> = (0..frame_count)
        .map(|i| {
            let t = (i as f32) / cfg.fps;
            let contributions = active_phonemes_at(events, t, cfg.blend_window);

            let mut weights = if contributions.is_empty() {
                silence_map.clone()
            } else {
                blend_viseme_weights(&contributions, viseme_map)
            };

            // Apply emphasis scale.
            if (cfg.emphasis_scale - 1.0).abs() > f32::EPSILON {
                for v in weights.values_mut() {
                    *v = (*v * cfg.emphasis_scale).clamp(0.0, 1.0);
                }
            }
            weights
        })
        .collect();

    BakedLipSync {
        fps: cfg.fps,
        frames,
        duration,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn ev(phoneme: &str, start: f32, end: f32) -> PhonemeEvent {
        PhonemeEvent {
            phoneme: phoneme.to_string(),
            start,
            end,
        }
    }

    // 1. active_phonemes_at: in middle of phoneme → 100% weight
    #[test]
    fn test_active_in_middle_full_weight() {
        let events = vec![ev("AA", 0.0, 1.0)];
        let result = active_phonemes_at(&events, 0.5, 0.05);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "AA");
        assert!((result[0].1 - 1.0).abs() < 1e-5);
    }

    // 2. active_phonemes_at: in crossfade → both phonemes active
    #[test]
    fn test_active_in_crossfade_both_present() {
        let events = vec![ev("AA", 0.0, 1.0), ev("IY", 1.0, 2.0)];
        // At t=0.98, we're in AA's fade-out AND before IY starts: blend_window=0.05
        // AA ends at 1.0, so fade-out covers 1.0..1.05 (after end).
        // IY starts at 1.0, fade-in covers 0.95..1.0 (before start).
        let result = active_phonemes_at(&events, 0.97, 0.05);
        // Only AA should be fully active here (IY fade-in starts at 0.95).
        let has_iy = result.iter().any(|(p, _)| p == "IY");
        let has_aa = result.iter().any(|(p, _)| p == "AA");
        assert!(has_aa, "AA should be active at t=0.97");
        // IY fade-in at t=0.97: d=1.0-0.97=0.03 < blend_window=0.05, weight = 1 - 0.03/0.05 = 0.4 > 0
        assert!(has_iy, "IY should be in fade-in at t=0.97");
    }

    // 3. active_phonemes_at: before first event → empty (no silence event)
    #[test]
    fn test_active_before_first_event_empty() {
        let events = vec![ev("AA", 1.0, 2.0)];
        let result = active_phonemes_at(&events, 0.0, 0.05);
        assert!(result.is_empty());
    }

    // 4. active_phonemes_at: after last event (past blend_window) → empty
    #[test]
    fn test_active_after_last_event_empty() {
        let events = vec![ev("AA", 0.0, 1.0)];
        let result = active_phonemes_at(&events, 1.2, 0.05);
        assert!(result.is_empty());
    }

    // 5. blend_viseme_weights: weighted sum is correct
    #[test]
    fn test_blend_viseme_weights_sum() {
        let vm = build_default_viseme_map();
        let contributions = vec![("AA".to_string(), 1.0_f32)];
        let weights = blend_viseme_weights(&contributions, &vm);
        let aa = vm.get("AA").unwrap();
        for (k, &v) in aa {
            assert!((weights[k] - v).abs() < 1e-5, "key {} mismatch", k);
        }
    }

    // 6. bake_phoneme_sequence: correct frame count
    #[test]
    fn test_bake_frame_count() {
        let events = vec![ev("AA", 0.0, 1.0)];
        let vm = build_default_viseme_map();
        let cfg = BakerConfig::default();
        let baked = bake_phoneme_sequence(&events, &vm, &cfg);
        let expected = (1.0_f32 * 30.0).ceil() as usize + 1;
        assert_eq!(baked.frames.len(), expected);
    }

    // 7. baked frame at a phoneme-active time has expected morph keys
    #[test]
    fn test_baked_frame_has_morph_keys() {
        let events = vec![ev("AA", 0.0, 1.0)];
        let vm = build_default_viseme_map();
        let cfg = BakerConfig::default();
        let baked = bake_phoneme_sequence(&events, &vm, &cfg);
        // Frame 0 corresponds to t=0.0 (inside AA).
        let frame = &baked.frames[0];
        assert!(frame.contains_key("mouth_open"));
        assert!(frame.contains_key("lip_round"));
    }

    // 8. build_default_viseme_map contains "SIL"
    #[test]
    fn test_default_viseme_map_contains_sil() {
        let vm = build_default_viseme_map();
        assert!(vm.contains_key("SIL"));
    }

    // 9. BakerConfig defaults
    #[test]
    fn test_baker_config_defaults() {
        let cfg = BakerConfig::default();
        assert!((cfg.fps - 30.0).abs() < 1e-5);
        assert!((cfg.blend_window - 0.05).abs() < 1e-5);
        assert!((cfg.emphasis_scale - 1.0).abs() < 1e-5);
        assert_eq!(cfg.silence_phoneme, "SIL");
    }

    // 10. emphasis_scale applies to baked weights
    #[test]
    fn test_emphasis_scale_applies() {
        let events = vec![ev("AA", 0.0, 1.0)];
        let vm = build_default_viseme_map();
        let cfg_normal = BakerConfig::default();
        let cfg_half = BakerConfig {
            emphasis_scale: 0.5,
            ..Default::default()
        };

        let baked_normal = bake_phoneme_sequence(&events, &vm, &cfg_normal);
        let baked_half = bake_phoneme_sequence(&events, &vm, &cfg_half);

        let frame_idx = 5; // somewhere in the middle
        let mouth_open_normal = baked_normal.frames[frame_idx]
            .get("mouth_open")
            .copied()
            .unwrap_or(0.0);
        let mouth_open_half = baked_half.frames[frame_idx]
            .get("mouth_open")
            .copied()
            .unwrap_or(0.0);
        // With emphasis 0.5, mouth_open should be approximately half.
        if mouth_open_normal > 0.01 {
            assert!(
                mouth_open_half < mouth_open_normal,
                "half scale should be smaller"
            );
        }
    }

    // 11. blend_viseme_weights: two contributions average correctly
    #[test]
    fn test_blend_viseme_weights_two_contributions() {
        let vm = build_default_viseme_map();
        // SIL = all zeros, AA has mouth_open=0.9 → blend at 0.5 each → mouth_open=0.45
        let contributions = vec![("SIL".to_string(), 0.5_f32), ("AA".to_string(), 0.5_f32)];
        let weights = blend_viseme_weights(&contributions, &vm);
        let expected = 0.9 * 0.5;
        assert!((weights["mouth_open"] - expected).abs() < 1e-5);
    }

    // 12. baked sequence: BakedLipSync fps matches config
    #[test]
    fn test_baked_fps_matches_config() {
        let events = vec![ev("IY", 0.0, 0.5)];
        let vm = build_default_viseme_map();
        let cfg = BakerConfig {
            fps: 24.0,
            ..Default::default()
        };
        let baked = bake_phoneme_sequence(&events, &vm, &cfg);
        assert!((baked.fps - 24.0).abs() < 1e-5);
    }
}
