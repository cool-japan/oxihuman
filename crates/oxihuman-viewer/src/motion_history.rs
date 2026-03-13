// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Motion history buffer — rolling per-object velocity and trajectory recording.

/// A single motion sample.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MotionSample {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub time_s: f32,
}

/// Per-object motion history.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MotionHistory {
    pub object_id: u32,
    samples: Vec<MotionSample>,
    capacity: usize,
}

impl MotionHistory {
    #[allow(dead_code)]
    pub fn new(object_id: u32, capacity: usize) -> Self {
        Self {
            object_id,
            samples: Vec::with_capacity(capacity),
            capacity: capacity.max(1),
        }
    }
}

#[allow(dead_code)]
pub fn new_motion_history(object_id: u32, capacity: usize) -> MotionHistory {
    MotionHistory::new(object_id, capacity)
}

#[allow(dead_code)]
pub fn mh_push(hist: &mut MotionHistory, pos: [f32; 3], vel: [f32; 3], time_s: f32) {
    if hist.samples.len() >= hist.capacity {
        hist.samples.remove(0);
    }
    hist.samples.push(MotionSample {
        position: pos,
        velocity: vel,
        time_s,
    });
}

#[allow(dead_code)]
pub fn mh_sample_count(hist: &MotionHistory) -> usize {
    hist.samples.len()
}

#[allow(dead_code)]
pub fn mh_latest(hist: &MotionHistory) -> Option<&MotionSample> {
    hist.samples.last()
}

#[allow(dead_code)]
pub fn mh_oldest(hist: &MotionHistory) -> Option<&MotionSample> {
    hist.samples.first()
}

#[allow(dead_code)]
pub fn mh_clear(hist: &mut MotionHistory) {
    hist.samples.clear();
}

/// Average speed over recorded history.
#[allow(dead_code)]
pub fn mh_average_speed(hist: &MotionHistory) -> f32 {
    if hist.samples.is_empty() {
        return 0.0;
    }
    let sum: f32 = hist
        .samples
        .iter()
        .map(|s| {
            let v = s.velocity;
            (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
        })
        .sum();
    sum / hist.samples.len() as f32
}

/// Peak speed.
#[allow(dead_code)]
pub fn mh_peak_speed(hist: &MotionHistory) -> f32 {
    hist.samples
        .iter()
        .map(|s| {
            let v = s.velocity;
            (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
        })
        .fold(0.0f32, f32::max)
}

/// Total path length traversed.
#[allow(dead_code)]
pub fn mh_path_length(hist: &MotionHistory) -> f32 {
    if hist.samples.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0f32;
    for i in 1..hist.samples.len() {
        let a = hist.samples[i - 1].position;
        let b = hist.samples[i].position;
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let dz = b[2] - a[2];
        total += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    total
}

/// Displacement from oldest to latest.
#[allow(dead_code)]
pub fn mh_net_displacement(hist: &MotionHistory) -> f32 {
    if hist.samples.len() < 2 {
        return 0.0;
    }
    let (a, b) = match (hist.samples.first(), hist.samples.last()) {
        (Some(first), Some(last)) => (first.position, last.position),
        _ => return 0.0,
    };
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[allow(dead_code)]
pub fn mh_to_json(hist: &MotionHistory) -> String {
    format!(
        "{{\"object_id\":{},\"samples\":{},\"avg_speed\":{:.4}}}",
        hist.object_id,
        mh_sample_count(hist),
        mh_average_speed(hist)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_history() {
        let h = new_motion_history(1, 10);
        assert_eq!(mh_sample_count(&h), 0);
    }

    #[test]
    fn push_increments() {
        let mut h = new_motion_history(1, 10);
        mh_push(&mut h, [0.0; 3], [1.0, 0.0, 0.0], 0.0);
        assert_eq!(mh_sample_count(&h), 1);
    }

    #[test]
    fn capacity_evicts_oldest() {
        let mut h = new_motion_history(1, 2);
        mh_push(&mut h, [0.0; 3], [0.0; 3], 0.0);
        mh_push(&mut h, [1.0, 0.0, 0.0], [0.0; 3], 1.0);
        mh_push(&mut h, [2.0, 0.0, 0.0], [0.0; 3], 2.0);
        assert_eq!(mh_sample_count(&h), 2);
        assert!((mh_oldest(&h).expect("should succeed").position[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn average_speed_correct() {
        let mut h = new_motion_history(1, 10);
        mh_push(&mut h, [0.0; 3], [3.0, 4.0, 0.0], 0.0); // speed 5
        assert!((mh_average_speed(&h) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn peak_speed_is_max() {
        let mut h = new_motion_history(1, 10);
        mh_push(&mut h, [0.0; 3], [1.0, 0.0, 0.0], 0.0);
        mh_push(&mut h, [0.0; 3], [3.0, 4.0, 0.0], 1.0);
        assert!((mh_peak_speed(&h) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn path_length_two_points() {
        let mut h = new_motion_history(1, 10);
        mh_push(&mut h, [0.0; 3], [0.0; 3], 0.0);
        mh_push(&mut h, [1.0, 0.0, 0.0], [0.0; 3], 1.0);
        assert!((mh_path_length(&h) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn clear_resets() {
        let mut h = new_motion_history(1, 10);
        mh_push(&mut h, [0.0; 3], [0.0; 3], 0.0);
        mh_clear(&mut h);
        assert_eq!(mh_sample_count(&h), 0);
    }

    #[test]
    fn net_displacement_zero_single() {
        let mut h = new_motion_history(1, 10);
        mh_push(&mut h, [5.0, 0.0, 0.0], [0.0; 3], 0.0);
        assert!(mh_net_displacement(&h) < 1e-6);
    }

    #[test]
    fn json_has_object_id() {
        let h = new_motion_history(42, 10);
        assert!(mh_to_json(&h).contains("42"));
    }

    #[test]
    fn latest_returns_last() {
        let mut h = new_motion_history(1, 10);
        mh_push(&mut h, [0.0; 3], [0.0; 3], 0.0);
        mh_push(&mut h, [5.0, 0.0, 0.0], [0.0; 3], 1.0);
        assert!((mh_latest(&h).expect("should succeed").time_s - 1.0).abs() < 1e-5);
    }
}
