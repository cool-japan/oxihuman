// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Camera bookmark system for saving and restoring named viewpoints.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraBookmarkEntry {
    pub name: String,
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub fov_deg: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraBookmarkStore {
    pub bookmarks: Vec<CameraBookmarkEntry>,
    pub max_bookmarks: usize,
}

#[allow(dead_code)]
pub fn new_camera_bookmark_store(max: usize) -> CameraBookmarkStore {
    CameraBookmarkStore {
        bookmarks: Vec::new(),
        max_bookmarks: max,
    }
}

#[allow(dead_code)]
pub fn add_bookmark(store: &mut CameraBookmarkStore, entry: CameraBookmarkEntry) -> bool {
    if store.bookmarks.len() >= store.max_bookmarks {
        return false;
    }
    store.bookmarks.push(entry);
    true
}

#[allow(dead_code)]
pub fn remove_bookmark(store: &mut CameraBookmarkStore, name: &str) -> bool {
    let before = store.bookmarks.len();
    store.bookmarks.retain(|b| b.name != name);
    store.bookmarks.len() < before
}

#[allow(dead_code)]
pub fn find_bookmark<'a>(store: &'a CameraBookmarkStore, name: &str) -> Option<&'a CameraBookmarkEntry> {
    store.bookmarks.iter().find(|b| b.name == name)
}

#[allow(dead_code)]
pub fn bookmark_count(store: &CameraBookmarkStore) -> usize {
    store.bookmarks.len()
}

#[allow(dead_code)]
pub fn bookmark_names(store: &CameraBookmarkStore) -> Vec<String> {
    store.bookmarks.iter().map(|b| b.name.clone()).collect()
}

#[allow(dead_code)]
pub fn lerp_bookmark(a: &CameraBookmarkEntry, b: &CameraBookmarkEntry, t: f32) -> CameraBookmarkEntry {
    let t = t.clamp(0.0, 1.0);
    CameraBookmarkEntry {
        name: if t < 0.5 { a.name.clone() } else { b.name.clone() },
        position: [
            a.position[0] + (b.position[0] - a.position[0]) * t,
            a.position[1] + (b.position[1] - a.position[1]) * t,
            a.position[2] + (b.position[2] - a.position[2]) * t,
        ],
        target: [
            a.target[0] + (b.target[0] - a.target[0]) * t,
            a.target[1] + (b.target[1] - a.target[1]) * t,
            a.target[2] + (b.target[2] - a.target[2]) * t,
        ],
        fov_deg: a.fov_deg + (b.fov_deg - a.fov_deg) * t,
    }
}

#[allow(dead_code)]
pub fn bookmark_to_json(entry: &CameraBookmarkEntry) -> String {
    format!(
        r#"{{"name":"{}","pos":[{},{},{}],"target":[{},{},{}],"fov":{}}}"#,
        entry.name,
        entry.position[0], entry.position[1], entry.position[2],
        entry.target[0], entry.target[1], entry.target[2],
        entry.fov_deg
    )
}

#[allow(dead_code)]
pub fn clear_bookmarks(store: &mut CameraBookmarkStore) {
    store.bookmarks.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(name: &str) -> CameraBookmarkEntry {
        CameraBookmarkEntry {
            name: name.to_string(),
            position: [0.0, 1.0, -3.0],
            target: [0.0, 0.9, 0.0],
            fov_deg: 60.0,
        }
    }

    #[test]
    fn test_new_store() {
        let s = new_camera_bookmark_store(10);
        assert_eq!(s.max_bookmarks, 10);
        assert!(s.bookmarks.is_empty());
    }

    #[test]
    fn test_add_bookmark() {
        let mut s = new_camera_bookmark_store(5);
        assert!(add_bookmark(&mut s, sample_entry("front")));
        assert_eq!(bookmark_count(&s), 1);
    }

    #[test]
    fn test_add_over_limit() {
        let mut s = new_camera_bookmark_store(1);
        assert!(add_bookmark(&mut s, sample_entry("a")));
        assert!(!add_bookmark(&mut s, sample_entry("b")));
    }

    #[test]
    fn test_remove_bookmark() {
        let mut s = new_camera_bookmark_store(5);
        add_bookmark(&mut s, sample_entry("front"));
        assert!(remove_bookmark(&mut s, "front"));
        assert_eq!(bookmark_count(&s), 0);
    }

    #[test]
    fn test_find_bookmark() {
        let mut s = new_camera_bookmark_store(5);
        add_bookmark(&mut s, sample_entry("side"));
        assert!(find_bookmark(&s, "side").is_some());
        assert!(find_bookmark(&s, "back").is_none());
    }

    #[test]
    fn test_bookmark_names() {
        let mut s = new_camera_bookmark_store(5);
        add_bookmark(&mut s, sample_entry("a"));
        add_bookmark(&mut s, sample_entry("b"));
        let names = bookmark_names(&s);
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn test_lerp_bookmark() {
        let a = sample_entry("a");
        let mut b = sample_entry("b");
        b.fov_deg = 80.0;
        let mid = lerp_bookmark(&a, &b, 0.5);
        assert!((mid.fov_deg - 70.0).abs() < 1e-4);
    }

    #[test]
    fn test_to_json() {
        let e = sample_entry("test");
        let j = bookmark_to_json(&e);
        assert!(j.contains("test"));
    }

    #[test]
    fn test_clear() {
        let mut s = new_camera_bookmark_store(5);
        add_bookmark(&mut s, sample_entry("x"));
        clear_bookmarks(&mut s);
        assert!(s.bookmarks.is_empty());
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut s = new_camera_bookmark_store(5);
        assert!(!remove_bookmark(&mut s, "nope"));
    }
}
