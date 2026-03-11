// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

// ── Enumerations ──────────────────────────────────────────────────────────────

/// Pixel format of a texture.
pub enum TextureFormat {
    Rgba8,
    Rgba16Float,
    R8,
    Rg8,
    Bc1,
    Bc3,
    Bc7,
}

/// Filtering mode.
pub enum TextureFilter {
    Nearest,
    Linear,
    Anisotropic(u8),
}

/// Texture address / wrap mode.
pub enum TextureWrap {
    Clamp,
    Repeat,
    Mirror,
}

// ── Descriptors ───────────────────────────────────────────────────────────────

/// Describes a texture without holding pixel data.
pub struct TextureDescriptor {
    pub label: String,
    pub width: u32,
    pub height: u32,
    pub mip_levels: u32,
    pub format: TextureFormat,
    pub filter: TextureFilter,
    pub wrap_u: TextureWrap,
    pub wrap_v: TextureWrap,
}

/// A texture entry held in the cache.
pub struct TextureEntry {
    pub id: u32,
    pub descriptor: TextureDescriptor,
    /// `None` = placeholder / streaming not yet loaded.
    pub data: Option<Vec<u8>>,
    pub loaded: bool,
}

// ── Cache ─────────────────────────────────────────────────────────────────────

/// Manages a collection of GPU textures (CPU-side stubs for Phase 2).
pub struct TextureCache {
    entries: Vec<TextureEntry>,
    next_id: u32,
}

impl TextureCache {
    /// Create an empty texture cache.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 0,
        }
    }

    /// Insert a new texture, returning its unique id.
    pub fn insert(&mut self, desc: TextureDescriptor, data: Option<Vec<u8>>) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let loaded = data.is_some();
        self.entries.push(TextureEntry {
            id,
            descriptor: desc,
            data,
            loaded,
        });
        id
    }

    /// Retrieve a reference to a texture by id.
    pub fn get(&self, id: u32) -> Option<&TextureEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Remove a texture by id. Returns `true` if it was present.
    pub fn remove(&mut self, id: u32) -> bool {
        if let Some(pos) = self.entries.iter().position(|e| e.id == id) {
            self.entries.remove(pos);
            true
        } else {
            false
        }
    }

    /// Total number of entries (loaded or not).
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Number of entries where `loaded == true`.
    pub fn loaded_count(&self) -> usize {
        self.entries.iter().filter(|e| e.loaded).count()
    }

    /// Supply pixel data for a previously-inserted placeholder texture.
    /// Returns `false` if the id is not found.
    pub fn mark_loaded(&mut self, id: u32, data: Vec<u8>) -> bool {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
            entry.data = Some(data);
            entry.loaded = true;
            true
        } else {
            false
        }
    }

    /// Evict pixel data from all entries (streaming eviction), keeping descriptors.
    pub fn evict_all(&mut self) {
        for entry in &mut self.entries {
            entry.data = None;
            entry.loaded = false;
        }
    }

    /// Sum of memory used by all loaded textures.
    pub fn total_memory_bytes(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.loaded)
            .map(|e| texture_memory_bytes(&e.descriptor))
            .sum()
    }
}

impl Default for TextureCache {
    fn default() -> Self {
        Self::new()
    }
}

// ── Helper functions ──────────────────────────────────────────────────────────

/// Bytes per pixel for a given format (BC formats use sub-pixel values rounded up).
pub fn texture_format_bytes(fmt: &TextureFormat) -> u32 {
    match fmt {
        TextureFormat::R8 => 1,
        TextureFormat::Rg8 => 2,
        TextureFormat::Rgba8 => 4,
        TextureFormat::Rgba16Float => 8,
        // BC1: 0.5 bytes/pixel → represented as 0 for integer math; caller uses special path
        TextureFormat::Bc1 => 0, // handled separately in texture_memory_bytes
        // BC3/BC7: 1 byte/pixel
        TextureFormat::Bc3 => 1,
        TextureFormat::Bc7 => 1,
    }
}

/// Approximate memory size in bytes for width×height texels (ignoring mips/compression exactly).
pub fn texture_memory_bytes(desc: &TextureDescriptor) -> usize {
    let pixels = desc.width as usize * desc.height as usize;
    match &desc.format {
        TextureFormat::Bc1 => {
            // 0.5 bytes per pixel = pixels / 2
            pixels / 2
        }
        other => pixels * texture_format_bytes(other) as usize,
    }
}

/// Return a 1×1 pink RGBA8 placeholder texture.
pub fn default_placeholder_texture() -> TextureEntry {
    let desc = TextureDescriptor {
        label: "placeholder".to_string(),
        width: 1,
        height: 1,
        mip_levels: 1,
        format: TextureFormat::Rgba8,
        filter: TextureFilter::Nearest,
        wrap_u: TextureWrap::Clamp,
        wrap_v: TextureWrap::Clamp,
    };
    // Pink: R=255, G=105, B=180, A=255
    let data = vec![255u8, 105, 180, 255];
    TextureEntry {
        id: u32::MAX,
        descriptor: desc,
        data: Some(data),
        loaded: true,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn rgba8_desc(label: &str, w: u32, h: u32) -> TextureDescriptor {
        TextureDescriptor {
            label: label.to_string(),
            width: w,
            height: h,
            mip_levels: 1,
            format: TextureFormat::Rgba8,
            filter: TextureFilter::Linear,
            wrap_u: TextureWrap::Repeat,
            wrap_v: TextureWrap::Repeat,
        }
    }

    #[test]
    fn new_cache_is_empty() {
        let cache = TextureCache::new();
        assert_eq!(cache.count(), 0);
        assert_eq!(cache.loaded_count(), 0);
    }

    #[test]
    fn insert_returns_incrementing_ids() {
        let mut cache = TextureCache::new();
        let id0 = cache.insert(rgba8_desc("a", 4, 4), None);
        let id1 = cache.insert(rgba8_desc("b", 4, 4), None);
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
    }

    #[test]
    fn get_found() {
        let mut cache = TextureCache::new();
        let id = cache.insert(rgba8_desc("tex", 8, 8), None);
        let entry = cache.get(id);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().id, id);
    }

    #[test]
    fn get_not_found() {
        let cache = TextureCache::new();
        assert!(cache.get(999).is_none());
    }

    #[test]
    fn remove_true_when_present() {
        let mut cache = TextureCache::new();
        let id = cache.insert(rgba8_desc("x", 2, 2), None);
        assert!(cache.remove(id));
        assert_eq!(cache.count(), 0);
    }

    #[test]
    fn remove_false_when_absent() {
        let mut cache = TextureCache::new();
        assert!(!cache.remove(42));
    }

    #[test]
    fn count_and_loaded_count() {
        let mut cache = TextureCache::new();
        cache.insert(rgba8_desc("a", 4, 4), None);
        cache.insert(rgba8_desc("b", 4, 4), Some(vec![0u8; 64]));
        assert_eq!(cache.count(), 2);
        assert_eq!(cache.loaded_count(), 1);
    }

    #[test]
    fn mark_loaded_sets_data_and_flag() {
        let mut cache = TextureCache::new();
        let id = cache.insert(rgba8_desc("stream", 4, 4), None);
        assert_eq!(cache.loaded_count(), 0);
        let ok = cache.mark_loaded(id, vec![0u8; 64]);
        assert!(ok);
        assert_eq!(cache.loaded_count(), 1);
        assert!(cache.get(id).unwrap().data.is_some());
    }

    #[test]
    fn mark_loaded_returns_false_for_missing_id() {
        let mut cache = TextureCache::new();
        assert!(!cache.mark_loaded(999, vec![0u8; 4]));
    }

    #[test]
    fn evict_all_clears_data_and_loaded_flag() {
        let mut cache = TextureCache::new();
        cache.insert(rgba8_desc("t", 2, 2), Some(vec![0u8; 16]));
        cache.insert(rgba8_desc("u", 2, 2), Some(vec![0u8; 16]));
        assert_eq!(cache.loaded_count(), 2);
        cache.evict_all();
        assert_eq!(cache.loaded_count(), 0);
        assert_eq!(cache.count(), 2); // descriptors remain
        for entry in &cache.entries {
            assert!(entry.data.is_none());
        }
    }

    #[test]
    fn texture_format_bytes_rgba8_is_4() {
        assert_eq!(texture_format_bytes(&TextureFormat::Rgba8), 4);
    }

    #[test]
    fn texture_format_bytes_r8_is_1() {
        assert_eq!(texture_format_bytes(&TextureFormat::R8), 1);
    }

    #[test]
    fn texture_format_bytes_rgba16float_is_8() {
        assert_eq!(texture_format_bytes(&TextureFormat::Rgba16Float), 8);
    }

    #[test]
    fn texture_memory_bytes_rgba8_formula() {
        let desc = rgba8_desc("t", 4, 4);
        // 4*4*4 = 64
        assert_eq!(texture_memory_bytes(&desc), 64);
    }

    #[test]
    fn texture_memory_bytes_bc1_half_pixel() {
        let desc = TextureDescriptor {
            label: "bc".to_string(),
            width: 4,
            height: 4,
            mip_levels: 1,
            format: TextureFormat::Bc1,
            filter: TextureFilter::Linear,
            wrap_u: TextureWrap::Clamp,
            wrap_v: TextureWrap::Clamp,
        };
        // 4*4/2 = 8
        assert_eq!(texture_memory_bytes(&desc), 8);
    }

    #[test]
    fn total_memory_bytes_sum_of_loaded() {
        let mut cache = TextureCache::new();
        // Insert two 4×4 RGBA8 textures with data — each = 64 bytes
        cache.insert(rgba8_desc("a", 4, 4), Some(vec![0u8; 64]));
        cache.insert(rgba8_desc("b", 4, 4), Some(vec![0u8; 64]));
        // One placeholder without data
        cache.insert(rgba8_desc("c", 4, 4), None);
        assert_eq!(cache.total_memory_bytes(), 128);
    }

    #[test]
    fn default_placeholder_texture_non_null() {
        let entry = default_placeholder_texture();
        assert!(entry.data.is_some());
        let data = entry.data.unwrap();
        assert_eq!(data.len(), 4);
        // Pink: R=255, G=105, B=180, A=255
        assert_eq!(data[0], 255);
        assert_eq!(data[2], 180);
    }
}
