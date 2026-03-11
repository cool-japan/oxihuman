#![allow(dead_code)]

/// A fixed-capacity stack-like vector (no heap allocation beyond initial).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StackVec<T> {
    data: Vec<T>,
    capacity: usize,
}

/// Creates a new stack vec with given capacity.
#[allow(dead_code)]
pub fn new_stack_vec<T>(capacity: usize) -> StackVec<T> {
    StackVec {
        data: Vec::with_capacity(capacity),
        capacity,
    }
}

/// Pushes a value onto the stack. Returns false if full.
#[allow(dead_code)]
pub fn stack_push<T>(sv: &mut StackVec<T>, value: T) -> bool {
    if sv.data.len() >= sv.capacity {
        return false;
    }
    sv.data.push(value);
    true
}

/// Pops a value from the top of the stack.
#[allow(dead_code)]
pub fn stack_pop<T>(sv: &mut StackVec<T>) -> Option<T> {
    sv.data.pop()
}

/// Peeks at the top of the stack.
#[allow(dead_code)]
pub fn stack_peek<T>(sv: &StackVec<T>) -> Option<&T> {
    sv.data.last()
}

/// Returns the current number of elements.
#[allow(dead_code)]
pub fn stack_len<T>(sv: &StackVec<T>) -> usize {
    sv.data.len()
}

/// Returns true if the stack is empty.
#[allow(dead_code)]
pub fn stack_is_empty<T>(sv: &StackVec<T>) -> bool {
    sv.data.is_empty()
}

/// Returns true if the stack is at capacity.
#[allow(dead_code)]
pub fn stack_is_full<T>(sv: &StackVec<T>) -> bool {
    sv.data.len() >= sv.capacity
}

/// Clears the stack.
#[allow(dead_code)]
pub fn stack_clear<T>(sv: &mut StackVec<T>) {
    sv.data.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stack_vec() {
        let sv: StackVec<i32> = new_stack_vec(10);
        assert!(stack_is_empty(&sv));
        assert_eq!(stack_len(&sv), 0);
    }

    #[test]
    fn test_push_pop() {
        let mut sv = new_stack_vec(5);
        assert!(stack_push(&mut sv, 42));
        assert_eq!(stack_pop(&mut sv), Some(42));
    }

    #[test]
    fn test_push_full() {
        let mut sv = new_stack_vec(2);
        assert!(stack_push(&mut sv, 1));
        assert!(stack_push(&mut sv, 2));
        assert!(!stack_push(&mut sv, 3));
    }

    #[test]
    fn test_peek() {
        let mut sv = new_stack_vec(5);
        stack_push(&mut sv, 10);
        assert_eq!(stack_peek(&sv), Some(&10));
        assert_eq!(stack_len(&sv), 1);
    }

    #[test]
    fn test_is_full() {
        let mut sv = new_stack_vec(1);
        assert!(!stack_is_full(&sv));
        stack_push(&mut sv, 1);
        assert!(stack_is_full(&sv));
    }

    #[test]
    fn test_clear() {
        let mut sv = new_stack_vec(5);
        stack_push(&mut sv, 1);
        stack_push(&mut sv, 2);
        stack_clear(&mut sv);
        assert!(stack_is_empty(&sv));
    }

    #[test]
    fn test_lifo_order() {
        let mut sv = new_stack_vec(5);
        stack_push(&mut sv, 1);
        stack_push(&mut sv, 2);
        stack_push(&mut sv, 3);
        assert_eq!(stack_pop(&mut sv), Some(3));
        assert_eq!(stack_pop(&mut sv), Some(2));
    }

    #[test]
    fn test_pop_empty() {
        let mut sv: StackVec<i32> = new_stack_vec(5);
        assert_eq!(stack_pop(&mut sv), None);
    }

    #[test]
    fn test_peek_empty() {
        let sv: StackVec<i32> = new_stack_vec(5);
        assert_eq!(stack_peek(&sv), None);
    }

    #[test]
    fn test_len() {
        let mut sv = new_stack_vec(10);
        stack_push(&mut sv, 1);
        stack_push(&mut sv, 2);
        assert_eq!(stack_len(&sv), 2);
    }
}
