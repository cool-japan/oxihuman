//! Phoneme to viseme mapping for speech animation.
//!
//! Note: `Phoneme` and `Viseme` are defined locally here (distinct from speech_viseme's types).
//! They are exported with aliases to avoid collisions.

#[allow(dead_code)]
#[derive(Clone)]
pub struct PhonemeEntry {
    pub symbol: String,
    pub ipa: String,
    pub voiced: bool,
    pub duration_hint_ms: f32,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct VisemeEntry {
    pub name: String,
    pub jaw_open: f32,
    pub lip_spread: f32,
    pub lip_round: f32,
    pub tongue_up: f32,
    pub mouth_wide: f32,
}

#[allow(dead_code)]
pub struct PhonemeMapConfig {
    pub coarticulation_factor: f32,
    pub default_duration_ms: f32,
    pub blend_frames: usize,
}

#[allow(dead_code)]
pub struct PhonemeMap {
    pub config: PhonemeMapConfig,
    pub phonemes: Vec<PhonemeEntry>,
    pub visemes: Vec<VisemeEntry>,
    /// maps phoneme symbol -> viseme name
    pub mapping: Vec<(String, String)>,
}

#[allow(dead_code)]
pub fn default_phoneme_map_config() -> PhonemeMapConfig {
    PhonemeMapConfig {
        coarticulation_factor: 0.25,
        default_duration_ms: 80.0,
        blend_frames: 3,
    }
}

#[allow(dead_code)]
pub fn new_phoneme_map(config: PhonemeMapConfig) -> PhonemeMap {
    PhonemeMap {
        config,
        phonemes: Vec::new(),
        visemes: Vec::new(),
        mapping: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_phoneme_mapping(map: &mut PhonemeMap, phoneme_sym: &str, viseme_name: &str) {
    // Remove existing mapping for same phoneme
    map.mapping.retain(|(p, _)| p != phoneme_sym);
    map.mapping.push((phoneme_sym.to_string(), viseme_name.to_string()));
}

#[allow(dead_code)]
pub fn phoneme_to_viseme<'a>(map: &'a PhonemeMap, phoneme_sym: &str) -> Option<&'a str> {
    map.mapping.iter()
        .find(|(p, _)| p == phoneme_sym)
        .map(|(_, v)| v.as_str())
}

#[allow(dead_code)]
pub fn viseme_to_morph_weights(map: &PhonemeMap, viseme_name: &str) -> Vec<f32> {
    if let Some(v) = map.visemes.iter().find(|v| v.name == viseme_name) {
        vec![v.jaw_open, v.lip_spread, v.lip_round, v.tongue_up, v.mouth_wide]
    } else {
        vec![0.0; 5]
    }
}

#[allow(dead_code)]
pub fn phoneme_count(map: &PhonemeMap) -> usize {
    map.mapping.len()
}

#[allow(dead_code)]
pub fn viseme_count(map: &PhonemeMap) -> usize {
    map.visemes.len()
}

#[allow(dead_code)]
pub fn phoneme_map_to_json(map: &PhonemeMap) -> String {
    let mut parts = Vec::new();
    parts.push(format!("\"phoneme_count\":{}", phoneme_count(map)));
    parts.push(format!("\"viseme_count\":{}", viseme_count(map)));
    parts.push(format!("\"coarticulation_factor\":{}", map.config.coarticulation_factor));
    parts.push(format!("\"default_duration_ms\":{}", map.config.default_duration_ms));
    let mappings: Vec<String> = map.mapping.iter()
        .map(|(p, v)| format!("[\"{p}\",\"{v}\"]"))
        .collect();
    parts.push(format!("\"mappings\":[{}]", mappings.join(",")));
    format!("{{{}}}", parts.join(","))
}

/// Build a standard ARPAbet phoneme-to-viseme map.
#[allow(dead_code)]
pub fn build_arpabet_map() -> PhonemeMap {
    let cfg = default_phoneme_map_config();
    let mut map = new_phoneme_map(cfg);

    // Silence / rest
    add_phoneme_mapping(&mut map, "SIL", "rest");
    add_phoneme_mapping(&mut map, "SP", "rest");
    // Vowels
    add_phoneme_mapping(&mut map, "AA", "aa");
    add_phoneme_mapping(&mut map, "AE", "aa");
    add_phoneme_mapping(&mut map, "AH", "aa");
    add_phoneme_mapping(&mut map, "AO", "oh");
    add_phoneme_mapping(&mut map, "AW", "oh");
    add_phoneme_mapping(&mut map, "AY", "aa");
    add_phoneme_mapping(&mut map, "EH", "ee");
    add_phoneme_mapping(&mut map, "ER", "ee");
    add_phoneme_mapping(&mut map, "EY", "ee");
    add_phoneme_mapping(&mut map, "IH", "ih");
    add_phoneme_mapping(&mut map, "IY", "ih");
    add_phoneme_mapping(&mut map, "OW", "oh");
    add_phoneme_mapping(&mut map, "OY", "oh");
    add_phoneme_mapping(&mut map, "UH", "oo");
    add_phoneme_mapping(&mut map, "UW", "oo");
    // Consonants - bilabial
    add_phoneme_mapping(&mut map, "B", "pp");
    add_phoneme_mapping(&mut map, "P", "pp");
    add_phoneme_mapping(&mut map, "M", "pp");
    // Consonants - labiodental
    add_phoneme_mapping(&mut map, "F", "ff");
    add_phoneme_mapping(&mut map, "V", "ff");
    // Consonants - dental/alveolar
    add_phoneme_mapping(&mut map, "TH", "th");
    add_phoneme_mapping(&mut map, "DH", "th");
    add_phoneme_mapping(&mut map, "T", "dd");
    add_phoneme_mapping(&mut map, "D", "dd");
    add_phoneme_mapping(&mut map, "N", "nn");
    add_phoneme_mapping(&mut map, "L", "nn");
    add_phoneme_mapping(&mut map, "S", "ss");
    add_phoneme_mapping(&mut map, "Z", "ss");
    // Consonants - postalveolar
    add_phoneme_mapping(&mut map, "SH", "sh");
    add_phoneme_mapping(&mut map, "ZH", "sh");
    add_phoneme_mapping(&mut map, "CH", "sh");
    add_phoneme_mapping(&mut map, "JH", "sh");
    // Consonants - velar
    add_phoneme_mapping(&mut map, "K", "kk");
    add_phoneme_mapping(&mut map, "G", "kk");
    add_phoneme_mapping(&mut map, "NG", "nn");
    // Consonants - other
    add_phoneme_mapping(&mut map, "R", "rr");
    add_phoneme_mapping(&mut map, "W", "oo");
    add_phoneme_mapping(&mut map, "Y", "ih");
    add_phoneme_mapping(&mut map, "HH", "rest");

    // Add standard viseme entries
    let viseme_defs: &[(&str, f32, f32, f32, f32, f32)] = &[
        ("rest", 0.0, 0.0, 0.0, 0.0, 0.0),
        ("aa",   0.7, 0.3, 0.0, 0.0, 0.5),
        ("ee",   0.4, 0.8, 0.0, 0.0, 0.8),
        ("ih",   0.3, 0.6, 0.0, 0.2, 0.6),
        ("oh",   0.5, 0.0, 0.7, 0.0, 0.3),
        ("oo",   0.3, 0.0, 0.9, 0.0, 0.2),
        ("pp",   0.0, 0.0, 0.0, 0.0, 0.0),
        ("ff",   0.1, 0.2, 0.0, 0.0, 0.1),
        ("th",   0.2, 0.3, 0.0, 0.5, 0.2),
        ("dd",   0.2, 0.1, 0.0, 0.4, 0.2),
        ("nn",   0.1, 0.0, 0.0, 0.6, 0.1),
        ("ss",   0.15,0.5, 0.0, 0.3, 0.3),
        ("sh",   0.2, 0.3, 0.3, 0.1, 0.3),
        ("kk",   0.3, 0.0, 0.0, 0.0, 0.3),
        ("rr",   0.3, 0.2, 0.2, 0.3, 0.2),
    ];
    for (name, jaw, spread, round, tongue, wide) in viseme_defs {
        map.visemes.push(VisemeEntry {
            name: name.to_string(),
            jaw_open: *jaw,
            lip_spread: *spread,
            lip_round: *round,
            tongue_up: *tongue,
            mouth_wide: *wide,
        });
    }

    map
}

#[allow(dead_code)]
pub fn phoneme_duration(map: &PhonemeMap, phoneme_sym: &str) -> f32 {
    if let Some(p) = map.phonemes.iter().find(|p| p.symbol == phoneme_sym) {
        p.duration_hint_ms
    } else {
        map.config.default_duration_ms
    }
}

#[allow(dead_code)]
pub fn coarticulation_blend(map: &PhonemeMap, from: &str, to: &str, t: f32) -> Vec<f32> {
    let w_from = viseme_to_morph_weights(
        map,
        phoneme_to_viseme(map, from).unwrap_or("rest"),
    );
    let w_to = viseme_to_morph_weights(
        map,
        phoneme_to_viseme(map, to).unwrap_or("rest"),
    );
    let factor = map.config.coarticulation_factor;
    let t_adj = t + (1.0 - t) * factor;
    let t_clamped = t_adj.clamp(0.0, 1.0);
    w_from.iter().zip(w_to.iter())
        .map(|(a, b)| a + (b - a) * t_clamped)
        .collect()
}

#[allow(dead_code)]
pub fn dominant_viseme(map: &PhonemeMap, viseme_name: &str) -> f32 {
    let weights = viseme_to_morph_weights(map, viseme_name);
    weights.iter().cloned().fold(0.0f32, f32::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_phoneme_map_config();
        assert!((cfg.coarticulation_factor - 0.25).abs() < 1e-6);
        assert!((cfg.default_duration_ms - 80.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_phoneme_map_empty() {
        let map = new_phoneme_map(default_phoneme_map_config());
        assert_eq!(phoneme_count(&map), 0);
        assert_eq!(viseme_count(&map), 0);
    }

    #[test]
    fn test_add_phoneme_mapping() {
        let mut map = new_phoneme_map(default_phoneme_map_config());
        add_phoneme_mapping(&mut map, "AA", "aa");
        assert_eq!(phoneme_count(&map), 1);
    }

    #[test]
    fn test_phoneme_to_viseme_found() {
        let mut map = new_phoneme_map(default_phoneme_map_config());
        add_phoneme_mapping(&mut map, "AH", "aa");
        let v = phoneme_to_viseme(&map, "AH");
        assert_eq!(v, Some("aa"));
    }

    #[test]
    fn test_phoneme_to_viseme_not_found() {
        let map = new_phoneme_map(default_phoneme_map_config());
        assert!(phoneme_to_viseme(&map, "ZZ").is_none());
    }

    #[test]
    fn test_add_mapping_replaces_existing() {
        let mut map = new_phoneme_map(default_phoneme_map_config());
        add_phoneme_mapping(&mut map, "AA", "aa");
        add_phoneme_mapping(&mut map, "AA", "oh");
        assert_eq!(phoneme_count(&map), 1);
        assert_eq!(phoneme_to_viseme(&map, "AA"), Some("oh"));
    }

    #[test]
    fn test_viseme_to_morph_weights_found() {
        let map = build_arpabet_map();
        let weights = viseme_to_morph_weights(&map, "aa");
        assert_eq!(weights.len(), 5);
        assert!(weights[0] > 0.0);
    }

    #[test]
    fn test_viseme_to_morph_weights_not_found() {
        let map = new_phoneme_map(default_phoneme_map_config());
        let weights = viseme_to_morph_weights(&map, "nonexistent");
        assert_eq!(weights, vec![0.0; 5]);
    }

    #[test]
    fn test_build_arpabet_map_phoneme_count() {
        let map = build_arpabet_map();
        assert!(phoneme_count(&map) >= 30);
    }

    #[test]
    fn test_build_arpabet_map_viseme_count() {
        let map = build_arpabet_map();
        assert!(viseme_count(&map) >= 10);
    }

    #[test]
    fn test_phoneme_duration_default() {
        let map = new_phoneme_map(default_phoneme_map_config());
        let dur = phoneme_duration(&map, "AA");
        assert!((dur - 80.0).abs() < 1e-5);
    }

    #[test]
    fn test_phoneme_map_to_json() {
        let map = build_arpabet_map();
        let json = phoneme_map_to_json(&map);
        assert!(json.contains("phoneme_count"));
        assert!(json.contains("viseme_count"));
        assert!(json.contains("mappings"));
    }

    #[test]
    fn test_coarticulation_blend_at_zero() {
        let map = build_arpabet_map();
        let w = coarticulation_blend(&map, "SIL", "AA", 0.0);
        assert_eq!(w.len(), 5);
    }

    #[test]
    fn test_coarticulation_blend_at_one() {
        let map = build_arpabet_map();
        let w1 = coarticulation_blend(&map, "SIL", "AA", 1.0);
        let w2 = viseme_to_morph_weights(&map, "aa");
        for (a, b) in w1.iter().zip(w2.iter()) {
            assert!((a - b).abs() < 1e-5);
        }
    }

    #[test]
    fn test_dominant_viseme_rest_is_zero() {
        let map = build_arpabet_map();
        let d = dominant_viseme(&map, "rest");
        assert!((d).abs() < 1e-6);
    }

    #[test]
    fn test_dominant_viseme_aa_positive() {
        let map = build_arpabet_map();
        let d = dominant_viseme(&map, "aa");
        assert!(d > 0.0);
    }

    #[test]
    fn test_arpabet_bilabial_maps_to_pp() {
        let map = build_arpabet_map();
        assert_eq!(phoneme_to_viseme(&map, "B"), Some("pp"));
        assert_eq!(phoneme_to_viseme(&map, "P"), Some("pp"));
        assert_eq!(phoneme_to_viseme(&map, "M"), Some("pp"));
    }
}
