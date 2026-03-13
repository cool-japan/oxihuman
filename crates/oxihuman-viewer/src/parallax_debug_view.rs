// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct ParallaxDebugView {
    pub show_depth: bool,
    pub show_iterations: bool,
    pub scale: f32,
}

pub fn new_parallax_debug_view() -> ParallaxDebugView {
    ParallaxDebugView {
        show_depth: true,
        show_iterations: false,
        scale: 0.05,
    }
}

pub fn parallax_depth_color(depth: f32) -> [f32; 3] {
    let d = depth.clamp(0.0, 1.0);
    [d, d, d]
}

pub fn parallax_iteration_color(iters: u32, max_iters: u32) -> [f32; 3] {
    let t = if max_iters == 0 {
        0.0
    } else {
        (iters as f32 / max_iters as f32).clamp(0.0, 1.0)
    };
    [t, 1.0 - t, 0.0]
}

pub fn parallax_offset(uv: [f32; 2], height: f32, view_dir: [f32; 3], scale: f32) -> [f32; 2] {
    let len_xy = (view_dir[0] * view_dir[0] + view_dir[1] * view_dir[1])
        .sqrt()
        .max(1e-9);
    let parallax_x = view_dir[0] / len_xy;
    let parallax_y = view_dir[1] / len_xy;
    let h = (height - 0.5) * scale;
    [uv[0] + parallax_x * h, uv[1] + parallax_y * h]
}

pub fn parallax_relief_steps(quality: u32) -> u32 {
    (quality * 8).max(8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_parallax_debug_view() {
        /* scale defaults to 0.05 */
        let v = new_parallax_debug_view();
        assert!((v.scale - 0.05).abs() < 1e-6);
    }

    #[test]
    fn test_parallax_depth_color() {
        /* depth=0 -> black */
        let c = parallax_depth_color(0.0);
        assert!((c[0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_parallax_iteration_color() {
        /* iters=max -> [1,0,0] */
        let c = parallax_iteration_color(8, 8);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_parallax_offset_no_height() {
        /* height=0.5 (midlevel) -> no offset */
        let uv = [0.5f32, 0.5f32];
        let out = parallax_offset(uv, 0.5, [1.0, 0.0, 0.0], 0.05);
        assert!((out[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_parallax_relief_steps() {
        /* quality=1 -> at least 8 */
        let s = parallax_relief_steps(1);
        assert!(s >= 8);
    }
}
