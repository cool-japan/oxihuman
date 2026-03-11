// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! Sequence player: plays a morph animation sequence.

#[allow(dead_code)]
pub struct MorphSequenceFrame {
    pub weight: f32,
    pub duration: f32,
}

#[allow(dead_code)]
pub struct MorphSequencePlayer {
    pub frames: Vec<MorphSequenceFrame>,
    pub current_time: f32,
    pub looping: bool,
    pub playing: bool,
}

#[allow(dead_code)]
pub fn new_morph_sequence_player(looping: bool) -> MorphSequencePlayer {
    MorphSequencePlayer { frames: Vec::new(), current_time: 0.0, looping, playing: false }
}

#[allow(dead_code)]
pub fn msp_add_frame(p: &mut MorphSequencePlayer, weight: f32, duration: f32) {
    p.frames.push(MorphSequenceFrame { weight, duration });
}

#[allow(dead_code)]
pub fn msp_total_duration(p: &MorphSequencePlayer) -> f32 {
    p.frames.iter().map(|f| f.duration).sum()
}

#[allow(dead_code)]
pub fn msp_step(p: &mut MorphSequencePlayer, dt: f32) {
    if !p.playing {
        return;
    }
    p.current_time += dt;
    let total = msp_total_duration(p);
    if total < 1e-7 {
        return;
    }
    if p.looping {
        p.current_time = p.current_time.rem_euclid(total);
    } else if p.current_time >= total {
        p.current_time = total;
        p.playing = false;
    }
}

#[allow(dead_code)]
pub fn msp_current_weight(p: &MorphSequencePlayer) -> f32 {
    if p.frames.is_empty() {
        return 0.0;
    }
    let mut t = 0.0f32;
    for f in &p.frames {
        t += f.duration;
        if p.current_time <= t {
            return f.weight;
        }
    }
    p.frames.last().map(|f| f.weight).unwrap_or(0.0)
}

#[allow(dead_code)]
pub fn msp_play(p: &mut MorphSequencePlayer) {
    p.playing = true;
}

#[allow(dead_code)]
pub fn msp_stop(p: &mut MorphSequencePlayer) {
    p.playing = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_frame() {
        let mut p = new_morph_sequence_player(false);
        msp_add_frame(&mut p, 1.0, 0.5);
        assert_eq!(p.frames.len(), 1);
    }

    #[test]
    fn test_total_duration() {
        let mut p = new_morph_sequence_player(false);
        msp_add_frame(&mut p, 0.0, 0.3);
        msp_add_frame(&mut p, 1.0, 0.7);
        assert!((msp_total_duration(&p) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_play_stop() {
        let mut p = new_morph_sequence_player(false);
        msp_play(&mut p);
        assert!(p.playing);
        msp_stop(&mut p);
        assert!(!p.playing);
    }

    #[test]
    fn test_step_advances() {
        let mut p = new_morph_sequence_player(false);
        msp_add_frame(&mut p, 1.0, 2.0);
        msp_play(&mut p);
        msp_step(&mut p, 0.5);
        assert!((p.current_time - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_step_stops_at_end() {
        let mut p = new_morph_sequence_player(false);
        msp_add_frame(&mut p, 1.0, 1.0);
        msp_play(&mut p);
        msp_step(&mut p, 2.0);
        assert!(!p.playing);
    }

    #[test]
    fn test_step_loops() {
        let mut p = new_morph_sequence_player(true);
        msp_add_frame(&mut p, 1.0, 1.0);
        msp_play(&mut p);
        msp_step(&mut p, 1.5);
        assert!(p.playing);
        assert!(p.current_time < 1.0);
    }

    #[test]
    fn test_current_weight() {
        let mut p = new_morph_sequence_player(false);
        msp_add_frame(&mut p, 0.5, 1.0);
        assert!((msp_current_weight(&p) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_step_not_playing() {
        let mut p = new_morph_sequence_player(false);
        msp_add_frame(&mut p, 1.0, 2.0);
        msp_step(&mut p, 1.0);
        assert!((p.current_time - 0.0).abs() < 1e-5);
    }
}
