// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A buffer that accumulates byte-level patches (offset, data) for later application.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Patch {
    pub offset: usize,
    pub data: Vec<u8>,
}

#[allow(dead_code)]
pub struct PatchBuffer {
    patches: Vec<Patch>,
    applied_count: u32,
}

#[allow(dead_code)]
impl PatchBuffer {
    pub fn new() -> Self {
        Self {
            patches: Vec::new(),
            applied_count: 0,
        }
    }
    pub fn add_patch(&mut self, offset: usize, data: &[u8]) {
        self.patches.push(Patch {
            offset,
            data: data.to_vec(),
        });
    }
    pub fn apply_to(&mut self, buf: &mut [u8]) -> u32 {
        let mut applied = 0u32;
        for p in &self.patches {
            let end = (p.offset + p.data.len()).min(buf.len());
            if p.offset < buf.len() {
                let len = end - p.offset;
                buf[p.offset..end].copy_from_slice(&p.data[..len]);
                applied += 1;
            }
        }
        self.applied_count += applied;
        self.patches.clear();
        applied
    }
    pub fn patch_count(&self) -> usize {
        self.patches.len()
    }
    pub fn is_empty(&self) -> bool {
        self.patches.is_empty()
    }
    pub fn applied_count(&self) -> u32 {
        self.applied_count
    }
    pub fn total_bytes(&self) -> usize {
        self.patches.iter().map(|p| p.data.len()).sum()
    }
    pub fn clear(&mut self) {
        self.patches.clear();
    }
    pub fn patches(&self) -> &[Patch] {
        &self.patches
    }
    pub fn max_offset(&self) -> usize {
        self.patches
            .iter()
            .map(|p| p.offset + p.data.len())
            .max()
            .unwrap_or(0)
    }
}

impl Default for PatchBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub fn new_patch_buffer() -> PatchBuffer {
    PatchBuffer::new()
}
#[allow(dead_code)]
pub fn pb_add(b: &mut PatchBuffer, offset: usize, data: &[u8]) {
    b.add_patch(offset, data);
}
#[allow(dead_code)]
pub fn pb_apply(b: &mut PatchBuffer, buf: &mut [u8]) -> u32 {
    b.apply_to(buf)
}
#[allow(dead_code)]
pub fn pb_count(b: &PatchBuffer) -> usize {
    b.patch_count()
}
#[allow(dead_code)]
pub fn pb_is_empty(b: &PatchBuffer) -> bool {
    b.is_empty()
}
#[allow(dead_code)]
pub fn pb_applied_count(b: &PatchBuffer) -> u32 {
    b.applied_count()
}
#[allow(dead_code)]
pub fn pb_total_bytes(b: &PatchBuffer) -> usize {
    b.total_bytes()
}
#[allow(dead_code)]
pub fn pb_clear(b: &mut PatchBuffer) {
    b.clear();
}
#[allow(dead_code)]
pub fn pb_max_offset(b: &PatchBuffer) -> usize {
    b.max_offset()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_add_apply() {
        let mut b = new_patch_buffer();
        pb_add(&mut b, 2, &[9, 8]);
        let mut buf = [0u8; 8];
        pb_apply(&mut b, &mut buf);
        assert_eq!(buf[2], 9);
        assert_eq!(buf[3], 8);
    }
    #[test]
    fn test_clears_after_apply() {
        let mut b = new_patch_buffer();
        pb_add(&mut b, 0, &[1]);
        let mut buf = [0u8; 4];
        pb_apply(&mut b, &mut buf);
        assert!(pb_is_empty(&b));
    }
    #[test]
    fn test_applied_count() {
        let mut b = new_patch_buffer();
        pb_add(&mut b, 0, &[1]);
        let mut buf = [0u8; 4];
        pb_apply(&mut b, &mut buf);
        assert_eq!(pb_applied_count(&b), 1);
    }
    #[test]
    fn test_patch_count() {
        let mut b = new_patch_buffer();
        pb_add(&mut b, 0, &[1]);
        pb_add(&mut b, 1, &[2]);
        assert_eq!(pb_count(&b), 2);
    }
    #[test]
    fn test_total_bytes() {
        let mut b = new_patch_buffer();
        pb_add(&mut b, 0, &[1, 2, 3]);
        assert_eq!(pb_total_bytes(&b), 3);
    }
    #[test]
    fn test_out_of_bounds_skipped() {
        let mut b = new_patch_buffer();
        pb_add(&mut b, 100, &[1, 2]);
        let mut buf = [0u8; 4];
        let n = pb_apply(&mut b, &mut buf);
        assert_eq!(n, 0);
    }
    #[test]
    fn test_clear() {
        let mut b = new_patch_buffer();
        pb_add(&mut b, 0, &[1]);
        pb_clear(&mut b);
        assert!(pb_is_empty(&b));
    }
    #[test]
    fn test_max_offset() {
        let mut b = new_patch_buffer();
        pb_add(&mut b, 5, &[1, 2, 3]);
        assert_eq!(pb_max_offset(&b), 8);
    }
    #[test]
    fn test_empty_initially() {
        let b = new_patch_buffer();
        assert!(pb_is_empty(&b));
    }
    #[test]
    fn test_multiple_patches() {
        let mut b = new_patch_buffer();
        pb_add(&mut b, 0, &[0xAA]);
        pb_add(&mut b, 3, &[0xBB]);
        let mut buf = [0u8; 8];
        pb_apply(&mut b, &mut buf);
        assert_eq!(buf[0], 0xAA);
        assert_eq!(buf[3], 0xBB);
    }
}
