//! Memory usage tracking and budget management.

/// Categories of memory allocations.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MemoryCategory {
    /// Mesh geometry buffers.
    Meshes,
    /// Texture / image data.
    Textures,
    /// Physics simulation data.
    Physics,
    /// Audio buffers.
    Audio,
    /// Uncategorised allocations.
    Other,
}

/// A record of a single allocation or free event.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct AllocationRecord {
    /// Human-readable label.
    pub label: String,
    /// Size in bytes.
    pub size: u64,
    /// Category.
    pub category: MemoryCategory,
    /// `true` for allocation, `false` for free.
    pub is_alloc: bool,
}

/// Tracks memory allocations, frees, budgets and peak usage.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct MemoryTracker {
    /// Running total of current usage in bytes.
    current: u64,
    /// Peak usage ever observed.
    peak: u64,
    /// Per-category current usage.
    by_category: Vec<(MemoryCategory, u64)>,
    /// Budget cap in bytes (0 = unlimited).
    budget: u64,
    /// Total allocation operations.
    alloc_count: u64,
    /// Total free operations.
    free_count: u64,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Find or insert a category entry in the tracker's category vec.
#[allow(dead_code)]
fn cat_index(tracker: &mut MemoryTracker, cat: &MemoryCategory) -> usize {
    if let Some(idx) = tracker.by_category.iter().position(|(c, _)| c == cat) {
        idx
    } else {
        tracker.by_category.push((cat.clone(), 0));
        tracker.by_category.len() - 1
    }
}

/// Find category entry (immutable).
#[allow(dead_code)]
fn cat_usage(tracker: &MemoryTracker, cat: &MemoryCategory) -> u64 {
    tracker
        .by_category
        .iter()
        .find(|(c, _)| c == cat)
        .map_or(0, |(_, v)| *v)
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

/// Create a new `MemoryTracker` with no allocations and no budget.
#[allow(dead_code)]
pub fn new_memory_tracker() -> MemoryTracker {
    MemoryTracker {
        current: 0,
        peak: 0,
        by_category: Vec::new(),
        budget: 0,
        alloc_count: 0,
        free_count: 0,
    }
}

// ---------------------------------------------------------------------------
// Track operations
// ---------------------------------------------------------------------------

/// Record an allocation of `size` bytes under `category`.
#[allow(dead_code)]
pub fn track_alloc(tracker: &mut MemoryTracker, size: u64, category: MemoryCategory) {
    tracker.current += size;
    tracker.alloc_count += 1;
    if tracker.current > tracker.peak {
        tracker.peak = tracker.current;
    }
    let idx = cat_index(tracker, &category);
    tracker.by_category[idx].1 += size;
}

/// Record a free of `size` bytes under `category`.
/// Saturates at zero if the free exceeds current usage.
#[allow(dead_code)]
pub fn track_free(tracker: &mut MemoryTracker, size: u64, category: MemoryCategory) {
    tracker.current = tracker.current.saturating_sub(size);
    tracker.free_count += 1;
    let idx = cat_index(tracker, &category);
    tracker.by_category[idx].1 = tracker.by_category[idx].1.saturating_sub(size);
}

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

/// Return the current total memory usage in bytes.
#[allow(dead_code)]
pub fn current_usage(tracker: &MemoryTracker) -> u64 {
    tracker.current
}

/// Return the current usage for a specific category.
#[allow(dead_code)]
pub fn usage_by_category(tracker: &MemoryTracker, category: &MemoryCategory) -> u64 {
    cat_usage(tracker, category)
}

/// Return the peak memory usage ever recorded.
#[allow(dead_code)]
pub fn peak_usage(tracker: &MemoryTracker) -> u64 {
    tracker.peak
}

/// Return the remaining bytes under the budget. Returns 0 if no budget is set
/// or if usage exceeds the budget.
#[allow(dead_code)]
pub fn budget_remaining(tracker: &MemoryTracker) -> u64 {
    if tracker.budget == 0 {
        return 0;
    }
    tracker.budget.saturating_sub(tracker.current)
}

/// Set the memory budget in bytes.  0 means unlimited.
#[allow(dead_code)]
pub fn set_budget(tracker: &mut MemoryTracker, budget: u64) {
    tracker.budget = budget;
}

/// Return `true` if current usage exceeds the budget (and budget > 0).
#[allow(dead_code)]
pub fn over_budget(tracker: &MemoryTracker) -> bool {
    tracker.budget > 0 && tracker.current > tracker.budget
}

/// Return the total number of allocation operations.
#[allow(dead_code)]
pub fn allocation_count(tracker: &MemoryTracker) -> u64 {
    tracker.alloc_count
}

/// Return the total number of free operations.
#[allow(dead_code)]
pub fn free_count(tracker: &MemoryTracker) -> u64 {
    tracker.free_count
}

/// Return the category with the highest current usage, or `None` if empty.
#[allow(dead_code)]
pub fn largest_category(tracker: &MemoryTracker) -> Option<MemoryCategory> {
    tracker
        .by_category
        .iter()
        .max_by_key(|(_, v)| *v)
        .map(|(c, _)| c.clone())
}

// ---------------------------------------------------------------------------
// Reset
// ---------------------------------------------------------------------------

/// Reset all counters and category data to zero.
#[allow(dead_code)]
pub fn reset_tracker(tracker: &mut MemoryTracker) {
    tracker.current = 0;
    tracker.peak = 0;
    tracker.by_category.clear();
    tracker.budget = 0;
    tracker.alloc_count = 0;
    tracker.free_count = 0;
}

// ---------------------------------------------------------------------------
// Serialization
// ---------------------------------------------------------------------------

/// Produce a minimal JSON representation of the tracker state.
#[allow(dead_code)]
pub fn memory_tracker_to_json(tracker: &MemoryTracker) -> String {
    let mut s = String::from("{");
    s.push_str(&format!("\"current\":{}", tracker.current));
    s.push_str(&format!(",\"peak\":{}", tracker.peak));
    s.push_str(&format!(",\"budget\":{}", tracker.budget));
    s.push_str(&format!(",\"alloc_count\":{}", tracker.alloc_count));
    s.push_str(&format!(",\"free_count\":{}", tracker.free_count));
    s.push_str(",\"categories\":{");
    for (i, (cat, val)) in tracker.by_category.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!("\"{:?}\":{}", cat, val));
    }
    s.push_str("}}");
    s
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_memory_tracker() {
        let t = new_memory_tracker();
        assert_eq!(current_usage(&t), 0);
        assert_eq!(peak_usage(&t), 0);
    }

    #[test]
    fn test_track_alloc() {
        let mut t = new_memory_tracker();
        track_alloc(&mut t, 1024, MemoryCategory::Meshes);
        assert_eq!(current_usage(&t), 1024);
        assert_eq!(allocation_count(&t), 1);
    }

    #[test]
    fn test_track_free() {
        let mut t = new_memory_tracker();
        track_alloc(&mut t, 1000, MemoryCategory::Textures);
        track_free(&mut t, 400, MemoryCategory::Textures);
        assert_eq!(current_usage(&t), 600);
        assert_eq!(free_count(&t), 1);
    }

    #[test]
    fn test_track_free_saturates() {
        let mut t = new_memory_tracker();
        track_alloc(&mut t, 100, MemoryCategory::Audio);
        track_free(&mut t, 999, MemoryCategory::Audio);
        assert_eq!(current_usage(&t), 0);
    }

    #[test]
    fn test_usage_by_category() {
        let mut t = new_memory_tracker();
        track_alloc(&mut t, 500, MemoryCategory::Meshes);
        track_alloc(&mut t, 300, MemoryCategory::Textures);
        assert_eq!(usage_by_category(&t, &MemoryCategory::Meshes), 500);
        assert_eq!(usage_by_category(&t, &MemoryCategory::Textures), 300);
        assert_eq!(usage_by_category(&t, &MemoryCategory::Physics), 0);
    }

    #[test]
    fn test_peak_usage() {
        let mut t = new_memory_tracker();
        track_alloc(&mut t, 1000, MemoryCategory::Meshes);
        track_alloc(&mut t, 500, MemoryCategory::Textures);
        track_free(&mut t, 1000, MemoryCategory::Meshes);
        assert_eq!(peak_usage(&t), 1500);
        assert_eq!(current_usage(&t), 500);
    }

    #[test]
    fn test_set_budget_and_remaining() {
        let mut t = new_memory_tracker();
        set_budget(&mut t, 2000);
        track_alloc(&mut t, 800, MemoryCategory::Audio);
        assert_eq!(budget_remaining(&t), 1200);
    }

    #[test]
    fn test_budget_remaining_no_budget() {
        let t = new_memory_tracker();
        assert_eq!(budget_remaining(&t), 0);
    }

    #[test]
    fn test_over_budget() {
        let mut t = new_memory_tracker();
        set_budget(&mut t, 100);
        track_alloc(&mut t, 200, MemoryCategory::Physics);
        assert!(over_budget(&t));
    }

    #[test]
    fn test_not_over_budget() {
        let mut t = new_memory_tracker();
        set_budget(&mut t, 1000);
        track_alloc(&mut t, 500, MemoryCategory::Meshes);
        assert!(!over_budget(&t));
    }

    #[test]
    fn test_allocation_and_free_counts() {
        let mut t = new_memory_tracker();
        track_alloc(&mut t, 100, MemoryCategory::Meshes);
        track_alloc(&mut t, 200, MemoryCategory::Textures);
        track_free(&mut t, 50, MemoryCategory::Meshes);
        assert_eq!(allocation_count(&t), 2);
        assert_eq!(free_count(&t), 1);
    }

    #[test]
    fn test_largest_category() {
        let mut t = new_memory_tracker();
        track_alloc(&mut t, 100, MemoryCategory::Meshes);
        track_alloc(&mut t, 500, MemoryCategory::Textures);
        track_alloc(&mut t, 200, MemoryCategory::Audio);
        assert_eq!(largest_category(&t), Some(MemoryCategory::Textures));
    }

    #[test]
    fn test_largest_category_empty() {
        let t = new_memory_tracker();
        assert!(largest_category(&t).is_none());
    }

    #[test]
    fn test_reset_tracker() {
        let mut t = new_memory_tracker();
        track_alloc(&mut t, 1000, MemoryCategory::Meshes);
        set_budget(&mut t, 5000);
        reset_tracker(&mut t);
        assert_eq!(current_usage(&t), 0);
        assert_eq!(peak_usage(&t), 0);
        assert_eq!(allocation_count(&t), 0);
        assert_eq!(free_count(&t), 0);
    }

    #[test]
    fn test_memory_tracker_to_json() {
        let mut t = new_memory_tracker();
        track_alloc(&mut t, 256, MemoryCategory::Meshes);
        let json = memory_tracker_to_json(&t);
        assert!(json.contains("\"current\":256"));
        assert!(json.contains("\"peak\":256"));
        assert!(json.contains("\"alloc_count\":1"));
    }

    #[test]
    fn test_multiple_categories_independent() {
        let mut t = new_memory_tracker();
        track_alloc(&mut t, 100, MemoryCategory::Meshes);
        track_alloc(&mut t, 200, MemoryCategory::Audio);
        track_free(&mut t, 100, MemoryCategory::Meshes);
        assert_eq!(usage_by_category(&t, &MemoryCategory::Meshes), 0);
        assert_eq!(usage_by_category(&t, &MemoryCategory::Audio), 200);
        assert_eq!(current_usage(&t), 200);
    }
}
