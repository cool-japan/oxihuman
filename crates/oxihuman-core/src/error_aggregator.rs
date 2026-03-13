// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct ErrorAggregator {
    pub errors: Vec<String>,
}

impl ErrorAggregator {
    pub fn new() -> Self {
        ErrorAggregator { errors: Vec::new() }
    }

    pub fn push(&mut self, msg: &str) {
        self.errors.push(msg.to_string());
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn count(&self) -> usize {
        self.errors.len()
    }

    pub fn clear(&mut self) {
        self.errors.clear();
    }
}

impl Default for ErrorAggregator {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_error_aggregator() -> ErrorAggregator {
    ErrorAggregator::new()
}

pub fn aggregator_push(a: &mut ErrorAggregator, msg: &str) {
    a.push(msg);
}

pub fn aggregator_has_errors(a: &ErrorAggregator) -> bool {
    a.has_errors()
}

pub fn aggregator_count(a: &ErrorAggregator) -> usize {
    a.count()
}

pub fn aggregator_clear(a: &mut ErrorAggregator) {
    a.clear();
}

pub fn aggregator_messages(a: &ErrorAggregator) -> Vec<String> {
    a.errors.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        /* new aggregator has no errors */
        let a = new_error_aggregator();
        assert!(!aggregator_has_errors(&a));
        assert_eq!(aggregator_count(&a), 0);
    }

    #[test]
    fn test_push_single() {
        /* pushing one error */
        let mut a = new_error_aggregator();
        aggregator_push(&mut a, "oops");
        assert!(aggregator_has_errors(&a));
        assert_eq!(aggregator_count(&a), 1);
    }

    #[test]
    fn test_push_multiple() {
        /* pushing multiple errors */
        let mut a = new_error_aggregator();
        aggregator_push(&mut a, "err1");
        aggregator_push(&mut a, "err2");
        aggregator_push(&mut a, "err3");
        assert_eq!(aggregator_count(&a), 3);
    }

    #[test]
    fn test_clear() {
        /* clear removes all errors */
        let mut a = new_error_aggregator();
        aggregator_push(&mut a, "e");
        aggregator_clear(&mut a);
        assert!(!aggregator_has_errors(&a));
        assert_eq!(aggregator_count(&a), 0);
    }

    #[test]
    fn test_messages_content() {
        /* messages returns correct strings */
        let mut a = new_error_aggregator();
        aggregator_push(&mut a, "alpha");
        aggregator_push(&mut a, "beta");
        let msgs = aggregator_messages(&a);
        assert_eq!(msgs[0], "alpha");
        assert_eq!(msgs[1], "beta");
    }

    #[test]
    fn test_messages_empty() {
        /* messages on empty aggregator returns empty vec */
        let a = new_error_aggregator();
        assert!(aggregator_messages(&a).is_empty());
    }

    #[test]
    fn test_method_push() {
        /* method-level push works */
        let mut a = ErrorAggregator::new();
        a.push("x");
        assert_eq!(a.count(), 1);
    }

    #[test]
    fn test_default() {
        /* Default impl works */
        let a = ErrorAggregator::default();
        assert_eq!(a.count(), 0);
    }
}
