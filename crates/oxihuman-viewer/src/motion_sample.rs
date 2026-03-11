// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Motion sample ring buffer for temporal effects.

use std::f32::consts::FRAC_PI_4;

pub const MOTION_SAMPLE_CAPACITY: usize = 16;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct MotionSampleEntry {
    pub velocity: [f32; 2],
    pub timestamp: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MotionSampleBuffer {
    pub entries: [Option<MotionSampleEntry>; MOTION_SAMPLE_CAPACITY],
    pub head: usize,
    pub len: usize,
}

#[allow(dead_code)]
pub fn new_motion_sample_buffer() -> MotionSampleBuffer {
    MotionSampleBuffer {
        entries: [None; MOTION_SAMPLE_CAPACITY],
        head: 0,
        len: 0,
    }
}

#[allow(dead_code)]
pub fn msb_push(buf: &mut MotionSampleBuffer, velocity: [f32; 2], timestamp: f32) {
    let idx = buf.head % MOTION_SAMPLE_CAPACITY;
    buf.entries[idx] = Some(MotionSampleEntry {
        velocity,
        timestamp,
    });
    buf.head = (buf.head + 1) % MOTION_SAMPLE_CAPACITY;
    if buf.len < MOTION_SAMPLE_CAPACITY {
        buf.len += 1;
    }
}

#[allow(dead_code)]
pub fn msb_clear(buf: &mut MotionSampleBuffer) {
    buf.entries = [None; MOTION_SAMPLE_CAPACITY];
    buf.head = 0;
    buf.len = 0;
}

#[allow(dead_code)]
pub fn msb_count(buf: &MotionSampleBuffer) -> usize {
    buf.len
}

#[allow(dead_code)]
pub fn msb_is_empty(buf: &MotionSampleBuffer) -> bool {
    buf.len == 0
}

#[allow(dead_code)]
pub fn msb_average_velocity(buf: &MotionSampleBuffer) -> [f32; 2] {
    if buf.len == 0 {
        return [0.0, 0.0];
    }
    let mut sx = 0.0f32;
    let mut sy = 0.0f32;
    for e in buf.entries.iter().flatten() {
        sx += e.velocity[0];
        sy += e.velocity[1];
    }
    [sx / buf.len as f32, sy / buf.len as f32]
}

#[allow(dead_code)]
pub fn msb_peak_speed(buf: &MotionSampleBuffer) -> f32 {
    buf.entries
        .iter()
        .flatten()
        .map(|e| (e.velocity[0] * e.velocity[0] + e.velocity[1] * e.velocity[1]).sqrt())
        .fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn msb_speed_angle_rad(buf: &MotionSampleBuffer) -> f32 {
    let s = msb_peak_speed(buf);
    if s > 0.0 {
        (1.0 / s).atan().min(FRAC_PI_4)
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn msb_to_json(buf: &MotionSampleBuffer) -> String {
    let avg = msb_average_velocity(buf);
    format!(
        "{{\"count\":{},\"avg_vel\":[{:.4},{:.4}]}}",
        msb_count(buf),
        avg[0],
        avg[1]
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_is_empty() {
        assert!(msb_is_empty(&new_motion_sample_buffer()));
    }
    #[test]
    fn push_increments_count() {
        let mut b = new_motion_sample_buffer();
        msb_push(&mut b, [1.0, 0.0], 0.0);
        assert_eq!(msb_count(&b), 1);
    }
    #[test]
    fn clear_empties() {
        let mut b = new_motion_sample_buffer();
        msb_push(&mut b, [1.0, 0.0], 0.0);
        msb_clear(&mut b);
        assert!(msb_is_empty(&b));
    }
    #[test]
    fn wraps_at_capacity() {
        let mut b = new_motion_sample_buffer();
        for i in 0..MOTION_SAMPLE_CAPACITY + 2 {
            msb_push(&mut b, [i as f32, 0.0], i as f32);
        }
        assert_eq!(msb_count(&b), MOTION_SAMPLE_CAPACITY);
    }
    #[test]
    fn average_velocity_empty_zero() {
        let b = new_motion_sample_buffer();
        let avg = msb_average_velocity(&b);
        assert!(avg[0].abs() < 1e-6 && avg[1].abs() < 1e-6);
    }
    #[test]
    fn average_velocity_one_entry() {
        let mut b = new_motion_sample_buffer();
        msb_push(&mut b, [4.0, 2.0], 0.0);
        let avg = msb_average_velocity(&b);
        assert!((avg[0] - 4.0).abs() < 1e-5);
    }
    #[test]
    fn peak_speed_correct() {
        let mut b = new_motion_sample_buffer();
        msb_push(&mut b, [3.0, 4.0], 0.0);
        assert!((msb_peak_speed(&b) - 5.0).abs() < 1e-4);
    }
    #[test]
    fn speed_angle_nonneg() {
        assert!(msb_speed_angle_rad(&new_motion_sample_buffer()) >= 0.0);
    }
    #[test]
    fn to_json_has_count() {
        assert!(msb_to_json(&new_motion_sample_buffer()).contains("\"count\""));
    }
    #[test]
    fn capacity_is_sixteen() {
        assert_eq!(MOTION_SAMPLE_CAPACITY, 16);
    }
}
