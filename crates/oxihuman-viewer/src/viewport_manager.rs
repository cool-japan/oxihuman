// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Multi-viewport layout manager.

#[allow(dead_code)]
pub struct Viewport {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub camera_idx: usize,
}

#[allow(dead_code)]
pub struct ViewportManager {
    pub viewports: Vec<Viewport>,
    pub screen_width: u32,
    pub screen_height: u32,
}

#[allow(dead_code)]
pub fn new_viewport_manager(screen_width: u32, screen_height: u32) -> ViewportManager {
    ViewportManager { viewports: Vec::new(), screen_width, screen_height }
}

#[allow(dead_code)]
pub fn vpm_add_viewport(m: &mut ViewportManager, x: u32, y: u32, width: u32, height: u32) -> usize {
    let idx = m.viewports.len();
    m.viewports.push(Viewport { x, y, width, height, camera_idx: 0 });
    idx
}

#[allow(dead_code)]
pub fn vpm_count(m: &ViewportManager) -> usize {
    m.viewports.len()
}

#[allow(dead_code)]
pub fn vpm_coverage(m: &ViewportManager) -> f32 {
    let screen_area = m.screen_width as f64 * m.screen_height as f64;
    if screen_area < 1.0 {
        return 0.0;
    }
    let vp_area: f64 = m.viewports.iter().map(|v| v.width as f64 * v.height as f64).sum();
    (vp_area / screen_area) as f32
}

#[allow(dead_code)]
pub fn vpm_set_camera(m: &mut ViewportManager, idx: usize, camera_idx: usize) {
    if idx < m.viewports.len() {
        m.viewports[idx].camera_idx = camera_idx;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_viewport() {
        let mut m = new_viewport_manager(1920, 1080);
        let idx = vpm_add_viewport(&mut m, 0, 0, 960, 1080);
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_count() {
        let mut m = new_viewport_manager(1920, 1080);
        vpm_add_viewport(&mut m, 0, 0, 960, 1080);
        vpm_add_viewport(&mut m, 960, 0, 960, 1080);
        assert_eq!(vpm_count(&m), 2);
    }

    #[test]
    fn test_coverage_full() {
        let mut m = new_viewport_manager(1920, 1080);
        vpm_add_viewport(&mut m, 0, 0, 1920, 1080);
        assert!((vpm_coverage(&m) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_coverage_half() {
        let mut m = new_viewport_manager(1000, 1000);
        vpm_add_viewport(&mut m, 0, 0, 500, 1000);
        assert!((vpm_coverage(&m) - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_set_camera() {
        let mut m = new_viewport_manager(1920, 1080);
        vpm_add_viewport(&mut m, 0, 0, 1920, 1080);
        vpm_set_camera(&mut m, 0, 2);
        assert_eq!(m.viewports[0].camera_idx, 2);
    }

    #[test]
    fn test_screen_size() {
        let m = new_viewport_manager(800, 600);
        assert_eq!(m.screen_width, 800);
        assert_eq!(m.screen_height, 600);
    }

    #[test]
    fn test_coverage_empty() {
        let m = new_viewport_manager(1920, 1080);
        assert_eq!(vpm_coverage(&m), 0.0);
    }

    #[test]
    fn test_set_camera_out_of_bounds_safe() {
        let mut m = new_viewport_manager(1920, 1080);
        vpm_set_camera(&mut m, 99, 5); /* should not panic */
    }
}
