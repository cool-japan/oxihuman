//! Circular FIFO queue — fixed-capacity circular buffer for streaming
//! event and command processing.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CircularQueueConfig {
    pub capacity: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CircularQueue<T: Clone> {
    pub config: CircularQueueConfig,
    buffer: Vec<Option<T>>,
    head: usize,
    tail: usize,
    len: usize,
}

#[allow(dead_code)]
pub fn default_circular_queue_config() -> CircularQueueConfig {
    CircularQueueConfig { capacity: 64 }
}

#[allow(dead_code)]
pub fn new_circular_queue<T: Clone>(config: CircularQueueConfig) -> CircularQueue<T> {
    let cap = config.capacity;
    CircularQueue {
        buffer: vec![None; cap],
        head: 0,
        tail: 0,
        len: 0,
        config,
    }
}

#[allow(dead_code)]
pub fn cq_push<T: Clone>(q: &mut CircularQueue<T>, item: T) -> bool {
    if q.len >= q.config.capacity {
        return false;
    }
    q.buffer[q.tail] = Some(item);
    q.tail = (q.tail + 1) % q.config.capacity;
    q.len += 1;
    true
}

#[allow(dead_code)]
pub fn cq_pop<T: Clone>(q: &mut CircularQueue<T>) -> Option<T> {
    if q.len == 0 {
        return None;
    }
    let item = q.buffer[q.head].take();
    q.head = (q.head + 1) % q.config.capacity;
    q.len -= 1;
    item
}

#[allow(dead_code)]
pub fn cq_peek<T: Clone>(q: &CircularQueue<T>) -> Option<&T> {
    if q.len == 0 {
        return None;
    }
    q.buffer[q.head].as_ref()
}

#[allow(dead_code)]
pub fn cq_len<T: Clone>(q: &CircularQueue<T>) -> usize {
    q.len
}

#[allow(dead_code)]
pub fn cq_is_empty<T: Clone>(q: &CircularQueue<T>) -> bool {
    q.len == 0
}

#[allow(dead_code)]
pub fn cq_is_full<T: Clone>(q: &CircularQueue<T>) -> bool {
    q.len >= q.config.capacity
}

#[allow(dead_code)]
pub fn cq_capacity<T: Clone>(q: &CircularQueue<T>) -> usize {
    q.config.capacity
}

#[allow(dead_code)]
pub fn cq_to_json<T: Clone>(q: &CircularQueue<T>) -> String {
    format!(
        "{{\"capacity\":{},\"len\":{},\"head\":{},\"tail\":{}}}",
        q.config.capacity, q.len, q.head, q.tail
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_q() -> CircularQueue<u32> {
        new_circular_queue(CircularQueueConfig { capacity: 4 })
    }

    #[test]
    fn test_push_pop_fifo() {
        let mut q = make_q();
        cq_push(&mut q, 1u32);
        cq_push(&mut q, 2u32);
        cq_push(&mut q, 3u32);
        assert_eq!(cq_pop(&mut q), Some(1));
        assert_eq!(cq_pop(&mut q), Some(2));
        assert_eq!(cq_pop(&mut q), Some(3));
    }

    #[test]
    fn test_empty_pop_none() {
        let mut q = make_q();
        assert_eq!(cq_pop(&mut q), None);
    }

    #[test]
    fn test_full_push_returns_false() {
        let mut q = make_q();
        cq_push(&mut q, 1u32);
        cq_push(&mut q, 2u32);
        cq_push(&mut q, 3u32);
        cq_push(&mut q, 4u32);
        assert!(!cq_push(&mut q, 5u32));
    }

    #[test]
    fn test_is_empty() {
        let mut q = make_q();
        assert!(cq_is_empty(&q));
        cq_push(&mut q, 1u32);
        assert!(!cq_is_empty(&q));
    }

    #[test]
    fn test_is_full() {
        let mut q = make_q();
        for i in 0..4 {
            cq_push(&mut q, i);
        }
        assert!(cq_is_full(&q));
    }

    #[test]
    fn test_wrap_around() {
        let mut q = make_q();
        cq_push(&mut q, 1u32);
        cq_push(&mut q, 2u32);
        cq_pop(&mut q);
        cq_pop(&mut q);
        cq_push(&mut q, 3u32);
        cq_push(&mut q, 4u32);
        assert_eq!(cq_pop(&mut q), Some(3));
        assert_eq!(cq_pop(&mut q), Some(4));
    }

    #[test]
    fn test_peek_does_not_remove() {
        let mut q = make_q();
        cq_push(&mut q, 42u32);
        let _ = cq_peek(&q);
        assert_eq!(cq_len(&q), 1);
    }

    #[test]
    fn test_capacity() {
        let q = make_q();
        assert_eq!(cq_capacity(&q), 4);
    }

    #[test]
    fn test_to_json() {
        let mut q = make_q();
        cq_push(&mut q, 1u32);
        let j = cq_to_json(&q);
        assert!(j.contains("capacity"));
        assert!(j.contains("len"));
    }

    #[test]
    fn test_default_config() {
        let c = default_circular_queue_config();
        assert_eq!(c.capacity, 64);
    }
}
