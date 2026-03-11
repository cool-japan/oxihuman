// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Camera animation tracks with keyframes and interpolation.

// ── Structs ──────────────────────────────────────────────────────────────────

/// A single camera keyframe storing position, target, and up vector at a time.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct CameraKeyframe {
    /// Time in seconds.
    pub time: f32,
    /// Eye position.
    pub position: [f32; 3],
    /// Look-at target.
    pub target: [f32; 3],
    /// Up vector.
    pub up: [f32; 3],
    /// Vertical field-of-view in degrees.
    pub fov_deg: f32,
}

/// A collection of camera keyframes forming an animation track.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraTrack {
    /// Keyframes sorted by time.
    pub keyframes: Vec<CameraKeyframe>,
    /// Whether the track loops.
    pub looping: bool,
    /// Human-readable name.
    pub name: String,
}

/// Runtime state for a playing camera animation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraAnimState {
    /// Current playback time in seconds.
    pub current_time: f32,
    /// Playback speed multiplier (1.0 = normal).
    pub speed: f32,
    /// Whether the animation is currently playing.
    pub playing: bool,
}

// ── Constructor ──────────────────────────────────────────────────────────────

/// Create a new empty camera track with the given name.
#[allow(dead_code)]
pub fn new_camera_track(name: &str) -> CameraTrack {
    CameraTrack {
        keyframes: Vec::new(),
        looping: false,
        name: name.to_string(),
    }
}

// ── Keyframe management ──────────────────────────────────────────────────────

/// Add a keyframe to the track, maintaining sorted order by time.
#[allow(dead_code)]
pub fn add_keyframe(track: &mut CameraTrack, kf: CameraKeyframe) {
    let pos = track
        .keyframes
        .binary_search_by(|k| {
            k.time
                .partial_cmp(&kf.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or_else(|e| e);
    track.keyframes.insert(pos, kf);
}

/// Remove the keyframe at `index`.  Does nothing if out of range.
#[allow(dead_code)]
pub fn remove_keyframe(track: &mut CameraTrack, index: usize) {
    if index < track.keyframes.len() {
        track.keyframes.remove(index);
    }
}

/// Return the number of keyframes in the track.
#[allow(dead_code)]
pub fn keyframe_count(track: &CameraTrack) -> usize {
    track.keyframes.len()
}

// ── Sampling / interpolation ─────────────────────────────────────────────────

/// Sample the track at the given time, linearly interpolating between keyframes.
///
/// Returns `None` if the track has no keyframes.
#[allow(dead_code)]
pub fn sample_track(track: &CameraTrack, time: f32) -> Option<CameraKeyframe> {
    if track.keyframes.is_empty() {
        return None;
    }
    if track.keyframes.len() == 1 {
        return Some(track.keyframes[0].clone());
    }

    let effective_time = if track.looping {
        let dur = track_duration(track);
        if dur < 1e-9 {
            0.0
        } else {
            ((time % dur) + dur) % dur
        }
    } else {
        time
    };

    // Before first keyframe
    if effective_time <= track.keyframes[0].time {
        return Some(track.keyframes[0].clone());
    }
    // After last keyframe
    let last = track.keyframes.len() - 1;
    if effective_time >= track.keyframes[last].time {
        return Some(track.keyframes[last].clone());
    }

    // Find the two surrounding keyframes
    for i in 0..last {
        let a = &track.keyframes[i];
        let b = &track.keyframes[i + 1];
        if effective_time >= a.time && effective_time <= b.time {
            let span = b.time - a.time;
            let t = if span < 1e-9 {
                0.0
            } else {
                (effective_time - a.time) / span
            };
            return Some(lerp_keyframe(a, b, t));
        }
    }

    Some(track.keyframes[last].clone())
}

/// Return the total duration of the track (last time - first time).
#[allow(dead_code)]
pub fn track_duration(track: &CameraTrack) -> f32 {
    if track.keyframes.len() < 2 {
        return 0.0;
    }
    let first = track.keyframes[0].time;
    let last = track.keyframes[track.keyframes.len() - 1].time;
    (last - first).max(0.0)
}

// ── Track modifiers ──────────────────────────────────────────────────────────

/// Set whether the track loops.
#[allow(dead_code)]
pub fn set_looping(track: &mut CameraTrack, looping: bool) {
    track.looping = looping;
}

/// Advance the animation state by `dt` seconds.
///
/// Returns the new current time.
#[allow(dead_code)]
pub fn advance_camera_anim(state: &mut CameraAnimState, dt: f32) -> f32 {
    if state.playing {
        state.current_time += dt * state.speed;
    }
    state.current_time
}

/// Return a keyframe representing the camera at the start of the track.
#[allow(dead_code)]
pub fn camera_at_start(track: &CameraTrack) -> Option<CameraKeyframe> {
    track.keyframes.first().cloned()
}

/// Return a keyframe representing the camera at the end of the track.
#[allow(dead_code)]
pub fn camera_at_end(track: &CameraTrack) -> Option<CameraKeyframe> {
    track.keyframes.last().cloned()
}

/// Find the index of the keyframe nearest to the given time.
///
/// Returns `None` if the track is empty.
#[allow(dead_code)]
pub fn nearest_keyframe(track: &CameraTrack, time: f32) -> Option<usize> {
    if track.keyframes.is_empty() {
        return None;
    }
    let mut best_idx = 0;
    let mut best_dist = f32::MAX;
    for (i, kf) in track.keyframes.iter().enumerate() {
        let dist = (kf.time - time).abs();
        if dist < best_dist {
            best_dist = dist;
            best_idx = i;
        }
    }
    Some(best_idx)
}

/// Serialise a camera track to a JSON string.
#[allow(dead_code)]
pub fn track_to_json(track: &CameraTrack) -> String {
    let mut s = String::from("{\n");
    s.push_str(&format!("  \"name\": \"{}\",\n", track.name));
    s.push_str(&format!("  \"looping\": {},\n", track.looping));
    s.push_str("  \"keyframes\": [\n");
    for (i, kf) in track.keyframes.iter().enumerate() {
        s.push_str(&format!(
            "    {{\"time\":{:.6},\"position\":[{:.6},{:.6},{:.6}],\"target\":[{:.6},{:.6},{:.6}],\"fov_deg\":{:.2}}}",
            kf.time,
            kf.position[0], kf.position[1], kf.position[2],
            kf.target[0], kf.target[1], kf.target[2],
            kf.fov_deg,
        ));
        if i + 1 < track.keyframes.len() {
            s.push(',');
        }
        s.push('\n');
    }
    s.push_str("  ]\n}");
    s
}

/// Reverse the order of keyframes in the track, remapping times so the first
/// keyframe starts at the original first-keyframe time.
#[allow(dead_code)]
pub fn reverse_track(track: &mut CameraTrack) {
    if track.keyframes.len() < 2 {
        return;
    }
    let first_time = track.keyframes[0].time;
    let last_time = track.keyframes[track.keyframes.len() - 1].time;
    track.keyframes.reverse();
    for kf in &mut track.keyframes {
        kf.time = first_time + (last_time - kf.time);
    }
    // Re-normalise times relative to first_time
    // (already correct due to the formula above, but clamp for safety)
    for kf in &mut track.keyframes {
        kf.time = kf.time.max(first_time);
    }
}

/// Blend two camera tracks by linearly interpolating each keyframe pair at
/// corresponding indices.  Returns `None` if the tracks have different keyframe
/// counts.
#[allow(dead_code)]
pub fn blend_tracks(a: &CameraTrack, b: &CameraTrack, t: f32) -> Option<CameraTrack> {
    if a.keyframes.len() != b.keyframes.len() {
        return None;
    }
    let t = t.clamp(0.0, 1.0);
    let mut result = new_camera_track(&a.name);
    result.looping = a.looping;
    for (ka, kb) in a.keyframes.iter().zip(b.keyframes.iter()) {
        result.keyframes.push(lerp_keyframe(ka, kb, t));
    }
    Some(result)
}

// ── Private helpers ──────────────────────────────────────────────────────────

fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        lerp_f32(a[0], b[0], t),
        lerp_f32(a[1], b[1], t),
        lerp_f32(a[2], b[2], t),
    ]
}

fn lerp_keyframe(a: &CameraKeyframe, b: &CameraKeyframe, t: f32) -> CameraKeyframe {
    CameraKeyframe {
        time: lerp_f32(a.time, b.time, t),
        position: lerp3(a.position, b.position, t),
        target: lerp3(a.target, b.target, t),
        up: lerp3(a.up, b.up, t),
        fov_deg: lerp_f32(a.fov_deg, b.fov_deg, t),
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_keyframe(time: f32, x: f32) -> CameraKeyframe {
        CameraKeyframe {
            time,
            position: [x, 0.0, 0.0],
            target: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            fov_deg: 60.0,
        }
    }

    #[test]
    fn new_camera_track_is_empty() {
        let track = new_camera_track("test");
        assert!(track.keyframes.is_empty());
        assert!(!track.looping);
        assert_eq!(track.name, "test");
    }

    #[test]
    fn add_keyframe_inserts_sorted() {
        let mut track = new_camera_track("t");
        add_keyframe(&mut track, make_keyframe(2.0, 2.0));
        add_keyframe(&mut track, make_keyframe(0.0, 0.0));
        add_keyframe(&mut track, make_keyframe(1.0, 1.0));
        assert_eq!(keyframe_count(&track), 3);
        assert!((track.keyframes[0].time - 0.0).abs() < 1e-6);
        assert!((track.keyframes[1].time - 1.0).abs() < 1e-6);
        assert!((track.keyframes[2].time - 2.0).abs() < 1e-6);
    }

    #[test]
    fn remove_keyframe_valid() {
        let mut track = new_camera_track("t");
        add_keyframe(&mut track, make_keyframe(0.0, 0.0));
        add_keyframe(&mut track, make_keyframe(1.0, 1.0));
        remove_keyframe(&mut track, 0);
        assert_eq!(keyframe_count(&track), 1);
    }

    #[test]
    fn remove_keyframe_out_of_range() {
        let mut track = new_camera_track("t");
        add_keyframe(&mut track, make_keyframe(0.0, 0.0));
        remove_keyframe(&mut track, 99);
        assert_eq!(keyframe_count(&track), 1);
    }

    #[test]
    fn track_duration_empty() {
        let track = new_camera_track("t");
        assert!((track_duration(&track) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn track_duration_two_keyframes() {
        let mut track = new_camera_track("t");
        add_keyframe(&mut track, make_keyframe(1.0, 0.0));
        add_keyframe(&mut track, make_keyframe(4.0, 0.0));
        assert!((track_duration(&track) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn sample_track_empty_returns_none() {
        let track = new_camera_track("t");
        assert!(sample_track(&track, 0.0).is_none());
    }

    #[test]
    fn sample_track_single_keyframe() {
        let mut track = new_camera_track("t");
        add_keyframe(&mut track, make_keyframe(0.0, 5.0));
        let kf = sample_track(&track, 0.5).unwrap();
        assert!((kf.position[0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn sample_track_interpolation() {
        let mut track = new_camera_track("t");
        add_keyframe(&mut track, make_keyframe(0.0, 0.0));
        add_keyframe(&mut track, make_keyframe(2.0, 10.0));
        let kf = sample_track(&track, 1.0).unwrap();
        assert!((kf.position[0] - 5.0).abs() < 1e-4);
    }

    #[test]
    fn sample_track_before_first() {
        let mut track = new_camera_track("t");
        add_keyframe(&mut track, make_keyframe(1.0, 3.0));
        add_keyframe(&mut track, make_keyframe(2.0, 6.0));
        let kf = sample_track(&track, 0.0).unwrap();
        assert!((kf.position[0] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn set_looping_works() {
        let mut track = new_camera_track("t");
        set_looping(&mut track, true);
        assert!(track.looping);
        set_looping(&mut track, false);
        assert!(!track.looping);
    }

    #[test]
    fn advance_camera_anim_increments() {
        let mut state = CameraAnimState {
            current_time: 0.0,
            speed: 1.0,
            playing: true,
        };
        advance_camera_anim(&mut state, 0.5);
        assert!((state.current_time - 0.5).abs() < 1e-6);
    }

    #[test]
    fn advance_camera_anim_paused() {
        let mut state = CameraAnimState {
            current_time: 1.0,
            speed: 1.0,
            playing: false,
        };
        advance_camera_anim(&mut state, 0.5);
        assert!((state.current_time - 1.0).abs() < 1e-6);
    }

    #[test]
    fn camera_at_start_and_end() {
        let mut track = new_camera_track("t");
        add_keyframe(&mut track, make_keyframe(0.0, 1.0));
        add_keyframe(&mut track, make_keyframe(5.0, 9.0));
        let start = camera_at_start(&track).unwrap();
        let end = camera_at_end(&track).unwrap();
        assert!((start.position[0] - 1.0).abs() < 1e-6);
        assert!((end.position[0] - 9.0).abs() < 1e-6);
    }

    #[test]
    fn nearest_keyframe_finds_closest() {
        let mut track = new_camera_track("t");
        add_keyframe(&mut track, make_keyframe(0.0, 0.0));
        add_keyframe(&mut track, make_keyframe(1.0, 0.0));
        add_keyframe(&mut track, make_keyframe(3.0, 0.0));
        assert_eq!(nearest_keyframe(&track, 0.8), Some(1));
        assert_eq!(nearest_keyframe(&track, 2.5), Some(2));
    }

    #[test]
    fn track_to_json_contains_name() {
        let mut track = new_camera_track("fly_by");
        add_keyframe(&mut track, make_keyframe(0.0, 0.0));
        let json = track_to_json(&track);
        assert!(json.contains("fly_by"));
        assert!(json.contains("keyframes"));
    }

    #[test]
    fn reverse_track_reverses_order() {
        let mut track = new_camera_track("t");
        add_keyframe(&mut track, make_keyframe(0.0, 1.0));
        add_keyframe(&mut track, make_keyframe(1.0, 2.0));
        add_keyframe(&mut track, make_keyframe(2.0, 3.0));
        reverse_track(&mut track);
        // After reversal the first keyframe should have position from the old last
        assert!((track.keyframes[0].position[0] - 3.0).abs() < 1e-6);
        assert!((track.keyframes[2].position[0] - 1.0).abs() < 1e-6);
        // Times should still start at 0.0
        assert!((track.keyframes[0].time - 0.0).abs() < 1e-6);
    }

    #[test]
    fn blend_tracks_at_zero_gives_a() {
        let mut a = new_camera_track("a");
        let mut b = new_camera_track("b");
        add_keyframe(&mut a, make_keyframe(0.0, 0.0));
        add_keyframe(&mut b, make_keyframe(0.0, 10.0));
        let result = blend_tracks(&a, &b, 0.0).unwrap();
        assert!((result.keyframes[0].position[0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn blend_tracks_mismatched_returns_none() {
        let mut a = new_camera_track("a");
        let b = new_camera_track("b");
        add_keyframe(&mut a, make_keyframe(0.0, 0.0));
        assert!(blend_tracks(&a, &b, 0.5).is_none());
    }
}
