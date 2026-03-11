#![allow(dead_code)]
//! Timeline-based morph animation with keyframes.

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct TimelineKey {
    pub time: f32,
    pub value: f32,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct MorphTimeline {
    keys: Vec<TimelineKey>,
}

#[allow(dead_code)]
pub fn new_morph_timeline() -> MorphTimeline {
    MorphTimeline { keys: Vec::new() }
}

#[allow(dead_code)]
pub fn add_timeline_key(tl: &mut MorphTimeline, time: f32, value: f32) {
    tl.keys.push(TimelineKey { time, value });
    tl.keys.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
}

#[allow(dead_code)]
pub fn evaluate_timeline(tl: &MorphTimeline, time: f32) -> f32 {
    if tl.keys.is_empty() {
        return 0.0;
    }
    if time <= tl.keys[0].time {
        return tl.keys[0].value;
    }
    let last = tl.keys.len() - 1;
    if time >= tl.keys[last].time {
        return tl.keys[last].value;
    }
    for i in 0..last {
        let a = &tl.keys[i];
        let b = &tl.keys[i + 1];
        if (a.time..=b.time).contains(&time) {
            let t = if (b.time - a.time).abs() < 1e-9 {
                0.0
            } else {
                (time - a.time) / (b.time - a.time)
            };
            return a.value + (b.value - a.value) * t;
        }
    }
    0.0
}

#[allow(dead_code)]
pub fn timeline_duration(tl: &MorphTimeline) -> f32 {
    if tl.keys.is_empty() {
        return 0.0;
    }
    tl.keys.last().map_or(0.0, |k| k.time) - tl.keys[0].time
}

#[allow(dead_code)]
pub fn key_count(tl: &MorphTimeline) -> usize {
    tl.keys.len()
}

#[allow(dead_code)]
pub fn key_at_time(tl: &MorphTimeline, time: f32) -> Option<&TimelineKey> {
    tl.keys.iter().find(|k| (k.time - time).abs() < 1e-6)
}

#[allow(dead_code)]
pub fn timeline_to_json(tl: &MorphTimeline) -> String {
    let entries: Vec<String> = tl
        .keys
        .iter()
        .map(|k| format!("{{\"time\":{},\"value\":{}}}", k.time, k.value))
        .collect();
    format!("[{}]", entries.join(","))
}

#[allow(dead_code)]
pub fn timeline_clear(tl: &mut MorphTimeline) {
    tl.keys.clear();
}

// ── Multi-weight MorphTimeline (new API) ──────────────────────────────────────

#[allow(dead_code)]
pub struct MorphTimelineKf {
    pub time: f32,
    pub weights: Vec<f32>,
}

#[allow(dead_code)]
pub struct MorphTimelineMulti {
    pub keyframes: Vec<MorphTimelineKf>,
    pub morph_count: usize,
}

#[allow(dead_code)]
pub fn new_morph_timeline_multi(morph_count: usize) -> MorphTimelineMulti {
    MorphTimelineMulti { keyframes: Vec::new(), morph_count }
}

#[allow(dead_code)]
pub fn mt_add_keyframe(t: &mut MorphTimelineMulti, time: f32, weights: Vec<f32>) -> bool {
    if weights.len() != t.morph_count {
        return false;
    }
    let pos = t.keyframes.partition_point(|k| k.time <= time);
    t.keyframes.insert(pos, MorphTimelineKf { time, weights });
    true
}

#[allow(dead_code)]
pub fn mt_evaluate(t: &MorphTimelineMulti, time: f32) -> Vec<f32> {
    if t.keyframes.is_empty() {
        return vec![0.0; t.morph_count];
    }
    if time <= t.keyframes[0].time {
        return t.keyframes[0].weights.clone();
    }
    let last = &t.keyframes[t.keyframes.len() - 1];
    if time >= last.time {
        return last.weights.clone();
    }
    let idx = t.keyframes.partition_point(|k| k.time <= time);
    let a = &t.keyframes[idx - 1];
    let b = &t.keyframes[idx];
    let span = b.time - a.time;
    let frac = if span > 1e-7 { (time - a.time) / span } else { 0.0 };
    a.weights.iter().zip(b.weights.iter()).map(|(wa, wb)| wa + (wb - wa) * frac).collect()
}

#[allow(dead_code)]
pub fn mt_keyframe_count(t: &MorphTimelineMulti) -> usize {
    t.keyframes.len()
}

#[allow(dead_code)]
pub fn mt_duration(t: &MorphTimelineMulti) -> f32 {
    if t.keyframes.is_empty() {
        return 0.0;
    }
    t.keyframes.last().map_or(0.0, |k| k.time) - t.keyframes[0].time
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_morph_timeline() {
        let tl = new_morph_timeline();
        assert_eq!(key_count(&tl), 0);
    }

    #[test]
    fn test_add_timeline_key() {
        let mut tl = new_morph_timeline();
        add_timeline_key(&mut tl, 0.0, 0.0);
        add_timeline_key(&mut tl, 1.0, 1.0);
        assert_eq!(key_count(&tl), 2);
    }

    #[test]
    fn test_evaluate_timeline_lerp() {
        let mut tl = new_morph_timeline();
        add_timeline_key(&mut tl, 0.0, 0.0);
        add_timeline_key(&mut tl, 1.0, 1.0);
        assert!((evaluate_timeline(&tl, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_timeline_before() {
        let mut tl = new_morph_timeline();
        add_timeline_key(&mut tl, 1.0, 0.5);
        assert!((evaluate_timeline(&tl, 0.0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_timeline_after() {
        let mut tl = new_morph_timeline();
        add_timeline_key(&mut tl, 0.0, 0.0);
        add_timeline_key(&mut tl, 1.0, 0.8);
        assert!((evaluate_timeline(&tl, 2.0) - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_timeline_duration() {
        let mut tl = new_morph_timeline();
        add_timeline_key(&mut tl, 0.5, 0.0);
        add_timeline_key(&mut tl, 2.5, 1.0);
        assert!((timeline_duration(&tl) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_key_at_time() {
        let mut tl = new_morph_timeline();
        add_timeline_key(&mut tl, 1.0, 0.5);
        assert!(key_at_time(&tl, 1.0).is_some());
        assert!(key_at_time(&tl, 2.0).is_none());
    }

    #[test]
    fn test_timeline_to_json() {
        let mut tl = new_morph_timeline();
        add_timeline_key(&mut tl, 0.0, 1.0);
        let json = timeline_to_json(&tl);
        assert!(json.contains("\"time\":"));
    }

    #[test]
    fn test_timeline_clear() {
        let mut tl = new_morph_timeline();
        add_timeline_key(&mut tl, 0.0, 0.0);
        timeline_clear(&mut tl);
        assert_eq!(key_count(&tl), 0);
    }

    #[test]
    fn test_evaluate_empty() {
        let tl = new_morph_timeline();
        assert!((evaluate_timeline(&tl, 0.5)).abs() < 1e-6);
    }

    /* multi-weight tests */
    #[test]
    fn test_mt_add_keyframe() {
        let mut t = new_morph_timeline_multi(2);
        assert!(mt_add_keyframe(&mut t, 0.0, vec![0.0, 1.0]));
    }

    #[test]
    fn test_mt_add_keyframe_mismatch() {
        let mut t = new_morph_timeline_multi(2);
        assert!(!mt_add_keyframe(&mut t, 0.0, vec![0.0]));
    }

    #[test]
    fn test_mt_evaluate_between() {
        let mut t = new_morph_timeline_multi(1);
        mt_add_keyframe(&mut t, 0.0, vec![0.0]);
        mt_add_keyframe(&mut t, 1.0, vec![1.0]);
        let v = mt_evaluate(&t, 0.5);
        assert!((v[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_mt_keyframe_count() {
        let mut t = new_morph_timeline_multi(1);
        mt_add_keyframe(&mut t, 0.0, vec![0.0]);
        mt_add_keyframe(&mut t, 1.0, vec![1.0]);
        assert_eq!(mt_keyframe_count(&t), 2);
    }

    #[test]
    fn test_mt_duration() {
        let mut t = new_morph_timeline_multi(1);
        mt_add_keyframe(&mut t, 0.0, vec![0.0]);
        mt_add_keyframe(&mut t, 2.5, vec![1.0]);
        assert!((mt_duration(&t) - 2.5).abs() < 1e-5);
    }
}
