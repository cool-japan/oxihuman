#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// A handle to an object with generation tracking.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectHandle {
    index: u32,
    generation: u32,
}

/// Manages object handles with generation tracking.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HandleManager {
    generations: Vec<u32>,
    free_indices: Vec<u32>,
    count: usize,
}

#[allow(dead_code)]
pub fn new_handle_manager() -> HandleManager {
    HandleManager {
        generations: Vec::new(),
        free_indices: Vec::new(),
        count: 0,
    }
}

#[allow(dead_code)]
pub fn create_handle(mgr: &mut HandleManager) -> ObjectHandle {
    let (index, generation) = if let Some(idx) = mgr.free_indices.pop() {
        mgr.generations[idx as usize] += 1;
        (idx, mgr.generations[idx as usize])
    } else {
        let idx = mgr.generations.len() as u32;
        mgr.generations.push(0);
        (idx, 0)
    };
    mgr.count += 1;
    ObjectHandle { index, generation }
}

#[allow(dead_code)]
pub fn destroy_handle(mgr: &mut HandleManager, handle: ObjectHandle) -> bool {
    let idx = handle.index as usize;
    if idx < mgr.generations.len() && mgr.generations[idx] == handle.generation {
        mgr.generations[idx] += 1;
        mgr.free_indices.push(handle.index);
        mgr.count -= 1;
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn is_handle_valid(mgr: &HandleManager, handle: ObjectHandle) -> bool {
    let idx = handle.index as usize;
    idx < mgr.generations.len() && mgr.generations[idx] == handle.generation
}

#[allow(dead_code)]
pub fn handle_count_hm(mgr: &HandleManager) -> usize {
    mgr.count
}

#[allow(dead_code)]
pub fn handle_generation_hm(mgr: &HandleManager, handle: ObjectHandle) -> Option<u32> {
    let idx = handle.index as usize;
    if idx < mgr.generations.len() {
        Some(mgr.generations[idx])
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn handle_to_u64(handle: ObjectHandle) -> u64 {
    ((handle.generation as u64) << 32) | (handle.index as u64)
}

#[allow(dead_code)]
pub fn handle_manager_clear(mgr: &mut HandleManager) {
    mgr.generations.clear();
    mgr.free_indices.clear();
    mgr.count = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_handle_manager() {
        let mgr = new_handle_manager();
        assert_eq!(handle_count_hm(&mgr), 0);
    }

    #[test]
    fn test_create_handle() {
        let mut mgr = new_handle_manager();
        let h = create_handle(&mut mgr);
        assert!(is_handle_valid(&mgr, h));
    }

    #[test]
    fn test_destroy_handle() {
        let mut mgr = new_handle_manager();
        let h = create_handle(&mut mgr);
        assert!(destroy_handle(&mut mgr, h));
        assert!(!is_handle_valid(&mgr, h));
    }

    #[test]
    fn test_handle_count() {
        let mut mgr = new_handle_manager();
        create_handle(&mut mgr);
        create_handle(&mut mgr);
        assert_eq!(handle_count_hm(&mgr), 2);
    }

    #[test]
    fn test_handle_generation() {
        let mut mgr = new_handle_manager();
        let h = create_handle(&mut mgr);
        assert_eq!(handle_generation_hm(&mgr, h), Some(0));
    }

    #[test]
    fn test_handle_to_u64() {
        let h = ObjectHandle {
            index: 5,
            generation: 3,
        };
        let v = handle_to_u64(h);
        assert_eq!(v & 0xFFFF_FFFF, 5);
    }

    #[test]
    fn test_handle_manager_clear() {
        let mut mgr = new_handle_manager();
        create_handle(&mut mgr);
        handle_manager_clear(&mut mgr);
        assert_eq!(handle_count_hm(&mgr), 0);
    }

    #[test]
    fn test_recycle_index() {
        let mut mgr = new_handle_manager();
        let h1 = create_handle(&mut mgr);
        destroy_handle(&mut mgr, h1);
        let h2 = create_handle(&mut mgr);
        assert_eq!(h2.index, h1.index);
        assert_ne!(h2.generation, h1.generation);
    }

    #[test]
    fn test_destroy_invalid() {
        let mut mgr = new_handle_manager();
        let h = ObjectHandle {
            index: 999,
            generation: 0,
        };
        assert!(!destroy_handle(&mut mgr, h));
    }

    #[test]
    fn test_double_destroy() {
        let mut mgr = new_handle_manager();
        let h = create_handle(&mut mgr);
        destroy_handle(&mut mgr, h);
        assert!(!destroy_handle(&mut mgr, h));
    }
}
