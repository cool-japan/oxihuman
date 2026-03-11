//! Monotonically-increasing ID allocator with optional recycling of freed IDs.
//!
//! IDs start at 1 (0 is reserved as the null/invalid ID). When recycling is
//! enabled, freed IDs are pushed onto a free-list and reused before the counter
//! is advanced.

/// Configuration for the ID allocator.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IdAllocatorConfig {
    /// Whether freed IDs are recycled for future allocations.
    pub recycle: bool,
    /// Maximum ID value before the allocator is considered exhausted (0 = no limit).
    pub max_id: u32,
}

/// A monotonically-increasing ID allocator.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IdAllocator {
    config: IdAllocatorConfig,
    next: u32,
    live: std::collections::HashSet<u32>,
    free_list: Vec<u32>,
    peak: u32,
}

/// Return a default [`IdAllocatorConfig`].
#[allow(dead_code)]
pub fn default_id_allocator_config() -> IdAllocatorConfig {
    IdAllocatorConfig {
        recycle: true,
        max_id: 0,
    }
}

/// Create a new [`IdAllocator`].
#[allow(dead_code)]
pub fn new_id_allocator(config: IdAllocatorConfig) -> IdAllocator {
    IdAllocator {
        config,
        next: 1,
        live: std::collections::HashSet::new(),
        free_list: Vec::new(),
        peak: 0,
    }
}

/// Allocate a new ID.  Returns `None` if the allocator is exhausted.
#[allow(dead_code)]
pub fn id_alloc(alloc: &mut IdAllocator) -> Option<u32> {
    let id = if alloc.config.recycle && !alloc.free_list.is_empty() {
        alloc.free_list.pop()?
    } else {
        let candidate = alloc.next;
        if alloc.config.max_id > 0 && candidate > alloc.config.max_id {
            return None;
        }
        alloc.next += 1;
        candidate
    };
    alloc.live.insert(id);
    if id > alloc.peak {
        alloc.peak = id;
    }
    Some(id)
}

/// Free an ID.  Returns `true` if the ID was live and has now been freed.
#[allow(dead_code)]
pub fn id_free(alloc: &mut IdAllocator, id: u32) -> bool {
    if !alloc.live.remove(&id) {
        return false;
    }
    if alloc.config.recycle {
        alloc.free_list.push(id);
    }
    true
}

/// Return `true` if the given ID is currently live (allocated and not freed).
#[allow(dead_code)]
pub fn id_is_alive(alloc: &IdAllocator, id: u32) -> bool {
    alloc.live.contains(&id)
}

/// Return the number of IDs currently in the free-list (recycled but not reused).
#[allow(dead_code)]
pub fn id_recycled_count(alloc: &IdAllocator) -> usize {
    alloc.free_list.len()
}

/// Return the number of IDs currently live.
#[allow(dead_code)]
pub fn id_live_count(alloc: &IdAllocator) -> usize {
    alloc.live.len()
}

/// Return the highest ID ever allocated.
#[allow(dead_code)]
pub fn id_peak(alloc: &IdAllocator) -> u32 {
    alloc.peak
}

/// Serialize the allocator state to a compact JSON string.
#[allow(dead_code)]
pub fn id_allocator_to_json(alloc: &IdAllocator) -> String {
    format!(
        r#"{{"live_count":{},"recycled_count":{},"peak":{},"next":{}}}"#,
        alloc.live.len(),
        alloc.free_list.len(),
        alloc.peak,
        alloc.next,
    )
}

/// Reset the allocator to its initial state (clears live set, free-list, and counter).
#[allow(dead_code)]
pub fn id_allocator_reset(alloc: &mut IdAllocator) {
    alloc.next = 1;
    alloc.live.clear();
    alloc.free_list.clear();
    alloc.peak = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_starts_at_one() {
        let mut a = new_id_allocator(default_id_allocator_config());
        let id = id_alloc(&mut a).unwrap();
        assert_eq!(id, 1);
    }

    #[test]
    fn test_alloc_monotone_without_recycle() {
        let cfg = IdAllocatorConfig {
            recycle: false,
            max_id: 0,
        };
        let mut a = new_id_allocator(cfg);
        let i1 = id_alloc(&mut a).unwrap();
        id_free(&mut a, i1);
        let i2 = id_alloc(&mut a).unwrap();
        assert!(i2 > i1);
    }

    #[test]
    fn test_free_and_recycle() {
        let mut a = new_id_allocator(default_id_allocator_config());
        let id1 = id_alloc(&mut a).unwrap();
        id_free(&mut a, id1);
        assert_eq!(id_recycled_count(&a), 1);
        let id2 = id_alloc(&mut a).unwrap();
        assert_eq!(id1, id2); // recycled
        assert_eq!(id_recycled_count(&a), 0);
    }

    #[test]
    fn test_is_alive() {
        let mut a = new_id_allocator(default_id_allocator_config());
        let id = id_alloc(&mut a).unwrap();
        assert!(id_is_alive(&a, id));
        id_free(&mut a, id);
        assert!(!id_is_alive(&a, id));
    }

    #[test]
    fn test_free_unknown_returns_false() {
        let mut a = new_id_allocator(default_id_allocator_config());
        assert!(!id_free(&mut a, 999));
    }

    #[test]
    fn test_peak_tracks_highest() {
        let mut a = new_id_allocator(default_id_allocator_config());
        for _ in 0..5 {
            id_alloc(&mut a);
        }
        assert_eq!(id_peak(&a), 5);
    }

    #[test]
    fn test_live_count() {
        let mut a = new_id_allocator(default_id_allocator_config());
        id_alloc(&mut a);
        id_alloc(&mut a);
        assert_eq!(id_live_count(&a), 2);
    }

    #[test]
    fn test_max_id_exhaustion() {
        let cfg = IdAllocatorConfig {
            recycle: false,
            max_id: 2,
        };
        let mut a = new_id_allocator(cfg);
        assert!(id_alloc(&mut a).is_some());
        assert!(id_alloc(&mut a).is_some());
        assert!(id_alloc(&mut a).is_none());
    }

    #[test]
    fn test_reset() {
        let mut a = new_id_allocator(default_id_allocator_config());
        id_alloc(&mut a);
        id_alloc(&mut a);
        id_allocator_reset(&mut a);
        assert_eq!(id_live_count(&a), 0);
        assert_eq!(id_peak(&a), 0);
        let id = id_alloc(&mut a).unwrap();
        assert_eq!(id, 1);
    }

    #[test]
    fn test_to_json() {
        let a = new_id_allocator(default_id_allocator_config());
        let json = id_allocator_to_json(&a);
        assert!(json.contains("live_count"));
        assert!(json.contains("peak"));
    }
}
