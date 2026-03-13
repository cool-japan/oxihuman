// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Animation playback controller.

#[allow(dead_code)]
pub struct AnimationState {
    pub current_time: f32,
    pub speed: f32,
    pub looping: bool,
    pub playing: bool,
}

#[allow(dead_code)]
pub fn new_animation_state(_duration_hint: f32) -> AnimationState {
    AnimationState { current_time: 0.0, speed: 1.0, looping: false, playing: false }
}

#[allow(dead_code)]
pub fn ap_play(s: &mut AnimationState) {
    s.playing = true;
}

#[allow(dead_code)]
pub fn ap_stop(s: &mut AnimationState) {
    s.playing = false;
}

#[allow(dead_code)]
pub fn ap_step(s: &mut AnimationState, dt: f32, duration: f32) {
    if !s.playing {
        return;
    }
    s.current_time += dt * s.speed;
    if duration > 1e-7 {
        if s.looping {
            s.current_time = s.current_time.rem_euclid(duration);
        } else if s.current_time >= duration {
            s.current_time = duration;
            s.playing = false;
        }
    }
}

#[allow(dead_code)]
pub fn ap_current_time(s: &AnimationState) -> f32 {
    s.current_time
}

#[allow(dead_code)]
pub fn ap_set_speed(s: &mut AnimationState, speed: f32) {
    s.speed = speed;
}

#[allow(dead_code)]
pub fn ap_is_playing(s: &AnimationState) -> bool {
    s.playing
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_play() {
        let mut s = new_animation_state(1.0);
        ap_play(&mut s);
        assert!(ap_is_playing(&s));
    }

    #[test]
    fn test_stop() {
        let mut s = new_animation_state(1.0);
        ap_play(&mut s);
        ap_stop(&mut s);
        assert!(!ap_is_playing(&s));
    }

    #[test]
    fn test_step_advances() {
        let mut s = new_animation_state(2.0);
        s.looping = true;
        ap_play(&mut s);
        ap_step(&mut s, 0.5, 2.0);
        assert!((ap_current_time(&s) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_step_not_playing() {
        let mut s = new_animation_state(2.0);
        ap_step(&mut s, 1.0, 2.0);
        assert!((ap_current_time(&s) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_looping_wraps() {
        let mut s = new_animation_state(1.0);
        s.looping = true;
        ap_play(&mut s);
        ap_step(&mut s, 1.5, 1.0);
        assert!(ap_current_time(&s) < 1.0);
        assert!(ap_is_playing(&s));
    }

    #[test]
    fn test_non_looping_stops() {
        let mut s = new_animation_state(1.0);
        ap_play(&mut s);
        ap_step(&mut s, 2.0, 1.0);
        assert!(!ap_is_playing(&s));
    }

    #[test]
    fn test_set_speed() {
        let mut s = new_animation_state(1.0);
        ap_set_speed(&mut s, 2.0);
        ap_play(&mut s);
        ap_step(&mut s, 0.5, 10.0);
        assert!((ap_current_time(&s) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_is_playing_default_false() {
        let s = new_animation_state(1.0);
        assert!(!ap_is_playing(&s));
    }
}
