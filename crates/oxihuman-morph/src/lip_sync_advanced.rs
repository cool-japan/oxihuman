//! Advanced lip sync with phoneme scheduling and coarticulation.

#[allow(dead_code)]
#[derive(Clone)]
pub struct PhonemeEvent {
    pub phoneme: String,
    pub start_time: f32,
    pub duration: f32,
    pub intensity: f32,
}

#[allow(dead_code)]
pub struct CoarticulationParams {
    /// How far ahead to blend toward next phoneme (secs).
    pub lookahead: f32,
    /// Blend tail from previous phoneme (secs).
    pub lookbehind: f32,
    /// Smoothing factor in 0..1.
    pub smoothing: f32,
}

#[allow(dead_code)]
pub struct LipSyncTrack {
    pub events: Vec<PhonemeEvent>,
    pub duration: f32,
    pub coarticulation: CoarticulationParams,
}

#[allow(dead_code)]
pub struct LipSyncFrame {
    pub time: f32,
    pub active_phoneme: String,
    pub blend_phoneme: Option<String>,
    pub blend_weight: f32,
    pub mouth_open: f32,
    pub lip_corner_pull: f32,
    pub lip_press: f32,
}

// ---------------------------------------------------------------------------
// Construction helpers
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn default_coarticulation() -> CoarticulationParams {
    CoarticulationParams {
        lookahead: 0.05,
        lookbehind: 0.03,
        smoothing: 0.3,
    }
}

#[allow(dead_code)]
pub fn new_lip_sync_track(duration: f32) -> LipSyncTrack {
    LipSyncTrack {
        events: Vec::new(),
        duration,
        coarticulation: default_coarticulation(),
    }
}

#[allow(dead_code)]
pub fn add_phoneme_event(track: &mut LipSyncTrack, event: PhonemeEvent) {
    track.events.push(event);
}

// ---------------------------------------------------------------------------
// Sorting / indexing
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn sort_phoneme_events(track: &mut LipSyncTrack) {
    track.events.sort_by(|a, b| {
        a.start_time
            .partial_cmp(&b.start_time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

#[allow(dead_code)]
pub fn event_count(track: &LipSyncTrack) -> usize {
    track.events.len()
}

#[allow(dead_code)]
pub fn phonemes_at_time(track: &LipSyncTrack, time: f32) -> Vec<&PhonemeEvent> {
    track
        .events
        .iter()
        .filter(|e| time >= e.start_time && time < e.start_time + e.duration)
        .collect()
}

// ---------------------------------------------------------------------------
// Phoneme → mouth shape
// ---------------------------------------------------------------------------

/// Returns `(mouth_open, lip_corner_pull, lip_press)` for a given phoneme.
#[allow(dead_code)]
pub fn phoneme_to_mouth_shape(phoneme: &str) -> (f32, f32, f32) {
    match phoneme.to_uppercase().as_str() {
        // Vowels
        "AA" | "AH" => (0.8, 0.1, 0.0),
        "AE" => (0.7, 0.3, 0.0),
        "AO" => (0.6, 0.0, 0.0),
        "AW" => (0.5, 0.0, 0.1),
        "AY" => (0.7, 0.2, 0.0),
        "EH" => (0.5, 0.4, 0.0),
        "ER" => (0.4, 0.1, 0.1),
        "EY" => (0.4, 0.5, 0.0),
        "IH" | "IY" => (0.2, 0.6, 0.0),
        "OW" => (0.5, 0.0, 0.2),
        "OY" => (0.5, 0.0, 0.3),
        "UH" | "UW" => (0.3, 0.0, 0.4),
        // Bilabials
        "B" | "P" | "M" => (0.0, 0.0, 0.8),
        // Labiodentals
        "F" | "V" => (0.1, 0.0, 0.6),
        // Dentals / sibilants
        "TH" | "DH" => (0.2, 0.0, 0.2),
        "S" | "Z" => (0.1, 0.3, 0.3),
        "SH" | "ZH" => (0.2, 0.1, 0.4),
        // Rest / silence
        "SIL" | "" => (0.0, 0.0, 0.0),
        _ => (0.3, 0.1, 0.1),
    }
}

// ---------------------------------------------------------------------------
// Evaluation
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn evaluate_lip_sync(track: &LipSyncTrack, time: f32) -> LipSyncFrame {
    // Find the active phoneme event (last one whose window covers `time`).
    let active = track
        .events
        .iter()
        .rfind(|e| time >= e.start_time && time < e.start_time + e.duration);

    // Find the next phoneme event (soonest starting after `time` within lookahead).
    let next = track
        .events
        .iter()
        .filter(|e| e.start_time > time && e.start_time - time <= track.coarticulation.lookahead)
        .min_by(|a, b| {
            a.start_time
                .partial_cmp(&b.start_time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

    let (active_phoneme, base_open, base_corner, base_press) = if let Some(ev) = active {
        let (o, c, p) = phoneme_to_mouth_shape(&ev.phoneme);
        (
            ev.phoneme.clone(),
            o * ev.intensity,
            c * ev.intensity,
            p * ev.intensity,
        )
    } else {
        (String::new(), 0.0, 0.0, 0.0)
    };

    let (blend_phoneme, blend_weight, mouth_open, lip_corner_pull, lip_press) =
        if let Some(nev) = next {
            let dist = nev.start_time - time;
            let weight = (1.0 - dist / track.coarticulation.lookahead).clamp(0.0, 1.0);
            let (no, nc, np) = phoneme_to_mouth_shape(&nev.phoneme);
            let weight_scaled = weight * nev.intensity;
            let w_inv = 1.0 - weight_scaled;
            (
                Some(nev.phoneme.clone()),
                weight,
                base_open * w_inv + no * weight_scaled,
                base_corner * w_inv + nc * weight_scaled,
                base_press * w_inv + np * weight_scaled,
            )
        } else {
            (None, 0.0, base_open, base_corner, base_press)
        };

    LipSyncFrame {
        time,
        active_phoneme,
        blend_phoneme,
        blend_weight,
        mouth_open,
        lip_corner_pull,
        lip_press,
    }
}

// ---------------------------------------------------------------------------
// Viseme weights
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn lip_sync_to_viseme_weights(track: &LipSyncTrack, time: f32) -> Vec<(String, f32)> {
    let frame = evaluate_lip_sync(track, time);
    let mut weights: Vec<(String, f32)> = Vec::new();
    if !frame.active_phoneme.is_empty() {
        let w = 1.0 - frame.blend_weight;
        if w > 0.001 {
            weights.push((frame.active_phoneme.clone(), w));
        }
    }
    if let Some(blend) = frame.blend_phoneme {
        if frame.blend_weight > 0.001 {
            weights.push((blend, frame.blend_weight));
        }
    }
    weights
}

// ---------------------------------------------------------------------------
// Editing helpers
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn trim_lip_sync(track: &mut LipSyncTrack, start: f32, end: f32) {
    track.events.retain(|e| {
        let ev_end = e.start_time + e.duration;
        ev_end > start && e.start_time < end
    });
    track.duration = end - start;
}

#[allow(dead_code)]
pub fn scale_lip_sync_timing(track: &mut LipSyncTrack, factor: f32) {
    for event in track.events.iter_mut() {
        event.start_time *= factor;
        event.duration *= factor;
    }
    track.duration *= factor;
}

#[allow(dead_code)]
pub fn merge_lip_sync_tracks(a: &LipSyncTrack, b: &LipSyncTrack) -> LipSyncTrack {
    let mut merged = new_lip_sync_track(a.duration.max(b.duration));
    for ev in &a.events {
        merged.events.push(ev.clone());
    }
    for ev in &b.events {
        merged.events.push(ev.clone());
    }
    sort_phoneme_events(&mut merged);
    merged
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(phoneme: &str, start: f32, dur: f32) -> PhonemeEvent {
        PhonemeEvent {
            phoneme: phoneme.to_string(),
            start_time: start,
            duration: dur,
            intensity: 1.0,
        }
    }

    #[test]
    fn test_new_track() {
        let track = new_lip_sync_track(5.0);
        assert!((track.duration - 5.0).abs() < 1e-6);
        assert!(track.events.is_empty());
    }

    #[test]
    fn test_add_event() {
        let mut track = new_lip_sync_track(3.0);
        add_phoneme_event(&mut track, make_event("AA", 0.0, 0.2));
        assert_eq!(track.events.len(), 1);
    }

    #[test]
    fn test_event_count() {
        let mut track = new_lip_sync_track(3.0);
        assert_eq!(event_count(&track), 0);
        add_phoneme_event(&mut track, make_event("AA", 0.0, 0.2));
        add_phoneme_event(&mut track, make_event("B", 0.2, 0.1));
        assert_eq!(event_count(&track), 2);
    }

    #[test]
    fn test_evaluate_lip_sync_active() {
        let mut track = new_lip_sync_track(2.0);
        add_phoneme_event(&mut track, make_event("AA", 0.0, 0.5));
        let frame = evaluate_lip_sync(&track, 0.2);
        assert_eq!(frame.active_phoneme, "AA");
        assert!(frame.mouth_open > 0.0);
    }

    #[test]
    fn test_evaluate_lip_sync_silence() {
        let track = new_lip_sync_track(2.0);
        let frame = evaluate_lip_sync(&track, 0.5);
        assert_eq!(frame.active_phoneme, "");
        assert!((frame.mouth_open).abs() < 1e-6);
    }

    #[test]
    fn test_phoneme_to_mouth_shape_vowels() {
        let (o, _c, _p) = phoneme_to_mouth_shape("AA");
        assert!(o > 0.5, "AA should have large mouth open");
        let (o2, c2, _) = phoneme_to_mouth_shape("IY");
        assert!(o2 < 0.4, "IY should have smaller opening");
        assert!(c2 > 0.4, "IY should pull corners");
    }

    #[test]
    fn test_phoneme_to_mouth_shape_bilabial() {
        let (o, _c, p) = phoneme_to_mouth_shape("B");
        assert!((o).abs() < 1e-6, "B should close mouth");
        assert!(p > 0.5, "B should press lips");
    }

    #[test]
    fn test_phoneme_to_mouth_shape_silence() {
        let (o, c, p) = phoneme_to_mouth_shape("SIL");
        assert!((o + c + p).abs() < 1e-6);
    }

    #[test]
    fn test_phonemes_at_time() {
        let mut track = new_lip_sync_track(3.0);
        add_phoneme_event(&mut track, make_event("AA", 0.0, 0.5));
        add_phoneme_event(&mut track, make_event("B", 0.6, 0.3));
        let at_01 = phonemes_at_time(&track, 0.1);
        assert_eq!(at_01.len(), 1);
        assert_eq!(at_01[0].phoneme, "AA");
        let at_05 = phonemes_at_time(&track, 0.55);
        assert!(at_05.is_empty());
    }

    #[test]
    fn test_sort_phoneme_events() {
        let mut track = new_lip_sync_track(3.0);
        add_phoneme_event(&mut track, make_event("B", 0.5, 0.2));
        add_phoneme_event(&mut track, make_event("AA", 0.0, 0.4));
        sort_phoneme_events(&mut track);
        assert!((track.events[0].start_time - 0.0).abs() < 1e-6);
        assert!((track.events[1].start_time - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_trim_lip_sync() {
        let mut track = new_lip_sync_track(5.0);
        add_phoneme_event(&mut track, make_event("AA", 0.0, 0.5));
        add_phoneme_event(&mut track, make_event("B", 1.0, 0.3));
        add_phoneme_event(&mut track, make_event("IY", 3.0, 0.5));
        trim_lip_sync(&mut track, 0.5, 2.0);
        // AA ends at 0.5 so its ev_end == 0.5 which is NOT > start=0.5, should be removed
        assert_eq!(event_count(&track), 1);
        assert_eq!(track.events[0].phoneme, "B");
    }

    #[test]
    fn test_scale_lip_sync_timing() {
        let mut track = new_lip_sync_track(2.0);
        add_phoneme_event(&mut track, make_event("AA", 0.0, 0.5));
        add_phoneme_event(&mut track, make_event("B", 0.5, 0.5));
        scale_lip_sync_timing(&mut track, 2.0);
        assert!((track.duration - 4.0).abs() < 1e-6);
        assert!((track.events[0].duration - 1.0).abs() < 1e-6);
        assert!((track.events[1].start_time - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_merge_lip_sync_tracks() {
        let mut a = new_lip_sync_track(1.0);
        add_phoneme_event(&mut a, make_event("AA", 0.0, 0.5));
        let mut b = new_lip_sync_track(2.0);
        add_phoneme_event(&mut b, make_event("B", 1.0, 0.5));
        add_phoneme_event(&mut b, make_event("IY", 1.5, 0.5));
        let merged = merge_lip_sync_tracks(&a, &b);
        assert_eq!(event_count(&merged), 3);
        assert!((merged.duration - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_viseme_weights_empty() {
        let track = new_lip_sync_track(1.0);
        let weights = lip_sync_to_viseme_weights(&track, 0.5);
        assert!(weights.is_empty());
    }

    #[test]
    fn test_default_coarticulation() {
        let p = default_coarticulation();
        assert!(p.lookahead > 0.0);
        assert!(p.lookbehind >= 0.0);
        assert!(p.smoothing >= 0.0 && p.smoothing <= 1.0);
    }

    #[test]
    fn test_viseme_weights_active() {
        let mut track = new_lip_sync_track(2.0);
        add_phoneme_event(&mut track, make_event("AA", 0.0, 1.0));
        // No next event, so weight=1.0 for "AA"
        let weights = lip_sync_to_viseme_weights(&track, 0.3);
        assert!(!weights.is_empty());
        assert_eq!(weights[0].0, "AA");
    }
}
