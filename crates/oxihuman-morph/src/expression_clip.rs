#![allow(dead_code)]
//! Expression clip: a sequence of keyed expression weights over time.

/// A single key in a clip at a given time.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClipKey {
    pub time: f32,
    pub weight: f32,
}

/// A clip holding a sequence of [`ClipKey`]s.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionClip {
    name: String,
    keys: Vec<ClipKey>,
    looping: bool,
}

/// Create a new empty [`ExpressionClip`].
#[allow(dead_code)]
pub fn new_expression_clip(name: &str) -> ExpressionClip {
    ExpressionClip {
        name: name.to_string(),
        keys: Vec::new(),
        looping: false,
    }
}

/// Add a key at the given time with the given weight.
#[allow(dead_code)]
pub fn add_clip_key(clip: &mut ExpressionClip, time: f32, weight: f32) {
    clip.keys.push(ClipKey { time, weight });
    clip.keys.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
}

/// Evaluate the clip at `t` using linear interpolation.
#[allow(dead_code)]
pub fn evaluate_clip(clip: &ExpressionClip, t: f32) -> f32 {
    if clip.keys.is_empty() {
        return 0.0;
    }
    if clip.keys.len() == 1 {
        return clip.keys[0].weight;
    }
    let clamped = if clip.looping && clip_duration(clip) > 0.0 {
        t % clip_duration(clip)
    } else {
        t.clamp(clip.keys[0].time, clip.keys[clip.keys.len() - 1].time)
    };
    for i in 0..clip.keys.len() - 1 {
        let a = &clip.keys[i];
        let b = &clip.keys[i + 1];
        if (a.time..=b.time).contains(&clamped) {
            let span = b.time - a.time;
            if span.abs() < 1e-9 {
                return a.weight;
            }
            let frac = (clamped - a.time) / span;
            return a.weight + (b.weight - a.weight) * frac;
        }
    }
    clip.keys.last().map_or(0.0, |k| k.weight)
}

/// Return the total duration (last key time - first key time).
#[allow(dead_code)]
pub fn clip_duration(clip: &ExpressionClip) -> f32 {
    if clip.keys.len() < 2 {
        return 0.0;
    }
    clip.keys.last().map_or(0.0, |k| k.time) - clip.keys[0].time
}

/// Return the number of keys.
#[allow(dead_code)]
pub fn clip_key_count(clip: &ExpressionClip) -> usize {
    clip.keys.len()
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn clip_to_json(clip: &ExpressionClip) -> String {
    let keys_str: Vec<String> = clip
        .keys
        .iter()
        .map(|k| format!("{{\"time\":{},\"weight\":{}}}", k.time, k.weight))
        .collect();
    format!(
        "{{\"name\":\"{}\",\"looping\":{},\"keys\":[{}]}}",
        clip.name,
        clip.looping,
        keys_str.join(",")
    )
}

/// Set the clip to loop.
#[allow(dead_code)]
pub fn clip_loop(clip: &mut ExpressionClip, looping: bool) {
    clip.looping = looping;
}

/// Reverse the order of keys (time values are remapped).
#[allow(dead_code)]
pub fn clip_reverse(clip: &mut ExpressionClip) {
    if clip.keys.len() < 2 {
        return;
    }
    let dur = clip_duration(clip);
    let start = clip.keys[0].time;
    clip.keys.reverse();
    for key in &mut clip.keys {
        key.time = start + dur - (key.time - start);
    }
    clip.keys.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_clip() {
        let c = new_expression_clip("test");
        assert_eq!(clip_key_count(&c), 0);
    }

    #[test]
    fn test_add_key() {
        let mut c = new_expression_clip("test");
        add_clip_key(&mut c, 0.0, 0.0);
        add_clip_key(&mut c, 1.0, 1.0);
        assert_eq!(clip_key_count(&c), 2);
    }

    #[test]
    fn test_evaluate_empty() {
        let c = new_expression_clip("e");
        assert!((evaluate_clip(&c, 0.5) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_single() {
        let mut c = new_expression_clip("s");
        add_clip_key(&mut c, 0.0, 0.7);
        assert!((evaluate_clip(&c, 0.5) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_interpolation() {
        let mut c = new_expression_clip("i");
        add_clip_key(&mut c, 0.0, 0.0);
        add_clip_key(&mut c, 1.0, 1.0);
        assert!((evaluate_clip(&c, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clip_duration() {
        let mut c = new_expression_clip("d");
        add_clip_key(&mut c, 1.0, 0.0);
        add_clip_key(&mut c, 3.0, 1.0);
        assert!((clip_duration(&c) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_clip_to_json() {
        let c = new_expression_clip("j");
        let json = clip_to_json(&c);
        assert!(json.contains("\"name\":\"j\""));
    }

    #[test]
    fn test_clip_loop() {
        let mut c = new_expression_clip("l");
        clip_loop(&mut c, true);
        assert!(c.looping);
    }

    #[test]
    fn test_clip_reverse() {
        let mut c = new_expression_clip("r");
        add_clip_key(&mut c, 0.0, 0.0);
        add_clip_key(&mut c, 1.0, 1.0);
        clip_reverse(&mut c);
        assert!((evaluate_clip(&c, 0.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_clip_reverse_single() {
        let mut c = new_expression_clip("r");
        add_clip_key(&mut c, 0.0, 0.5);
        clip_reverse(&mut c);
        assert_eq!(clip_key_count(&c), 1);
    }
}
