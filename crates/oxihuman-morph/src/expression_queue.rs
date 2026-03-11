#![allow(dead_code)]
//! Queue of expressions to be applied sequentially.

use std::collections::VecDeque;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct QueuedExpression {
    pub name: String,
    pub weight: f32,
    pub duration: f32,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ExpressionQueue {
    queue: VecDeque<QueuedExpression>,
}

#[allow(dead_code)]
pub fn new_expression_queue() -> ExpressionQueue {
    ExpressionQueue {
        queue: VecDeque::new(),
    }
}

#[allow(dead_code)]
pub fn enqueue_expression(q: &mut ExpressionQueue, name: &str, weight: f32, duration: f32) {
    q.queue.push_back(QueuedExpression {
        name: name.to_string(),
        weight: weight.clamp(0.0, 1.0),
        duration: duration.max(0.0),
    });
}

#[allow(dead_code)]
pub fn dequeue_expression(q: &mut ExpressionQueue) -> Option<QueuedExpression> {
    q.queue.pop_front()
}

#[allow(dead_code)]
pub fn queue_len(q: &ExpressionQueue) -> usize {
    q.queue.len()
}

#[allow(dead_code)]
pub fn queue_is_empty(q: &ExpressionQueue) -> bool {
    q.queue.is_empty()
}

#[allow(dead_code)]
pub fn peek_expression(q: &ExpressionQueue) -> Option<&QueuedExpression> {
    q.queue.front()
}

#[allow(dead_code)]
pub fn clear_expression_queue(q: &mut ExpressionQueue) {
    q.queue.clear();
}

#[allow(dead_code)]
pub fn queue_to_json(q: &ExpressionQueue) -> String {
    let entries: Vec<String> = q
        .queue
        .iter()
        .map(|e| {
            format!(
                "{{\"name\":\"{}\",\"weight\":{},\"duration\":{}}}",
                e.name, e.weight, e.duration
            )
        })
        .collect();
    format!("[{}]", entries.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_expression_queue() {
        let q = new_expression_queue();
        assert!(queue_is_empty(&q));
    }

    #[test]
    fn test_enqueue_expression() {
        let mut q = new_expression_queue();
        enqueue_expression(&mut q, "smile", 0.5, 1.0);
        assert_eq!(queue_len(&q), 1);
    }

    #[test]
    fn test_dequeue_expression() {
        let mut q = new_expression_queue();
        enqueue_expression(&mut q, "smile", 0.5, 1.0);
        let e = dequeue_expression(&mut q).unwrap();
        assert_eq!(e.name, "smile");
        assert!(queue_is_empty(&q));
    }

    #[test]
    fn test_queue_len() {
        let mut q = new_expression_queue();
        enqueue_expression(&mut q, "a", 0.1, 0.5);
        enqueue_expression(&mut q, "b", 0.2, 0.5);
        assert_eq!(queue_len(&q), 2);
    }

    #[test]
    fn test_peek_expression() {
        let mut q = new_expression_queue();
        enqueue_expression(&mut q, "first", 0.3, 1.0);
        let p = peek_expression(&q).unwrap();
        assert_eq!(p.name, "first");
        assert_eq!(queue_len(&q), 1);
    }

    #[test]
    fn test_clear_expression_queue() {
        let mut q = new_expression_queue();
        enqueue_expression(&mut q, "a", 0.1, 0.5);
        clear_expression_queue(&mut q);
        assert!(queue_is_empty(&q));
    }

    #[test]
    fn test_queue_to_json() {
        let mut q = new_expression_queue();
        enqueue_expression(&mut q, "x", 0.5, 1.0);
        let json = queue_to_json(&q);
        assert!(json.contains("\"name\":\"x\""));
    }

    #[test]
    fn test_fifo_order() {
        let mut q = new_expression_queue();
        enqueue_expression(&mut q, "first", 0.1, 1.0);
        enqueue_expression(&mut q, "second", 0.2, 1.0);
        assert_eq!(dequeue_expression(&mut q).unwrap().name, "first");
        assert_eq!(dequeue_expression(&mut q).unwrap().name, "second");
    }

    #[test]
    fn test_dequeue_empty() {
        let mut q = new_expression_queue();
        assert!(dequeue_expression(&mut q).is_none());
    }

    #[test]
    fn test_weight_clamp() {
        let mut q = new_expression_queue();
        enqueue_expression(&mut q, "x", 5.0, 1.0);
        let e = dequeue_expression(&mut q).unwrap();
        assert!((e.weight - 1.0).abs() < 1e-6);
    }
}
