// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! Film strip: captures a sequence of frames for preview thumbnails.

/// A single captured frame thumbnail.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FilmFrame {
    pub frame_index: u32,
    pub time: f32,
    pub thumbnail_width: u32,
    pub thumbnail_height: u32,
}

/// Film strip collection.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FilmStrip {
    pub frames: Vec<FilmFrame>,
    pub max_frames: usize,
    pub fps: f32,
}

#[allow(dead_code)]
pub fn new_film_strip(max_frames: usize, fps: f32) -> FilmStrip {
    FilmStrip {
        frames: Vec::new(),
        max_frames,
        fps: fps.max(1.0),
    }
}

#[allow(dead_code)]
pub fn capture_frame(strip: &mut FilmStrip, time: f32, width: u32, height: u32) {
    let index = strip.frames.len() as u32;
    if strip.frames.len() >= strip.max_frames {
        strip.frames.remove(0);
    }
    strip.frames.push(FilmFrame {
        frame_index: index,
        time,
        thumbnail_width: width,
        thumbnail_height: height,
    });
}

#[allow(dead_code)]
pub fn frame_count(strip: &FilmStrip) -> usize {
    strip.frames.len()
}

#[allow(dead_code)]
pub fn clear_film_strip(strip: &mut FilmStrip) {
    strip.frames.clear();
}

#[allow(dead_code)]
pub fn total_duration(strip: &FilmStrip) -> f32 {
    if strip.frames.len() < 2 {
        return 0.0;
    }
    match strip.frames.last() {
        Some(last) => last.time - strip.frames[0].time,
        None => 0.0,
    }
}

#[allow(dead_code)]
pub fn frame_at_time(strip: &FilmStrip, time: f32) -> Option<&FilmFrame> {
    strip.frames.iter().min_by(|a, b| {
        let da = (a.time - time).abs();
        let db = (b.time - time).abs();
        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
    })
}

#[allow(dead_code)]
pub fn film_strip_to_json(strip: &FilmStrip) -> String {
    format!(
        r#"{{"frame_count":{},"max_frames":{},"fps":{:.1},"duration":{:.4}}}"#,
        strip.frames.len(),
        strip.max_frames,
        strip.fps,
        total_duration(strip)
    )
}

#[allow(dead_code)]
pub fn set_fps(strip: &mut FilmStrip, fps: f32) {
    strip.fps = fps.max(1.0);
}

#[allow(dead_code)]
pub fn time_per_frame(strip: &FilmStrip) -> f32 {
    1.0 / strip.fps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_film_strip() {
        let s = new_film_strip(100, 30.0);
        assert_eq!(s.max_frames, 100);
        assert!((s.fps - 30.0).abs() < 1e-6);
    }

    #[test]
    fn test_capture_frame() {
        let mut s = new_film_strip(10, 30.0);
        capture_frame(&mut s, 0.0, 128, 72);
        assert_eq!(frame_count(&s), 1);
    }

    #[test]
    fn test_capture_overflow() {
        let mut s = new_film_strip(2, 30.0);
        capture_frame(&mut s, 0.0, 128, 72);
        capture_frame(&mut s, 0.5, 128, 72);
        capture_frame(&mut s, 1.0, 128, 72);
        assert_eq!(frame_count(&s), 2);
    }

    #[test]
    fn test_clear() {
        let mut s = new_film_strip(10, 30.0);
        capture_frame(&mut s, 0.0, 128, 72);
        clear_film_strip(&mut s);
        assert_eq!(frame_count(&s), 0);
    }

    #[test]
    fn test_total_duration() {
        let mut s = new_film_strip(10, 30.0);
        capture_frame(&mut s, 0.0, 128, 72);
        capture_frame(&mut s, 2.0, 128, 72);
        assert!((total_duration(&s) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_frame_at_time() {
        let mut s = new_film_strip(10, 30.0);
        capture_frame(&mut s, 0.0, 128, 72);
        capture_frame(&mut s, 1.0, 128, 72);
        let f = frame_at_time(&s, 0.8).unwrap();
        assert!((f.time - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_film_strip_to_json() {
        let s = new_film_strip(10, 24.0);
        let j = film_strip_to_json(&s);
        assert!(j.contains("frame_count"));
    }

    #[test]
    fn test_set_fps() {
        let mut s = new_film_strip(10, 30.0);
        set_fps(&mut s, 60.0);
        assert!((s.fps - 60.0).abs() < 1e-6);
    }

    #[test]
    fn test_time_per_frame() {
        let s = new_film_strip(10, 30.0);
        let t = time_per_frame(&s);
        assert!((t - 1.0 / 30.0).abs() < 1e-6);
    }

    #[test]
    fn test_total_duration_single() {
        let mut s = new_film_strip(10, 30.0);
        capture_frame(&mut s, 1.0, 128, 72);
        assert!(total_duration(&s).abs() < 1e-6);
    }
}
