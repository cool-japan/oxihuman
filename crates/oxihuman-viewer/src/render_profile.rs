// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! Render profiling and timing.

#[allow(dead_code)]
pub struct RenderProfileEntry {
    pub name: String,
    pub start_ms: f32,
    pub end_ms: f32,
}

#[allow(dead_code)]
pub struct RenderProfile {
    pub entries: Vec<RenderProfileEntry>,
    pub frame_idx: u64,
}

#[allow(dead_code)]
pub fn new_render_profile() -> RenderProfile {
    RenderProfile { entries: Vec::new(), frame_idx: 0 }
}

#[allow(dead_code)]
pub fn rp_begin(profile: &mut RenderProfile, name: &str, start_ms: f32) -> usize {
    let idx = profile.entries.len();
    profile.entries.push(RenderProfileEntry { name: name.to_string(), start_ms, end_ms: start_ms });
    idx
}

#[allow(dead_code)]
pub fn rp_end(profile: &mut RenderProfile, idx: usize, end_ms: f32) {
    if idx < profile.entries.len() {
        profile.entries[idx].end_ms = end_ms;
    }
}

#[allow(dead_code)]
pub fn rp_total_time(profile: &RenderProfile) -> f32 {
    profile.entries.iter().map(|e| (e.end_ms - e.start_ms).max(0.0)).sum()
}

#[allow(dead_code)]
pub fn rp_entry_count(profile: &RenderProfile) -> usize {
    profile.entries.len()
}

#[allow(dead_code)]
pub fn rp_clear(profile: &mut RenderProfile) {
    profile.entries.clear();
    profile.frame_idx += 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_begin() {
        let mut p = new_render_profile();
        let idx = rp_begin(&mut p, "shadow", 0.0);
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_end() {
        let mut p = new_render_profile();
        let idx = rp_begin(&mut p, "gbuffer", 0.0);
        rp_end(&mut p, idx, 2.5);
        assert!((p.entries[idx].end_ms - 2.5).abs() < 1e-5);
    }

    #[test]
    fn test_total_time() {
        let mut p = new_render_profile();
        let i1 = rp_begin(&mut p, "a", 0.0);
        rp_end(&mut p, i1, 1.0);
        let i2 = rp_begin(&mut p, "b", 1.0);
        rp_end(&mut p, i2, 3.0);
        assert!((rp_total_time(&p) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_entry_count() {
        let mut p = new_render_profile();
        rp_begin(&mut p, "a", 0.0);
        rp_begin(&mut p, "b", 1.0);
        assert_eq!(rp_entry_count(&p), 2);
    }

    #[test]
    fn test_clear() {
        let mut p = new_render_profile();
        rp_begin(&mut p, "x", 0.0);
        rp_clear(&mut p);
        assert_eq!(rp_entry_count(&p), 0);
    }

    #[test]
    fn test_clear_increments_frame_idx() {
        let mut p = new_render_profile();
        rp_clear(&mut p);
        assert_eq!(p.frame_idx, 1);
    }

    #[test]
    fn test_total_time_empty() {
        let p = new_render_profile();
        assert_eq!(rp_total_time(&p), 0.0);
    }

    #[test]
    fn test_end_out_of_bounds_safe() {
        let mut p = new_render_profile();
        rp_end(&mut p, 99, 1.0); /* should not panic */
    }
}
