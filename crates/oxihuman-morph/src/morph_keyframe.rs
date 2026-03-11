#![allow(dead_code)]

//! Morph keyframe sequence with interpolation.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphKeyframe {
    pub time: f32,
    pub weights: Vec<f32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphKeyframeSeq {
    pub name: String,
    pub keyframes: Vec<MorphKeyframe>,
    pub looping: bool,
    pub duration: f32,
}

#[allow(dead_code)]
pub fn new_morph_keyframe_seq(name: &str, looping: bool) -> MorphKeyframeSeq {
    MorphKeyframeSeq {
        name: name.to_string(),
        keyframes: Vec::new(),
        looping,
        duration: 0.0,
    }
}

#[allow(dead_code)]
pub fn mks_add_keyframe(seq: &mut MorphKeyframeSeq, time: f32, weights: Vec<f32>) {
    seq.keyframes.push(MorphKeyframe { time, weights });
    seq.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
    seq.duration = seq.keyframes.last().map(|k| k.time).unwrap_or(0.0);
}

#[allow(dead_code)]
pub fn mks_evaluate(seq: &MorphKeyframeSeq, time: f32) -> Vec<f32> {
    if seq.keyframes.is_empty() {
        return Vec::new();
    }
    let t = if seq.looping && seq.duration > 0.0 {
        time % seq.duration
    } else {
        time.clamp(0.0, seq.duration)
    };
    let after = seq.keyframes.iter().position(|k| k.time > t);
    match after {
        None => seq.keyframes[seq.keyframes.len() - 1].weights.clone(),
        Some(0) => seq.keyframes[0].weights.clone(),
        Some(i) => {
            let k0 = &seq.keyframes[i - 1];
            let k1 = &seq.keyframes[i];
            let span = k1.time - k0.time;
            let alpha = if span > 0.0 { (t - k0.time) / span } else { 0.0 };
            let len = k0.weights.len().min(k1.weights.len());
            (0..len)
                .map(|j| k0.weights[j] * (1.0 - alpha) + k1.weights[j] * alpha)
                .collect()
        }
    }
}

#[allow(dead_code)]
pub fn mks_keyframe_count(seq: &MorphKeyframeSeq) -> usize {
    seq.keyframes.len()
}

#[allow(dead_code)]
pub fn mks_clear(seq: &mut MorphKeyframeSeq) {
    seq.keyframes.clear();
    seq.duration = 0.0;
}

#[allow(dead_code)]
pub fn mks_to_json(seq: &MorphKeyframeSeq) -> String {
    format!(
        "{{\"name\":\"{}\",\"keyframe_count\":{},\"duration\":{},\"looping\":{}}}",
        seq.name,
        seq.keyframes.len(),
        seq.duration,
        seq.looping
    )
}

#[allow(dead_code)]
pub fn mks_duration(seq: &MorphKeyframeSeq) -> f32 {
    seq.duration
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_seq() {
        let seq = new_morph_keyframe_seq("blink", false);
        assert_eq!(mks_keyframe_count(&seq), 0);
        assert!((mks_duration(&seq) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_add_keyframe() {
        let mut seq = new_morph_keyframe_seq("blink", false);
        mks_add_keyframe(&mut seq, 0.5, vec![1.0]);
        assert_eq!(mks_keyframe_count(&seq), 1);
    }

    #[test]
    fn test_duration_updated() {
        let mut seq = new_morph_keyframe_seq("blink", false);
        mks_add_keyframe(&mut seq, 1.0, vec![0.0]);
        mks_add_keyframe(&mut seq, 2.0, vec![1.0]);
        assert!((mks_duration(&seq) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_before_start() {
        let mut seq = new_morph_keyframe_seq("blink", false);
        mks_add_keyframe(&mut seq, 1.0, vec![0.5]);
        let w = mks_evaluate(&seq, 0.0);
        assert!((w[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_at_end() {
        let mut seq = new_morph_keyframe_seq("blink", false);
        mks_add_keyframe(&mut seq, 0.0, vec![0.0]);
        mks_add_keyframe(&mut seq, 1.0, vec![1.0]);
        let w = mks_evaluate(&seq, 1.0);
        assert!((w[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_interpolated() {
        let mut seq = new_morph_keyframe_seq("blink", false);
        mks_add_keyframe(&mut seq, 0.0, vec![0.0]);
        mks_add_keyframe(&mut seq, 1.0, vec![1.0]);
        let w = mks_evaluate(&seq, 0.5);
        assert!((w[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_empty() {
        let seq = new_morph_keyframe_seq("empty", false);
        let w = mks_evaluate(&seq, 0.5);
        assert!(w.is_empty());
    }

    #[test]
    fn test_looping_wraps() {
        let mut seq = new_morph_keyframe_seq("loop", true);
        mks_add_keyframe(&mut seq, 0.0, vec![0.0]);
        mks_add_keyframe(&mut seq, 1.0, vec![1.0]);
        let w1 = mks_evaluate(&seq, 0.5);
        let w2 = mks_evaluate(&seq, 1.5);
        assert!((w1[0] - w2[0]).abs() < 1e-5);
    }

    #[test]
    fn test_clear() {
        let mut seq = new_morph_keyframe_seq("blink", false);
        mks_add_keyframe(&mut seq, 1.0, vec![1.0]);
        mks_clear(&mut seq);
        assert_eq!(mks_keyframe_count(&seq), 0);
    }

    #[test]
    fn test_to_json() {
        let seq = new_morph_keyframe_seq("jaw", false);
        let json = mks_to_json(&seq);
        assert!(json.contains("\"name\":\"jaw\""));
    }
}
