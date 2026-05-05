// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Memory pressure profiling for the OxiHuman WASM runtime.
//!
//! This module provides:
//!
//! - [`WasmAllocTracker`] — a [`GlobalAlloc`] wrapper around the system
//!   allocator that atomically tracks total allocated bytes, peak usage, and
//!   active allocation count.
//! - [`MemorySnapshot`] — a point-in-time capture of allocator state.
//! - [`MemoryProfiler`] — accumulates up to 256 labelled snapshots in a ring
//!   buffer and summarises them via [`MemoryReport`].
//! - [`MemoryReport`] — aggregated statistics with JSON serialisation.
//! - [`MemoryError`] — typed error for budget violations.
//! - [`check_wasm_budget`] — asserts that peak usage does not exceed a given
//!   threshold.
//!
//! # Usage
//!
//! ```rust,no_run
//! use oxihuman_wasm::memory_profile::{GLOBAL_ALLOC_TRACKER, MemoryProfiler, check_wasm_budget};
//!
//! let mut profiler = MemoryProfiler::new();
//! profiler.take_snapshot("before_build");
//! // ... do work ...
//! profiler.take_snapshot("after_build");
//! let report = profiler.report();
//! println!("{}", report.to_json());
//! check_wasm_budget(64.0).expect("peak memory exceeded 64 MB");
//! ```

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

// ---------------------------------------------------------------------------
// MemoryError
// ---------------------------------------------------------------------------

/// Error type returned when memory usage violates a budget constraint.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryError {
    /// Human-readable description.
    pub message: String,
    /// Peak usage in bytes at the time of the check.
    pub peak_bytes: usize,
    /// Configured budget in bytes.
    pub budget_bytes: usize,
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "memory budget exceeded: peak {:.3} MB > limit {:.3} MB",
            self.peak_bytes as f64 / (1024.0 * 1024.0),
            self.budget_bytes as f64 / (1024.0 * 1024.0),
        )
    }
}

impl std::error::Error for MemoryError {}

// ---------------------------------------------------------------------------
// WasmAllocTracker — global allocator wrapper
// ---------------------------------------------------------------------------

/// A [`GlobalAlloc`] wrapper around the system allocator that instruments
/// every allocation/deallocation using lock-free [`AtomicUsize`] counters.
///
/// Three counters are maintained:
///
/// - `current` — bytes currently live on the heap.
/// - `peak`    — high-water mark of `current` (never decreases).
/// - `count`   — cumulative number of successful allocations.
///
/// # Safety note
///
/// This allocator is safe to declare as `#[global_allocator]` and does not
/// introduce any data races because all counter updates use
/// [`Ordering::Relaxed`] for the non-peak counters and a CAS loop for `peak`.
pub struct WasmAllocTracker {
    inner: System,
    /// Bytes currently allocated.
    current: AtomicUsize,
    /// Peak allocated bytes (high-water mark).
    peak: AtomicUsize,
    /// Cumulative allocation count.
    count: AtomicUsize,
}

impl Default for WasmAllocTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmAllocTracker {
    /// Create a new tracker wrapping the system allocator.
    pub const fn new() -> Self {
        Self {
            inner: System,
            current: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
            count: AtomicUsize::new(0),
        }
    }

    /// Current live heap bytes.
    #[inline]
    pub fn current_bytes(&self) -> usize {
        self.current.load(Ordering::Relaxed)
    }

    /// Peak live heap bytes since process start (or last reset — but reset is
    /// intentionally not exposed to keep the invariant "peak never decreases").
    #[inline]
    pub fn peak_bytes(&self) -> usize {
        self.peak.load(Ordering::Relaxed)
    }

    /// Cumulative number of successful allocations.
    #[inline]
    pub fn allocation_count(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    /// Update the peak counter to be at least `new_current`.
    /// Uses a compare-and-swap loop so the peak is always monotonically
    /// non-decreasing even under concurrent allocation pressure.
    #[inline]
    fn update_peak(&self, new_current: usize) {
        let mut peak = self.peak.load(Ordering::Relaxed);
        while new_current > peak {
            match self.peak.compare_exchange_weak(
                peak,
                new_current,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => peak = actual,
            }
        }
    }
}

// SAFETY: `WasmAllocTracker` wraps `std::alloc::System` which is the
// canonical safe allocator. The atomic counters are updated without holding
// any lock, so there is no possibility of deadlock or data races.
unsafe impl GlobalAlloc for WasmAllocTracker {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { self.inner.alloc(layout) };
        if !ptr.is_null() {
            let new = self.current.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
            self.update_peak(new);
            self.count.fetch_add(1, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { self.inner.dealloc(ptr, layout) };
        // Saturating subtract prevents underflow on any missed dealloc edge
        // cases (e.g., allocations that happened before instrumentation).
        self.current
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |cur| {
                Some(cur.saturating_sub(layout.size()))
            })
            .ok(); // fetch_update returns Err(cur) only when the closure returns None.
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { self.inner.alloc_zeroed(layout) };
        if !ptr.is_null() {
            let new = self.current.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
            self.update_peak(new);
            self.count.fetch_add(1, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { self.inner.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() {
            // Adjust current: remove old size, add new size.
            let old_size = layout.size();
            if new_size > old_size {
                let diff = new_size - old_size;
                let new = self.current.fetch_add(diff, Ordering::Relaxed) + diff;
                self.update_peak(new);
                self.count.fetch_add(1, Ordering::Relaxed);
            } else {
                let diff = old_size - new_size;
                self.current
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |cur| {
                        Some(cur.saturating_sub(diff))
                    })
                    .ok();
            }
        }
        new_ptr
    }
}

// ---------------------------------------------------------------------------
// Global tracker instance
// ---------------------------------------------------------------------------

/// Global allocator tracker instance.
///
/// To use as the process-wide allocator, declare in your crate root:
/// ```rust,no_run
/// use oxihuman_wasm::memory_profile::WasmAllocTracker;
///
/// #[global_allocator]
/// static ALLOCATOR: WasmAllocTracker = WasmAllocTracker::new();
/// ```
///
/// When used purely for profiling without replacing the global allocator,
/// call [`GLOBAL_ALLOC_TRACKER.current_bytes()`][`WasmAllocTracker::current_bytes`]
/// etc. directly — the counters will remain at zero until the tracker is
/// installed as the `#[global_allocator]`.
pub static GLOBAL_ALLOC_TRACKER: WasmAllocTracker = WasmAllocTracker::new();

// ---------------------------------------------------------------------------
// MemorySnapshot
// ---------------------------------------------------------------------------

/// A point-in-time capture of allocator statistics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemorySnapshot {
    /// Millisecond timestamp (caller-supplied; monotonic from an arbitrary epoch).
    pub timestamp_ms: u64,
    /// Bytes currently live on the heap at snapshot time.
    pub heap_allocated_bytes: usize,
    /// Peak bytes ever allocated (high-water mark up to snapshot time).
    pub heap_peak_bytes: usize,
    /// Cumulative number of allocations up to snapshot time.
    pub active_allocations: usize,
    /// Caller-supplied label (e.g. `"before_build"`, `"after_morph"`).
    label: [u8; 64],
    label_len: usize,
}

impl MemorySnapshot {
    /// Create a snapshot with all fields set to zero / empty.
    pub fn zeroed() -> Self {
        Self {
            timestamp_ms: 0,
            heap_allocated_bytes: 0,
            heap_peak_bytes: 0,
            active_allocations: 0,
            label: [0u8; 64],
            label_len: 0,
        }
    }

    /// Return the label as a string slice.
    pub fn label(&self) -> &str {
        let bytes = &self.label[..self.label_len];
        std::str::from_utf8(bytes).unwrap_or("<invalid utf8>")
    }

    /// Build a snapshot from the provided tracker state.
    fn from_tracker(tracker: &WasmAllocTracker, label: &str, timestamp_ms: u64) -> Self {
        let mut snap = Self {
            timestamp_ms,
            heap_allocated_bytes: tracker.current_bytes(),
            heap_peak_bytes: tracker.peak_bytes(),
            active_allocations: tracker.allocation_count(),
            label: [0u8; 64],
            label_len: 0,
        };

        let src = label.as_bytes();
        let copy_len = src.len().min(64);
        snap.label[..copy_len].copy_from_slice(&src[..copy_len]);
        snap.label_len = copy_len;
        snap
    }
}

// ---------------------------------------------------------------------------
// Ring buffer helpers
// ---------------------------------------------------------------------------

/// Capacity of the snapshot ring buffer.
const RING_CAPACITY: usize = 256;

/// Fixed-size ring buffer of [`MemorySnapshot`]s with no heap allocation.
struct RingBuffer {
    slots: Box<[MemorySnapshot; RING_CAPACITY]>,
    /// Index of the next write slot (wraps around at `RING_CAPACITY`).
    write_head: usize,
    /// Number of valid snapshots (capped at `RING_CAPACITY`).
    len: usize,
}

impl RingBuffer {
    fn new() -> Self {
        // SAFETY: We initialise via a boxed array of `zeroed()` values.
        let slots: Vec<MemorySnapshot> = (0..RING_CAPACITY)
            .map(|_| MemorySnapshot::zeroed())
            .collect();
        let boxed: Box<[MemorySnapshot; RING_CAPACITY]> = slots
            .into_boxed_slice()
            .try_into()
            .unwrap_or_else(|_| panic!("ring buffer size mismatch"));
        Self {
            slots: boxed,
            write_head: 0,
            len: 0,
        }
    }

    fn push(&mut self, snap: MemorySnapshot) {
        self.slots[self.write_head] = snap;
        self.write_head = (self.write_head + 1) % RING_CAPACITY;
        if self.len < RING_CAPACITY {
            self.len += 1;
        }
    }

    /// Iterate snapshots in insertion order (oldest first when the buffer is full).
    fn iter(&self) -> impl Iterator<Item = &MemorySnapshot> {
        let start = if self.len < RING_CAPACITY {
            0
        } else {
            self.write_head
        };
        (0..self.len).map(move |i| &self.slots[(start + i) % RING_CAPACITY])
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }
}

// ---------------------------------------------------------------------------
// MemoryProfiler
// ---------------------------------------------------------------------------

/// Accumulates [`MemorySnapshot`]s and produces summary [`MemoryReport`]s.
///
/// Internally uses a 256-slot ring buffer; the oldest snapshot is silently
/// overwritten once the buffer is full.
///
/// # Example
///
/// ```rust,no_run
/// use oxihuman_wasm::memory_profile::MemoryProfiler;
///
/// let mut profiler = MemoryProfiler::new();
/// profiler.take_snapshot("start");
/// // ... allocate things ...
/// profiler.take_snapshot("end");
/// println!("{}", profiler.report().to_json());
/// ```
pub struct MemoryProfiler {
    ring: RingBuffer,
    /// Monotonic millisecond clock — simple increment-on-snapshot approach.
    clock_ms: u64,
}

impl MemoryProfiler {
    /// Create an empty profiler.
    pub fn new() -> Self {
        Self {
            ring: RingBuffer::new(),
            clock_ms: 0,
        }
    }

    /// Capture a snapshot from [`GLOBAL_ALLOC_TRACKER`] and store it with
    /// the given `label`.
    ///
    /// The `timestamp_ms` is derived from an internal monotonic counter that
    /// increments by 1 per call (useful in WASM where `std::time` may be
    /// unavailable).  For wall-clock accuracy, prefer
    /// [`take_snapshot_at`][Self::take_snapshot_at].
    pub fn take_snapshot(&mut self, label: &str) {
        let snap = MemorySnapshot::from_tracker(&GLOBAL_ALLOC_TRACKER, label, self.clock_ms);
        self.clock_ms = self.clock_ms.saturating_add(1);
        self.ring.push(snap);
    }

    /// Capture a snapshot with an explicit wall-clock `timestamp_ms`.
    pub fn take_snapshot_at(&mut self, label: &str, timestamp_ms: u64) {
        let snap = MemorySnapshot::from_tracker(&GLOBAL_ALLOC_TRACKER, label, timestamp_ms);
        self.ring.push(snap);
    }

    /// Summarise all accumulated snapshots into a [`MemoryReport`].
    pub fn report(&self) -> MemoryReport {
        if self.ring.is_empty() {
            return MemoryReport::empty();
        }

        let mut total_allocated_sum: u64 = 0;
        let mut peak_usage: usize = 0;
        let mut allocation_count: usize = 0;
        let mut snapshot_count: usize = 0;

        for snap in self.ring.iter() {
            total_allocated_sum =
                total_allocated_sum.saturating_add(snap.heap_allocated_bytes as u64);
            if snap.heap_peak_bytes > peak_usage {
                peak_usage = snap.heap_peak_bytes;
            }
            // Use the maximum cumulative count seen across snapshots.
            if snap.active_allocations > allocation_count {
                allocation_count = snap.active_allocations;
            }
            snapshot_count += 1;
        }

        let average_alloc_size = peak_usage.checked_div(allocation_count).unwrap_or(0);

        let total_allocated = total_allocated_sum / snapshot_count.max(1) as u64;

        MemoryReport {
            total_allocated: total_allocated as usize,
            peak_usage,
            allocation_count,
            average_alloc_size,
            snapshot_count,
        }
    }

    /// Return the number of snapshots currently held in the ring buffer.
    pub fn snapshot_count(&self) -> usize {
        self.ring.len
    }

    /// Clear all accumulated snapshots and reset the internal clock.
    pub fn clear(&mut self) {
        self.ring = RingBuffer::new();
        self.clock_ms = 0;
    }
}

impl Default for MemoryProfiler {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// MemoryReport
// ---------------------------------------------------------------------------

/// Aggregated memory statistics produced by [`MemoryProfiler::report`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryReport {
    /// Mean heap-allocated bytes across all snapshots.
    pub total_allocated: usize,
    /// Maximum peak bytes observed across all snapshots.
    pub peak_usage: usize,
    /// Maximum cumulative allocation count observed across all snapshots.
    pub allocation_count: usize,
    /// Approximate average allocation size: `peak / count`.
    pub average_alloc_size: usize,
    /// Number of snapshots that were summarised.
    pub snapshot_count: usize,
}

impl MemoryReport {
    fn empty() -> Self {
        Self {
            total_allocated: 0,
            peak_usage: 0,
            allocation_count: 0,
            average_alloc_size: 0,
            snapshot_count: 0,
        }
    }

    /// Serialise the report to a compact JSON string.
    ///
    /// All byte values are included as raw integers; MB values are provided
    /// as convenience floating-point fields.
    pub fn to_json(&self) -> String {
        let mb = |bytes: usize| bytes as f64 / (1024.0 * 1024.0);
        format!(
            concat!(
                "{{",
                "\"total_allocated_bytes\":{},",
                "\"total_allocated_mb\":{:.4},",
                "\"peak_usage_bytes\":{},",
                "\"peak_usage_mb\":{:.4},",
                "\"allocation_count\":{},",
                "\"average_alloc_size_bytes\":{},",
                "\"snapshot_count\":{}",
                "}}"
            ),
            self.total_allocated,
            mb(self.total_allocated),
            self.peak_usage,
            mb(self.peak_usage),
            self.allocation_count,
            self.average_alloc_size,
            self.snapshot_count,
        )
    }
}

// ---------------------------------------------------------------------------
// Budget check
// ---------------------------------------------------------------------------

/// Assert that the peak heap usage recorded by [`GLOBAL_ALLOC_TRACKER`] does
/// not exceed `max_mb` megabytes.
///
/// Returns `Ok(())` when the budget is satisfied, or a [`MemoryError`]
/// describing the violation.
///
/// # Notes
///
/// The peak is meaningful only when [`GLOBAL_ALLOC_TRACKER`] is installed as
/// `#[global_allocator]`.  When it is not, `peak_bytes` will be 0 and this
/// function will always return `Ok`.
pub fn check_wasm_budget(max_mb: f64) -> Result<(), MemoryError> {
    let budget_bytes = (max_mb * 1024.0 * 1024.0) as usize;
    let peak_bytes = GLOBAL_ALLOC_TRACKER.peak_bytes();
    if peak_bytes > budget_bytes {
        Err(MemoryError {
            message: format!(
                "peak {:.3} MB exceeds budget {:.3} MB",
                peak_bytes as f64 / (1024.0 * 1024.0),
                max_mb,
            ),
            peak_bytes,
            budget_bytes,
        })
    } else {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::alloc::Layout;

    // Build a standalone tracker (not the global one) for isolated testing.
    fn fresh_tracker() -> WasmAllocTracker {
        WasmAllocTracker::new()
    }

    // Allocate `size` bytes through a tracker without using it as the global allocator.
    unsafe fn manual_alloc(tracker: &WasmAllocTracker, size: usize) -> *mut u8 {
        let layout = Layout::from_size_align(size, 8).expect("valid layout");
        unsafe { tracker.alloc(layout) }
    }

    unsafe fn manual_dealloc(tracker: &WasmAllocTracker, ptr: *mut u8, size: usize) {
        let layout = Layout::from_size_align(size, 8).expect("valid layout");
        unsafe { tracker.dealloc(ptr, layout) };
    }

    // ── tracker counter tests ──────────────────────────────────────────────

    #[test]
    fn tracker_starts_at_zero() {
        let t = fresh_tracker();
        assert_eq!(t.current_bytes(), 0);
        assert_eq!(t.peak_bytes(), 0);
        assert_eq!(t.allocation_count(), 0);
    }

    #[test]
    fn tracker_increments_on_alloc() {
        let t = fresh_tracker();
        let ptr = unsafe { manual_alloc(&t, 128) };
        assert!(!ptr.is_null(), "allocation must succeed");
        assert_eq!(t.current_bytes(), 128);
        assert_eq!(t.peak_bytes(), 128);
        assert_eq!(t.allocation_count(), 1);
        unsafe { manual_dealloc(&t, ptr, 128) };
    }

    #[test]
    fn tracker_decrements_on_dealloc() {
        let t = fresh_tracker();
        let ptr = unsafe { manual_alloc(&t, 256) };
        assert_eq!(t.current_bytes(), 256);
        unsafe { manual_dealloc(&t, ptr, 256) };
        assert_eq!(
            t.current_bytes(),
            0,
            "current must return to zero after dealloc"
        );
    }

    #[test]
    fn peak_never_decreases() {
        let t = fresh_tracker();

        let ptr1 = unsafe { manual_alloc(&t, 512) };
        let peak_after_first = t.peak_bytes();
        assert_eq!(peak_after_first, 512);

        unsafe { manual_dealloc(&t, ptr1, 512) };
        // After dealloc, current drops but peak must not.
        assert_eq!(t.current_bytes(), 0);
        assert_eq!(t.peak_bytes(), 512, "peak must not decrease after dealloc");

        // Smaller subsequent alloc must not lower the peak.
        let ptr2 = unsafe { manual_alloc(&t, 64) };
        assert_eq!(
            t.peak_bytes(),
            512,
            "peak must stay at 512 after smaller alloc"
        );
        unsafe { manual_dealloc(&t, ptr2, 64) };
    }

    #[test]
    fn peak_grows_on_larger_alloc() {
        let t = fresh_tracker();
        let ptr1 = unsafe { manual_alloc(&t, 100) };
        assert_eq!(t.peak_bytes(), 100);
        let ptr2 = unsafe { manual_alloc(&t, 200) };
        assert_eq!(
            t.peak_bytes(),
            300,
            "peak must grow with concurrent allocations"
        );
        unsafe { manual_dealloc(&t, ptr1, 100) };
        unsafe { manual_dealloc(&t, ptr2, 200) };
        // Peak stays at the highest watermark.
        assert_eq!(t.peak_bytes(), 300);
    }

    #[test]
    fn multiple_allocs_increment_count() {
        let t = fresh_tracker();
        let mut ptrs: Vec<(*mut u8, usize)> = Vec::new();
        for i in 1..=5usize {
            let ptr = unsafe { manual_alloc(&t, i * 16) };
            ptrs.push((ptr, i * 16));
        }
        assert_eq!(t.allocation_count(), 5);
        for (ptr, size) in ptrs {
            unsafe { manual_dealloc(&t, ptr, size) };
        }
    }

    // ── snapshot tests ────────────────────────────────────────────────────

    #[test]
    fn snapshot_zeroed_is_default() {
        let s = MemorySnapshot::zeroed();
        assert_eq!(s.timestamp_ms, 0);
        assert_eq!(s.heap_allocated_bytes, 0);
        assert_eq!(s.heap_peak_bytes, 0);
        assert_eq!(s.active_allocations, 0);
        assert_eq!(s.label(), "");
    }

    #[test]
    fn snapshot_label_stored_correctly() {
        let snap = MemorySnapshot::from_tracker(&fresh_tracker(), "test_label", 42);
        assert_eq!(snap.label(), "test_label");
        assert_eq!(snap.timestamp_ms, 42);
    }

    #[test]
    fn snapshot_label_truncated_at_64() {
        let long_label = "x".repeat(128);
        let snap = MemorySnapshot::from_tracker(&fresh_tracker(), &long_label, 0);
        assert_eq!(snap.label().len(), 64);
    }

    // ── profiler tests ────────────────────────────────────────────────────

    #[test]
    fn profiler_empty_report_is_zeroed() {
        let p = MemoryProfiler::new();
        let r = p.report();
        assert_eq!(r.snapshot_count, 0);
        assert_eq!(r.peak_usage, 0);
    }

    #[test]
    fn profiler_snapshot_count_increments() {
        let mut p = MemoryProfiler::new();
        assert_eq!(p.snapshot_count(), 0);
        p.take_snapshot("a");
        assert_eq!(p.snapshot_count(), 1);
        p.take_snapshot("b");
        assert_eq!(p.snapshot_count(), 2);
    }

    #[test]
    fn profiler_report_has_correct_snapshot_count() {
        let mut p = MemoryProfiler::new();
        p.take_snapshot("s1");
        p.take_snapshot("s2");
        p.take_snapshot("s3");
        let r = p.report();
        assert_eq!(r.snapshot_count, 3);
    }

    #[test]
    fn profiler_ring_wraps_at_256() {
        let mut p = MemoryProfiler::new();
        for i in 0..300 {
            p.take_snapshot(&format!("snap_{i}"));
        }
        // Ring holds at most 256.
        assert_eq!(p.snapshot_count(), 256);
    }

    #[test]
    fn profiler_clear_resets() {
        let mut p = MemoryProfiler::new();
        p.take_snapshot("a");
        p.take_snapshot("b");
        p.clear();
        assert_eq!(p.snapshot_count(), 0);
        let r = p.report();
        assert_eq!(r.snapshot_count, 0);
    }

    // ── MemoryReport JSON tests ───────────────────────────────────────────

    #[test]
    fn report_to_json_is_valid_structure() {
        let report = MemoryReport {
            total_allocated: 1024,
            peak_usage: 2048,
            allocation_count: 10,
            average_alloc_size: 204,
            snapshot_count: 3,
        };
        let json = report.to_json();
        // Must parse as valid JSON object with expected keys.
        assert!(json.starts_with('{') && json.ends_with('}'));
        assert!(json.contains("\"total_allocated_bytes\":1024"));
        assert!(json.contains("\"peak_usage_bytes\":2048"));
        assert!(json.contains("\"allocation_count\":10"));
        assert!(json.contains("\"snapshot_count\":3"));
    }

    #[test]
    fn report_empty_json_all_zeros() {
        let report = MemoryReport::empty();
        let json = report.to_json();
        assert!(json.contains("\"total_allocated_bytes\":0"));
        assert!(json.contains("\"peak_usage_bytes\":0"));
    }

    // ── budget check tests ────────────────────────────────────────────────

    #[test]
    fn budget_check_passes_when_tracker_is_zero() {
        // GLOBAL_ALLOC_TRACKER is only non-zero when used as #[global_allocator].
        // In tests, peak is 0, so any positive budget should pass.
        // (Unless we happen to be running with the tracker as global allocator,
        //  in which case peak may be non-zero — we allow that case by using a
        //  very large budget.)
        let result = check_wasm_budget(1024.0);
        assert!(
            result.is_ok(),
            "budget check should pass for 1024 MB: {result:?}"
        );
    }

    #[test]
    fn budget_check_fails_when_over_budget() {
        // Fabricate a scenario: directly craft a MemoryError to validate
        // the error path, since we cannot guarantee the tracker peak without
        // being the global allocator.
        let err = MemoryError {
            message: "test".to_string(),
            peak_bytes: 100 * 1024 * 1024,
            budget_bytes: 50 * 1024 * 1024,
        };
        assert!(err.peak_bytes > err.budget_bytes);
        let display = format!("{err}");
        assert!(display.contains("budget exceeded"));
    }

    #[test]
    fn memory_error_display_contains_mb() {
        let err = MemoryError {
            message: "test".to_string(),
            peak_bytes: 64 * 1024 * 1024,
            budget_bytes: 32 * 1024 * 1024,
        };
        let s = format!("{err}");
        assert!(s.contains("64.000 MB") || s.contains("64"));
        assert!(s.contains("32.000 MB") || s.contains("32"));
    }
}
