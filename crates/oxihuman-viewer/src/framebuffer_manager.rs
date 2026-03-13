// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Framebuffer/render target manager.

#[allow(dead_code)]
pub struct Framebuffer {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub color_attachments: u32,
    pub has_depth: bool,
}

#[allow(dead_code)]
pub struct FramebufferManager {
    pub framebuffers: Vec<Framebuffer>,
    pub next_id: u32,
}

#[allow(dead_code)]
pub fn new_framebuffer_manager() -> FramebufferManager {
    FramebufferManager { framebuffers: Vec::new(), next_id: 1 }
}

#[allow(dead_code)]
pub fn fb_create(mgr: &mut FramebufferManager, width: u32, height: u32, color_attachments: u32, has_depth: bool) -> u32 {
    let id = mgr.next_id;
    mgr.next_id += 1;
    mgr.framebuffers.push(Framebuffer { id, width, height, color_attachments, has_depth });
    id
}

#[allow(dead_code)]
pub fn fb_destroy(mgr: &mut FramebufferManager, id: u32) -> bool {
    if let Some(idx) = mgr.framebuffers.iter().position(|f| f.id == id) {
        mgr.framebuffers.remove(idx);
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn fb_count(mgr: &FramebufferManager) -> usize {
    mgr.framebuffers.len()
}

#[allow(dead_code)]
pub fn fb_total_pixels(mgr: &FramebufferManager) -> u64 {
    mgr.framebuffers.iter().map(|f| f.width as u64 * f.height as u64).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let mut mgr = new_framebuffer_manager();
        let id = fb_create(&mut mgr, 1920, 1080, 1, true);
        assert!(id > 0);
    }

    #[test]
    fn test_count() {
        let mut mgr = new_framebuffer_manager();
        fb_create(&mut mgr, 800, 600, 1, false);
        fb_create(&mut mgr, 1024, 768, 2, true);
        assert_eq!(fb_count(&mgr), 2);
    }

    #[test]
    fn test_destroy() {
        let mut mgr = new_framebuffer_manager();
        let id = fb_create(&mut mgr, 800, 600, 1, false);
        assert!(fb_destroy(&mut mgr, id));
        assert_eq!(fb_count(&mgr), 0);
    }

    #[test]
    fn test_destroy_missing() {
        let mut mgr = new_framebuffer_manager();
        assert!(!fb_destroy(&mut mgr, 999));
    }

    #[test]
    fn test_total_pixels() {
        let mut mgr = new_framebuffer_manager();
        fb_create(&mut mgr, 100, 100, 1, false);
        assert_eq!(fb_total_pixels(&mgr), 10000);
    }

    #[test]
    fn test_total_pixels_multiple() {
        let mut mgr = new_framebuffer_manager();
        fb_create(&mut mgr, 100, 100, 1, false);
        fb_create(&mut mgr, 200, 100, 1, false);
        assert_eq!(fb_total_pixels(&mgr), 30000);
    }

    #[test]
    fn test_ids_unique() {
        let mut mgr = new_framebuffer_manager();
        let id1 = fb_create(&mut mgr, 100, 100, 1, false);
        let id2 = fb_create(&mut mgr, 200, 200, 1, false);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_empty_total_pixels() {
        let mgr = new_framebuffer_manager();
        assert_eq!(fb_total_pixels(&mgr), 0);
    }
}
