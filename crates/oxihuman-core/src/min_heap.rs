//! Min-heap — fixed-capacity binary min-heap for priority scheduling
//! using f32 key and u32 value pairs.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MinHeapConfig {
    pub capacity: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeapEntry {
    pub key: f32,
    pub value: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MinHeap {
    pub config: MinHeapConfig,
    pub data: Vec<HeapEntry>,
}

#[allow(dead_code)]
pub fn default_min_heap_config() -> MinHeapConfig {
    MinHeapConfig { capacity: 64 }
}

#[allow(dead_code)]
pub fn new_min_heap(config: MinHeapConfig) -> MinHeap {
    MinHeap {
        data: Vec::with_capacity(config.capacity),
        config,
    }
}

/// Sift entry at `idx` up toward root to restore min-heap property.
fn sift_up(data: &mut [HeapEntry], mut idx: usize) {
    while idx > 0 {
        let parent = (idx - 1) / 2;
        if data[idx].key < data[parent].key {
            data.swap(idx, parent);
            idx = parent;
        } else {
            break;
        }
    }
}

/// Sift entry at `idx` down to restore min-heap property.
fn sift_down(data: &mut [HeapEntry], mut idx: usize) {
    let len = data.len();
    loop {
        let left = 2 * idx + 1;
        let right = 2 * idx + 2;
        let mut smallest = idx;

        if left < len && data[left].key < data[smallest].key {
            smallest = left;
        }
        if right < len && data[right].key < data[smallest].key {
            smallest = right;
        }

        if smallest == idx {
            break;
        }
        data.swap(idx, smallest);
        idx = smallest;
    }
}

#[allow(dead_code)]
pub fn heap_push(heap: &mut MinHeap, key: f32, value: u32) -> bool {
    if heap.data.len() >= heap.config.capacity {
        return false;
    }
    heap.data.push(HeapEntry { key, value });
    let idx = heap.data.len() - 1;
    sift_up(&mut heap.data, idx);
    true
}

#[allow(dead_code)]
pub fn heap_pop(heap: &mut MinHeap) -> Option<HeapEntry> {
    if heap.data.is_empty() {
        return None;
    }
    let n = heap.data.len();
    heap.data.swap(0, n - 1);
    let entry = heap.data.pop();
    if !heap.data.is_empty() {
        sift_down(&mut heap.data, 0);
    }
    entry
}

#[allow(dead_code)]
pub fn heap_peek(heap: &MinHeap) -> Option<&HeapEntry> {
    heap.data.first()
}

#[allow(dead_code)]
pub fn heap_len(heap: &MinHeap) -> usize {
    heap.data.len()
}

#[allow(dead_code)]
pub fn heap_is_empty(heap: &MinHeap) -> bool {
    heap.data.is_empty()
}

#[allow(dead_code)]
pub fn heap_is_full(heap: &MinHeap) -> bool {
    heap.data.len() >= heap.config.capacity
}

#[allow(dead_code)]
pub fn heap_to_json(heap: &MinHeap) -> String {
    let entries: Vec<String> = heap
        .data
        .iter()
        .map(|e| format!("{{\"key\":{:.4},\"value\":{}}}", e.key, e.value))
        .collect();
    format!(
        "{{\"capacity\":{},\"len\":{},\"entries\":[{}]}}",
        heap.config.capacity,
        heap.data.len(),
        entries.join(",")
    )
}

#[allow(dead_code)]
pub fn heap_clear(heap: &mut MinHeap) {
    heap.data.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_heap() -> MinHeap {
        new_min_heap(MinHeapConfig { capacity: 8 })
    }

    #[test]
    fn test_push_pop_order() {
        let mut h = make_heap();
        heap_push(&mut h, 3.0, 3);
        heap_push(&mut h, 1.0, 1);
        heap_push(&mut h, 2.0, 2);
        let first = heap_pop(&mut h).expect("should succeed");
        assert!((first.key - 1.0).abs() < 1e-6);
        let second = heap_pop(&mut h).expect("should succeed");
        assert!((second.key - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_empty_pop_returns_none() {
        let mut h = make_heap();
        assert!(heap_pop(&mut h).is_none());
    }

    #[test]
    fn test_peek_does_not_remove() {
        let mut h = make_heap();
        heap_push(&mut h, 1.0, 1);
        let _ = heap_peek(&h);
        assert_eq!(heap_len(&h), 1);
    }

    #[test]
    fn test_is_empty() {
        let mut h = make_heap();
        assert!(heap_is_empty(&h));
        heap_push(&mut h, 1.0, 1);
        assert!(!heap_is_empty(&h));
    }

    #[test]
    fn test_is_full() {
        let mut h = new_min_heap(MinHeapConfig { capacity: 2 });
        heap_push(&mut h, 1.0, 1);
        heap_push(&mut h, 2.0, 2);
        assert!(heap_is_full(&h));
        assert!(!heap_push(&mut h, 3.0, 3));
    }

    #[test]
    fn test_clear() {
        let mut h = make_heap();
        heap_push(&mut h, 1.0, 1);
        heap_clear(&mut h);
        assert!(heap_is_empty(&h));
    }

    #[test]
    fn test_to_json() {
        let mut h = make_heap();
        heap_push(&mut h, 0.5, 99);
        let j = heap_to_json(&h);
        assert!(j.contains("capacity"));
        assert!(j.contains("entries"));
    }

    #[test]
    fn test_heap_property_maintained() {
        let mut h = make_heap();
        for i in [5u32, 3, 7, 1, 4, 6, 2, 8] {
            heap_push(&mut h, i as f32, i);
        }
        let mut prev = f32::NEG_INFINITY;
        while let Some(e) = heap_pop(&mut h) {
            assert!(e.key >= prev);
            prev = e.key;
        }
    }

    #[test]
    fn test_default_config_capacity() {
        let c = default_min_heap_config();
        assert_eq!(c.capacity, 64);
    }
}
