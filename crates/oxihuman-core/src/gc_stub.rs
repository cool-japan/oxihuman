// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Mark-and-sweep GC stub — demonstrates the two-phase collection cycle using
//! object indices instead of actual heap pointers.

/// Identifier for a GC-managed object.
pub type GcId = usize;

/// State of a GC object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GcState {
    White, /* unreachable — candidate for collection */
    Grey,  /* reachable but children not yet scanned */
    Black, /* reachable and fully scanned */
}

/// A managed object with an adjacency list of references.
#[derive(Debug)]
pub struct GcObject {
    pub id: GcId,
    pub state: GcState,
    pub refs: Vec<GcId>,
}

impl GcObject {
    fn new(id: GcId) -> Self {
        Self {
            id,
            state: GcState::White,
            refs: Vec::new(),
        }
    }
}

/// Simple mark-and-sweep garbage collector stub.
pub struct GcStub {
    objects: Vec<GcObject>,
    roots: Vec<GcId>,
    collected: usize,
}

impl GcStub {
    /// Create an empty GC.
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            roots: Vec::new(),
            collected: 0,
        }
    }

    /// Allocate a new managed object. Returns its id.
    pub fn alloc(&mut self) -> GcId {
        let id = self.objects.len();
        self.objects.push(GcObject::new(id));
        id
    }

    /// Add a reference from object `from` to object `to`.
    pub fn add_ref(&mut self, from: GcId, to: GcId) {
        if from < self.objects.len() {
            self.objects[from].refs.push(to);
        }
    }

    /// Mark `id` as a root (always reachable).
    pub fn add_root(&mut self, id: GcId) {
        self.roots.push(id);
    }

    /// Run mark phase: mark all objects reachable from roots.
    pub fn mark(&mut self) {
        /* reset all to white */
        for obj in &mut self.objects {
            obj.state = GcState::White;
        }
        /* seed grey set from roots */
        let mut grey: Vec<GcId> = self.roots.clone();
        for &r in &self.roots {
            if r < self.objects.len() {
                self.objects[r].state = GcState::Grey;
            }
        }
        while let Some(id) = grey.pop() {
            if id >= self.objects.len() {
                continue;
            }
            let children: Vec<GcId> = self.objects[id].refs.clone();
            for child in children {
                if child < self.objects.len() && self.objects[child].state == GcState::White {
                    self.objects[child].state = GcState::Grey;
                    grey.push(child);
                }
            }
            self.objects[id].state = GcState::Black;
        }
    }

    /// Run sweep phase: remove all white objects and return count collected.
    pub fn sweep(&mut self) -> usize {
        let before = self.objects.len();
        self.objects.retain(|o| o.state != GcState::White);
        let freed = before - self.objects.len();
        self.collected += freed;
        freed
    }

    /// Full collection cycle (mark + sweep).
    pub fn collect(&mut self) -> usize {
        self.mark();
        self.sweep()
    }

    /// Total number of live managed objects.
    pub fn live_count(&self) -> usize {
        self.objects.len()
    }

    /// Cumulative objects collected since creation.
    pub fn total_collected(&self) -> usize {
        self.collected
    }
}

impl Default for GcStub {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new GC stub.
pub fn new_gc_stub() -> GcStub {
    GcStub::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc() {
        let mut gc = GcStub::new();
        let id = gc.alloc();
        assert_eq!(id, 0); /* first object has id 0 */
    }

    #[test]
    fn test_collect_unreachable() {
        let mut gc = GcStub::new();
        gc.alloc(); /* unreachable object */
        let freed = gc.collect();
        assert_eq!(freed, 1); /* one object collected */
    }

    #[test]
    fn test_root_not_collected() {
        let mut gc = GcStub::new();
        let id = gc.alloc();
        gc.add_root(id);
        let freed = gc.collect();
        assert_eq!(freed, 0); /* root survives */
    }

    #[test]
    fn test_reachable_via_root() {
        let mut gc = GcStub::new();
        let root = gc.alloc();
        let child = gc.alloc();
        gc.add_root(root);
        gc.add_ref(root, child);
        let freed = gc.collect();
        assert_eq!(freed, 0); /* both reachable */
    }

    #[test]
    fn test_unreachable_child() {
        let mut gc = GcStub::new();
        let root = gc.alloc();
        let orphan = gc.alloc();
        gc.add_root(root);
        let _ = orphan;
        let freed = gc.collect();
        assert_eq!(freed, 1); /* orphan collected */
    }

    #[test]
    fn test_live_count() {
        let mut gc = GcStub::new();
        let id = gc.alloc();
        gc.add_root(id);
        gc.collect();
        assert_eq!(gc.live_count(), 1); /* root alive */
    }

    #[test]
    fn test_total_collected() {
        let mut gc = GcStub::new();
        gc.alloc();
        gc.alloc();
        gc.collect();
        assert_eq!(gc.total_collected(), 2); /* cumulative count */
    }

    #[test]
    fn test_multiple_roots() {
        let mut gc = GcStub::new();
        let a = gc.alloc();
        let b = gc.alloc();
        gc.add_root(a);
        gc.add_root(b);
        let freed = gc.collect();
        assert_eq!(freed, 0); /* both roots survive */
    }

    #[test]
    fn test_default() {
        let gc = GcStub::default();
        assert_eq!(gc.live_count(), 0); /* default creates empty GC */
    }

    #[test]
    fn test_new_helper() {
        let gc = new_gc_stub();
        assert_eq!(gc.live_count(), 0); /* helper creates empty GC */
    }
}
