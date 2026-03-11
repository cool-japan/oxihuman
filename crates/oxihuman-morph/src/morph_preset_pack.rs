#![allow(dead_code)]
//! Morph preset pack: a collection of named preset entries.

use std::collections::HashMap;

/// A single preset entry with name and weight map.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PresetEntry {
    /// Human-readable preset name.
    pub name: String,
    /// Morph weights keyed by target name.
    pub weights: HashMap<String, f32>,
}

/// A collection of morph presets.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphPresetPack {
    /// The presets in this pack.
    pub entries: Vec<PresetEntry>,
    /// Pack label.
    pub label: String,
}

/// Create a new empty [`MorphPresetPack`].
#[allow(dead_code)]
pub fn new_morph_preset_pack(label: &str) -> MorphPresetPack {
    MorphPresetPack {
        entries: Vec::new(),
        label: label.to_string(),
    }
}

/// Add a preset to the pack.
#[allow(dead_code)]
pub fn add_preset(pack: &mut MorphPresetPack, name: &str, weights: HashMap<String, f32>) {
    pack.entries.push(PresetEntry {
        name: name.to_string(),
        weights,
    });
}

/// Get a preset by name.
#[allow(dead_code)]
pub fn get_preset<'a>(pack: &'a MorphPresetPack, name: &str) -> Option<&'a PresetEntry> {
    pack.entries.iter().find(|e| e.name == name)
}

/// Return the number of presets.
#[allow(dead_code)]
pub fn preset_count(pack: &MorphPresetPack) -> usize {
    pack.entries.len()
}

/// Serialize pack to a simple JSON-like string.
#[allow(dead_code)]
pub fn pack_to_json(pack: &MorphPresetPack) -> String {
    let mut s = format!("{{\"label\":\"{}\",\"presets\":[", pack.label);
    for (i, e) in pack.entries.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!("{{\"name\":\"{}\"", e.name));
        s.push_str(",\"weights\":{");
        for (j, (k, v)) in e.weights.iter().enumerate() {
            if j > 0 {
                s.push(',');
            }
            s.push_str(&format!("\"{}\":{:.4}", k, v));
        }
        s.push_str("}}");
    }
    s.push_str("]}");
    s
}

/// Stub: parse pack from JSON (returns empty pack with label extracted).
#[allow(dead_code)]
pub fn pack_from_json_stub(json: &str) -> MorphPresetPack {
    // Extract label from between first "label":" and next "
    let label = json
        .find("\"label\":\"")
        .and_then(|start| {
            let rest = &json[start + 9..];
            rest.find('"').map(|end| &rest[..end])
        })
        .unwrap_or("unknown");
    new_morph_preset_pack(label)
}

/// Remove a preset by name. Returns true if removed.
#[allow(dead_code)]
pub fn remove_preset(pack: &mut MorphPresetPack, name: &str) -> bool {
    let before = pack.entries.len();
    pack.entries.retain(|e| e.name != name);
    pack.entries.len() < before
}

/// Return a list of all preset names.
#[allow(dead_code)]
pub fn preset_names(pack: &MorphPresetPack) -> Vec<String> {
    pack.entries.iter().map(|e| e.name.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_morph_preset_pack() {
        let pack = new_morph_preset_pack("test");
        assert_eq!(pack.label, "test");
        assert_eq!(preset_count(&pack), 0);
    }

    #[test]
    fn test_add_and_get_preset() {
        let mut pack = new_morph_preset_pack("p");
        let mut w = HashMap::new();
        w.insert("smile".to_string(), 0.8);
        add_preset(&mut pack, "happy", w);
        assert_eq!(preset_count(&pack), 1);
        let found = get_preset(&pack, "happy");
        assert!(found.is_some());
    }

    #[test]
    fn test_get_preset_missing() {
        let pack = new_morph_preset_pack("p");
        assert!(get_preset(&pack, "nope").is_none());
    }

    #[test]
    fn test_remove_preset() {
        let mut pack = new_morph_preset_pack("p");
        add_preset(&mut pack, "a", HashMap::new());
        assert!(remove_preset(&mut pack, "a"));
        assert_eq!(preset_count(&pack), 0);
    }

    #[test]
    fn test_remove_preset_missing() {
        let mut pack = new_morph_preset_pack("p");
        assert!(!remove_preset(&mut pack, "nope"));
    }

    #[test]
    fn test_preset_names() {
        let mut pack = new_morph_preset_pack("p");
        add_preset(&mut pack, "a", HashMap::new());
        add_preset(&mut pack, "b", HashMap::new());
        let names = preset_names(&pack);
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"a".to_string()));
    }

    #[test]
    fn test_pack_to_json() {
        let pack = new_morph_preset_pack("my_pack");
        let json = pack_to_json(&pack);
        assert!(json.contains("my_pack"));
    }

    #[test]
    fn test_pack_from_json_stub() {
        let json = r#"{"label":"hello","presets":[]}"#;
        let pack = pack_from_json_stub(json);
        assert_eq!(pack.label, "hello");
    }

    #[test]
    fn test_preset_weights() {
        let mut pack = new_morph_preset_pack("p");
        let mut w = HashMap::new();
        w.insert("x".to_string(), 0.5);
        add_preset(&mut pack, "test", w);
        let entry = get_preset(&pack, "test").unwrap();
        assert!((entry.weights["x"] - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_multiple_presets() {
        let mut pack = new_morph_preset_pack("p");
        add_preset(&mut pack, "a", HashMap::new());
        add_preset(&mut pack, "b", HashMap::new());
        add_preset(&mut pack, "c", HashMap::new());
        assert_eq!(preset_count(&pack), 3);
    }
}
