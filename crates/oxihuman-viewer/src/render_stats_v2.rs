// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! Render statistics v2 with history.

#[allow(dead_code)]
pub struct RenderStatsV2 {
    pub frame_times: Vec<f32>,
    pub draw_calls: Vec<u32>,
    pub max_history: usize,
}

#[allow(dead_code)]
pub fn new_render_stats_v2(max_history: usize) -> RenderStatsV2 {
    RenderStatsV2 { frame_times: Vec::new(), draw_calls: Vec::new(), max_history }
}

#[allow(dead_code)]
pub fn rsv2_record(stats: &mut RenderStatsV2, frame_time_ms: f32, draw_calls: u32) {
    stats.frame_times.push(frame_time_ms);
    stats.draw_calls.push(draw_calls);
    while stats.frame_times.len() > stats.max_history {
        stats.frame_times.remove(0);
        stats.draw_calls.remove(0);
    }
}

#[allow(dead_code)]
pub fn rsv2_avg_frame_time(stats: &RenderStatsV2) -> f32 {
    if stats.frame_times.is_empty() {
        return 0.0;
    }
    stats.frame_times.iter().sum::<f32>() / stats.frame_times.len() as f32
}

#[allow(dead_code)]
pub fn rsv2_avg_fps(stats: &RenderStatsV2) -> f32 {
    let avg = rsv2_avg_frame_time(stats);
    if avg < 1e-7 {
        return 0.0;
    }
    1000.0 / avg
}

#[allow(dead_code)]
pub fn rsv2_avg_draw_calls(stats: &RenderStatsV2) -> f32 {
    if stats.draw_calls.is_empty() {
        return 0.0;
    }
    stats.draw_calls.iter().sum::<u32>() as f32 / stats.draw_calls.len() as f32
}

#[allow(dead_code)]
pub fn rsv2_record_count(stats: &RenderStatsV2) -> usize {
    stats.frame_times.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record() {
        let mut s = new_render_stats_v2(10);
        rsv2_record(&mut s, 16.0, 100);
        assert_eq!(rsv2_record_count(&s), 1);
    }

    #[test]
    fn test_avg_frame_time() {
        let mut s = new_render_stats_v2(10);
        rsv2_record(&mut s, 10.0, 0);
        rsv2_record(&mut s, 20.0, 0);
        assert!((rsv2_avg_frame_time(&s) - 15.0).abs() < 1e-4);
    }

    #[test]
    fn test_avg_fps() {
        let mut s = new_render_stats_v2(10);
        rsv2_record(&mut s, 16.666_67, 0);
        let fps = rsv2_avg_fps(&s);
        assert!(fps > 50.0 && fps < 70.0);
    }

    #[test]
    fn test_avg_fps_empty() {
        let s = new_render_stats_v2(10);
        assert_eq!(rsv2_avg_fps(&s), 0.0);
    }

    #[test]
    fn test_avg_draw_calls() {
        let mut s = new_render_stats_v2(10);
        rsv2_record(&mut s, 1.0, 100);
        rsv2_record(&mut s, 1.0, 200);
        assert!((rsv2_avg_draw_calls(&s) - 150.0).abs() < 1e-4);
    }

    #[test]
    fn test_max_history_enforced() {
        let mut s = new_render_stats_v2(3);
        for _ in 0..5 {
            rsv2_record(&mut s, 1.0, 0);
        }
        assert_eq!(rsv2_record_count(&s), 3);
    }

    #[test]
    fn test_empty_avg_frame_time() {
        let s = new_render_stats_v2(10);
        assert_eq!(rsv2_avg_frame_time(&s), 0.0);
    }

    #[test]
    fn test_record_count() {
        let mut s = new_render_stats_v2(10);
        rsv2_record(&mut s, 1.0, 1);
        rsv2_record(&mut s, 2.0, 2);
        assert_eq!(rsv2_record_count(&s), 2);
    }
}
