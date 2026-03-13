// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Speech viseme / lip-sync system for OxiHuman.
//!
//! Maps spoken phonemes (IPA-inspired, ARPAbet-compatible) to facial viseme
//! morph-target weights for real-time animated lip synchronization.

#![allow(dead_code)]

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Phoneme
// ---------------------------------------------------------------------------

/// English phonemes (IPA-inspired, ARPAbet subset).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Phoneme {
    // Silence
    Silence,
    // Vowels
    AA, // "f**ther"
    AE, // "c**t"
    AH, // "b**t"
    AO, // "b**ght"
    AW, // "c**w"
    AY, // "m**"
    EH, // "b**d"
    ER, // "b**d" (r-colored)
    EY, // "b**te"
    IH, // "b**t"
    IY, // "b**te"
    OW, // "b**ne"
    OY, // "b**y"
    UH, // "b**k"
    UW, // "b**te"
    // Consonants
    B,
    CH,
    D,
    DH,
    F,
    G,
    HH,
    JH,
    K,
    L,
    M,
    N,
    NG,
    P,
    R,
    S,
    SH,
    T,
    TH,
    V,
    W,
    Y,
    Z,
    ZH,
}

impl Phoneme {
    /// All phonemes in a fixed order.
    pub fn all() -> &'static [Phoneme] {
        &[
            Phoneme::Silence,
            Phoneme::AA,
            Phoneme::AE,
            Phoneme::AH,
            Phoneme::AO,
            Phoneme::AW,
            Phoneme::AY,
            Phoneme::EH,
            Phoneme::ER,
            Phoneme::EY,
            Phoneme::IH,
            Phoneme::IY,
            Phoneme::OW,
            Phoneme::OY,
            Phoneme::UH,
            Phoneme::UW,
            Phoneme::B,
            Phoneme::CH,
            Phoneme::D,
            Phoneme::DH,
            Phoneme::F,
            Phoneme::G,
            Phoneme::HH,
            Phoneme::JH,
            Phoneme::K,
            Phoneme::L,
            Phoneme::M,
            Phoneme::N,
            Phoneme::NG,
            Phoneme::P,
            Phoneme::R,
            Phoneme::S,
            Phoneme::SH,
            Phoneme::T,
            Phoneme::TH,
            Phoneme::V,
            Phoneme::W,
            Phoneme::Y,
            Phoneme::Z,
            Phoneme::ZH,
        ]
    }

    /// Human-readable name (ARPAbet string).
    pub fn name(&self) -> &'static str {
        match self {
            Phoneme::Silence => "Silence",
            Phoneme::AA => "AA",
            Phoneme::AE => "AE",
            Phoneme::AH => "AH",
            Phoneme::AO => "AO",
            Phoneme::AW => "AW",
            Phoneme::AY => "AY",
            Phoneme::EH => "EH",
            Phoneme::ER => "ER",
            Phoneme::EY => "EY",
            Phoneme::IH => "IH",
            Phoneme::IY => "IY",
            Phoneme::OW => "OW",
            Phoneme::OY => "OY",
            Phoneme::UH => "UH",
            Phoneme::UW => "UW",
            Phoneme::B => "B",
            Phoneme::CH => "CH",
            Phoneme::D => "D",
            Phoneme::DH => "DH",
            Phoneme::F => "F",
            Phoneme::G => "G",
            Phoneme::HH => "HH",
            Phoneme::JH => "JH",
            Phoneme::K => "K",
            Phoneme::L => "L",
            Phoneme::M => "M",
            Phoneme::N => "N",
            Phoneme::NG => "NG",
            Phoneme::P => "P",
            Phoneme::R => "R",
            Phoneme::S => "S",
            Phoneme::SH => "SH",
            Phoneme::T => "T",
            Phoneme::TH => "TH",
            Phoneme::V => "V",
            Phoneme::W => "W",
            Phoneme::Y => "Y",
            Phoneme::Z => "Z",
            Phoneme::ZH => "ZH",
        }
    }

    /// Returns `true` for vowel phonemes.
    pub fn is_vowel(&self) -> bool {
        matches!(
            self,
            Phoneme::AA
                | Phoneme::AE
                | Phoneme::AH
                | Phoneme::AO
                | Phoneme::AW
                | Phoneme::AY
                | Phoneme::EH
                | Phoneme::ER
                | Phoneme::EY
                | Phoneme::IH
                | Phoneme::IY
                | Phoneme::OW
                | Phoneme::OY
                | Phoneme::UH
                | Phoneme::UW
        )
    }

    /// Returns `true` for consonant phonemes.
    pub fn is_consonant(&self) -> bool {
        matches!(
            self,
            Phoneme::B
                | Phoneme::CH
                | Phoneme::D
                | Phoneme::DH
                | Phoneme::F
                | Phoneme::G
                | Phoneme::HH
                | Phoneme::JH
                | Phoneme::K
                | Phoneme::L
                | Phoneme::M
                | Phoneme::N
                | Phoneme::NG
                | Phoneme::P
                | Phoneme::R
                | Phoneme::S
                | Phoneme::SH
                | Phoneme::T
                | Phoneme::TH
                | Phoneme::V
                | Phoneme::W
                | Phoneme::Y
                | Phoneme::Z
                | Phoneme::ZH
        )
    }

    /// Parse an ARPAbet string (case-insensitive) into a `Phoneme`.
    ///
    /// # Examples
    /// ```
    /// use oxihuman_morph::speech_viseme::Phoneme;
    /// assert_eq!(Phoneme::from_arpabet("AA"), Some(Phoneme::AA));
    /// assert_eq!(Phoneme::from_arpabet("sil"), Some(Phoneme::Silence));
    /// ```
    pub fn from_arpabet(s: &str) -> Option<Phoneme> {
        match s.to_uppercase().as_str() {
            "SILENCE" | "SIL" | "SP" | "_" => Some(Phoneme::Silence),
            "AA" => Some(Phoneme::AA),
            "AE" => Some(Phoneme::AE),
            "AH" => Some(Phoneme::AH),
            "AO" => Some(Phoneme::AO),
            "AW" => Some(Phoneme::AW),
            "AY" => Some(Phoneme::AY),
            "EH" => Some(Phoneme::EH),
            "ER" => Some(Phoneme::ER),
            "EY" => Some(Phoneme::EY),
            "IH" => Some(Phoneme::IH),
            "IY" => Some(Phoneme::IY),
            "OW" => Some(Phoneme::OW),
            "OY" => Some(Phoneme::OY),
            "UH" => Some(Phoneme::UH),
            "UW" => Some(Phoneme::UW),
            "B" => Some(Phoneme::B),
            "CH" => Some(Phoneme::CH),
            "D" => Some(Phoneme::D),
            "DH" => Some(Phoneme::DH),
            "F" => Some(Phoneme::F),
            "G" => Some(Phoneme::G),
            "HH" => Some(Phoneme::HH),
            "JH" => Some(Phoneme::JH),
            "K" => Some(Phoneme::K),
            "L" => Some(Phoneme::L),
            "M" => Some(Phoneme::M),
            "N" => Some(Phoneme::N),
            "NG" => Some(Phoneme::NG),
            "P" => Some(Phoneme::P),
            "R" => Some(Phoneme::R),
            "S" => Some(Phoneme::S),
            "SH" => Some(Phoneme::SH),
            "T" => Some(Phoneme::T),
            "TH" => Some(Phoneme::TH),
            "V" => Some(Phoneme::V),
            "W" => Some(Phoneme::W),
            "Y" => Some(Phoneme::Y),
            "Z" => Some(Phoneme::Z),
            "ZH" => Some(Phoneme::ZH),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Viseme
// ---------------------------------------------------------------------------

/// A viseme: the canonical mouth shape associated with one or more phonemes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Viseme {
    /// Mouth closed (silence).
    Silence,
    /// Bilabial plosive/nasal: B, M, P.
    PP,
    /// Labiodental fricative: F, V.
    FF,
    /// Dental fricative: TH, DH.
    TH,
    /// Alveolar: D, T, N, L.
    DD,
    /// Velar: K, G, NG.
    KK,
    /// Palatal/affricate: CH, SH, ZH, JH.
    CH,
    /// Sibilant fricative: S, Z.
    SS,
    /// Open vowel: AA, AE, AH.
    Aa,
    /// Mid vowel: EH, ER, AY (and EY).
    E,
    /// Close-front vowel: IH, IY.
    I,
    /// Rounded mid vowel: OW, AO, OY.
    O,
    /// Close-back / rounded vowel: UH, UW, AW.
    U,
    /// Retroflex / rhotic: R, ER.
    RR,
    /// Mid-neutral: HH, W, Y.
    Neutral,
}

impl Viseme {
    /// All visemes in a fixed order.
    pub fn all() -> &'static [Viseme] {
        &[
            Viseme::Silence,
            Viseme::PP,
            Viseme::FF,
            Viseme::TH,
            Viseme::DD,
            Viseme::KK,
            Viseme::CH,
            Viseme::SS,
            Viseme::Aa,
            Viseme::E,
            Viseme::I,
            Viseme::O,
            Viseme::U,
            Viseme::RR,
            Viseme::Neutral,
        ]
    }

    /// Human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            Viseme::Silence => "Silence",
            Viseme::PP => "PP",
            Viseme::FF => "FF",
            Viseme::TH => "TH",
            Viseme::DD => "DD",
            Viseme::KK => "KK",
            Viseme::CH => "CH",
            Viseme::SS => "SS",
            Viseme::Aa => "Aa",
            Viseme::E => "E",
            Viseme::I => "I",
            Viseme::O => "O",
            Viseme::U => "U",
            Viseme::RR => "RR",
            Viseme::Neutral => "Neutral",
        }
    }
}

// ---------------------------------------------------------------------------
// phoneme_to_viseme
// ---------------------------------------------------------------------------

/// Map a phoneme to its canonical viseme.
pub fn phoneme_to_viseme(phoneme: &Phoneme) -> Viseme {
    match phoneme {
        // Silence
        Phoneme::Silence => Viseme::Silence,
        // Bilabial: B, M, P
        Phoneme::B | Phoneme::M | Phoneme::P => Viseme::PP,
        // Labiodental: F, V
        Phoneme::F | Phoneme::V => Viseme::FF,
        // Dental: TH, DH
        Phoneme::TH | Phoneme::DH => Viseme::TH,
        // Alveolar: D, T, N, L
        Phoneme::D | Phoneme::T | Phoneme::N | Phoneme::L => Viseme::DD,
        // Velar: K, G, NG
        Phoneme::K | Phoneme::G | Phoneme::NG => Viseme::KK,
        // Palatal/affricate: CH, SH, ZH, JH
        Phoneme::CH | Phoneme::SH | Phoneme::ZH | Phoneme::JH => Viseme::CH,
        // Sibilant: S, Z
        Phoneme::S | Phoneme::Z => Viseme::SS,
        // Open vowels: AA, AE, AH
        Phoneme::AA | Phoneme::AE | Phoneme::AH => Viseme::Aa,
        // Mid vowels: EH, AY (+ EY)
        Phoneme::EH | Phoneme::AY | Phoneme::EY => Viseme::E,
        // Rhotic (ER maps to RR as primary viseme)
        Phoneme::ER => Viseme::RR,
        // Close-front vowels: IH, IY
        Phoneme::IH | Phoneme::IY => Viseme::I,
        // Rounded mid vowels: OW, AO, OY
        Phoneme::AO | Phoneme::OW | Phoneme::OY => Viseme::O,
        // Close-back / rounded: UH, UW, AW
        Phoneme::UH | Phoneme::UW | Phoneme::AW => Viseme::U,
        // Retroflex: R
        Phoneme::R => Viseme::RR,
        // Mid-neutral: HH, W, Y
        Phoneme::HH | Phoneme::W | Phoneme::Y => Viseme::Neutral,
    }
}

// ---------------------------------------------------------------------------
// VisemeMorphWeights / VisemeMapper
// ---------------------------------------------------------------------------

/// A set of morph-target name → weight pairs for one viseme.
pub type VisemeMorphWeights = HashMap<String, f32>;

/// Maps visemes to morph-target weight sets.
pub struct VisemeMapper {
    mappings: HashMap<Viseme, VisemeMorphWeights>,
}

impl VisemeMapper {
    /// Create an empty mapper.
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    /// Register or replace the morph weights for `viseme`.
    pub fn set_viseme(&mut self, viseme: Viseme, weights: VisemeMorphWeights) {
        self.mappings.insert(viseme, weights);
    }

    /// Return the morph weights for `viseme` (empty map if not registered).
    pub fn get_weights(&self, viseme: &Viseme) -> VisemeMorphWeights {
        self.mappings.get(viseme).cloned().unwrap_or_default()
    }

    /// Evaluate the morph weights for the viseme corresponding to `phoneme`.
    pub fn evaluate_phoneme(&self, phoneme: &Phoneme) -> VisemeMorphWeights {
        let viseme = phoneme_to_viseme(phoneme);
        self.get_weights(&viseme)
    }
}

impl Default for VisemeMapper {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// default_viseme_mapper
// ---------------------------------------------------------------------------

/// Build a `VisemeMapper` pre-loaded with MakeHuman-style morph names and
/// sensible default weights.
pub fn default_viseme_mapper() -> VisemeMapper {
    let mut mapper = VisemeMapper::new();

    // Helper closure to build a weight map from key-value pairs.
    let weights = |pairs: &[(&str, f32)]| -> VisemeMorphWeights {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    };

    // Silence — mouth fully closed, no tension.
    mapper.set_viseme(Viseme::Silence, weights(&[("lips_closed", 1.0)]));

    // PP — bilabial: B, M, P
    mapper.set_viseme(
        Viseme::PP,
        weights(&[("lips_closed", 0.9), ("lips_press", 0.5)]),
    );

    // FF — labiodental: F, V
    mapper.set_viseme(
        Viseme::FF,
        weights(&[("lower_lip_up", 0.6), ("upper_teeth_show", 0.5)]),
    );

    // TH — dental: TH, DH
    mapper.set_viseme(
        Viseme::TH,
        weights(&[("lips_part", 0.4), ("tongue_tip_up", 0.7)]),
    );

    // DD — alveolar: D, T, N, L
    mapper.set_viseme(
        Viseme::DD,
        weights(&[("lips_part", 0.3), ("jaw_drop", 0.15)]),
    );

    // KK — velar: K, G, NG
    mapper.set_viseme(
        Viseme::KK,
        weights(&[("lips_part", 0.25), ("jaw_drop", 0.2)]),
    );

    // CH — palatal / affricate: CH, SH, ZH, JH
    mapper.set_viseme(
        Viseme::CH,
        weights(&[("lips_round", 0.4), ("lips_part", 0.35), ("jaw_drop", 0.1)]),
    );

    // SS — sibilant: S, Z
    mapper.set_viseme(
        Viseme::SS,
        weights(&[("lips_part", 0.2), ("teeth_show", 0.4)]),
    );

    // Aa — open vowel: AA, AE, AH
    mapper.set_viseme(
        Viseme::Aa,
        weights(&[("jaw_drop", 0.7), ("lips_open", 0.8)]),
    );

    // E — mid vowel: EH, AY, EY
    mapper.set_viseme(
        Viseme::E,
        weights(&[("lips_wide", 0.5), ("jaw_drop", 0.35), ("lips_open", 0.4)]),
    );

    // I — close-front vowel: IH, IY
    mapper.set_viseme(Viseme::I, weights(&[("lips_wide", 0.6), ("jaw_drop", 0.2)]));

    // O — rounded mid vowel: OW, AO, OY
    mapper.set_viseme(
        Viseme::O,
        weights(&[("lips_round", 0.8), ("jaw_drop", 0.4)]),
    );

    // U — close-back / rounded: UH, UW, AW
    mapper.set_viseme(
        Viseme::U,
        weights(&[("lips_round", 0.9), ("jaw_drop", 0.3), ("lips_pucker", 0.5)]),
    );

    // RR — retroflex: R, ER
    mapper.set_viseme(
        Viseme::RR,
        weights(&[("lips_part", 0.35), ("jaw_drop", 0.25), ("lips_round", 0.3)]),
    );

    // Neutral — mid-neutral: HH, W, Y
    mapper.set_viseme(
        Viseme::Neutral,
        weights(&[("lips_part", 0.15), ("jaw_drop", 0.1)]),
    );

    mapper
}

// ---------------------------------------------------------------------------
// PhonemeEvent / LipSyncTrack
// ---------------------------------------------------------------------------

/// A single timed phoneme event in a lip-sync timeline.
pub struct PhonemeEvent {
    /// Start time in seconds.
    pub start: f32,
    /// End time in seconds.
    pub end: f32,
    /// The phoneme being spoken.
    pub phoneme: Phoneme,
    /// Amplitude / intensity in [0, 1].
    pub intensity: f32,
}

/// A complete lip-sync track: an ordered sequence of `PhonemeEvent`s.
pub struct LipSyncTrack {
    /// Phoneme events sorted by start time.
    pub events: Vec<PhonemeEvent>,
    /// Total duration of the track in seconds.
    pub duration: f32,
}

impl LipSyncTrack {
    /// Create an empty track.
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            duration: 0.0,
        }
    }

    /// Append a phoneme event, updating `duration` as needed.
    pub fn add_event(&mut self, event: PhonemeEvent) {
        if event.end > self.duration {
            self.duration = event.end;
        }
        self.events.push(event);
    }

    /// Number of events in the track.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Evaluate morph weights at time `t`.
    ///
    /// Finds the active event (start ≤ t < end) and applies a short
    /// coarticulation blend: in the last 0.05 s of an event the weights are
    /// linearly interpolated toward the *next* event's weights.
    pub fn evaluate(&self, t: f32, mapper: &VisemeMapper) -> VisemeMorphWeights {
        // Locate the current event index.
        let maybe_idx = self
            .events
            .iter()
            .enumerate()
            .find(|(_, ev)| ev.start <= t && t < ev.end)
            .map(|(i, _)| i);

        let Some(idx) = maybe_idx else {
            // Outside all events — return silence.
            return mapper.get_weights(&Viseme::Silence);
        };

        let current = &self.events[idx];
        let current_weights = mapper.evaluate_phoneme(&current.phoneme);

        // Scale by intensity.
        let scale_weights = |w: &VisemeMorphWeights, scale: f32| -> VisemeMorphWeights {
            w.iter().map(|(k, v)| (k.clone(), v * scale)).collect()
        };

        // Coarticulation blend: lerp toward next event in the last 0.05 s.
        const BLEND_WINDOW: f32 = 0.05;
        let time_left = current.end - t;

        if time_left < BLEND_WINDOW {
            if let Some(next) = self.events.get(idx + 1) {
                let alpha = 1.0 - time_left / BLEND_WINDOW; // 0→1 as we approach end
                let next_weights = mapper.evaluate_phoneme(&next.phoneme);

                // Lerp: current * (1-alpha) + next * alpha, scaled by intensity.
                let mut blended: VisemeMorphWeights = HashMap::new();

                // Collect all keys.
                let mut all_keys: std::collections::HashSet<&String> =
                    std::collections::HashSet::new();
                for k in current_weights.keys() {
                    all_keys.insert(k);
                }
                for k in next_weights.keys() {
                    all_keys.insert(k);
                }

                for key in all_keys {
                    let cw = *current_weights.get(key).unwrap_or(&0.0);
                    let nw = *next_weights.get(key).unwrap_or(&0.0);
                    let lerped = cw * (1.0 - alpha) + nw * alpha;
                    blended.insert(key.clone(), lerped * current.intensity);
                }
                return blended;
            }
        }

        scale_weights(&current_weights, current.intensity)
    }

    /// Parse a simple phoneme timeline string.
    ///
    /// Format: `"0.0:AA 0.2:B 0.4:IY"`
    ///
    /// Each token is `<start>:<PHONEME>`.  The duration of each event is
    /// inferred as the gap to the next token, or 0.1 s for the last token.
    pub fn from_string(s: &str) -> Self {
        let mut track = LipSyncTrack::new();

        // Collect (start, phoneme) pairs.
        let pairs: Vec<(f32, Phoneme)> = s
            .split_whitespace()
            .filter_map(|token| {
                let (time_str, phon_str) = token.split_once(':')?;
                let start: f32 = time_str.parse().ok()?;
                let phoneme = Phoneme::from_arpabet(phon_str)?;
                Some((start, phoneme))
            })
            .collect();

        for (i, (start, phoneme)) in pairs.iter().enumerate() {
            let end = pairs.get(i + 1).map(|(t, _)| *t).unwrap_or(start + 0.1);

            track.add_event(PhonemeEvent {
                start: *start,
                end,
                phoneme: phoneme.clone(),
                intensity: 1.0,
            });
        }

        track
    }
}

impl Default for LipSyncTrack {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phoneme_all() {
        let all = Phoneme::all();
        // Must contain Silence and all 39 phonemes = 40 total.
        assert_eq!(all.len(), 40);
        assert!(all.contains(&Phoneme::Silence));
        assert!(all.contains(&Phoneme::AA));
        assert!(all.contains(&Phoneme::ZH));
    }

    #[test]
    fn test_phoneme_is_vowel() {
        assert!(Phoneme::AA.is_vowel());
        assert!(Phoneme::IY.is_vowel());
        assert!(Phoneme::UW.is_vowel());
        assert!(Phoneme::ER.is_vowel());
        assert!(!Phoneme::B.is_vowel());
        assert!(!Phoneme::M.is_vowel());
        assert!(!Phoneme::Silence.is_vowel());
    }

    #[test]
    fn test_phoneme_is_consonant() {
        assert!(Phoneme::B.is_consonant());
        assert!(Phoneme::ZH.is_consonant());
        assert!(Phoneme::NG.is_consonant());
        assert!(!Phoneme::AA.is_consonant());
        assert!(!Phoneme::Silence.is_consonant());
    }

    #[test]
    fn test_phoneme_from_arpabet() {
        assert_eq!(Phoneme::from_arpabet("AA"), Some(Phoneme::AA));
        assert_eq!(Phoneme::from_arpabet("aa"), Some(Phoneme::AA));
        assert_eq!(Phoneme::from_arpabet("sil"), Some(Phoneme::Silence));
        assert_eq!(Phoneme::from_arpabet("SIL"), Some(Phoneme::Silence));
        assert_eq!(Phoneme::from_arpabet("SP"), Some(Phoneme::Silence));
        assert_eq!(Phoneme::from_arpabet("ZH"), Some(Phoneme::ZH));
        assert_eq!(Phoneme::from_arpabet("NG"), Some(Phoneme::NG));
        assert_eq!(Phoneme::from_arpabet("NOPE"), None);
    }

    #[test]
    fn test_viseme_all() {
        let all = Viseme::all();
        assert_eq!(all.len(), 15);
        assert!(all.contains(&Viseme::Silence));
        assert!(all.contains(&Viseme::PP));
        assert!(all.contains(&Viseme::RR));
        assert!(all.contains(&Viseme::Neutral));
    }

    #[test]
    fn test_phoneme_to_viseme_bilabial() {
        assert_eq!(phoneme_to_viseme(&Phoneme::B), Viseme::PP);
        assert_eq!(phoneme_to_viseme(&Phoneme::M), Viseme::PP);
        assert_eq!(phoneme_to_viseme(&Phoneme::P), Viseme::PP);
    }

    #[test]
    fn test_phoneme_to_viseme_vowel() {
        assert_eq!(phoneme_to_viseme(&Phoneme::AA), Viseme::Aa);
        assert_eq!(phoneme_to_viseme(&Phoneme::AH), Viseme::Aa);
        assert_eq!(phoneme_to_viseme(&Phoneme::IH), Viseme::I);
        assert_eq!(phoneme_to_viseme(&Phoneme::IY), Viseme::I);
        assert_eq!(phoneme_to_viseme(&Phoneme::OW), Viseme::O);
        assert_eq!(phoneme_to_viseme(&Phoneme::UW), Viseme::U);
        assert_eq!(phoneme_to_viseme(&Phoneme::AW), Viseme::U);
        assert_eq!(phoneme_to_viseme(&Phoneme::R), Viseme::RR);
        assert_eq!(phoneme_to_viseme(&Phoneme::ER), Viseme::RR);
    }

    #[test]
    fn test_phoneme_to_viseme_silence() {
        assert_eq!(phoneme_to_viseme(&Phoneme::Silence), Viseme::Silence);
    }

    #[test]
    fn test_viseme_mapper_default() {
        let mapper = default_viseme_mapper();
        // PP should have lips_closed and lips_press.
        let pp = mapper.get_weights(&Viseme::PP);
        assert!(pp.contains_key("lips_closed"));
        assert!(pp.contains_key("lips_press"));
        assert!((pp["lips_closed"] - 0.9).abs() < 1e-5);

        // Silence should close lips fully.
        let sil = mapper.get_weights(&Viseme::Silence);
        assert_eq!(sil["lips_closed"], 1.0);

        // Aa should have jaw_drop and lips_open.
        let aa = mapper.get_weights(&Viseme::Aa);
        assert!(aa.contains_key("jaw_drop"));
        assert!((aa["jaw_drop"] - 0.7).abs() < 1e-5);
    }

    #[test]
    fn test_viseme_mapper_evaluate_phoneme() {
        let mapper = default_viseme_mapper();
        let weights = mapper.evaluate_phoneme(&Phoneme::B);
        // B → PP → lips_closed weight.
        assert!(weights.contains_key("lips_closed"));

        let weights_i = mapper.evaluate_phoneme(&Phoneme::IY);
        assert!(weights_i.contains_key("lips_wide"));

        let weights_u = mapper.evaluate_phoneme(&Phoneme::UW);
        assert!(weights_u.contains_key("lips_round"));
        assert!(weights_u.contains_key("lips_pucker"));
    }

    #[test]
    fn test_lip_sync_track_new() {
        let track = LipSyncTrack::new();
        assert_eq!(track.event_count(), 0);
        assert_eq!(track.duration, 0.0);

        let mut track2 = LipSyncTrack::default();
        track2.add_event(PhonemeEvent {
            start: 0.0,
            end: 0.2,
            phoneme: Phoneme::AA,
            intensity: 1.0,
        });
        assert_eq!(track2.event_count(), 1);
        assert!((track2.duration - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_lip_sync_track_evaluate() {
        let mapper = default_viseme_mapper();
        let mut track = LipSyncTrack::new();

        track.add_event(PhonemeEvent {
            start: 0.0,
            end: 0.3,
            phoneme: Phoneme::AA,
            intensity: 1.0,
        });
        track.add_event(PhonemeEvent {
            start: 0.3,
            end: 0.6,
            phoneme: Phoneme::B,
            intensity: 0.8,
        });

        // At t=0.1 we should be well inside the AA event.
        let w = track.evaluate(0.1, &mapper);
        assert!(w.contains_key("jaw_drop") || w.contains_key("lips_open"));

        // At t=-0.1 (before track) — should return silence.
        let w_before = track.evaluate(-0.1, &mapper);
        assert!(w_before.contains_key("lips_closed") || w_before.is_empty());

        // At t=0.7 (after track) — should return silence.
        let w_after = track.evaluate(0.7, &mapper);
        assert!(w_after.contains_key("lips_closed") || w_after.is_empty());
    }

    #[test]
    fn test_lip_sync_from_string() {
        let track = LipSyncTrack::from_string("0.0:AA 0.2:B 0.4:IY");
        assert_eq!(track.event_count(), 3);

        // First event: AA 0.0→0.2
        assert_eq!(track.events[0].phoneme, Phoneme::AA);
        assert!((track.events[0].start - 0.0).abs() < 1e-6);
        assert!((track.events[0].end - 0.2).abs() < 1e-6);

        // Second event: B 0.2→0.4
        assert_eq!(track.events[1].phoneme, Phoneme::B);
        assert!((track.events[1].end - 0.4).abs() < 1e-6);

        // Last event: IY 0.4→0.5 (inferred 0.1 s)
        assert_eq!(track.events[2].phoneme, Phoneme::IY);
        assert!((track.events[2].end - 0.5).abs() < 1e-6);

        // Duration should be 0.5.
        assert!((track.duration - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_phoneme_name() {
        assert_eq!(Phoneme::Silence.name(), "Silence");
        assert_eq!(Phoneme::AA.name(), "AA");
        assert_eq!(Phoneme::ZH.name(), "ZH");
        assert_eq!(Phoneme::NG.name(), "NG");
        assert_eq!(Phoneme::IY.name(), "IY");
        assert_eq!(Phoneme::B.name(), "B");

        // Viseme names
        assert_eq!(Viseme::Silence.name(), "Silence");
        assert_eq!(Viseme::PP.name(), "PP");
        assert_eq!(Viseme::Aa.name(), "Aa");
        assert_eq!(Viseme::RR.name(), "RR");
    }
}
