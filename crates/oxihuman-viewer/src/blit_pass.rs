// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Blit pass utilities for copying texture regions.

/// Blit filter mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlitFilter {
    Nearest,
    Linear,
}

/// A rectangular region for blit operations.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct BlitRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Blit pass descriptor.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlitPassDesc {
    pub src_rect: BlitRect,
    pub dst_rect: BlitRect,
    pub filter: BlitFilter,
    pub flip_y: bool,
}

#[allow(dead_code)]
pub fn new_blit_rect(x: u32, y: u32, width: u32, height: u32) -> BlitRect {
    BlitRect { x, y, width, height }
}

#[allow(dead_code)]
pub fn full_screen_rect(width: u32, height: u32) -> BlitRect {
    BlitRect { x: 0, y: 0, width, height }
}

#[allow(dead_code)]
pub fn default_blit_pass(width: u32, height: u32) -> BlitPassDesc {
    let rect = full_screen_rect(width, height);
    BlitPassDesc { src_rect: rect, dst_rect: rect, filter: BlitFilter::Linear, flip_y: false }
}

#[allow(dead_code)]
pub fn blit_area(rect: &BlitRect) -> u64 {
    rect.width as u64 * rect.height as u64
}

#[allow(dead_code)]
pub fn rects_overlap(a: &BlitRect, b: &BlitRect) -> bool {
    a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y
}

#[allow(dead_code)]
pub fn scale_rect(rect: &BlitRect, scale: f32) -> BlitRect {
    BlitRect {
        x: rect.x,
        y: rect.y,
        width: (rect.width as f32 * scale) as u32,
        height: (rect.height as f32 * scale) as u32,
    }
}

#[allow(dead_code)]
pub fn set_blit_filter(desc: &mut BlitPassDesc, filter: BlitFilter) {
    desc.filter = filter;
}

#[allow(dead_code)]
pub fn set_blit_flip_y(desc: &mut BlitPassDesc, flip: bool) {
    desc.flip_y = flip;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_blit_rect() {
        let r = new_blit_rect(10, 20, 100, 200);
        assert_eq!(r.x, 10);
        assert_eq!(r.height, 200);
    }

    #[test]
    fn test_full_screen_rect() {
        let r = full_screen_rect(1920, 1080);
        assert_eq!(r.x, 0);
        assert_eq!(r.width, 1920);
    }

    #[test]
    fn test_default_blit_pass() {
        let desc = default_blit_pass(800, 600);
        assert_eq!(desc.filter, BlitFilter::Linear);
        assert!(!desc.flip_y);
    }

    #[test]
    fn test_blit_area() {
        let r = new_blit_rect(0, 0, 100, 200);
        assert_eq!(blit_area(&r), 20000);
    }

    #[test]
    fn test_rects_overlap_true() {
        let a = new_blit_rect(0, 0, 100, 100);
        let b = new_blit_rect(50, 50, 100, 100);
        assert!(rects_overlap(&a, &b));
    }

    #[test]
    fn test_rects_overlap_false() {
        let a = new_blit_rect(0, 0, 50, 50);
        let b = new_blit_rect(100, 100, 50, 50);
        assert!(!rects_overlap(&a, &b));
    }

    #[test]
    fn test_scale_rect() {
        let r = new_blit_rect(0, 0, 100, 100);
        let scaled = scale_rect(&r, 0.5);
        assert_eq!(scaled.width, 50);
    }

    #[test]
    fn test_set_filter() {
        let mut desc = default_blit_pass(800, 600);
        set_blit_filter(&mut desc, BlitFilter::Nearest);
        assert_eq!(desc.filter, BlitFilter::Nearest);
    }

    #[test]
    fn test_set_flip_y() {
        let mut desc = default_blit_pass(800, 600);
        set_blit_flip_y(&mut desc, true);
        assert!(desc.flip_y);
    }

    #[test]
    fn test_zero_area() {
        let r = new_blit_rect(0, 0, 0, 100);
        assert_eq!(blit_area(&r), 0);
    }
}
