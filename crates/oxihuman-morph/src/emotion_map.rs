//! Emotion map — maps high-level emotion labels (joy, sadness, anger, etc.)
//! to expression parameter vectors.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EmotionMapConfig {
    pub param_count: usize,
    pub blend_epsilon: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EmotionEntry {
    pub name: String,
    pub params: Vec<f32>,
    pub intensity: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EmotionMap {
    pub config: EmotionMapConfig,
    pub entries: Vec<EmotionEntry>,
}

#[allow(dead_code)]
pub fn default_emotion_map_config() -> EmotionMapConfig {
    EmotionMapConfig {
        param_count: 8,
        blend_epsilon: 1e-6,
    }
}

#[allow(dead_code)]
pub fn new_emotion_map(config: EmotionMapConfig) -> EmotionMap {
    EmotionMap {
        config,
        entries: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn emotion_map_add(map: &mut EmotionMap, name: &str, params: Vec<f32>, intensity: f32) {
    let entry = EmotionEntry {
        name: name.to_string(),
        params,
        intensity: intensity.clamp(0.0, 1.0),
    };
    // Replace if exists
    if let Some(pos) = map.entries.iter().position(|e| e.name == name) {
        map.entries[pos] = entry;
    } else {
        map.entries.push(entry);
    }
}

#[allow(dead_code)]
pub fn emotion_map_get<'a>(map: &'a EmotionMap, name: &str) -> Option<&'a EmotionEntry> {
    map.entries.iter().find(|e| e.name == name)
}

#[allow(dead_code)]
pub fn emotion_map_blend(map: &EmotionMap, weights: &[(&str, f32)]) -> Vec<f32> {
    let n = map.config.param_count;
    let mut result = vec![0.0f32; n];
    let mut total_weight = 0.0f32;

    for (name, w) in weights {
        if *w < map.config.blend_epsilon {
            continue;
        }
        if let Some(entry) = emotion_map_get(map, name) {
            for (r, &p) in result.iter_mut().zip(entry.params.iter()).take(n) {
                *r += p * w;
            }
            total_weight += w;
        }
    }

    if total_weight > map.config.blend_epsilon {
        for v in &mut result {
            *v /= total_weight;
        }
    }

    result
}

#[allow(dead_code)]
pub fn emotion_map_count(map: &EmotionMap) -> usize {
    map.entries.len()
}

#[allow(dead_code)]
pub fn emotion_map_names(map: &EmotionMap) -> Vec<&str> {
    map.entries.iter().map(|e| e.name.as_str()).collect()
}

#[allow(dead_code)]
pub fn emotion_map_to_json(map: &EmotionMap) -> String {
    let entries: Vec<String> = map
        .entries
        .iter()
        .map(|e| {
            let ps: Vec<String> = e.params.iter().map(|p| format!("{p:.6}")).collect();
            format!(
                "{{\"name\":\"{}\",\"params\":[{}],\"intensity\":{:.6}}}",
                e.name,
                ps.join(","),
                e.intensity
            )
        })
        .collect();
    format!(
        "{{\"param_count\":{},\"entries\":[{}]}}",
        map.config.param_count,
        entries.join(",")
    )
}

#[allow(dead_code)]
pub fn emotion_map_clear(map: &mut EmotionMap) {
    map.entries.clear();
}

#[allow(dead_code)]
pub fn emotion_map_has(map: &EmotionMap, name: &str) -> bool {
    map.entries.iter().any(|e| e.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_map() -> EmotionMap {
        let cfg = default_emotion_map_config();
        let mut m = new_emotion_map(cfg);
        emotion_map_add(&mut m, "joy", vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 1.0);
        emotion_map_add(
            &mut m,
            "sadness",
            vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            0.8,
        );
        emotion_map_add(
            &mut m,
            "anger",
            vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            0.9,
        );
        m
    }

    #[test]
    fn test_add_and_count() {
        let m = make_map();
        assert_eq!(emotion_map_count(&m), 3);
    }

    #[test]
    fn test_has() {
        let m = make_map();
        assert!(emotion_map_has(&m, "joy"));
        assert!(!emotion_map_has(&m, "fear"));
    }

    #[test]
    fn test_get_entry() {
        let m = make_map();
        let e = emotion_map_get(&m, "joy").expect("should succeed");
        assert!((e.params[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_add_replaces() {
        let mut m = make_map();
        emotion_map_add(&mut m, "joy", vec![0.5; 8], 0.5);
        assert_eq!(emotion_map_count(&m), 3);
        let e = emotion_map_get(&m, "joy").expect("should succeed");
        assert!((e.params[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_single() {
        let m = make_map();
        let result = emotion_map_blend(&m, &[("joy", 1.0)]);
        assert!((result[0] - 1.0).abs() < 1e-5);
        assert!(result[1].abs() < 1e-5);
    }

    #[test]
    fn test_blend_two() {
        let m = make_map();
        let result = emotion_map_blend(&m, &[("joy", 1.0), ("sadness", 1.0)]);
        assert!((result[0] - 0.5).abs() < 1e-5);
        assert!((result[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_names() {
        let m = make_map();
        let names = emotion_map_names(&m);
        assert!(names.contains(&"joy"));
        assert!(names.contains(&"sadness"));
    }

    #[test]
    fn test_clear() {
        let mut m = make_map();
        emotion_map_clear(&mut m);
        assert_eq!(emotion_map_count(&m), 0);
    }

    #[test]
    fn test_to_json() {
        let m = make_map();
        let j = emotion_map_to_json(&m);
        assert!(j.contains("joy"));
        assert!(j.contains("entries"));
    }

    #[test]
    fn test_intensity_clamped() {
        let mut m = new_emotion_map(default_emotion_map_config());
        emotion_map_add(&mut m, "test", vec![0.0; 8], 2.0);
        let e = emotion_map_get(&m, "test").expect("should succeed");
        assert!(e.intensity <= 1.0);
    }
}
