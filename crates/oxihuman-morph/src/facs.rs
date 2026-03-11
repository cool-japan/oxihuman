// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Facial Action Coding System (FACS) implementation.
//!
//! Maps the standard FACS Action Units (AUs) to morph-target weights used in
//! the OxiHuman pipeline.  The FACS system was developed by Paul Ekman and
//! Wallace Friesen and describes individual facial muscle activations with a
//! set of numbered Action Units.

#![allow(dead_code)]

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// ActionUnit
// ---------------------------------------------------------------------------

/// A FACS Action Unit representing an individual facial muscle activation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ActionUnit {
    // Upper face
    AU1, // Inner Brow Raise
    AU2, // Outer Brow Raise
    AU4, // Brow Lowerer
    AU5, // Upper Lid Raiser
    AU6, // Cheek Raiser
    AU7, // Lid Tightener
    // Nose
    AU9,  // Nose Wrinkler
    AU10, // Upper Lip Raiser
    // Mouth
    AU11, // Nasolabial Deepener
    AU12, // Lip Corner Puller (Smile)
    AU13, // Cheek Puffer
    AU14, // Dimpler
    AU15, // Lip Corner Depressor
    AU16, // Lower Lip Depressor
    AU17, // Chin Raiser
    AU18, // Lip Puckerer
    AU20, // Lip Stretcher
    AU22, // Lip Funneler
    AU23, // Lip Tightener
    AU24, // Lip Pressor
    AU25, // Lips Part
    AU26, // Jaw Drop
    AU27, // Mouth Stretch
    AU28, // Lip Suck
    // Eye
    AU41, // Lid Droop
    AU42, // Slit
    AU43, // Eyes Closed
    AU44, // Squint
    AU45, // Blink
    AU46, // Wink
}

impl ActionUnit {
    /// Return the canonical FACS number for this Action Unit.
    pub fn number(&self) -> u32 {
        match self {
            ActionUnit::AU1 => 1,
            ActionUnit::AU2 => 2,
            ActionUnit::AU4 => 4,
            ActionUnit::AU5 => 5,
            ActionUnit::AU6 => 6,
            ActionUnit::AU7 => 7,
            ActionUnit::AU9 => 9,
            ActionUnit::AU10 => 10,
            ActionUnit::AU11 => 11,
            ActionUnit::AU12 => 12,
            ActionUnit::AU13 => 13,
            ActionUnit::AU14 => 14,
            ActionUnit::AU15 => 15,
            ActionUnit::AU16 => 16,
            ActionUnit::AU17 => 17,
            ActionUnit::AU18 => 18,
            ActionUnit::AU20 => 20,
            ActionUnit::AU22 => 22,
            ActionUnit::AU23 => 23,
            ActionUnit::AU24 => 24,
            ActionUnit::AU25 => 25,
            ActionUnit::AU26 => 26,
            ActionUnit::AU27 => 27,
            ActionUnit::AU28 => 28,
            ActionUnit::AU41 => 41,
            ActionUnit::AU42 => 42,
            ActionUnit::AU43 => 43,
            ActionUnit::AU44 => 44,
            ActionUnit::AU45 => 45,
            ActionUnit::AU46 => 46,
        }
    }

    /// Short human-readable name for the AU.
    pub fn name(&self) -> &'static str {
        match self {
            ActionUnit::AU1 => "Inner Brow Raise",
            ActionUnit::AU2 => "Outer Brow Raise",
            ActionUnit::AU4 => "Brow Lowerer",
            ActionUnit::AU5 => "Upper Lid Raiser",
            ActionUnit::AU6 => "Cheek Raiser",
            ActionUnit::AU7 => "Lid Tightener",
            ActionUnit::AU9 => "Nose Wrinkler",
            ActionUnit::AU10 => "Upper Lip Raiser",
            ActionUnit::AU11 => "Nasolabial Deepener",
            ActionUnit::AU12 => "Lip Corner Puller",
            ActionUnit::AU13 => "Cheek Puffer",
            ActionUnit::AU14 => "Dimpler",
            ActionUnit::AU15 => "Lip Corner Depressor",
            ActionUnit::AU16 => "Lower Lip Depressor",
            ActionUnit::AU17 => "Chin Raiser",
            ActionUnit::AU18 => "Lip Puckerer",
            ActionUnit::AU20 => "Lip Stretcher",
            ActionUnit::AU22 => "Lip Funneler",
            ActionUnit::AU23 => "Lip Tightener",
            ActionUnit::AU24 => "Lip Pressor",
            ActionUnit::AU25 => "Lips Part",
            ActionUnit::AU26 => "Jaw Drop",
            ActionUnit::AU27 => "Mouth Stretch",
            ActionUnit::AU28 => "Lip Suck",
            ActionUnit::AU41 => "Lid Droop",
            ActionUnit::AU42 => "Slit",
            ActionUnit::AU43 => "Eyes Closed",
            ActionUnit::AU44 => "Squint",
            ActionUnit::AU45 => "Blink",
            ActionUnit::AU46 => "Wink",
        }
    }

    /// Longer description of the muscle action.
    pub fn description(&self) -> &'static str {
        match self {
            ActionUnit::AU1 => "Medial frontalis raises the inner portion of the brow",
            ActionUnit::AU2 => "Lateral frontalis raises the outer portion of the brow",
            ActionUnit::AU4 => "Corrugator and depressor supercilii lower the brows",
            ActionUnit::AU5 => "Levator palpebrae superioris raises the upper eyelid",
            ActionUnit::AU6 => "Orbicularis oculi (orbital) raises the cheek",
            ActionUnit::AU7 => "Orbicularis oculi (palpebral) tightens the lower lid",
            ActionUnit::AU9 => "Levator labii superioris alaeque nasi wrinkles the nose",
            ActionUnit::AU10 => "Levator labii superioris raises the upper lip",
            ActionUnit::AU11 => "Zygomaticus minor deepens the nasolabial fold",
            ActionUnit::AU12 => "Zygomaticus major pulls the lip corners upward and outward",
            ActionUnit::AU13 => "Levator anguli oris puffs the cheeks",
            ActionUnit::AU14 => "Buccinator creates dimples at the lip corners",
            ActionUnit::AU15 => "Depressor anguli oris pulls the lip corners downward",
            ActionUnit::AU16 => "Depressor labii inferioris lowers the lower lip",
            ActionUnit::AU17 => "Mentalis raises and wrinkles the chin",
            ActionUnit::AU18 => "Incisivii labii pucker the lips",
            ActionUnit::AU20 => "Risorius stretches the lip corners horizontally",
            ActionUnit::AU22 => "Orbicularis oris creates a funnel/O-shape with the lips",
            ActionUnit::AU23 => "Orbicularis oris narrows and tightens the lips",
            ActionUnit::AU24 => "Orbicularis oris presses the lips together",
            ActionUnit::AU25 => "Depressor labii or relaxed mentalis parts the lips",
            ActionUnit::AU26 => "Internal pterygoid, digastric, etc. drop the jaw",
            ActionUnit::AU27 => "Pterygoids, digastric open the mouth extremely wide",
            ActionUnit::AU28 => "Orbicularis oris sucks the lips inward",
            ActionUnit::AU41 => "Relaxation of levator palpebrae droops the upper lid",
            ActionUnit::AU42 => "Orbicularis oculi narrows the eye opening",
            ActionUnit::AU43 => "Relaxed levator palpebrae closes the eyes",
            ActionUnit::AU44 => "Orbicularis oculi squints the eyes",
            ActionUnit::AU45 => "Rapid closing and opening of the eyelids (blink)",
            ActionUnit::AU46 => "Closing one eye (wink)",
        }
    }

    /// All 30 action unit variants in canonical order.
    pub fn all() -> &'static [ActionUnit] {
        use ActionUnit::*;
        &[
            AU1, AU2, AU4, AU5, AU6, AU7, AU9, AU10, AU11, AU12, AU13, AU14, AU15, AU16, AU17,
            AU18, AU20, AU22, AU23, AU24, AU25, AU26, AU27, AU28, AU41, AU42, AU43, AU44, AU45,
            AU46,
        ]
    }

    /// Upper face action units (brow, nose region).
    pub fn upper_face() -> &'static [ActionUnit] {
        use ActionUnit::*;
        &[AU1, AU2, AU4, AU5, AU6, AU7, AU9, AU10]
    }

    /// Lower face action units (mouth region).
    pub fn lower_face() -> &'static [ActionUnit] {
        use ActionUnit::*;
        &[
            AU11, AU12, AU13, AU14, AU15, AU16, AU17, AU18, AU20, AU22, AU23, AU24, AU25, AU26,
            AU27, AU28,
        ]
    }

    /// Eye-related action units.
    pub fn eye_units() -> &'static [ActionUnit] {
        use ActionUnit::*;
        &[AU41, AU42, AU43, AU44, AU45, AU46]
    }

    /// Construct an ActionUnit from its canonical number, if it exists.
    pub fn from_number(n: u32) -> Option<ActionUnit> {
        match n {
            1 => Some(ActionUnit::AU1),
            2 => Some(ActionUnit::AU2),
            4 => Some(ActionUnit::AU4),
            5 => Some(ActionUnit::AU5),
            6 => Some(ActionUnit::AU6),
            7 => Some(ActionUnit::AU7),
            9 => Some(ActionUnit::AU9),
            10 => Some(ActionUnit::AU10),
            11 => Some(ActionUnit::AU11),
            12 => Some(ActionUnit::AU12),
            13 => Some(ActionUnit::AU13),
            14 => Some(ActionUnit::AU14),
            15 => Some(ActionUnit::AU15),
            16 => Some(ActionUnit::AU16),
            17 => Some(ActionUnit::AU17),
            18 => Some(ActionUnit::AU18),
            20 => Some(ActionUnit::AU20),
            22 => Some(ActionUnit::AU22),
            23 => Some(ActionUnit::AU23),
            24 => Some(ActionUnit::AU24),
            25 => Some(ActionUnit::AU25),
            26 => Some(ActionUnit::AU26),
            27 => Some(ActionUnit::AU27),
            28 => Some(ActionUnit::AU28),
            41 => Some(ActionUnit::AU41),
            42 => Some(ActionUnit::AU42),
            43 => Some(ActionUnit::AU43),
            44 => Some(ActionUnit::AU44),
            45 => Some(ActionUnit::AU45),
            46 => Some(ActionUnit::AU46),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// FacsState
// ---------------------------------------------------------------------------

/// FACS activation state: a mapping from Action Unit to intensity in [0..1].
pub type FacsState = HashMap<ActionUnit, f32>;

// ---------------------------------------------------------------------------
// FacsMapper
// ---------------------------------------------------------------------------

/// Maps FACS Action Units to morph-target weights.
///
/// Each AU can influence one or more named morph targets with a configurable
/// weight at full (1.0) intensity.  The [`Self::evaluate`] method scales each
/// morph contribution by the AU intensity and accumulates the results.
pub struct FacsMapper {
    /// AU → list of (morph_name, weight_at_full_intensity)
    mappings: HashMap<ActionUnit, Vec<(String, f32)>>,
}

impl FacsMapper {
    /// Create an empty mapper.
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    /// Register a morph target for the given AU.
    ///
    /// `weight` is the morph-target weight applied when the AU has intensity
    /// 1.0.  Multiple calls for the same AU accumulate additional mappings.
    pub fn add_mapping(&mut self, au: ActionUnit, morph: impl Into<String>, weight: f32) {
        self.mappings
            .entry(au)
            .or_default()
            .push((morph.into(), weight));
    }

    /// Return the list of `(morph_name, weight)` pairs registered for `au`.
    pub fn mappings_for(&self, au: &ActionUnit) -> &[(String, f32)] {
        self.mappings.get(au).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Convert a [`FacsState`] into a flat map of morph-target weights.
    ///
    /// For each active AU, each associated morph target receives a contribution
    /// of `intensity × weight_at_full`.  When multiple AUs influence the same
    /// morph target, contributions are summed and clamped to `[0.0, 1.0]`.
    pub fn evaluate(&self, state: &FacsState) -> HashMap<String, f32> {
        let mut result: HashMap<String, f32> = HashMap::new();
        for (au, &intensity) in state {
            if let Some(pairs) = self.mappings.get(au) {
                for (morph, max_weight) in pairs {
                    let contribution = intensity * *max_weight;
                    let entry = result.entry(morph.clone()).or_insert(0.0);
                    *entry = (*entry + contribution).min(1.0);
                }
            }
        }
        result
    }
}

impl Default for FacsMapper {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// default_facs_mapper
// ---------------------------------------------------------------------------

/// Build a default [`FacsMapper`] with MakeHuman-style morph names.
pub fn default_facs_mapper() -> FacsMapper {
    let mut m = FacsMapper::new();

    // Upper face / brow
    m.add_mapping(ActionUnit::AU1, "brow_inner_raise", 0.8);
    m.add_mapping(ActionUnit::AU2, "brow_outer_raise", 0.8);
    m.add_mapping(ActionUnit::AU4, "brow_lower", 0.9);
    m.add_mapping(ActionUnit::AU4, "brow_furrow", 0.6);
    m.add_mapping(ActionUnit::AU5, "upper_lid_raise", 0.8);
    m.add_mapping(ActionUnit::AU6, "cheek_raise", 0.7);
    m.add_mapping(ActionUnit::AU7, "lid_tighten", 0.7);

    // Nose
    m.add_mapping(ActionUnit::AU9, "nose_wrinkle", 0.8);
    m.add_mapping(ActionUnit::AU10, "upper_lip_raise", 0.7);

    // Mouth
    m.add_mapping(ActionUnit::AU11, "nasolabial_deepen", 0.7);
    m.add_mapping(ActionUnit::AU12, "smile_mouth", 0.9);
    m.add_mapping(ActionUnit::AU12, "lip_corner_pull", 0.7);
    m.add_mapping(ActionUnit::AU13, "cheek_puff", 0.7);
    m.add_mapping(ActionUnit::AU14, "dimple", 0.7);
    m.add_mapping(ActionUnit::AU15, "lip_corner_depress", 0.8);
    m.add_mapping(ActionUnit::AU16, "lower_lip_depress", 0.8);
    m.add_mapping(ActionUnit::AU17, "chin_raise", 0.7);
    m.add_mapping(ActionUnit::AU18, "lip_pucker", 0.8);
    m.add_mapping(ActionUnit::AU20, "lip_stretch", 0.8);
    m.add_mapping(ActionUnit::AU22, "lip_funnel", 0.8);
    m.add_mapping(ActionUnit::AU23, "lip_tighten", 0.7);
    m.add_mapping(ActionUnit::AU24, "lip_press", 0.7);
    m.add_mapping(ActionUnit::AU25, "lips_part", 0.8);
    m.add_mapping(ActionUnit::AU25, "jaw_open", 0.3);
    m.add_mapping(ActionUnit::AU26, "jaw_drop", 0.9);
    m.add_mapping(ActionUnit::AU27, "mouth_stretch", 0.9);
    m.add_mapping(ActionUnit::AU27, "jaw_drop", 0.5);
    m.add_mapping(ActionUnit::AU28, "lip_suck", 0.8);

    // Eye
    m.add_mapping(ActionUnit::AU41, "lid_droop", 0.8);
    m.add_mapping(ActionUnit::AU42, "eye_slit", 0.7);
    m.add_mapping(ActionUnit::AU43, "eyes_closed", 1.0);
    m.add_mapping(ActionUnit::AU44, "eye_squint", 0.8);
    m.add_mapping(ActionUnit::AU45, "blink_l", 1.0);
    m.add_mapping(ActionUnit::AU45, "blink_r", 1.0);
    m.add_mapping(ActionUnit::AU46, "wink_l", 1.0);

    m
}

// ---------------------------------------------------------------------------
// emotion_to_facs
// ---------------------------------------------------------------------------

/// Convert an emotion name to a prototypical FACS state.
///
/// Supported emotions: `"happy"`, `"sad"`, `"angry"`, `"surprised"`,
/// `"fear"`, `"disgust"`.  Unknown names return an empty state.
pub fn emotion_to_facs(emotion: &str) -> FacsState {
    let mut state: FacsState = HashMap::new();

    match emotion.to_lowercase().as_str() {
        "happy" => {
            state.insert(ActionUnit::AU6, 0.8);
            state.insert(ActionUnit::AU12, 0.9);
        }
        "sad" => {
            state.insert(ActionUnit::AU1, 0.8);
            state.insert(ActionUnit::AU4, 0.6);
            state.insert(ActionUnit::AU15, 0.7);
        }
        "angry" => {
            state.insert(ActionUnit::AU4, 0.9);
            state.insert(ActionUnit::AU5, 0.6);
            state.insert(ActionUnit::AU7, 0.8);
            state.insert(ActionUnit::AU23, 0.7);
        }
        "surprised" => {
            state.insert(ActionUnit::AU1, 0.8);
            state.insert(ActionUnit::AU2, 0.8);
            state.insert(ActionUnit::AU5, 0.9);
            state.insert(ActionUnit::AU26, 0.7);
        }
        "fear" => {
            state.insert(ActionUnit::AU1, 0.8);
            state.insert(ActionUnit::AU2, 0.8);
            state.insert(ActionUnit::AU4, 0.6);
            state.insert(ActionUnit::AU5, 0.8);
            state.insert(ActionUnit::AU20, 0.7);
        }
        "disgust" => {
            state.insert(ActionUnit::AU9, 0.9);
            state.insert(ActionUnit::AU15, 0.7);
            state.insert(ActionUnit::AU16, 0.6);
        }
        _ => {}
    }

    state
}

// ---------------------------------------------------------------------------
// FacsIntensity
// ---------------------------------------------------------------------------

/// A FACS intensity value on a normalized `[0.0, 1.0]` scale.
///
/// In the FACS literature, intensity is often described with a five-letter
/// scale: A (trace) through E (maximum).
pub struct FacsIntensity(pub f32);

impl FacsIntensity {
    /// Parse a FACS letter-scale intensity.
    ///
    /// | Letter | Label   | Normalized |
    /// |--------|---------|------------|
    /// | A      | Trace   | 0.10       |
    /// | B      | Slight  | 0.30       |
    /// | C      | Marked  | 0.50       |
    /// | D      | Extreme | 0.75       |
    /// | E      | Maximum | 1.00       |
    pub fn from_letter(letter: char) -> Option<Self> {
        let v = match letter.to_ascii_uppercase() {
            'A' => 0.10,
            'B' => 0.30,
            'C' => 0.50,
            'D' => 0.75,
            'E' => 1.00,
            _ => return None,
        };
        Some(FacsIntensity(v))
    }

    /// Return the normalized `[0.0, 1.0]` intensity.
    pub fn to_normalized(&self) -> f32 {
        self.0.clamp(0.0, 1.0)
    }

    /// Construct from a normalized `[0.0, 1.0]` value.
    pub fn from_normalized(v: f32) -> Self {
        FacsIntensity(v.clamp(0.0, 1.0))
    }
}

// ---------------------------------------------------------------------------
// parse_facs_string
// ---------------------------------------------------------------------------

/// Parse a FACS string such as `"AU1+AU6+AU12"` or `"AU1A+AU12E"`.
///
/// Tokens are separated by `'+'`.  Each token must start with `"AU"` (case-
/// insensitive) followed by a decimal number and an optional intensity letter
/// (`A`–`E`).  Missing intensity defaults to 1.0.
pub fn parse_facs_string(s: &str) -> FacsState {
    let mut state: FacsState = HashMap::new();

    for token in s.split('+') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }

        // Strip leading "AU" or "au" prefix (case-insensitive).
        let without_prefix = if token.to_uppercase().starts_with("AU") {
            &token[2..]
        } else {
            continue;
        };

        // Split numeric part from optional trailing letter.
        let last = without_prefix.chars().last();
        let (num_str, intensity) = match last {
            Some(c) if c.is_ascii_alphabetic() => {
                let num_part = &without_prefix[..without_prefix.len() - c.len_utf8()];
                let intensity = FacsIntensity::from_letter(c)
                    .map(|fi| fi.to_normalized())
                    .unwrap_or(1.0);
                (num_part, intensity)
            }
            _ => (without_prefix, 1.0_f32),
        };

        if let Ok(n) = num_str.parse::<u32>() {
            if let Some(au) = ActionUnit::from_number(n) {
                state.insert(au, intensity);
            }
        }
    }

    state
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_unit_number() {
        assert_eq!(ActionUnit::AU1.number(), 1);
        assert_eq!(ActionUnit::AU12.number(), 12);
        assert_eq!(ActionUnit::AU45.number(), 45);
        assert_eq!(ActionUnit::AU46.number(), 46);
    }

    #[test]
    fn test_action_unit_name() {
        assert_eq!(ActionUnit::AU1.name(), "Inner Brow Raise");
        assert_eq!(ActionUnit::AU12.name(), "Lip Corner Puller");
        assert_eq!(ActionUnit::AU45.name(), "Blink");
        assert_eq!(ActionUnit::AU26.name(), "Jaw Drop");
    }

    #[test]
    fn test_action_unit_all() {
        let all = ActionUnit::all();
        assert_eq!(all.len(), 30);
        // First and last
        assert_eq!(all[0], ActionUnit::AU1);
        assert_eq!(all[all.len() - 1], ActionUnit::AU46);
        // Every AU number is unique
        let numbers: Vec<u32> = all.iter().map(|au| au.number()).collect();
        let mut sorted = numbers.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), numbers.len());
    }

    #[test]
    fn test_upper_face_units() {
        let upper = ActionUnit::upper_face();
        assert_eq!(upper.len(), 8);
        assert!(upper.contains(&ActionUnit::AU1));
        assert!(upper.contains(&ActionUnit::AU6));
        assert!(upper.contains(&ActionUnit::AU10));
        // No mouth or eye AUs
        assert!(!upper.contains(&ActionUnit::AU12));
        assert!(!upper.contains(&ActionUnit::AU43));
    }

    #[test]
    fn test_lower_face_units() {
        let lower = ActionUnit::lower_face();
        assert_eq!(lower.len(), 16);
        assert!(lower.contains(&ActionUnit::AU12));
        assert!(lower.contains(&ActionUnit::AU26));
        assert!(lower.contains(&ActionUnit::AU28));
        // No upper-face or eye AUs
        assert!(!lower.contains(&ActionUnit::AU1));
        assert!(!lower.contains(&ActionUnit::AU45));
    }

    #[test]
    fn test_eye_units() {
        let eyes = ActionUnit::eye_units();
        assert_eq!(eyes.len(), 6);
        assert!(eyes.contains(&ActionUnit::AU41));
        assert!(eyes.contains(&ActionUnit::AU43));
        assert!(eyes.contains(&ActionUnit::AU46));
        // No brow or mouth AUs
        assert!(!eyes.contains(&ActionUnit::AU1));
        assert!(!eyes.contains(&ActionUnit::AU12));
    }

    #[test]
    fn test_facs_mapper_add_and_evaluate() {
        let mut mapper = FacsMapper::new();
        mapper.add_mapping(ActionUnit::AU12, "smile", 0.9);
        mapper.add_mapping(ActionUnit::AU6, "cheek", 0.7);

        let mut state: FacsState = HashMap::new();
        state.insert(ActionUnit::AU12, 1.0);
        state.insert(ActionUnit::AU6, 0.5);

        let weights = mapper.evaluate(&state);
        let smile = *weights.get("smile").expect("smile morph missing");
        let cheek = *weights.get("cheek").expect("cheek morph missing");

        assert!((smile - 0.9).abs() < 1e-5, "smile={smile}");
        assert!((cheek - 0.35).abs() < 1e-5, "cheek={cheek}");
    }

    #[test]
    fn test_default_facs_mapper() {
        let mapper = default_facs_mapper();

        // AU12 should map to smile_mouth and lip_corner_pull
        let m12 = mapper.mappings_for(&ActionUnit::AU12);
        assert!(!m12.is_empty());
        let has_smile = m12.iter().any(|(n, _)| n == "smile_mouth");
        assert!(has_smile, "AU12 should map to smile_mouth");

        // AU45 should map to blink_l and blink_r
        let m45 = mapper.mappings_for(&ActionUnit::AU45);
        let has_blink_l = m45.iter().any(|(n, _)| n == "blink_l");
        let has_blink_r = m45.iter().any(|(n, _)| n == "blink_r");
        assert!(has_blink_l, "AU45 missing blink_l");
        assert!(has_blink_r, "AU45 missing blink_r");

        // AU26 should map to jaw_drop at 0.9
        let m26 = mapper.mappings_for(&ActionUnit::AU26);
        let jaw_w = m26.iter().find(|(n, _)| n == "jaw_drop").map(|(_, w)| *w);
        assert_eq!(jaw_w, Some(0.9));
    }

    #[test]
    fn test_emotion_to_facs_happy() {
        let state = emotion_to_facs("happy");
        assert_eq!(state.get(&ActionUnit::AU6), Some(&0.8));
        assert_eq!(state.get(&ActionUnit::AU12), Some(&0.9));
        // Should not contain brow-lowering AU
        assert!(!state.contains_key(&ActionUnit::AU4));
    }

    #[test]
    fn test_emotion_to_facs_angry() {
        let state = emotion_to_facs("angry");
        assert_eq!(state.get(&ActionUnit::AU4), Some(&0.9));
        assert_eq!(state.get(&ActionUnit::AU5), Some(&0.6));
        assert_eq!(state.get(&ActionUnit::AU7), Some(&0.8));
        assert_eq!(state.get(&ActionUnit::AU23), Some(&0.7));
    }

    #[test]
    fn test_facs_intensity_from_letter() {
        assert_eq!(
            FacsIntensity::from_letter('A').unwrap().to_normalized(),
            0.10
        );
        assert_eq!(
            FacsIntensity::from_letter('B').unwrap().to_normalized(),
            0.30
        );
        assert_eq!(
            FacsIntensity::from_letter('C').unwrap().to_normalized(),
            0.50
        );
        assert_eq!(
            FacsIntensity::from_letter('D').unwrap().to_normalized(),
            0.75
        );
        assert_eq!(
            FacsIntensity::from_letter('E').unwrap().to_normalized(),
            1.00
        );
        // Lowercase should work too
        assert_eq!(
            FacsIntensity::from_letter('e').unwrap().to_normalized(),
            1.00
        );
        // Unknown letter
        assert!(FacsIntensity::from_letter('Z').is_none());
    }

    #[test]
    fn test_facs_intensity_normalized() {
        let fi = FacsIntensity::from_normalized(0.6);
        assert!((fi.to_normalized() - 0.6).abs() < 1e-6);

        // Clamping
        let over = FacsIntensity::from_normalized(1.5);
        assert_eq!(over.to_normalized(), 1.0);
        let under = FacsIntensity::from_normalized(-0.5);
        assert_eq!(under.to_normalized(), 0.0);
    }

    #[test]
    fn test_parse_facs_string_simple() {
        let state = parse_facs_string("AU12");
        assert_eq!(state.get(&ActionUnit::AU12), Some(&1.0));
        assert_eq!(state.len(), 1);
    }

    #[test]
    fn test_parse_facs_string_multi() {
        let state = parse_facs_string("AU1+AU6+AU12E");
        assert_eq!(state.get(&ActionUnit::AU1), Some(&1.0));
        assert_eq!(state.get(&ActionUnit::AU6), Some(&1.0));
        assert_eq!(state.get(&ActionUnit::AU12), Some(&1.0)); // 'E' → 1.0
        assert_eq!(state.len(), 3);

        // With intensity letters
        let state2 = parse_facs_string("AU4A+AU12C");
        assert_eq!(state2.get(&ActionUnit::AU4), Some(&0.10));
        assert_eq!(state2.get(&ActionUnit::AU12), Some(&0.50));
    }
}
