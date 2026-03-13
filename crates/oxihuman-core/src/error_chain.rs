// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Error cause chain tracking.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChainedError {
    pub message: String,
    pub code: u32,
    pub cause: Vec<ChainedError>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ErrorChain {
    pub errors: Vec<ChainedError>,
}

#[allow(dead_code)]
pub fn new_chained_error(msg: &str, code: u32) -> ChainedError {
    ChainedError { message: msg.to_string(), code, cause: Vec::new() }
}

#[allow(dead_code)]
pub fn chain_cause(mut err: ChainedError, cause: ChainedError) -> ChainedError {
    err.cause.push(cause);
    err
}

#[allow(dead_code)]
pub fn new_error_chain() -> ErrorChain {
    ErrorChain { errors: Vec::new() }
}

#[allow(dead_code)]
pub fn ec_push(chain: &mut ErrorChain, err: ChainedError) {
    chain.errors.push(err);
}

#[allow(dead_code)]
pub fn ec_pop(chain: &mut ErrorChain) -> Option<ChainedError> {
    chain.errors.pop()
}

#[allow(dead_code)]
pub fn ec_len(chain: &ErrorChain) -> usize {
    chain.errors.len()
}

#[allow(dead_code)]
pub fn ec_is_empty(chain: &ErrorChain) -> bool {
    chain.errors.is_empty()
}

#[allow(dead_code)]
pub fn ec_last(chain: &ErrorChain) -> Option<&ChainedError> {
    chain.errors.last()
}

#[allow(dead_code)]
pub fn ec_to_json(chain: &ErrorChain) -> String {
    let entries: Vec<String> = chain
        .errors
        .iter()
        .map(|e| format!("{{\"code\":{},\"message\":\"{}\"}}", e.code, e.message))
        .collect();
    format!("[{}]", entries.join(","))
}

#[allow(dead_code)]
pub fn ec_has_code(chain: &ErrorChain, code: u32) -> bool {
    chain.errors.iter().any(|e| e.code == code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_chained_error() {
        let err = new_chained_error("test error", 42);
        assert_eq!(err.code, 42);
        assert_eq!(err.message, "test error");
        assert!(err.cause.is_empty());
    }

    #[test]
    fn test_chain_cause() {
        let cause = new_chained_error("root cause", 1);
        let err = new_chained_error("top error", 2);
        let chained = chain_cause(err, cause);
        assert_eq!(chained.cause.len(), 1);
        assert_eq!(chained.cause[0].code, 1);
    }

    #[test]
    fn test_new_error_chain_empty() {
        let chain = new_error_chain();
        assert!(ec_is_empty(&chain));
        assert_eq!(ec_len(&chain), 0);
    }

    #[test]
    fn test_push_and_len() {
        let mut chain = new_error_chain();
        ec_push(&mut chain, new_chained_error("err", 5));
        assert_eq!(ec_len(&chain), 1);
    }

    #[test]
    fn test_pop() {
        let mut chain = new_error_chain();
        ec_push(&mut chain, new_chained_error("err", 99));
        let popped = ec_pop(&mut chain);
        assert!(popped.is_some());
        assert_eq!(popped.expect("should succeed").code, 99);
        assert!(ec_is_empty(&chain));
    }

    #[test]
    fn test_last() {
        let mut chain = new_error_chain();
        ec_push(&mut chain, new_chained_error("first", 1));
        ec_push(&mut chain, new_chained_error("last", 2));
        assert_eq!(ec_last(&chain).expect("should succeed").code, 2);
    }

    #[test]
    fn test_has_code() {
        let mut chain = new_error_chain();
        ec_push(&mut chain, new_chained_error("err", 404));
        assert!(ec_has_code(&chain, 404));
        assert!(!ec_has_code(&chain, 500));
    }

    #[test]
    fn test_to_json() {
        let mut chain = new_error_chain();
        ec_push(&mut chain, new_chained_error("oops", 1));
        let json = ec_to_json(&chain);
        assert!(json.contains("\"code\":1"));
        assert!(json.contains("\"message\":\"oops\""));
    }
}
