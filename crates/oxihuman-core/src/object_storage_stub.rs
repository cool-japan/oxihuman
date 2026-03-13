// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Object storage (S3-like) stub.

use std::collections::HashMap;

/// Object metadata.
#[derive(Debug, Clone)]
pub struct ObjectMeta {
    pub key: String,
    pub size_bytes: u64,
    pub etag: String,
    pub content_type: String,
}

impl ObjectMeta {
    pub fn new(key: &str, size_bytes: u64) -> Self {
        ObjectMeta {
            key: key.to_string(),
            size_bytes,
            etag: format!("{:x}", size_bytes ^ 0xDEAD_BEEF),
            content_type: "application/octet-stream".to_string(),
        }
    }
}

/// An object stored in the stub.
#[derive(Debug, Clone)]
pub struct StoredObject {
    pub meta: ObjectMeta,
    pub data: Vec<u8>,
}

impl StoredObject {
    pub fn new(key: &str, data: Vec<u8>) -> Self {
        let size = data.len() as u64;
        StoredObject {
            meta: ObjectMeta::new(key, size),
            data,
        }
    }
}

/// In-memory object storage stub.
pub struct ObjectStorage {
    bucket: String,
    objects: HashMap<String, StoredObject>,
}

impl ObjectStorage {
    pub fn new(bucket: &str) -> Self {
        ObjectStorage {
            bucket: bucket.to_string(),
            objects: HashMap::new(),
        }
    }

    pub fn put(&mut self, key: &str, data: Vec<u8>) {
        self.objects
            .insert(key.to_string(), StoredObject::new(key, data));
    }

    pub fn get(&self, key: &str) -> Option<&StoredObject> {
        self.objects.get(key)
    }

    pub fn delete(&mut self, key: &str) -> bool {
        self.objects.remove(key).is_some()
    }

    pub fn list(&self, prefix: &str) -> Vec<&ObjectMeta> {
        self.objects
            .values()
            .filter(|o| o.meta.key.starts_with(prefix))
            .map(|o| &o.meta)
            .collect()
    }

    pub fn exists(&self, key: &str) -> bool {
        self.objects.contains_key(key)
    }

    pub fn object_count(&self) -> usize {
        self.objects.len()
    }

    pub fn total_size(&self) -> u64 {
        self.objects.values().map(|o| o.meta.size_bytes).sum()
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }
}

impl Default for ObjectStorage {
    fn default() -> Self {
        Self::new("default-bucket")
    }
}

/// Create a new object storage.
pub fn new_object_storage(bucket: &str) -> ObjectStorage {
    ObjectStorage::new(bucket)
}

/// Upload bytes and return the key.
pub fn upload(storage: &mut ObjectStorage, key: &str, data: &[u8]) -> String {
    storage.put(key, data.to_vec());
    key.to_string()
}

/// Download bytes for a key.
pub fn download(storage: &ObjectStorage, key: &str) -> Option<Vec<u8>> {
    storage.get(key).map(|o| o.data.clone())
}

/// Copy object from one key to another.
pub fn copy_object(storage: &mut ObjectStorage, src_key: &str, dst_key: &str) -> bool {
    if let Some(obj) = storage.get(src_key).cloned() {
        storage.put(dst_key, obj.data);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_and_get() {
        let mut s = new_object_storage("bucket");
        s.put("key1", b"hello".to_vec());
        assert!(s.get("key1").is_some());
    }

    #[test]
    fn test_get_missing() {
        let s = new_object_storage("bucket");
        assert!(s.get("nonexistent").is_none());
    }

    #[test]
    fn test_delete() {
        let mut s = new_object_storage("bucket");
        s.put("k", b"data".to_vec());
        assert!(s.delete("k"));
        assert!(!s.exists("k"));
    }

    #[test]
    fn test_list_prefix() {
        let mut s = new_object_storage("bucket");
        s.put("images/a.png", b"img".to_vec());
        s.put("docs/b.txt", b"doc".to_vec());
        let imgs = s.list("images/");
        assert_eq!(imgs.len(), 1);
    }

    #[test]
    fn test_total_size() {
        let mut s = new_object_storage("bucket");
        s.put("a", vec![0u8; 100]);
        s.put("b", vec![0u8; 200]);
        assert_eq!(s.total_size(), 300);
    }

    #[test]
    fn test_upload_download() {
        let mut s = new_object_storage("bucket");
        upload(&mut s, "file.bin", &[1, 2, 3]);
        let data = download(&s, "file.bin").expect("should succeed");
        assert_eq!(data, vec![1, 2, 3]);
    }

    #[test]
    fn test_copy_object() {
        let mut s = new_object_storage("bucket");
        s.put("src", b"payload".to_vec());
        assert!(copy_object(&mut s, "src", "dst"));
        assert!(s.exists("dst"));
    }

    #[test]
    fn test_copy_missing_returns_false() {
        let mut s = new_object_storage("bucket");
        assert!(!copy_object(&mut s, "missing", "dst"));
    }

    #[test]
    fn test_object_count() {
        let mut s = new_object_storage("bucket");
        s.put("x", vec![]);
        s.put("y", vec![]);
        assert_eq!(s.object_count(), 2);
    }

    #[test]
    fn test_bucket_name() {
        let s = ObjectStorage::new("my-bucket");
        assert_eq!(s.bucket(), "my-bucket");
    }
}
