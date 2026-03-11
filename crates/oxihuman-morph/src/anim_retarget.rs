// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::interpolate::{Keyframe, MorphTrack};
use crate::params::ParamState;

/// A mapping entry: scale one parameter by a factor and optionally offset it.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ParamRetarget {
    /// Factor to multiply the parameter value (default 1.0).
    pub scale: f32,
    /// Offset to add after scaling (default 0.0).
    pub offset: f32,
    /// Clamp result to [0, 1] range (default true).
    pub clamp: bool,
}

impl ParamRetarget {
    /// Identity mapping: scale=1.0, offset=0.0, clamp=true.
    #[allow(dead_code)]
    pub fn identity() -> Self {
        Self {
            scale: 1.0,
            offset: 0.0,
            clamp: true,
        }
    }

    /// Scaled mapping: scale=factor, offset=0.0, clamp=true.
    #[allow(dead_code)]
    pub fn scaled(factor: f32) -> Self {
        Self {
            scale: factor,
            offset: 0.0,
            clamp: true,
        }
    }

    /// Apply the retarget to a single value.
    #[allow(dead_code)]
    pub fn apply(&self, value: f32) -> f32 {
        let result = value * self.scale + self.offset;
        if self.clamp {
            result.clamp(0.0, 1.0)
        } else {
            result
        }
    }
}

/// Configuration for retargeting a full `ParamState`.
#[allow(dead_code)]
pub struct AnimRetargetConfig {
    pub height: ParamRetarget,
    pub weight: ParamRetarget,
    pub muscle: ParamRetarget,
    pub age: ParamRetarget,
}

impl AnimRetargetConfig {
    /// All identity (no-op).
    #[allow(dead_code)]
    pub fn identity() -> Self {
        Self {
            height: ParamRetarget::identity(),
            weight: ParamRetarget::identity(),
            muscle: ParamRetarget::identity(),
            age: ParamRetarget::identity(),
        }
    }

    /// Apply the retarget config to a single `ParamState`.
    #[allow(dead_code)]
    pub fn apply(&self, state: &ParamState) -> ParamState {
        ParamState {
            height: self.height.apply(state.height),
            weight: self.weight.apply(state.weight),
            muscle: self.muscle.apply(state.muscle),
            age: self.age.apply(state.age),
            extra: state.extra.clone(),
        }
    }
}

/// Retarget a single keyframe.
#[allow(dead_code)]
pub fn retarget_keyframe(kf: &Keyframe, config: &AnimRetargetConfig) -> Keyframe {
    Keyframe {
        time: kf.time,
        params: config.apply(&kf.params),
        label: kf.label.clone(),
    }
}

/// Retarget an entire `MorphTrack`.
#[allow(dead_code)]
pub fn retarget_track(track: &MorphTrack, config: &AnimRetargetConfig) -> MorphTrack {
    let mut new_track = MorphTrack::new(track.name.clone());
    for kf in track.keyframes_iter() {
        new_track.add_keyframe(retarget_keyframe(kf, config));
    }
    new_track
}

/// Scale animation duration: compress/expand timeline by a factor.
/// All keyframe times are multiplied by `time_scale`.
#[allow(dead_code)]
pub fn scale_track_time(track: &MorphTrack, time_scale: f32) -> MorphTrack {
    let mut new_track = MorphTrack::new(track.name.clone());
    for kf in track.keyframes_iter() {
        new_track.add_keyframe(Keyframe {
            time: kf.time * time_scale,
            params: kf.params.clone(),
            label: kf.label.clone(),
        });
    }
    new_track
}

/// Trim a track to a time window [start_time, end_time].
/// Keyframes outside the window are removed. Times are not shifted.
#[allow(dead_code)]
pub fn trim_track(track: &MorphTrack, start_time: f32, end_time: f32) -> MorphTrack {
    let mut new_track = MorphTrack::new(track.name.clone());
    for kf in track.keyframes_iter() {
        if kf.time >= start_time && kf.time <= end_time {
            new_track.add_keyframe(kf.clone());
        }
    }
    new_track
}

/// Reverse a track (flip time: new_time = duration - old_time).
#[allow(dead_code)]
pub fn reverse_track(track: &MorphTrack) -> MorphTrack {
    let mut new_track = MorphTrack::new(track.name.clone());
    if track.is_empty() {
        return new_track;
    }
    let kfs: Vec<&Keyframe> = track.keyframes_iter().collect();
    let last_time = kfs[kfs.len() - 1].time;
    for kf in kfs {
        new_track.add_keyframe(Keyframe {
            time: last_time - kf.time,
            params: kf.params.clone(),
            label: kf.label.clone(),
        });
    }
    new_track
}

/// Concatenate two tracks. The second track's times are offset by the first track's last keyframe time.
#[allow(dead_code)]
pub fn concat_tracks(first: &MorphTrack, second: &MorphTrack) -> MorphTrack {
    let mut new_track = MorphTrack::new(first.name.clone());

    for kf in first.keyframes_iter() {
        new_track.add_keyframe(kf.clone());
    }

    let offset = first
        .keyframes_iter()
        .last()
        .map(|kf| kf.time)
        .unwrap_or(0.0);

    for kf in second.keyframes_iter() {
        new_track.add_keyframe(Keyframe {
            time: kf.time + offset,
            params: kf.params.clone(),
            label: kf.label.clone(),
        });
    }

    new_track
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpolate::{Keyframe, MorphTrack};
    use crate::params::ParamState;

    fn make_params(h: f32, w: f32, m: f32, a: f32) -> ParamState {
        ParamState::new(h, w, m, a)
    }

    fn make_track_two_kf() -> MorphTrack {
        let mut track = MorphTrack::new("test");
        track.add_keyframe(Keyframe::new(0.0, make_params(0.2, 0.3, 0.4, 0.5)));
        track.add_keyframe(Keyframe::new(1.0, make_params(0.6, 0.7, 0.8, 0.9)));
        track
    }

    #[test]
    fn param_retarget_identity_unchanged() {
        let r = ParamRetarget::identity();
        assert!((r.apply(0.7) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn param_retarget_scaled_doubles() {
        let r = ParamRetarget::scaled(2.0);
        // 0.3 * 2.0 = 0.6 (within [0,1], no clamp needed)
        assert!((r.apply(0.3) - 0.6).abs() < 1e-6);
    }

    #[test]
    fn param_retarget_clamps_above_one() {
        let r = ParamRetarget::scaled(3.0);
        // 0.8 * 3.0 = 2.4, clamped to 1.0
        assert!((r.apply(0.8) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn param_retarget_clamps_below_zero() {
        let r = ParamRetarget {
            scale: 1.0,
            offset: -0.5,
            clamp: true,
        };
        // 0.2 * 1.0 - 0.5 = -0.3, clamped to 0.0
        assert!((r.apply(0.2) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn anim_retarget_config_identity_preserves_params() {
        let config = AnimRetargetConfig::identity();
        let state = make_params(0.1, 0.2, 0.3, 0.4);
        let result = config.apply(&state);
        assert!((result.height - 0.1).abs() < 1e-6);
        assert!((result.weight - 0.2).abs() < 1e-6);
        assert!((result.muscle - 0.3).abs() < 1e-6);
        assert!((result.age - 0.4).abs() < 1e-6);
    }

    #[test]
    fn anim_retarget_config_scale_weight() {
        let config = AnimRetargetConfig {
            height: ParamRetarget::identity(),
            weight: ParamRetarget::scaled(0.5),
            muscle: ParamRetarget::identity(),
            age: ParamRetarget::identity(),
        };
        let state = make_params(0.4, 0.8, 0.6, 0.5);
        let result = config.apply(&state);
        assert!((result.height - 0.4).abs() < 1e-6);
        // 0.8 * 0.5 = 0.4
        assert!((result.weight - 0.4).abs() < 1e-6);
    }

    #[test]
    fn retarget_keyframe_applies_config() {
        let config = AnimRetargetConfig {
            height: ParamRetarget::scaled(0.5),
            weight: ParamRetarget::identity(),
            muscle: ParamRetarget::identity(),
            age: ParamRetarget::identity(),
        };
        let kf = Keyframe::new(2.5, make_params(1.0, 0.5, 0.5, 0.5));
        let result = retarget_keyframe(&kf, &config);
        assert!((result.time - 2.5).abs() < 1e-6);
        // 1.0 * 0.5 = 0.5
        assert!((result.params.height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn retarget_track_preserves_length() {
        let track = make_track_two_kf();
        let config = AnimRetargetConfig::identity();
        let result = retarget_track(&track, &config);
        assert_eq!(result.len(), track.len());
    }

    #[test]
    fn scale_track_time_doubles_durations() {
        let track = make_track_two_kf();
        let original_duration = track.duration();
        let scaled = scale_track_time(&track, 2.0);
        assert!((scaled.duration() - original_duration * 2.0).abs() < 1e-5);
    }

    #[test]
    fn trim_track_removes_outside_keyframes() {
        let mut track = MorphTrack::new("trim_test");
        track.add_keyframe(Keyframe::new(0.0, make_params(0.1, 0.1, 0.1, 0.1)));
        track.add_keyframe(Keyframe::new(1.0, make_params(0.2, 0.2, 0.2, 0.2)));
        track.add_keyframe(Keyframe::new(2.0, make_params(0.3, 0.3, 0.3, 0.3)));
        track.add_keyframe(Keyframe::new(3.0, make_params(0.4, 0.4, 0.4, 0.4)));
        let trimmed = trim_track(&track, 1.0, 2.0);
        assert_eq!(trimmed.len(), 2);
    }

    #[test]
    fn trim_track_keeps_inside_keyframes() {
        let mut track = MorphTrack::new("trim_keep");
        track.add_keyframe(Keyframe::new(0.5, make_params(0.1, 0.1, 0.1, 0.1)));
        track.add_keyframe(Keyframe::new(1.5, make_params(0.5, 0.5, 0.5, 0.5)));
        track.add_keyframe(Keyframe::new(2.5, make_params(0.9, 0.9, 0.9, 0.9)));
        let trimmed = trim_track(&track, 0.0, 3.0);
        assert_eq!(trimmed.len(), 3);
    }

    #[test]
    fn reverse_track_flips_order() {
        let track = make_track_two_kf();
        let reversed = reverse_track(&track);
        assert_eq!(reversed.len(), 2);
        // Original: kf at 0.0 and 1.0; reversed: kf at 1.0-0.0=1.0 and 1.0-1.0=0.0
        // After sorting by time: first is 0.0, second is 1.0
        // The params at time=0.0 in reversed should be the params that were at time=1.0 in original
        let original_last = track.keyframes_iter().last().unwrap().params.clone();
        let reversed_first = reversed.keyframes_iter().next().unwrap().params.clone();
        assert!((original_last.height - reversed_first.height).abs() < 1e-6);
    }

    #[test]
    fn concat_tracks_total_length() {
        let first = make_track_two_kf();
        let second = make_track_two_kf();
        let combined = concat_tracks(&first, &second);
        assert_eq!(combined.len(), 4);
    }

    #[test]
    fn concat_tracks_second_offset_correctly() {
        let first = make_track_two_kf(); // times: 0.0, 1.0
        let mut second = MorphTrack::new("second");
        second.add_keyframe(Keyframe::new(0.0, make_params(0.1, 0.1, 0.1, 0.1)));
        second.add_keyframe(Keyframe::new(0.5, make_params(0.9, 0.9, 0.9, 0.9)));
        let combined = concat_tracks(&first, &second);
        // second's kf at 0.0 + offset(1.0) = 1.0
        // second's kf at 0.5 + offset(1.0) = 1.5
        // But first already has a kf at 1.0, so combined should have 4 keyframes
        // The last kf should be at time 1.5
        let last_kf = combined.keyframes_iter().last().unwrap();
        assert!((last_kf.time - 1.5).abs() < 1e-6);
    }
}
