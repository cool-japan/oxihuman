#![allow(dead_code)]
//! Priority queue for edge split operations.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SplitEntry { pub edge: (u32, u32), pub length: f32 }

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SplitQueue { entries: Vec<SplitEntry> }

#[allow(dead_code)]
pub fn new_split_queue() -> SplitQueue { SplitQueue { entries: Vec::new() } }

#[allow(dead_code)]
pub fn push_split(q: &mut SplitQueue, edge: (u32, u32), length: f32) {
    q.entries.push(SplitEntry { edge, length });
    q.entries.sort_by(|a, b| b.length.partial_cmp(&a.length).unwrap_or(std::cmp::Ordering::Equal));
}

#[allow(dead_code)]
pub fn pop_split(q: &mut SplitQueue) -> Option<SplitEntry> {
    if q.entries.is_empty() { None } else { Some(q.entries.remove(0)) }
}

#[allow(dead_code)]
pub fn split_queue_len(q: &SplitQueue) -> usize { q.entries.len() }
#[allow(dead_code)]
pub fn split_is_empty(q: &SplitQueue) -> bool { q.entries.is_empty() }
#[allow(dead_code)]
pub fn peek_split(q: &SplitQueue) -> Option<&SplitEntry> { q.entries.first() }
#[allow(dead_code)]
pub fn clear_split_queue(q: &mut SplitQueue) { q.entries.clear(); }

#[allow(dead_code)]
pub fn split_queue_to_json(q: &SplitQueue) -> String {
    let es: Vec<String> = q.entries.iter().map(|e| format!("{{\"edge\":[{},{}],\"length\":{:.6}}}", e.edge.0, e.edge.1, e.length)).collect();
    format!("{{\"queue\":[{}]}}", es.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { assert!(split_is_empty(&new_split_queue())); }
    #[test] fn test_push() { let mut q = new_split_queue(); push_split(&mut q, (0,1), 1.0); assert_eq!(split_queue_len(&q), 1); }
    #[test] fn test_pop_order() { let mut q = new_split_queue(); push_split(&mut q, (0,1), 1.0); push_split(&mut q, (1,2), 3.0); let e = pop_split(&mut q).expect("should succeed"); assert!((e.length - 3.0).abs() < 1e-6); }
    #[test] fn test_peek() { let mut q = new_split_queue(); push_split(&mut q, (0,1), 1.0); assert!(peek_split(&q).is_some()); }
    #[test] fn test_clear() { let mut q = new_split_queue(); push_split(&mut q, (0,1), 1.0); clear_split_queue(&mut q); assert!(split_is_empty(&q)); }
    #[test] fn test_pop_empty() { assert!(pop_split(&mut new_split_queue()).is_none()); }
    #[test] fn test_json() { let mut q = new_split_queue(); push_split(&mut q, (0,1), 1.0); assert!(split_queue_to_json(&q).contains("queue")); }
    #[test] fn test_multi() { let mut q = new_split_queue(); for i in 0..5 { push_split(&mut q, (i,i+1), i as f32); } assert_eq!(split_queue_len(&q), 5); }
    #[test] fn test_sorted() { let mut q = new_split_queue(); push_split(&mut q, (0,1), 1.0); push_split(&mut q, (1,2), 5.0); push_split(&mut q, (2,3), 3.0); assert!((peek_split(&q).expect("should succeed").length - 5.0).abs() < 1e-6); }
    #[test] fn test_pop_all() { let mut q = new_split_queue(); push_split(&mut q, (0,1), 1.0); pop_split(&mut q); assert!(split_is_empty(&q)); }
}
