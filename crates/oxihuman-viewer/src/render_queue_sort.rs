#![allow(dead_code)]
//! Render queue sort: sorts renderable items by depth or priority.

/// A sortable item in the render queue.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct SortableItem {
    id: u32,
    depth: f32,
    priority: i32,
}

/// A render queue that can be sorted.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderQueueSort {
    items: Vec<SortableItem>,
}

/// Create a new empty render queue sort.
#[allow(dead_code)]
pub fn new_render_queue_sort() -> RenderQueueSort {
    RenderQueueSort { items: Vec::new() }
}

/// Add a sortable item.
#[allow(dead_code)]
pub fn add_sortable(queue: &mut RenderQueueSort, id: u32, depth: f32, priority: i32) {
    queue.items.push(SortableItem { id, depth, priority });
}

/// Sort the queue by priority first, then depth (front-to-back).
#[allow(dead_code)]
pub fn sort_queue(queue: &mut RenderQueueSort) {
    queue.items.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| a.depth.partial_cmp(&b.depth).unwrap_or(std::cmp::Ordering::Equal))
    });
}

/// Return the number of items.
#[allow(dead_code)]
pub fn queue_count_rqs(queue: &RenderQueueSort) -> usize {
    queue.items.len()
}

/// Sort front-to-back by depth.
#[allow(dead_code)]
pub fn front_to_back(queue: &mut RenderQueueSort) {
    queue
        .items
        .sort_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap_or(std::cmp::Ordering::Equal));
}

/// Sort back-to-front by depth.
#[allow(dead_code)]
pub fn back_to_front(queue: &mut RenderQueueSort) {
    queue
        .items
        .sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap_or(std::cmp::Ordering::Equal));
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn queue_to_json(queue: &RenderQueueSort) -> String {
    let entries: Vec<String> = queue
        .items
        .iter()
        .map(|i| format!("{{\"id\":{},\"depth\":{},\"priority\":{}}}", i.id, i.depth, i.priority))
        .collect();
    format!("{{\"items\":[{}]}}", entries.join(","))
}

/// Clear the queue.
#[allow(dead_code)]
pub fn queue_clear_rqs(queue: &mut RenderQueueSort) {
    queue.items.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_queue() {
        let q = new_render_queue_sort();
        assert_eq!(queue_count_rqs(&q), 0);
    }

    #[test]
    fn test_add_sortable() {
        let mut q = new_render_queue_sort();
        add_sortable(&mut q, 1, 5.0, 0);
        assert_eq!(queue_count_rqs(&q), 1);
    }

    #[test]
    fn test_sort_queue() {
        let mut q = new_render_queue_sort();
        add_sortable(&mut q, 1, 10.0, 1);
        add_sortable(&mut q, 2, 5.0, 0);
        sort_queue(&mut q);
        assert_eq!(q.items[0].id, 2);
    }

    #[test]
    fn test_front_to_back() {
        let mut q = new_render_queue_sort();
        add_sortable(&mut q, 1, 10.0, 0);
        add_sortable(&mut q, 2, 2.0, 0);
        front_to_back(&mut q);
        assert_eq!(q.items[0].id, 2);
    }

    #[test]
    fn test_back_to_front() {
        let mut q = new_render_queue_sort();
        add_sortable(&mut q, 1, 2.0, 0);
        add_sortable(&mut q, 2, 10.0, 0);
        back_to_front(&mut q);
        assert_eq!(q.items[0].id, 2);
    }

    #[test]
    fn test_to_json() {
        let q = new_render_queue_sort();
        let json = queue_to_json(&q);
        assert!(json.contains("\"items\":[]"));
    }

    #[test]
    fn test_clear() {
        let mut q = new_render_queue_sort();
        add_sortable(&mut q, 1, 5.0, 0);
        queue_clear_rqs(&mut q);
        assert_eq!(queue_count_rqs(&q), 0);
    }

    #[test]
    fn test_sort_empty() {
        let mut q = new_render_queue_sort();
        sort_queue(&mut q);
        assert_eq!(queue_count_rqs(&q), 0);
    }

    #[test]
    fn test_multiple_items() {
        let mut q = new_render_queue_sort();
        for i in 0..5 {
            add_sortable(&mut q, i, i as f32, 0);
        }
        assert_eq!(queue_count_rqs(&q), 5);
    }

    #[test]
    fn test_sort_same_priority() {
        let mut q = new_render_queue_sort();
        add_sortable(&mut q, 1, 10.0, 0);
        add_sortable(&mut q, 2, 1.0, 0);
        sort_queue(&mut q);
        assert_eq!(q.items[0].id, 2);
    }
}
