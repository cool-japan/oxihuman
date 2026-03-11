#![allow(dead_code)]
//! Priority queue for edge collapse operations.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollapseEntry {
    pub edge: (u32, u32),
    pub cost: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollapseQueue {
    entries: Vec<CollapseEntry>,
}

#[allow(dead_code)]
pub fn new_collapse_queue() -> CollapseQueue {
    CollapseQueue { entries: Vec::new() }
}

#[allow(dead_code)]
pub fn push_collapse(q: &mut CollapseQueue, edge: (u32, u32), cost: f32) {
    q.entries.push(CollapseEntry { edge, cost });
    q.entries.sort_by(|a, b| a.cost.partial_cmp(&b.cost).unwrap_or(std::cmp::Ordering::Equal));
}

#[allow(dead_code)]
pub fn pop_collapse(q: &mut CollapseQueue) -> Option<CollapseEntry> {
    if q.entries.is_empty() { None } else { Some(q.entries.remove(0)) }
}

#[allow(dead_code)]
pub fn collapse_queue_len(q: &CollapseQueue) -> usize { q.entries.len() }

#[allow(dead_code)]
pub fn collapse_is_empty(q: &CollapseQueue) -> bool { q.entries.is_empty() }

#[allow(dead_code)]
pub fn peek_collapse(q: &CollapseQueue) -> Option<&CollapseEntry> { q.entries.first() }

#[allow(dead_code)]
pub fn clear_collapse_queue(q: &mut CollapseQueue) { q.entries.clear(); }

#[allow(dead_code)]
pub fn collapse_queue_to_json(q: &CollapseQueue) -> String {
    let entries: Vec<String> = q.entries.iter().map(|e| {
        format!("{{\"edge\":[{},{}],\"cost\":{:.6}}}", e.edge.0, e.edge.1, e.cost)
    }).collect();
    format!("{{\"queue\":[{}]}}", entries.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_collapse_queue() {
        let q = new_collapse_queue();
        assert!(collapse_is_empty(&q));
    }
    #[test]
    fn test_push_and_len() {
        let mut q = new_collapse_queue();
        push_collapse(&mut q, (0, 1), 1.0);
        assert_eq!(collapse_queue_len(&q), 1);
    }
    #[test]
    fn test_pop_order() {
        let mut q = new_collapse_queue();
        push_collapse(&mut q, (0, 1), 2.0);
        push_collapse(&mut q, (1, 2), 0.5);
        let e = pop_collapse(&mut q).unwrap();
        assert!((e.cost - 0.5).abs() < 1e-6);
    }
    #[test]
    fn test_peek() {
        let mut q = new_collapse_queue();
        push_collapse(&mut q, (0, 1), 1.0);
        assert!(peek_collapse(&q).is_some());
    }
    #[test]
    fn test_clear() {
        let mut q = new_collapse_queue();
        push_collapse(&mut q, (0, 1), 1.0);
        clear_collapse_queue(&mut q);
        assert!(collapse_is_empty(&q));
    }
    #[test]
    fn test_pop_empty() {
        let mut q = new_collapse_queue();
        assert!(pop_collapse(&mut q).is_none());
    }
    #[test]
    fn test_to_json() {
        let mut q = new_collapse_queue();
        push_collapse(&mut q, (0, 1), 1.0);
        let j = collapse_queue_to_json(&q);
        assert!(j.contains("queue"));
    }
    #[test]
    fn test_multiple_push() {
        let mut q = new_collapse_queue();
        for i in 0..5 { push_collapse(&mut q, (i, i+1), i as f32); }
        assert_eq!(collapse_queue_len(&q), 5);
    }
    #[test]
    fn test_sorted_after_push() {
        let mut q = new_collapse_queue();
        push_collapse(&mut q, (0, 1), 3.0);
        push_collapse(&mut q, (1, 2), 1.0);
        push_collapse(&mut q, (2, 3), 2.0);
        let e = peek_collapse(&q).unwrap();
        assert!((e.cost - 1.0).abs() < 1e-6);
    }
    #[test]
    fn test_pop_all() {
        let mut q = new_collapse_queue();
        push_collapse(&mut q, (0, 1), 1.0);
        push_collapse(&mut q, (1, 2), 2.0);
        pop_collapse(&mut q);
        pop_collapse(&mut q);
        assert!(collapse_is_empty(&q));
    }
}
