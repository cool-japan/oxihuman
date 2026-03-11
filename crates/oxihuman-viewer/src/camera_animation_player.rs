#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraAnimPlayer {
    playing: bool,
    time: f32,
    duration: f32,
    speed: f32,
}

#[allow(dead_code)]
pub fn new_camera_anim_player(duration: f32) -> CameraAnimPlayer {
    CameraAnimPlayer { playing: false, time: 0.0, duration: duration.max(0.01), speed: 1.0 }
}

#[allow(dead_code)]
pub fn play_camera_anim(p: &mut CameraAnimPlayer) { p.playing = true; }

#[allow(dead_code)]
pub fn pause_anim(p: &mut CameraAnimPlayer) { p.playing = false; }

#[allow(dead_code)]
pub fn stop_anim(p: &mut CameraAnimPlayer) { p.playing = false; p.time = 0.0; }

#[allow(dead_code)]
pub fn anim_progress(p: &CameraAnimPlayer) -> f32 { p.time / p.duration }

#[allow(dead_code)]
pub fn anim_is_playing(p: &CameraAnimPlayer) -> bool { p.playing }

#[allow(dead_code)]
pub fn anim_to_json(p: &CameraAnimPlayer) -> String {
    format!("{{\"time\":{:.4},\"duration\":{:.4},\"playing\":{}}}", p.time, p.duration, p.playing)
}

#[allow(dead_code)]
pub fn anim_reset(p: &mut CameraAnimPlayer) { p.playing = false; p.time = 0.0; }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let p = new_camera_anim_player(2.0); assert!(!anim_is_playing(&p)); }
    #[test] fn test_play() { let mut p = new_camera_anim_player(1.0); play_camera_anim(&mut p); assert!(anim_is_playing(&p)); }
    #[test] fn test_pause() { let mut p = new_camera_anim_player(1.0); play_camera_anim(&mut p); pause_anim(&mut p); assert!(!anim_is_playing(&p)); }
    #[test] fn test_stop() { let mut p = new_camera_anim_player(1.0); play_camera_anim(&mut p); p.time = 0.5; stop_anim(&mut p); assert!(anim_progress(&p) < 1e-6); }
    #[test] fn test_progress() { let mut p = new_camera_anim_player(2.0); p.time = 1.0; assert!((anim_progress(&p) - 0.5).abs() < 1e-6); }
    #[test] fn test_json() { let p = new_camera_anim_player(1.0); assert!(anim_to_json(&p).contains("duration")); }
    #[test] fn test_reset() { let mut p = new_camera_anim_player(1.0); p.time = 0.8; anim_reset(&mut p); assert!(anim_progress(&p) < 1e-6); }
    #[test] fn test_min_duration() { let p = new_camera_anim_player(-1.0); assert!(p.duration >= 0.01); }
    #[test] fn test_initial_progress() { let p = new_camera_anim_player(1.0); assert!(anim_progress(&p) < 1e-6); }
    #[test] fn test_not_playing_init() { let p = new_camera_anim_player(5.0); assert!(!anim_is_playing(&p)); }
}
