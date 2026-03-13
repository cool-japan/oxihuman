#![allow(dead_code)]

use std::collections::VecDeque;

/// A message with an id and payload.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub id: u64,
    pub topic: String,
    pub payload: String,
}

/// A simple message queue.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MessageQueue {
    messages: VecDeque<Message>,
    next_id: u64,
}

/// Creates a new empty message queue.
#[allow(dead_code)]
pub fn new_message_queue() -> MessageQueue {
    MessageQueue {
        messages: VecDeque::new(),
        next_id: 1,
    }
}

/// Sends a message to the queue, returns the assigned id.
#[allow(dead_code)]
pub fn send_message(queue: &mut MessageQueue, topic: &str, payload: &str) -> u64 {
    let id = queue.next_id;
    queue.next_id += 1;
    queue.messages.push_back(Message {
        id,
        topic: topic.to_string(),
        payload: payload.to_string(),
    });
    id
}

/// Receives (dequeues) the next message.
#[allow(dead_code)]
pub fn receive_message(queue: &mut MessageQueue) -> Option<Message> {
    queue.messages.pop_front()
}

/// Returns the number of messages in the queue.
#[allow(dead_code)]
pub fn message_count(queue: &MessageQueue) -> usize {
    queue.messages.len()
}

/// Returns true if the queue is empty.
#[allow(dead_code)]
pub fn queue_is_empty_msg(queue: &MessageQueue) -> bool {
    queue.messages.is_empty()
}

/// Peeks at the front message without removing it.
#[allow(dead_code)]
pub fn peek_message(queue: &MessageQueue) -> Option<&Message> {
    queue.messages.front()
}

/// Drains all messages into a Vec.
#[allow(dead_code)]
pub fn drain_messages(queue: &mut MessageQueue) -> Vec<Message> {
    queue.messages.drain(..).collect()
}

/// Clears the message queue.
#[allow(dead_code)]
pub fn clear_message_queue(queue: &mut MessageQueue) {
    queue.messages.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_queue() {
        let q = new_message_queue();
        assert!(queue_is_empty_msg(&q));
        assert_eq!(message_count(&q), 0);
    }

    #[test]
    fn test_send_receive() {
        let mut q = new_message_queue();
        let id = send_message(&mut q, "test", "hello");
        assert_eq!(id, 1);
        let msg = receive_message(&mut q).expect("should succeed");
        assert_eq!(msg.topic, "test");
        assert_eq!(msg.payload, "hello");
    }

    #[test]
    fn test_message_count() {
        let mut q = new_message_queue();
        send_message(&mut q, "a", "1");
        send_message(&mut q, "b", "2");
        assert_eq!(message_count(&q), 2);
    }

    #[test]
    fn test_peek() {
        let mut q = new_message_queue();
        send_message(&mut q, "t", "p");
        let peeked = peek_message(&q).expect("should succeed");
        assert_eq!(peeked.topic, "t");
        assert_eq!(message_count(&q), 1);
    }

    #[test]
    fn test_drain() {
        let mut q = new_message_queue();
        send_message(&mut q, "a", "1");
        send_message(&mut q, "b", "2");
        let msgs = drain_messages(&mut q);
        assert_eq!(msgs.len(), 2);
        assert!(queue_is_empty_msg(&q));
    }

    #[test]
    fn test_clear() {
        let mut q = new_message_queue();
        send_message(&mut q, "x", "y");
        clear_message_queue(&mut q);
        assert!(queue_is_empty_msg(&q));
    }

    #[test]
    fn test_fifo_order() {
        let mut q = new_message_queue();
        send_message(&mut q, "first", "1");
        send_message(&mut q, "second", "2");
        let m1 = receive_message(&mut q).expect("should succeed");
        assert_eq!(m1.topic, "first");
    }

    #[test]
    fn test_receive_empty() {
        let mut q = new_message_queue();
        assert!(receive_message(&mut q).is_none());
    }

    #[test]
    fn test_ids_increment() {
        let mut q = new_message_queue();
        let id1 = send_message(&mut q, "a", "1");
        let id2 = send_message(&mut q, "b", "2");
        assert_eq!(id2, id1 + 1);
    }

    #[test]
    fn test_peek_empty() {
        let q = new_message_queue();
        assert!(peek_message(&q).is_none());
    }
}
