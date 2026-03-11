// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A generational handle map: stable handles survive removal of other entries.

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle {
    index: u32,
    generation: u32,
}

#[allow(dead_code)]
impl Handle {
    pub fn index(self) -> u32 { self.index }
    pub fn generation(self) -> u32 { self.generation }
}

#[allow(dead_code)]
struct Entry<T> {
    value: Option<T>,
    generation: u32,
}

#[allow(dead_code)]
pub struct HandleMap<T> {
    entries: Vec<Entry<T>>,
    free_list: Vec<u32>,
}

#[allow(dead_code)]
impl<T> HandleMap<T> {
    pub fn new() -> Self {
        Self { entries: Vec::new(), free_list: Vec::new() }
    }

    pub fn insert(&mut self, value: T) -> Handle {
        if let Some(idx) = self.free_list.pop() {
            let e = &mut self.entries[idx as usize];
            e.generation += 1;
            e.value = Some(value);
            Handle { index: idx, generation: e.generation }
        } else {
            let idx = self.entries.len() as u32;
            self.entries.push(Entry { value: Some(value), generation: 0 });
            Handle { index: idx, generation: 0 }
        }
    }

    pub fn remove(&mut self, handle: Handle) -> Option<T> {
        let idx = handle.index as usize;
        if idx < self.entries.len()
            && self.entries[idx].generation == handle.generation
            && self.entries[idx].value.is_some()
        {
            self.free_list.push(handle.index);
            self.entries[idx].value.take()
        } else {
            None
        }
    }

    pub fn get(&self, handle: Handle) -> Option<&T> {
        let idx = handle.index as usize;
        if idx < self.entries.len() && self.entries[idx].generation == handle.generation {
            self.entries[idx].value.as_ref()
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut T> {
        let idx = handle.index as usize;
        if idx < self.entries.len() && self.entries[idx].generation == handle.generation {
            self.entries[idx].value.as_mut()
        } else {
            None
        }
    }

    pub fn contains(&self, handle: Handle) -> bool {
        self.get(handle).is_some()
    }

    pub fn len(&self) -> usize {
        self.entries.iter().filter(|e| e.value.is_some()).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.free_list.clear();
    }
}

impl<T> Default for HandleMap<T> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_get() {
        let mut m = HandleMap::new();
        let h = m.insert(42);
        assert_eq!(m.get(h), Some(&42));
    }

    #[test]
    fn test_remove() {
        let mut m = HandleMap::new();
        let h = m.insert(10);
        assert_eq!(m.remove(h), Some(10));
        assert_eq!(m.get(h), None);
    }

    #[test]
    fn test_stale_handle() {
        let mut m = HandleMap::new();
        let h1 = m.insert(1);
        m.remove(h1);
        let _h2 = m.insert(2);
        assert_eq!(m.get(h1), None);
    }

    #[test]
    fn test_get_mut() {
        let mut m = HandleMap::new();
        let h = m.insert(5);
        *m.get_mut(h).unwrap() = 10;
        assert_eq!(m.get(h), Some(&10));
    }

    #[test]
    fn test_contains() {
        let mut m = HandleMap::new();
        let h = m.insert(1);
        assert!(m.contains(h));
        m.remove(h);
        assert!(!m.contains(h));
    }

    #[test]
    fn test_len() {
        let mut m = HandleMap::new();
        m.insert(1);
        m.insert(2);
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn test_is_empty() {
        let m: HandleMap<i32> = HandleMap::new();
        assert!(m.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut m = HandleMap::new();
        m.insert(1);
        m.clear();
        assert!(m.is_empty());
    }

    #[test]
    fn test_handle_fields() {
        let mut m = HandleMap::new();
        let h = m.insert(99);
        assert_eq!(h.index(), 0);
        assert_eq!(h.generation(), 0);
    }

    #[test]
    fn test_reuse_bumps_generation() {
        let mut m = HandleMap::new();
        let h1 = m.insert(1);
        m.remove(h1);
        let h2 = m.insert(2);
        assert_eq!(h2.index(), h1.index());
        assert_eq!(h2.generation(), 1);
    }
}
