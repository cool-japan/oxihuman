// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::params::ParamState;

/// A keyframe: a time value and a ParamState snapshot.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Keyframe {
    /// Time in seconds (or arbitrary units — must be sorted ascending).
    pub time: f32,
    pub params: ParamState,
    /// Optional label for this keyframe.
    pub label: Option<String>,
}

impl Keyframe {
    pub fn new(time: f32, params: ParamState) -> Self {
        Self {
            time,
            params,
            label: None,
        }
    }
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// An animation track: a sorted list of keyframes.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MorphTrack {
    pub name: String,
    keyframes: Vec<Keyframe>,
}

impl MorphTrack {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            keyframes: Vec::new(),
        }
    }

    /// Add a keyframe (will be sorted by time on insertion).
    pub fn add_keyframe(&mut self, kf: Keyframe) {
        let pos = self.keyframes.partition_point(|k| k.time <= kf.time);
        self.keyframes.insert(pos, kf);
    }

    /// Total duration (time of last keyframe - time of first).
    pub fn duration(&self) -> f32 {
        if self.keyframes.len() < 2 {
            return 0.0;
        }
        match (self.keyframes.first(), self.keyframes.last()) {
            (Some(first), Some(last)) => last.time - first.time,
            _ => 0.0,
        }
    }

    /// Number of keyframes.
    pub fn len(&self) -> usize {
        self.keyframes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keyframes.is_empty()
    }

    /// Iterate over all keyframes in order.
    pub fn keyframes_iter(&self) -> impl Iterator<Item = &Keyframe> {
        self.keyframes.iter()
    }

    /// Sample the track at time `t` using linear interpolation.
    /// If `t` is before the first keyframe, returns the first params.
    /// If `t` is after the last keyframe, returns the last params.
    pub fn sample_linear(&self, t: f32) -> Option<ParamState> {
        if self.keyframes.is_empty() {
            return None;
        }
        if let Some(first) = self.keyframes.first() {
            if t <= first.time {
                return Some(first.params.clone());
            }
        }
        if let Some(last) = self.keyframes.last() {
            if t >= last.time {
                return Some(last.params.clone());
            }
        }
        // Find last kf with time <= t
        let idx_b = self.keyframes.partition_point(|k| k.time <= t);
        let idx_a = idx_b - 1;
        let kf_a = &self.keyframes[idx_a];
        let kf_b = &self.keyframes[idx_b];
        let span = kf_b.time - kf_a.time;
        let local_t = if span > 0.0 {
            (t - kf_a.time) / span
        } else {
            0.0
        };
        Some(lerp_params(&kf_a.params, &kf_b.params, local_t))
    }

    /// Sample using Catmull-Rom spline interpolation (smoother than linear).
    /// Falls back to linear for tracks with < 4 keyframes.
    pub fn sample_catmull_rom(&self, t: f32) -> Option<ParamState> {
        if self.keyframes.is_empty() {
            return None;
        }
        if self.keyframes.len() < 4 {
            return self.sample_linear(t);
        }
        if let Some(first) = self.keyframes.first() {
            if t <= first.time {
                return Some(first.params.clone());
            }
        }
        if let Some(last) = self.keyframes.last() {
            if t >= last.time {
                return Some(last.params.clone());
            }
        }

        // Find surrounding segment: idx_b is first kf with time > t
        let idx_b = self.keyframes.partition_point(|k| k.time <= t);
        let idx_a = idx_b - 1;

        // Clamp neighbour indices
        let idx_p0 = idx_a.saturating_sub(1);
        let idx_p3 = (idx_b + 1).min(self.keyframes.len() - 1);

        let kf_a = &self.keyframes[idx_a];
        let kf_b = &self.keyframes[idx_b];
        let p0 = &self.keyframes[idx_p0].params;
        let p3 = &self.keyframes[idx_p3].params;

        let span = kf_b.time - kf_a.time;
        let u = if span > 0.0 {
            (t - kf_a.time) / span
        } else {
            0.0
        };

        let p1 = &kf_a.params;
        let p2 = &kf_b.params;

        let height = catmull_rom(p0.height, p1.height, p2.height, p3.height, u);
        let weight = catmull_rom(p0.weight, p1.weight, p2.weight, p3.weight, u);
        let muscle = catmull_rom(p0.muscle, p1.muscle, p2.muscle, p3.muscle, u);
        let age = catmull_rom(p0.age, p1.age, p2.age, p3.age, u);

        // Merge extra keys from all four control points
        let mut extra = std::collections::HashMap::new();
        for map in [&p0.extra, &p1.extra, &p2.extra, &p3.extra] {
            for key in map.keys() {
                extra.entry(key.clone()).or_insert(0.0_f32);
            }
        }
        for key in extra.keys().cloned().collect::<Vec<_>>() {
            let v0 = p0.extra.get(&key).copied().unwrap_or(0.0);
            let v1 = p1.extra.get(&key).copied().unwrap_or(0.0);
            let v2 = p2.extra.get(&key).copied().unwrap_or(0.0);
            let v3 = p3.extra.get(&key).copied().unwrap_or(0.0);
            extra.insert(key, catmull_rom(v0, v1, v2, v3, u));
        }

        Some(ParamState {
            height,
            weight,
            muscle,
            age,
            extra,
        })
    }

    /// Generate `count` evenly-spaced samples over the full duration.
    pub fn bake_linear(&self, count: usize) -> Vec<(f32, ParamState)> {
        if count == 0 || self.keyframes.is_empty() {
            return Vec::new();
        }
        let start = self.keyframes.first().map_or(0.0, |k| k.time);
        let end = self.keyframes.last().map_or(0.0, |k| k.time);
        (0..count)
            .filter_map(|i| {
                let t = if count == 1 {
                    start
                } else {
                    start + (end - start) * (i as f32 / (count - 1) as f32)
                };
                self.sample_linear(t).map(|p| (t, p))
            })
            .collect()
    }
}

/// Linearly interpolate between two ParamState values.
pub fn lerp_params(a: &ParamState, b: &ParamState, t: f32) -> ParamState {
    let t = t.clamp(0.0, 1.0);
    ParamState {
        height: a.height + (b.height - a.height) * t,
        weight: a.weight + (b.weight - a.weight) * t,
        muscle: a.muscle + (b.muscle - a.muscle) * t,
        age: a.age + (b.age - a.age) * t,
        extra: {
            // Merge both extra maps, interpolating shared keys
            let mut extra = a.extra.clone();
            for (k, &bv) in &b.extra {
                let av = a.extra.get(k).copied().unwrap_or(0.0);
                extra.insert(k.clone(), av + (bv - av) * t);
            }
            extra
        },
    }
}

/// Catmull-Rom spline value for 4 control points at parameter u in `[0,1]`.
/// Returns the interpolated value between p1 and p2.
pub fn catmull_rom(p0: f32, p1: f32, p2: f32, p3: f32, u: f32) -> f32 {
    let u2 = u * u;
    let u3 = u2 * u;
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * u
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * u2
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * u3)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params(v: f32) -> ParamState {
        ParamState {
            height: v,
            weight: v,
            muscle: v,
            age: v,
            extra: Default::default(),
        }
    }

    #[test]
    fn lerp_at_zero_is_a() {
        let a = default_params(0.2);
        let b = default_params(0.8);
        let r = lerp_params(&a, &b, 0.0);
        assert!((r.height - a.height).abs() < 1e-6);
    }

    #[test]
    fn lerp_at_one_is_b() {
        let a = default_params(0.2);
        let b = default_params(0.8);
        let r = lerp_params(&a, &b, 1.0);
        assert!((r.height - b.height).abs() < 1e-6);
    }

    #[test]
    fn lerp_midpoint() {
        let a = default_params(0.0);
        let b = default_params(1.0);
        let r = lerp_params(&a, &b, 0.5);
        assert!((r.height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn track_sample_before_start() {
        let mut track = MorphTrack::new("test");
        track.add_keyframe(Keyframe::new(1.0, default_params(0.3)));
        track.add_keyframe(Keyframe::new(2.0, default_params(0.7)));
        let r = track.sample_linear(0.0).unwrap();
        assert!((r.height - 0.3).abs() < 1e-6);
    }

    #[test]
    fn track_sample_after_end() {
        let mut track = MorphTrack::new("test");
        track.add_keyframe(Keyframe::new(0.0, default_params(0.1)));
        track.add_keyframe(Keyframe::new(1.0, default_params(0.9)));
        let r = track.sample_linear(5.0).unwrap();
        assert!((r.height - 0.9).abs() < 1e-6);
    }

    #[test]
    fn track_sample_midpoint() {
        let mut track = MorphTrack::new("test");
        track.add_keyframe(Keyframe::new(0.0, default_params(0.0)));
        track.add_keyframe(Keyframe::new(1.0, default_params(1.0)));
        let r = track.sample_linear(0.5).unwrap();
        assert!((r.height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn catmull_rom_endpoints() {
        let (p0, p1, p2, p3) = (0.0_f32, 0.25, 0.75, 1.0);
        assert!((catmull_rom(p0, p1, p2, p3, 0.0) - p1).abs() < 1e-6);
        assert!((catmull_rom(p0, p1, p2, p3, 1.0) - p2).abs() < 1e-6);
    }

    #[test]
    fn track_duration_correct() {
        let mut track = MorphTrack::new("test");
        track.add_keyframe(Keyframe::new(0.0, default_params(0.0)));
        track.add_keyframe(Keyframe::new(2.0, default_params(1.0)));
        assert!((track.duration() - 2.0).abs() < 1e-6);
    }

    #[test]
    fn bake_linear_count() {
        let mut track = MorphTrack::new("test");
        track.add_keyframe(Keyframe::new(0.0, default_params(0.0)));
        track.add_keyframe(Keyframe::new(1.0, default_params(1.0)));
        assert_eq!(track.bake_linear(10).len(), 10);
    }

    #[test]
    fn catmull_rom_sample_returns_some() {
        let mut track = MorphTrack::new("test");
        track.add_keyframe(Keyframe::new(0.0, default_params(0.0)));
        track.add_keyframe(Keyframe::new(1.0, default_params(0.33)));
        track.add_keyframe(Keyframe::new(2.0, default_params(0.66)));
        track.add_keyframe(Keyframe::new(3.0, default_params(1.0)));
        assert!(track.sample_catmull_rom(1.5).is_some());
    }
}
