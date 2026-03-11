// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! Texture asset manager.

#[allow(dead_code)]
pub struct TextureAsset {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub mip_levels: u32,
    pub format: String,
}

#[allow(dead_code)]
pub struct TextureManager {
    pub textures: Vec<TextureAsset>,
    pub next_id: u32,
}

#[allow(dead_code)]
pub fn new_texture_manager() -> TextureManager {
    TextureManager { textures: Vec::new(), next_id: 1 }
}

#[allow(dead_code)]
pub fn tm_create(mgr: &mut TextureManager, width: u32, height: u32, mip_levels: u32, format: &str) -> u32 {
    let id = mgr.next_id;
    mgr.next_id += 1;
    mgr.textures.push(TextureAsset { id, width, height, mip_levels, format: format.to_string() });
    id
}

#[allow(dead_code)]
pub fn tm_destroy(mgr: &mut TextureManager, id: u32) -> bool {
    if let Some(idx) = mgr.textures.iter().position(|t| t.id == id) {
        mgr.textures.remove(idx);
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn tm_count(mgr: &TextureManager) -> usize {
    mgr.textures.len()
}

#[allow(dead_code)]
pub fn tm_total_memory_estimate(mgr: &TextureManager) -> u64 {
    mgr.textures.iter().map(|t| t.width as u64 * t.height as u64 * 4).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let mut mgr = new_texture_manager();
        let id = tm_create(&mut mgr, 512, 512, 1, "RGBA8");
        assert!(id > 0);
    }

    #[test]
    fn test_count() {
        let mut mgr = new_texture_manager();
        tm_create(&mut mgr, 512, 512, 1, "RGBA8");
        tm_create(&mut mgr, 256, 256, 1, "RGB8");
        assert_eq!(tm_count(&mgr), 2);
    }

    #[test]
    fn test_destroy() {
        let mut mgr = new_texture_manager();
        let id = tm_create(&mut mgr, 512, 512, 1, "RGBA8");
        assert!(tm_destroy(&mut mgr, id));
        assert_eq!(tm_count(&mgr), 0);
    }

    #[test]
    fn test_destroy_missing() {
        let mut mgr = new_texture_manager();
        assert!(!tm_destroy(&mut mgr, 999));
    }

    #[test]
    fn test_total_memory_estimate() {
        let mut mgr = new_texture_manager();
        tm_create(&mut mgr, 100, 100, 1, "RGBA8");
        assert_eq!(tm_total_memory_estimate(&mgr), 40000);
    }

    #[test]
    fn test_total_memory_multiple() {
        let mut mgr = new_texture_manager();
        tm_create(&mut mgr, 100, 100, 1, "RGBA8");
        tm_create(&mut mgr, 50, 50, 1, "RGBA8");
        assert_eq!(tm_total_memory_estimate(&mgr), 40000 + 10000);
    }

    #[test]
    fn test_unique_ids() {
        let mut mgr = new_texture_manager();
        let id1 = tm_create(&mut mgr, 1, 1, 1, "X");
        let id2 = tm_create(&mut mgr, 1, 1, 1, "X");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_empty_memory() {
        let mgr = new_texture_manager();
        assert_eq!(tm_total_memory_estimate(&mgr), 0);
    }
}
