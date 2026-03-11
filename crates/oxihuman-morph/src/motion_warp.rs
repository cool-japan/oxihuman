// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Motion time-warping, speed scaling, and blend between motion clips.

#[allow(dead_code)]
pub struct MotionFrame {
    pub time: f32,
    pub pose: Vec<f32>,
}

#[allow(dead_code)]
pub struct MotionClip {
    pub name: String,
    pub frames: Vec<MotionFrame>,
    pub fps: f32,
}

#[allow(dead_code)]
pub struct WarpCurve {
    pub keys: Vec<(f32, f32)>,
}

#[allow(dead_code)]
pub enum WarpMode {
    Linear,
    Bezier,
    Hold,
}

#[allow(dead_code)]
pub struct WarpedClip {
    pub frames: Vec<MotionFrame>,
    pub original_duration: f32,
    pub warped_duration: f32,
}

#[allow(dead_code)]
pub fn clip_duration(clip: &MotionClip) -> f32 {
    if clip.frames.is_empty() {
        return 0.0;
    }
    clip.frames[clip.frames.len() - 1].time - clip.frames[0].time
}

#[allow(dead_code)]
pub fn sample_clip(clip: &MotionClip, time: f32) -> Vec<f32> {
    if clip.frames.is_empty() {
        return Vec::new();
    }
    if clip.frames.len() == 1 {
        return clip.frames[0].pose.clone();
    }
    let first = &clip.frames[0];
    let last = &clip.frames[clip.frames.len() - 1];
    if time <= first.time {
        return first.pose.clone();
    }
    if time >= last.time {
        return last.pose.clone();
    }
    // find surrounding frames
    let mut lo = 0usize;
    let mut hi = clip.frames.len() - 1;
    while lo + 1 < hi {
        let mid = (lo + hi) / 2;
        if clip.frames[mid].time <= time {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let fa = &clip.frames[lo];
    let fb = &clip.frames[hi];
    let span = fb.time - fa.time;
    let t = if span.abs() < 1e-9 {
        0.0
    } else {
        (time - fa.time) / span
    };
    pose_lerp(&fa.pose, &fb.pose, t)
}

#[allow(dead_code)]
pub fn warp_time(curve: &WarpCurve, t: f32) -> f32 {
    if curve.keys.is_empty() {
        return t;
    }
    if curve.keys.len() == 1 {
        return curve.keys[0].1;
    }
    let first = curve.keys[0];
    let last = curve.keys[curve.keys.len() - 1];
    if t <= first.0 {
        return first.1;
    }
    if t >= last.0 {
        return last.1;
    }
    let mut lo = 0usize;
    let mut hi = curve.keys.len() - 1;
    while lo + 1 < hi {
        let mid = (lo + hi) / 2;
        if curve.keys[mid].0 <= t {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let (t0, v0) = curve.keys[lo];
    let (t1, v1) = curve.keys[hi];
    let span = t1 - t0;
    let alpha = if span.abs() < 1e-9 {
        0.0
    } else {
        (t - t0) / span
    };
    v0 + (v1 - v0) * alpha
}

#[allow(dead_code)]
pub fn apply_warp(clip: &MotionClip, curve: &WarpCurve, output_fps: f32) -> WarpedClip {
    let orig_duration = clip_duration(clip);
    if clip.frames.is_empty() || output_fps <= 0.0 {
        return WarpedClip {
            frames: Vec::new(),
            original_duration: orig_duration,
            warped_duration: 0.0,
        };
    }
    let warped_end = warp_time(curve, orig_duration);
    let warped_duration = warped_end;
    let frame_count = ((warped_duration * output_fps).round() as usize).max(1);
    let mut frames = Vec::with_capacity(frame_count);
    for i in 0..frame_count {
        let warped_t = i as f32 / output_fps;
        let orig_t = warp_time(curve, warped_t);
        let pose = sample_clip(clip, orig_t);
        frames.push(MotionFrame {
            time: warped_t,
            pose,
        });
    }
    WarpedClip {
        frames,
        original_duration: orig_duration,
        warped_duration,
    }
}

#[allow(dead_code)]
pub fn speed_scale_clip(clip: &MotionClip, factor: f32) -> WarpedClip {
    let orig_duration = clip_duration(clip);
    let factor = factor.max(1e-6);
    let warped_duration = orig_duration / factor;
    let output_fps = clip.fps.max(1.0);
    let frame_count = ((warped_duration * output_fps).round() as usize).max(1);
    let mut frames = Vec::with_capacity(frame_count);
    for i in 0..frame_count {
        let warped_t = i as f32 / output_fps;
        let orig_t = warped_t * factor;
        let pose = sample_clip(clip, orig_t);
        frames.push(MotionFrame {
            time: warped_t,
            pose,
        });
    }
    WarpedClip {
        frames,
        original_duration: orig_duration,
        warped_duration,
    }
}

#[allow(dead_code)]
pub fn reverse_clip(clip: &MotionClip) -> MotionClip {
    let duration = clip_duration(clip);
    let frames: Vec<MotionFrame> = clip
        .frames
        .iter()
        .rev()
        .map(|f| MotionFrame {
            time: duration - f.time,
            pose: f.pose.clone(),
        })
        .collect();
    MotionClip {
        name: format!("{}_reversed", clip.name),
        frames,
        fps: clip.fps,
    }
}

#[allow(dead_code)]
pub fn trim_clip(clip: &MotionClip, start: f32, end: f32) -> MotionClip {
    let frames: Vec<MotionFrame> = clip
        .frames
        .iter()
        .filter(|f| f.time >= start && f.time <= end)
        .map(|f| MotionFrame {
            time: f.time - start,
            pose: f.pose.clone(),
        })
        .collect();
    MotionClip {
        name: clip.name.clone(),
        frames,
        fps: clip.fps,
    }
}

#[allow(dead_code)]
pub fn blend_clips(
    a: &MotionClip,
    b: &MotionClip,
    blend_weight: f32,
    output_fps: f32,
) -> MotionClip {
    let t = blend_weight.clamp(0.0, 1.0);
    let dur_a = clip_duration(a);
    let dur_b = clip_duration(b);
    let duration = dur_a * (1.0 - t) + dur_b * t;
    let fps = output_fps.max(1.0);
    let frame_count = ((duration * fps).round() as usize).max(1);
    let mut frames = Vec::with_capacity(frame_count);
    for i in 0..frame_count {
        let time = i as f32 / fps;
        let pa = sample_clip(a, time);
        let pb = sample_clip(b, time);
        let pose = pose_lerp(&pa, &pb, t);
        frames.push(MotionFrame { time, pose });
    }
    MotionClip {
        name: format!("blend_{}_{}", a.name, b.name),
        frames,
        fps,
    }
}

#[allow(dead_code)]
pub fn concat_clips(clips: &[MotionClip]) -> MotionClip {
    let mut frames = Vec::new();
    let mut offset = 0.0f32;
    let fps = clips.first().map(|c| c.fps).unwrap_or(30.0);
    for clip in clips {
        for f in &clip.frames {
            frames.push(MotionFrame {
                time: f.time + offset,
                pose: f.pose.clone(),
            });
        }
        offset += clip_duration(clip);
    }
    MotionClip {
        name: "concat".to_string(),
        frames,
        fps,
    }
}

#[allow(dead_code)]
pub fn loop_clip(clip: &MotionClip, loops: u32) -> MotionClip {
    if loops == 0 {
        return MotionClip {
            name: clip.name.clone(),
            frames: Vec::new(),
            fps: clip.fps,
        };
    }
    let duration = clip_duration(clip);
    let mut frames = Vec::new();
    for l in 0..loops {
        let offset = duration * l as f32;
        for f in &clip.frames {
            frames.push(MotionFrame {
                time: f.time + offset,
                pose: f.pose.clone(),
            });
        }
    }
    MotionClip {
        name: format!("{}_loop{}", clip.name, loops),
        frames,
        fps: clip.fps,
    }
}

#[allow(dead_code)]
pub fn identity_warp_curve() -> WarpCurve {
    WarpCurve {
        keys: vec![(0.0, 0.0), (1.0, 1.0)],
    }
}

#[allow(dead_code)]
pub fn linear_warp_curve(speed_factor: f32) -> WarpCurve {
    let factor = speed_factor.max(1e-6);
    WarpCurve {
        keys: vec![(0.0, 0.0), (1.0, 1.0 / factor)],
    }
}

#[allow(dead_code)]
pub fn pose_lerp(a: &[f32], b: &[f32], t: f32) -> Vec<f32> {
    let len = a.len().min(b.len());
    let mut out = Vec::with_capacity(len);
    for i in 0..len {
        out.push(a[i] + (b[i] - a[i]) * t);
    }
    out
}

#[allow(dead_code)]
fn make_test_clip(name: &str, frames: usize, fps: f32, joints: usize) -> MotionClip {
    let dt = 1.0 / fps;
    MotionClip {
        name: name.to_string(),
        frames: (0..frames)
            .map(|i| MotionFrame {
                time: i as f32 * dt,
                pose: vec![i as f32; joints],
            })
            .collect(),
        fps,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip_duration() {
        let clip = make_test_clip("test", 31, 30.0, 4);
        let d = clip_duration(&clip);
        assert!((d - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_clip_duration_empty() {
        let clip = MotionClip {
            name: "empty".to_string(),
            frames: Vec::new(),
            fps: 30.0,
        };
        assert_eq!(clip_duration(&clip), 0.0);
    }

    #[test]
    fn test_sample_at_t0_gives_first_frame() {
        let clip = make_test_clip("test", 10, 30.0, 3);
        let pose = sample_clip(&clip, 0.0);
        assert_eq!(pose, vec![0.0_f32, 0.0, 0.0]);
    }

    #[test]
    fn test_sample_at_end_gives_last_frame() {
        let clip = make_test_clip("test", 5, 30.0, 2);
        let d = clip_duration(&clip);
        let pose = sample_clip(&clip, d);
        assert_eq!(pose, vec![4.0_f32, 4.0]);
    }

    #[test]
    fn test_speed_scale_halves_duration() {
        let clip = make_test_clip("test", 31, 30.0, 2);
        let orig_dur = clip_duration(&clip);
        let warped = speed_scale_clip(&clip, 2.0);
        assert!((warped.warped_duration - orig_dur / 2.0).abs() < 0.05);
    }

    #[test]
    fn test_speed_scale_doubles_duration() {
        let clip = make_test_clip("test", 31, 30.0, 2);
        let orig_dur = clip_duration(&clip);
        let warped = speed_scale_clip(&clip, 0.5);
        assert!((warped.warped_duration - orig_dur * 2.0).abs() < 0.05);
    }

    #[test]
    fn test_reverse_clip_inverts() {
        let clip = make_test_clip("test", 5, 30.0, 1);
        let rev = reverse_clip(&clip);
        assert_eq!(rev.frames.len(), 5);
        // Last frame of original becomes first of reversed
        let orig_last_pose = clip.frames.last().unwrap().pose.clone();
        assert!((rev.frames[0].time).abs() < 1e-4);
        // original last frame value
        assert!((rev.frames[0].pose[0] - orig_last_pose[0]).abs() < 1e-4);
    }

    #[test]
    fn test_trim_clip_shrinks() {
        let clip = make_test_clip("test", 31, 30.0, 2);
        let trimmed = trim_clip(&clip, 0.0, 0.5);
        let dur = clip_duration(&trimmed);
        assert!(dur <= 0.5 + 0.04);
    }

    #[test]
    fn test_blend_identical_clips() {
        let a = make_test_clip("a", 10, 30.0, 3);
        let b = make_test_clip("b", 10, 30.0, 3);
        let blended = blend_clips(&a, &b, 0.5, 30.0);
        let pa = sample_clip(&a, 0.0);
        let pb = sample_clip(&blended, 0.0);
        for (va, vb) in pa.iter().zip(pb.iter()) {
            assert!((va - vb).abs() < 1e-4);
        }
    }

    #[test]
    fn test_concat_clips() {
        let a = make_test_clip("a", 31, 30.0, 2);
        let b = make_test_clip("b", 31, 30.0, 2);
        let dur_a = clip_duration(&a);
        let dur_b = clip_duration(&b);
        let cat = concat_clips(&[a, b]);
        let dur = clip_duration(&cat);
        assert!((dur - (dur_a + dur_b)).abs() < 0.05);
    }

    #[test]
    fn test_loop_doubles_duration() {
        let clip = make_test_clip("test", 31, 30.0, 2);
        let orig_dur = clip_duration(&clip);
        let looped = loop_clip(&clip, 2);
        let loop_dur = clip_duration(&looped);
        assert!((loop_dur - orig_dur * 2.0).abs() < 0.05);
    }

    #[test]
    fn test_loop_zero() {
        let clip = make_test_clip("test", 5, 30.0, 2);
        let looped = loop_clip(&clip, 0);
        assert!(looped.frames.is_empty());
    }

    #[test]
    fn test_identity_warp() {
        let curve = identity_warp_curve();
        assert!((warp_time(&curve, 0.5) - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_linear_warp_curve() {
        let curve = linear_warp_curve(2.0);
        // at t=1, output = 0.5
        assert!((warp_time(&curve, 1.0) - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_pose_lerp() {
        let a = vec![0.0_f32, 0.0, 0.0];
        let b = vec![2.0_f32, 4.0, 6.0];
        let result = pose_lerp(&a, &b, 0.5);
        assert!((result[0] - 1.0).abs() < 1e-5);
        assert!((result[1] - 2.0).abs() < 1e-5);
        assert!((result[2] - 3.0).abs() < 1e-5);
    }
}
