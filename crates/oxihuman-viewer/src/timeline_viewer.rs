//! Animation timeline UI widget (purely data/logic, no GPU).

#[allow(dead_code)]
pub struct TimelineTrack {
    pub id: u32,
    pub name: String,
    pub color: [f32; 4],
    pub keys: Vec<f32>,
    pub visible: bool,
    pub muted: bool,
    pub height: f32,
}

#[allow(dead_code)]
pub struct TimelineView {
    pub tracks: Vec<TimelineTrack>,
    pub current_time: f32,
    pub start_time: f32,
    pub end_time: f32,
    pub zoom: f32,
    pub fps: f32,
    pub playing: bool,
    pub next_id: u32,
}

#[allow(dead_code)]
pub struct TimelineSelection {
    pub track_id: u32,
    pub key_indices: Vec<usize>,
    pub time_range: Option<(f32, f32)>,
}

#[allow(dead_code)]
pub fn new_timeline(fps: f32, end_time: f32) -> TimelineView {
    TimelineView {
        tracks: Vec::new(),
        current_time: 0.0,
        start_time: 0.0,
        end_time,
        zoom: 1.0,
        fps,
        playing: false,
        next_id: 1,
    }
}

#[allow(dead_code)]
pub fn add_track(tl: &mut TimelineView, name: &str, color: [f32; 4]) -> u32 {
    let id = tl.next_id;
    tl.next_id += 1;
    tl.tracks.push(TimelineTrack {
        id,
        name: name.to_string(),
        color,
        keys: Vec::new(),
        visible: true,
        muted: false,
        height: 24.0,
    });
    id
}

#[allow(dead_code)]
pub fn remove_track(tl: &mut TimelineView, id: u32) -> bool {
    if let Some(pos) = tl.tracks.iter().position(|t| t.id == id) {
        tl.tracks.remove(pos);
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn add_key_to_track(tl: &mut TimelineView, track_id: u32, time: f32) -> bool {
    if let Some(track) = tl.tracks.iter_mut().find(|t| t.id == track_id) {
        let pos = track.keys.partition_point(|&k| k < time);
        track.keys.insert(pos, time);
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn remove_key_from_track(tl: &mut TimelineView, track_id: u32, key_idx: usize) -> bool {
    if let Some(track) = tl.tracks.iter_mut().find(|t| t.id == track_id) {
        if key_idx < track.keys.len() {
            track.keys.remove(key_idx);
            true
        } else {
            false
        }
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn set_current_time(tl: &mut TimelineView, time: f32) {
    tl.current_time = time.clamp(tl.start_time, tl.end_time);
}

#[allow(dead_code)]
pub fn step_frame(tl: &mut TimelineView) {
    let frame_dur = if tl.fps > 0.0 { 1.0 / tl.fps } else { 0.0 };
    tl.current_time = (tl.current_time + frame_dur).min(tl.end_time);
}

#[allow(dead_code)]
pub fn play(tl: &mut TimelineView) {
    tl.playing = true;
}

#[allow(dead_code)]
pub fn pause(tl: &mut TimelineView) {
    tl.playing = false;
}

#[allow(dead_code)]
pub fn get_track(tl: &TimelineView, id: u32) -> Option<&TimelineTrack> {
    tl.tracks.iter().find(|t| t.id == id)
}

#[allow(dead_code)]
pub fn keys_in_range(tl: &TimelineView, track_id: u32, start: f32, end: f32) -> Vec<usize> {
    if let Some(track) = tl.tracks.iter().find(|t| t.id == track_id) {
        track
            .keys
            .iter()
            .enumerate()
            .filter(|(_, &k)| k >= start && k <= end)
            .map(|(i, _)| i)
            .collect()
    } else {
        Vec::new()
    }
}

#[allow(dead_code)]
pub fn track_count(tl: &TimelineView) -> usize {
    tl.tracks.len()
}

#[allow(dead_code)]
pub fn time_to_frame(fps: f32, time: f32) -> u32 {
    if fps <= 0.0 {
        return 0;
    }
    (time * fps).round() as u32
}

#[allow(dead_code)]
pub fn frame_to_time(fps: f32, frame: u32) -> f32 {
    if fps <= 0.0 {
        return 0.0;
    }
    frame as f32 / fps
}

#[allow(dead_code)]
pub fn zoom_timeline(tl: &mut TimelineView, factor: f32) {
    tl.zoom = (tl.zoom * factor).clamp(0.1, 100.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_timeline() {
        let tl = new_timeline(24.0, 10.0);
        assert!((tl.fps - 24.0).abs() < 1e-6);
        assert!((tl.end_time - 10.0).abs() < 1e-6);
        assert!((tl.current_time).abs() < 1e-6);
        assert!(!tl.playing);
        assert_eq!(tl.tracks.len(), 0);
    }

    #[test]
    fn test_add_track() {
        let mut tl = new_timeline(24.0, 10.0);
        let id = add_track(&mut tl, "Anim", [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(track_count(&tl), 1);
        assert_eq!(id, 1);
        let track = get_track(&tl, id).expect("should succeed");
        assert_eq!(track.name, "Anim");
    }

    #[test]
    fn test_remove_track() {
        let mut tl = new_timeline(24.0, 10.0);
        let id = add_track(&mut tl, "T1", [0.0; 4]);
        assert_eq!(track_count(&tl), 1);
        let removed = remove_track(&mut tl, id);
        assert!(removed);
        assert_eq!(track_count(&tl), 0);
    }

    #[test]
    fn test_remove_nonexistent_track() {
        let mut tl = new_timeline(24.0, 10.0);
        assert!(!remove_track(&mut tl, 999));
    }

    #[test]
    fn test_add_key_to_track() {
        let mut tl = new_timeline(24.0, 10.0);
        let id = add_track(&mut tl, "T", [0.0; 4]);
        assert!(add_key_to_track(&mut tl, id, 1.0));
        assert!(add_key_to_track(&mut tl, id, 2.0));
        let track = get_track(&tl, id).expect("should succeed");
        assert_eq!(track.keys.len(), 2);
    }

    #[test]
    fn test_add_key_sorted() {
        let mut tl = new_timeline(24.0, 10.0);
        let id = add_track(&mut tl, "T", [0.0; 4]);
        add_key_to_track(&mut tl, id, 2.0);
        add_key_to_track(&mut tl, id, 0.5);
        add_key_to_track(&mut tl, id, 1.0);
        let track = get_track(&tl, id).expect("should succeed");
        assert!((track.keys[0] - 0.5).abs() < 1e-6);
        assert!((track.keys[1] - 1.0).abs() < 1e-6);
        assert!((track.keys[2] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_remove_key_from_track() {
        let mut tl = new_timeline(24.0, 10.0);
        let id = add_track(&mut tl, "T", [0.0; 4]);
        add_key_to_track(&mut tl, id, 1.0);
        add_key_to_track(&mut tl, id, 2.0);
        assert!(remove_key_from_track(&mut tl, id, 0));
        let track = get_track(&tl, id).expect("should succeed");
        assert_eq!(track.keys.len(), 1);
    }

    #[test]
    fn test_step_frame_advances_time() {
        let mut tl = new_timeline(24.0, 10.0);
        step_frame(&mut tl);
        let expected = 1.0 / 24.0;
        assert!((tl.current_time - expected).abs() < 1e-5);
    }

    #[test]
    fn test_step_frame_clamps_at_end() {
        let mut tl = new_timeline(24.0, 0.01);
        for _ in 0..100 {
            step_frame(&mut tl);
        }
        assert!(tl.current_time <= tl.end_time + 1e-6);
    }

    #[test]
    fn test_play_pause() {
        let mut tl = new_timeline(24.0, 10.0);
        assert!(!tl.playing);
        play(&mut tl);
        assert!(tl.playing);
        pause(&mut tl);
        assert!(!tl.playing);
    }

    #[test]
    fn test_keys_in_range() {
        let mut tl = new_timeline(24.0, 10.0);
        let id = add_track(&mut tl, "T", [0.0; 4]);
        add_key_to_track(&mut tl, id, 0.5);
        add_key_to_track(&mut tl, id, 1.0);
        add_key_to_track(&mut tl, id, 2.0);
        add_key_to_track(&mut tl, id, 3.0);
        let in_range = keys_in_range(&tl, id, 0.9, 2.1);
        assert_eq!(in_range.len(), 2);
    }

    #[test]
    fn test_time_to_frame() {
        assert_eq!(time_to_frame(24.0, 1.0), 24);
        assert_eq!(time_to_frame(24.0, 0.0), 0);
        assert_eq!(time_to_frame(30.0, 2.0), 60);
    }

    #[test]
    fn test_frame_to_time() {
        let t = frame_to_time(24.0, 24);
        assert!((t - 1.0).abs() < 1e-6);
        let t2 = frame_to_time(30.0, 60);
        assert!((t2 - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_zoom_clamp() {
        let mut tl = new_timeline(24.0, 10.0);
        zoom_timeline(&mut tl, 1000.0);
        assert!((tl.zoom - 100.0).abs() < 1e-6);
        zoom_timeline(&mut tl, 0.0001);
        assert!((tl.zoom - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_set_current_time_clamp() {
        let mut tl = new_timeline(24.0, 5.0);
        set_current_time(&mut tl, -1.0);
        assert!((tl.current_time).abs() < 1e-6);
        set_current_time(&mut tl, 100.0);
        assert!((tl.current_time - 5.0).abs() < 1e-6);
    }
}
