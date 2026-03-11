#![allow(dead_code)]
//! Expression presets with categories and named weight sets.

use std::collections::HashMap;

/// Category for an expression preset.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetCategory {
    Happy,
    Sad,
    Angry,
    Surprised,
    Neutral,
    Custom,
}

/// An expression preset with named weights.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionPreset {
    /// Preset name.
    pub name: String,
    /// Category classification.
    pub category: PresetCategory,
    /// Morph weights keyed by target name.
    pub weights: HashMap<String, f32>,
}

/// Create a new [`ExpressionPreset`].
#[allow(dead_code)]
pub fn new_expression_preset(
    name: &str,
    category: PresetCategory,
    weights: HashMap<String, f32>,
) -> ExpressionPreset {
    ExpressionPreset {
        name: name.to_string(),
        category,
        weights,
    }
}

/// Return the preset name.
#[allow(dead_code)]
pub fn preset_name(preset: &ExpressionPreset) -> &str {
    &preset.name
}

/// Return the preset category.
#[allow(dead_code)]
pub fn preset_category(preset: &ExpressionPreset) -> PresetCategory {
    preset.category
}

/// Return a reference to the preset weights.
#[allow(dead_code)]
pub fn preset_weights(preset: &ExpressionPreset) -> &HashMap<String, f32> {
    &preset.weights
}

/// Apply the preset weights to a mutable weight map, overwriting matching keys.
#[allow(dead_code)]
pub fn apply_expression_preset(preset: &ExpressionPreset, target: &mut HashMap<String, f32>) {
    for (k, v) in &preset.weights {
        target.insert(k.clone(), *v);
    }
}

/// Serialize the preset to a JSON-like string.
#[allow(dead_code)]
pub fn preset_to_json(preset: &ExpressionPreset) -> String {
    let cat_str = match preset.category {
        PresetCategory::Happy => "happy",
        PresetCategory::Sad => "sad",
        PresetCategory::Angry => "angry",
        PresetCategory::Surprised => "surprised",
        PresetCategory::Neutral => "neutral",
        PresetCategory::Custom => "custom",
    };
    let mut s = format!(
        "{{\"name\":\"{}\",\"category\":\"{}\",\"weights\":{{",
        preset.name, cat_str
    );
    for (i, (k, v)) in preset.weights.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!("\"{}\":{:.4}", k, v));
    }
    s.push_str("}}");
    s
}

/// Stub: parse a preset from JSON (returns a neutral preset with extracted name).
#[allow(dead_code)]
pub fn preset_from_json_stub(json: &str) -> ExpressionPreset {
    let name = json
        .find("\"name\":\"")
        .and_then(|start| {
            let rest = &json[start + 8..];
            rest.find('"').map(|end| &rest[..end])
        })
        .unwrap_or("unknown");
    new_expression_preset(name, PresetCategory::Neutral, HashMap::new())
}

/// Return the number of weights in a preset.
#[allow(dead_code)]
pub fn expression_preset_count(preset: &ExpressionPreset) -> usize {
    preset.weights.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_expression_preset() {
        let p = new_expression_preset("smile", PresetCategory::Happy, HashMap::new());
        assert_eq!(p.name, "smile");
        assert_eq!(p.category, PresetCategory::Happy);
    }

    #[test]
    fn test_preset_name() {
        let p = new_expression_preset("frown", PresetCategory::Sad, HashMap::new());
        assert_eq!(preset_name(&p), "frown");
    }

    #[test]
    fn test_preset_category() {
        let p = new_expression_preset("angry", PresetCategory::Angry, HashMap::new());
        assert_eq!(preset_category(&p), PresetCategory::Angry);
    }

    #[test]
    fn test_preset_weights() {
        let mut w = HashMap::new();
        w.insert("brow_raise".to_string(), 0.5);
        let p = new_expression_preset("surprise", PresetCategory::Surprised, w);
        assert!(preset_weights(&p).contains_key("brow_raise"));
    }

    #[test]
    fn test_apply_expression_preset() {
        let mut w = HashMap::new();
        w.insert("smile".to_string(), 0.9);
        let p = new_expression_preset("happy", PresetCategory::Happy, w);
        let mut target = HashMap::new();
        apply_expression_preset(&p, &mut target);
        assert!((target["smile"] - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_apply_expression_preset_overwrites() {
        let mut w = HashMap::new();
        w.insert("x".to_string(), 0.5);
        let p = new_expression_preset("p", PresetCategory::Neutral, w);
        let mut target = HashMap::new();
        target.insert("x".to_string(), 0.1);
        apply_expression_preset(&p, &mut target);
        assert!((target["x"] - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_preset_to_json() {
        let p = new_expression_preset("test", PresetCategory::Custom, HashMap::new());
        let json = preset_to_json(&p);
        assert!(json.contains("test"));
        assert!(json.contains("custom"));
    }

    #[test]
    fn test_preset_from_json_stub() {
        let json = r#"{"name":"hello","category":"happy","weights":{}}"#;
        let p = preset_from_json_stub(json);
        assert_eq!(p.name, "hello");
    }

    #[test]
    fn test_expression_preset_count() {
        let mut w = HashMap::new();
        w.insert("a".to_string(), 0.1);
        w.insert("b".to_string(), 0.2);
        let p = new_expression_preset("p", PresetCategory::Neutral, w);
        assert_eq!(expression_preset_count(&p), 2);
    }

    #[test]
    fn test_expression_preset_count_empty() {
        let p = new_expression_preset("p", PresetCategory::Neutral, HashMap::new());
        assert_eq!(expression_preset_count(&p), 0);
    }
}
