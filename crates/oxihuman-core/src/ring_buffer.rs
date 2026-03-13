//! Fixed-capacity ring buffer (circular queue) for streaming data.

#[allow(dead_code)]
pub struct RingBuffer<T> {
    pub data: Vec<Option<T>>,
    pub head: usize,
    pub tail: usize,
    pub count: usize,
    pub capacity: usize,
}

#[allow(dead_code)]
pub fn new_ring_buffer<T: Default + Clone>(capacity: usize) -> RingBuffer<T> {
    let cap = if capacity == 0 { 1 } else { capacity };
    RingBuffer {
        data: vec![None; cap],
        head: 0,
        tail: 0,
        count: 0,
        capacity: cap,
    }
}

/// Push item into the buffer. Returns `false` if the buffer is full.
#[allow(dead_code)]
pub fn ring_push<T: Clone>(buf: &mut RingBuffer<T>, item: T) -> bool {
    if buf.count == buf.capacity {
        return false;
    }
    buf.data[buf.tail] = Some(item);
    buf.tail = (buf.tail + 1) % buf.capacity;
    buf.count += 1;
    true
}

#[allow(dead_code)]
pub fn ring_pop<T: Clone>(buf: &mut RingBuffer<T>) -> Option<T> {
    if buf.count == 0 {
        return None;
    }
    let item = buf.data[buf.head].take();
    buf.head = (buf.head + 1) % buf.capacity;
    buf.count -= 1;
    item
}

#[allow(dead_code)]
pub fn ring_peek<T>(buf: &RingBuffer<T>) -> Option<&T> {
    if buf.count == 0 {
        return None;
    }
    buf.data[buf.head].as_ref()
}

#[allow(dead_code)]
pub fn ring_len<T>(buf: &RingBuffer<T>) -> usize {
    buf.count
}

#[allow(dead_code)]
pub fn ring_capacity<T>(buf: &RingBuffer<T>) -> usize {
    buf.capacity
}

#[allow(dead_code)]
pub fn ring_is_empty<T>(buf: &RingBuffer<T>) -> bool {
    buf.count == 0
}

#[allow(dead_code)]
pub fn ring_is_full<T>(buf: &RingBuffer<T>) -> bool {
    buf.count == buf.capacity
}

#[allow(dead_code)]
pub fn ring_clear<T>(buf: &mut RingBuffer<T>) {
    for slot in buf.data.iter_mut() {
        *slot = None;
    }
    buf.head = 0;
    buf.tail = 0;
    buf.count = 0;
}

/// Push an item, overwriting the oldest element if the buffer is full.
#[allow(dead_code)]
pub fn ring_push_overwrite<T: Clone>(buf: &mut RingBuffer<T>, item: T) {
    if buf.count == buf.capacity {
        // Drop the oldest
        buf.head = (buf.head + 1) % buf.capacity;
        buf.count -= 1;
    }
    buf.data[buf.tail] = Some(item);
    buf.tail = (buf.tail + 1) % buf.capacity;
    buf.count += 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ring_buffer() {
        let buf: RingBuffer<f32> = new_ring_buffer(4);
        assert_eq!(ring_capacity(&buf), 4);
        assert_eq!(ring_len(&buf), 0);
        assert!(ring_is_empty(&buf));
        assert!(!ring_is_full(&buf));
    }

    #[test]
    fn test_push_and_pop() {
        let mut buf: RingBuffer<f32> = new_ring_buffer(4);
        assert!(ring_push(&mut buf, 1.0));
        assert!(ring_push(&mut buf, 2.0));
        assert_eq!(ring_len(&buf), 2);
        let v = ring_pop(&mut buf).expect("should succeed");
        assert!((v - 1.0).abs() < 1e-6);
        let v = ring_pop(&mut buf).expect("should succeed");
        assert!((v - 2.0).abs() < 1e-6);
        assert!(ring_is_empty(&buf));
    }

    #[test]
    fn test_push_full_returns_false() {
        let mut buf: RingBuffer<u8> = new_ring_buffer(2);
        assert!(ring_push(&mut buf, 1));
        assert!(ring_push(&mut buf, 2));
        assert!(ring_is_full(&buf));
        assert!(!ring_push(&mut buf, 3));
    }

    #[test]
    fn test_pop_empty_returns_none() {
        let mut buf: RingBuffer<f32> = new_ring_buffer(2);
        assert!(ring_pop(&mut buf).is_none());
    }

    #[test]
    fn test_peek() {
        let mut buf: RingBuffer<f32> = new_ring_buffer(4);
        ring_push(&mut buf, 42.0);
        ring_push(&mut buf, 99.0);
        let peeked = ring_peek(&buf).copied().expect("should succeed");
        assert!((peeked - 42.0).abs() < 1e-6);
        assert_eq!(ring_len(&buf), 2);
    }

    #[test]
    fn test_ring_clear() {
        let mut buf: RingBuffer<f32> = new_ring_buffer(4);
        ring_push(&mut buf, 1.0);
        ring_push(&mut buf, 2.0);
        ring_clear(&mut buf);
        assert_eq!(ring_len(&buf), 0);
        assert!(ring_is_empty(&buf));
    }

    #[test]
    fn test_push_overwrite() {
        let mut buf: RingBuffer<u8> = new_ring_buffer(3);
        ring_push(&mut buf, 1);
        ring_push(&mut buf, 2);
        ring_push(&mut buf, 3);
        // Buffer full; overwrite oldest (1)
        ring_push_overwrite(&mut buf, 4);
        assert_eq!(ring_len(&buf), 3);
        let v = ring_pop(&mut buf).expect("should succeed");
        assert_eq!(v, 2);
    }

    #[test]
    fn test_push_overwrite_not_full() {
        let mut buf: RingBuffer<u8> = new_ring_buffer(4);
        ring_push_overwrite(&mut buf, 10);
        ring_push_overwrite(&mut buf, 20);
        assert_eq!(ring_len(&buf), 2);
        let v = ring_pop(&mut buf).expect("should succeed");
        assert_eq!(v, 10);
    }

    #[test]
    fn test_fifo_ordering() {
        let mut buf: RingBuffer<u8> = new_ring_buffer(8);
        for i in 0u8..5 {
            ring_push(&mut buf, i);
        }
        for i in 0u8..5 {
            let v = ring_pop(&mut buf).expect("should succeed");
            assert_eq!(v, i);
        }
    }

    #[test]
    fn test_wrap_around() {
        let mut buf: RingBuffer<u8> = new_ring_buffer(3);
        ring_push(&mut buf, 1);
        ring_push(&mut buf, 2);
        ring_pop(&mut buf);
        ring_push(&mut buf, 3);
        ring_push(&mut buf, 4);
        // Should contain 2, 3, 4
        assert_eq!(ring_len(&buf), 3);
        assert_eq!(ring_pop(&mut buf).expect("should succeed"), 2);
        assert_eq!(ring_pop(&mut buf).expect("should succeed"), 3);
        assert_eq!(ring_pop(&mut buf).expect("should succeed"), 4);
    }
}
