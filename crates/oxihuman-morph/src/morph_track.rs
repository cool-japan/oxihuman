#![allow(dead_code)]

/// A sample at a specific time in a morph track.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct TrackSample {
    pub time: f32,
    pub value: f32,
}

/// Time-based morph track with samples.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphTrack {
    samples: Vec<TrackSample>,
}

#[allow(dead_code)]
pub fn new_morph_track() -> MorphTrack { MorphTrack { samples: Vec::new() } }

#[allow(dead_code)]
pub fn add_track_sample(track: &mut MorphTrack, time: f32, value: f32) {
    track.samples.push(TrackSample { time, value });
    track.samples.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
}

#[allow(dead_code)]
pub fn evaluate_morph_track(track: &MorphTrack, time: f32) -> f32 {
    if track.samples.is_empty() { return 0.0; }
    if track.samples.len() == 1 { return track.samples[0].value; }
    if time <= track.samples[0].time { return track.samples[0].value; }
    let last = &track.samples[track.samples.len() - 1];
    if time >= last.time { return last.value; }
    for w in track.samples.windows(2) {
        if (w[0].time..=w[1].time).contains(&time) {
            let span = w[1].time - w[0].time;
            if span.abs() < 1e-9 { return w[0].value; }
            let t = (time - w[0].time) / span;
            return w[0].value + (w[1].value - w[0].value) * t;
        }
    }
    last.value
}

#[allow(dead_code)]
pub fn track_duration_mt(track: &MorphTrack) -> f32 {
    if track.samples.is_empty() { return 0.0; }
    track.samples.last().map_or(0.0, |s| s.time) - track.samples[0].time
}

#[allow(dead_code)]
pub fn track_sample_count(track: &MorphTrack) -> usize { track.samples.len() }

#[allow(dead_code)]
pub fn track_to_json_mt(track: &MorphTrack) -> String {
    let e: Vec<String> = track.samples.iter()
        .map(|s| format!("[{:.4},{:.4}]", s.time, s.value)).collect();
    format!("{{\"samples\":[{}]}}", e.join(","))
}

#[allow(dead_code)]
pub fn track_clear(track: &mut MorphTrack) { track.samples.clear(); }

#[allow(dead_code)]
pub fn track_reverse(track: &mut MorphTrack) {
    if track.samples.is_empty() { return; }
    let dur = track_duration_mt(track);
    let start = track.samples[0].time;
    for s in track.samples.iter_mut() { s.time = start + dur - (s.time - start); }
    track.samples.reverse();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { assert_eq!(track_sample_count(&new_morph_track()), 0); }
    #[test] fn test_add() {
        let mut t = new_morph_track();
        add_track_sample(&mut t, 0.0, 1.0);
        assert_eq!(track_sample_count(&t), 1);
    }
    #[test] fn test_evaluate_single() {
        let mut t = new_morph_track();
        add_track_sample(&mut t, 0.0, 0.5);
        assert!((evaluate_morph_track(&t, 0.0) - 0.5).abs() < 1e-6);
    }
    #[test] fn test_evaluate_lerp() {
        let mut t = new_morph_track();
        add_track_sample(&mut t, 0.0, 0.0);
        add_track_sample(&mut t, 1.0, 1.0);
        assert!((evaluate_morph_track(&t, 0.5) - 0.5).abs() < 1e-6);
    }
    #[test] fn test_evaluate_empty() { assert!((evaluate_morph_track(&new_morph_track(), 0.0)).abs() < 1e-6); }
    #[test] fn test_duration() {
        let mut t = new_morph_track();
        add_track_sample(&mut t, 1.0, 0.0);
        add_track_sample(&mut t, 3.0, 1.0);
        assert!((track_duration_mt(&t) - 2.0).abs() < 1e-6);
    }
    #[test] fn test_to_json() {
        let mut t = new_morph_track();
        add_track_sample(&mut t, 0.0, 1.0);
        assert!(track_to_json_mt(&t).contains("samples"));
    }
    #[test] fn test_clear() {
        let mut t = new_morph_track();
        add_track_sample(&mut t, 0.0, 1.0);
        track_clear(&mut t);
        assert_eq!(track_sample_count(&t), 0);
    }
    #[test] fn test_reverse() {
        let mut t = new_morph_track();
        add_track_sample(&mut t, 0.0, 0.0);
        add_track_sample(&mut t, 1.0, 1.0);
        track_reverse(&mut t);
        assert!((evaluate_morph_track(&t, 0.0) - 1.0).abs() < 1e-6);
    }
    #[test] fn test_evaluate_before_start() {
        let mut t = new_morph_track();
        add_track_sample(&mut t, 1.0, 0.5);
        assert!((evaluate_morph_track(&t, 0.0) - 0.5).abs() < 1e-6);
    }
}
