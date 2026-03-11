//! Lip sync morph controller that drives mouth morphs from phoneme/viseme data.

/// Configuration for the lip sync controller.
#[allow(dead_code)]
pub struct LipSyncConfig {
    /// Overall weight multiplier for all viseme morphs.
    pub weight_scale: f32,
    /// Smoothing factor for blending (0 = instant, 1 = no movement).
    pub smoothing: f32,
    /// Minimum jaw-open amount when speaking.
    pub jaw_bias: f32,
}

/// A viseme (mouth shape) used in lip sync.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Viseme {
    Silence,
    AA, // "ah" vowel
    EE, // "ee" vowel
    OO, // "oo" vowel
    FF, // labiodental fricative
    TH, // dental fricative
    SH, // palato-alveolar fricative
    NN, // nasal
}

/// Runtime state for the lip sync controller.
#[allow(dead_code)]
pub struct LipSyncState {
    /// Current active viseme.
    pub current_viseme: Viseme,
    /// Current blended weight for the active viseme [0..1].
    pub current_weight: f32,
    /// The target viseme being blended toward.
    pub target_viseme: Viseme,
    /// The target weight being blended toward.
    pub target_weight: f32,
    /// Jaw influence override [0..1].
    pub jaw_influence: f32,
    /// Cached config copy.
    pub config: LipSyncConfig,
}

/// Returns a default `LipSyncConfig`.
#[allow(dead_code)]
pub fn default_lip_sync_config() -> LipSyncConfig {
    LipSyncConfig {
        weight_scale: 1.0,
        smoothing: 0.1,
        jaw_bias: 0.0,
    }
}

/// Creates a new `LipSyncState` from a `LipSyncConfig`.
#[allow(dead_code)]
pub fn new_lip_sync_state(cfg: &LipSyncConfig) -> LipSyncState {
    LipSyncState {
        current_viseme: Viseme::Silence,
        current_weight: 0.0,
        target_viseme: Viseme::Silence,
        target_weight: 0.0,
        jaw_influence: cfg.jaw_bias,
        config: LipSyncConfig {
            weight_scale: cfg.weight_scale,
            smoothing: cfg.smoothing,
            jaw_bias: cfg.jaw_bias,
        },
    }
}

/// Instantly sets the current viseme and strength.
#[allow(dead_code)]
pub fn set_current_viseme(state: &mut LipSyncState, viseme: Viseme, strength: f32) {
    state.current_viseme = viseme;
    state.current_weight = strength.clamp(0.0, 1.0);
    state.target_viseme = viseme;
    state.target_weight = state.current_weight;
}

/// Blends toward a target viseme over time using blend_speed and dt.
#[allow(dead_code)]
pub fn blend_to_viseme(
    state: &mut LipSyncState,
    target: Viseme,
    strength: f32,
    blend_speed: f32,
    dt: f32,
) {
    state.target_viseme = target;
    state.target_weight = strength.clamp(0.0, 1.0);
    let alpha = (blend_speed * dt).clamp(0.0, 1.0);
    if state.current_viseme == state.target_viseme {
        state.current_weight += (state.target_weight - state.current_weight) * alpha;
    } else {
        // Fade out current, then switch
        state.current_weight -= state.current_weight * alpha;
        if state.current_weight < 0.01 {
            state.current_viseme = state.target_viseme;
            state.current_weight = 0.0;
        }
    }
}

/// Returns the morph weight array for all 8 visemes in enum order.
/// Index matches: [Silence, AA, EE, OO, FF, TH, SH, NN]
#[allow(dead_code)]
pub fn lip_sync_morph_weights(state: &LipSyncState) -> [f32; 8] {
    let mut weights = [0.0f32; 8];
    let idx = viseme_index(state.current_viseme);
    weights[idx] = (state.current_weight * state.config.weight_scale).clamp(0.0, 1.0);
    weights
}

/// Returns the human-readable name for a viseme.
#[allow(dead_code)]
pub fn viseme_name(v: Viseme) -> &'static str {
    match v {
        Viseme::Silence => "Silence",
        Viseme::AA => "AA",
        Viseme::EE => "EE",
        Viseme::OO => "OO",
        Viseme::FF => "FF",
        Viseme::TH => "TH",
        Viseme::SH => "SH",
        Viseme::NN => "NN",
    }
}

/// Resets the lip sync state to silent.
#[allow(dead_code)]
pub fn reset_lip_sync(state: &mut LipSyncState) {
    state.current_viseme = Viseme::Silence;
    state.current_weight = 0.0;
    state.target_viseme = Viseme::Silence;
    state.target_weight = 0.0;
    state.jaw_influence = state.config.jaw_bias;
}

/// Sets the jaw influence override amount [0..1].
#[allow(dead_code)]
pub fn set_jaw_influence(state: &mut LipSyncState, amount: f32) {
    state.jaw_influence = amount.clamp(0.0, 1.0);
}

/// Returns true if the current viseme is Silence and weight is effectively zero.
#[allow(dead_code)]
pub fn lip_sync_is_silent(state: &LipSyncState) -> bool {
    state.current_viseme == Viseme::Silence || state.current_weight < 0.01
}

/// Maps a phoneme string (IPA or ARPAbet-style) to a `Viseme`.
#[allow(dead_code)]
pub fn viseme_from_phoneme_str(phoneme: &str) -> Viseme {
    match phoneme.to_lowercase().as_str() {
        "aa" | "ah" | "ae" | "aw" | "ay" | "a" => Viseme::AA,
        "ee" | "ih" | "iy" | "eh" | "e" | "i" => Viseme::EE,
        "oo" | "uw" | "uh" | "oh" | "ow" | "o" | "u" => Viseme::OO,
        "f" | "v" => Viseme::FF,
        "th" | "dh" => Viseme::TH,
        "sh" | "zh" | "ch" | "jh" => Viseme::SH,
        "n" | "m" | "ng" => Viseme::NN,
        _ => Viseme::Silence,
    }
}

fn viseme_index(v: Viseme) -> usize {
    match v {
        Viseme::Silence => 0,
        Viseme::AA => 1,
        Viseme::EE => 2,
        Viseme::OO => 3,
        Viseme::FF => 4,
        Viseme::TH => 5,
        Viseme::SH => 6,
        Viseme::NN => 7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_lip_sync_config();
        assert!((cfg.weight_scale - 1.0).abs() < 1e-6);
        assert!((cfg.smoothing - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_is_silent() {
        let cfg = default_lip_sync_config();
        let state = new_lip_sync_state(&cfg);
        assert!(lip_sync_is_silent(&state));
        assert_eq!(state.current_viseme, Viseme::Silence);
    }

    #[test]
    fn test_set_current_viseme() {
        let cfg = default_lip_sync_config();
        let mut state = new_lip_sync_state(&cfg);
        set_current_viseme(&mut state, Viseme::AA, 0.8);
        assert_eq!(state.current_viseme, Viseme::AA);
        assert!((state.current_weight - 0.8).abs() < 1e-6);
        assert!(!lip_sync_is_silent(&state));
    }

    #[test]
    fn test_set_current_viseme_clamps() {
        let cfg = default_lip_sync_config();
        let mut state = new_lip_sync_state(&cfg);
        set_current_viseme(&mut state, Viseme::EE, 2.5);
        assert!((state.current_weight - 1.0).abs() < 1e-6);
        set_current_viseme(&mut state, Viseme::EE, -0.5);
        assert!((state.current_weight - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_lip_sync_morph_weights() {
        let cfg = default_lip_sync_config();
        let mut state = new_lip_sync_state(&cfg);
        set_current_viseme(&mut state, Viseme::OO, 1.0);
        let weights = lip_sync_morph_weights(&state);
        assert!((weights[3] - 1.0).abs() < 1e-6); // OO is index 3
        assert!((weights[0]).abs() < 1e-6);
        assert!((weights[1]).abs() < 1e-6);
    }

    #[test]
    fn test_viseme_name() {
        assert_eq!(viseme_name(Viseme::Silence), "Silence");
        assert_eq!(viseme_name(Viseme::AA), "AA");
        assert_eq!(viseme_name(Viseme::FF), "FF");
        assert_eq!(viseme_name(Viseme::NN), "NN");
    }

    #[test]
    fn test_reset_lip_sync() {
        let cfg = default_lip_sync_config();
        let mut state = new_lip_sync_state(&cfg);
        set_current_viseme(&mut state, Viseme::SH, 0.9);
        reset_lip_sync(&mut state);
        assert!(lip_sync_is_silent(&state));
        assert_eq!(state.current_viseme, Viseme::Silence);
        assert!((state.current_weight).abs() < 1e-6);
    }

    #[test]
    fn test_set_jaw_influence() {
        let cfg = default_lip_sync_config();
        let mut state = new_lip_sync_state(&cfg);
        set_jaw_influence(&mut state, 0.5);
        assert!((state.jaw_influence - 0.5).abs() < 1e-6);
        set_jaw_influence(&mut state, 3.0);
        assert!((state.jaw_influence - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_viseme_from_phoneme_str_vowels() {
        assert_eq!(viseme_from_phoneme_str("aa"), Viseme::AA);
        assert_eq!(viseme_from_phoneme_str("ee"), Viseme::EE);
        assert_eq!(viseme_from_phoneme_str("oo"), Viseme::OO);
        assert_eq!(viseme_from_phoneme_str("uw"), Viseme::OO);
    }

    #[test]
    fn test_viseme_from_phoneme_str_consonants() {
        assert_eq!(viseme_from_phoneme_str("f"), Viseme::FF);
        assert_eq!(viseme_from_phoneme_str("v"), Viseme::FF);
        assert_eq!(viseme_from_phoneme_str("th"), Viseme::TH);
        assert_eq!(viseme_from_phoneme_str("sh"), Viseme::SH);
        assert_eq!(viseme_from_phoneme_str("n"), Viseme::NN);
        assert_eq!(viseme_from_phoneme_str("m"), Viseme::NN);
    }

    #[test]
    fn test_viseme_from_phoneme_str_unknown() {
        assert_eq!(viseme_from_phoneme_str("xyz"), Viseme::Silence);
        assert_eq!(viseme_from_phoneme_str(""), Viseme::Silence);
    }

    #[test]
    fn test_blend_to_viseme_same() {
        let cfg = default_lip_sync_config();
        let mut state = new_lip_sync_state(&cfg);
        set_current_viseme(&mut state, Viseme::AA, 0.5);
        blend_to_viseme(&mut state, Viseme::AA, 1.0, 10.0, 0.1);
        // Should have moved toward 1.0
        assert!(state.current_weight > 0.5);
    }

    #[test]
    fn test_lip_sync_is_silent_zero_weight() {
        let cfg = default_lip_sync_config();
        let mut state = new_lip_sync_state(&cfg);
        set_current_viseme(&mut state, Viseme::AA, 0.005);
        // weight below threshold — treated as silent
        assert!(lip_sync_is_silent(&state));
    }
}
