#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackState { Stopped, Playing, Paused }

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphPlayback {
    state: PlaybackState,
    time: f32,
    duration: f32,
    speed: f32,
}

#[allow(dead_code)]
pub fn new_morph_playback(duration: f32) -> MorphPlayback {
    MorphPlayback { state: PlaybackState::Stopped, time: 0.0, duration: duration.max(0.01), speed: 1.0 }
}

#[allow(dead_code)]
pub fn playback_play(pb: &mut MorphPlayback) { pb.state = PlaybackState::Playing; }

#[allow(dead_code)]
pub fn playback_pause(pb: &mut MorphPlayback) { pb.state = PlaybackState::Paused; }

#[allow(dead_code)]
pub fn playback_stop(pb: &mut MorphPlayback) { pb.state = PlaybackState::Stopped; pb.time = 0.0; }

#[allow(dead_code)]
pub fn playback_seek(pb: &mut MorphPlayback, time: f32) { pb.time = time.clamp(0.0, pb.duration); }

#[allow(dead_code)]
pub fn playback_progress(pb: &MorphPlayback) -> f32 { pb.time / pb.duration }

#[allow(dead_code)]
pub fn playback_to_json(pb: &MorphPlayback) -> String {
    format!("{{\"time\":{:.4},\"duration\":{:.4},\"progress\":{:.4}}}", pb.time, pb.duration, playback_progress(pb))
}

#[allow(dead_code)]
pub fn playback_state(pb: &MorphPlayback) -> &PlaybackState { &pb.state }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let p = new_morph_playback(2.0); assert_eq!(*playback_state(&p), PlaybackState::Stopped); }
    #[test] fn test_play() { let mut p = new_morph_playback(1.0); playback_play(&mut p); assert_eq!(*playback_state(&p), PlaybackState::Playing); }
    #[test] fn test_pause() { let mut p = new_morph_playback(1.0); playback_play(&mut p); playback_pause(&mut p); assert_eq!(*playback_state(&p), PlaybackState::Paused); }
    #[test] fn test_stop() { let mut p = new_morph_playback(1.0); playback_play(&mut p); playback_stop(&mut p); assert_eq!(*playback_state(&p), PlaybackState::Stopped); }
    #[test] fn test_seek() { let mut p = new_morph_playback(2.0); playback_seek(&mut p, 1.0); assert!((playback_progress(&p) - 0.5).abs() < 1e-6); }
    #[test] fn test_seek_clamp() { let mut p = new_morph_playback(1.0); playback_seek(&mut p, 5.0); assert!((playback_progress(&p) - 1.0).abs() < 1e-6); }
    #[test] fn test_progress_zero() { let p = new_morph_playback(1.0); assert!(playback_progress(&p) < 1e-6); }
    #[test] fn test_json() { let p = new_morph_playback(1.0); assert!(playback_to_json(&p).contains("duration")); }
    #[test] fn test_stop_resets() { let mut p = new_morph_playback(1.0); playback_seek(&mut p, 0.5); playback_stop(&mut p); assert!(playback_progress(&p) < 1e-6); }
    #[test] fn test_min_duration() { let p = new_morph_playback(-1.0); assert!(p.duration >= 0.01); }
}
