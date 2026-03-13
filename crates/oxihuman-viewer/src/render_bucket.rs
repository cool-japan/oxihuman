// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Type of render bucket.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BucketType {
    Opaque,
    Transparent,
    Overlay,
    Shadow,
}

/// Entry in a render bucket.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct BucketEntry {
    sort_key: u64,
    mesh_id: u32,
}

/// A render bucket for draw call sorting.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderBucket {
    bucket_type: BucketType,
    entries: Vec<BucketEntry>,
}

/// Create a new render bucket.
#[allow(dead_code)]
pub fn new_render_bucket(bucket_type: BucketType) -> RenderBucket {
    RenderBucket {
        bucket_type,
        entries: Vec::new(),
    }
}

/// Add an item to the bucket.
#[allow(dead_code)]
pub fn add_to_bucket(bucket: &mut RenderBucket, mesh_id: u32, sort_key: u64) {
    bucket.entries.push(BucketEntry { sort_key, mesh_id });
}

/// Return the number of items in the bucket.
#[allow(dead_code)]
pub fn bucket_size(bucket: &RenderBucket) -> usize {
    bucket.entries.len()
}

/// Return the bucket type name.
#[allow(dead_code)]
pub fn bucket_type_name(bucket: &RenderBucket) -> &'static str {
    match bucket.bucket_type {
        BucketType::Opaque => "opaque",
        BucketType::Transparent => "transparent",
        BucketType::Overlay => "overlay",
        BucketType::Shadow => "shadow",
    }
}

/// Sort the bucket by sort key.
#[allow(dead_code)]
pub fn sort_bucket(bucket: &mut RenderBucket) {
    bucket.entries.sort_by_key(|e| e.sort_key);
}

/// Flush (clear) the bucket.
#[allow(dead_code)]
pub fn flush_bucket(bucket: &mut RenderBucket) {
    bucket.entries.clear();
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn bucket_to_json(bucket: &RenderBucket) -> String {
    format!(
        "{{\"type\":\"{}\",\"count\":{}}}",
        bucket_type_name(bucket),
        bucket.entries.len()
    )
}

/// Clear the bucket.
#[allow(dead_code)]
pub fn bucket_clear(bucket: &mut RenderBucket) {
    bucket.entries.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bucket() {
        let b = new_render_bucket(BucketType::Opaque);
        assert_eq!(bucket_size(&b), 0);
    }

    #[test]
    fn add_item() {
        let mut b = new_render_bucket(BucketType::Opaque);
        add_to_bucket(&mut b, 1, 100);
        assert_eq!(bucket_size(&b), 1);
    }

    #[test]
    fn type_name() {
        let b = new_render_bucket(BucketType::Transparent);
        assert_eq!(bucket_type_name(&b), "transparent");
    }

    #[test]
    fn sort_does_not_panic() {
        let mut b = new_render_bucket(BucketType::Opaque);
        add_to_bucket(&mut b, 1, 200);
        add_to_bucket(&mut b, 2, 100);
        sort_bucket(&mut b);
        assert_eq!(bucket_size(&b), 2);
    }

    #[test]
    fn flush_clears() {
        let mut b = new_render_bucket(BucketType::Opaque);
        add_to_bucket(&mut b, 1, 100);
        flush_bucket(&mut b);
        assert_eq!(bucket_size(&b), 0);
    }

    #[test]
    fn to_json() {
        let b = new_render_bucket(BucketType::Shadow);
        let j = bucket_to_json(&b);
        assert!(j.contains("shadow"));
    }

    #[test]
    fn clear_works() {
        let mut b = new_render_bucket(BucketType::Overlay);
        add_to_bucket(&mut b, 1, 1);
        bucket_clear(&mut b);
        assert_eq!(bucket_size(&b), 0);
    }

    #[test]
    fn overlay_type() {
        let b = new_render_bucket(BucketType::Overlay);
        assert_eq!(bucket_type_name(&b), "overlay");
    }

    #[test]
    fn multiple_items() {
        let mut b = new_render_bucket(BucketType::Opaque);
        for i in 0..5u32 {
            add_to_bucket(&mut b, i, i as u64);
        }
        assert_eq!(bucket_size(&b), 5);
    }

    #[test]
    fn shadow_type() {
        let b = new_render_bucket(BucketType::Shadow);
        assert_eq!(bucket_type_name(&b), "shadow");
    }
}
