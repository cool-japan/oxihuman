#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChainError {
    message: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ResultChain<T> {
    Ok(T),
    Err(ChainError),
}

#[allow(dead_code)]
pub fn new_result_chain<T>(val: T) -> ResultChain<T> {
    ResultChain::Ok(val)
}

#[allow(dead_code)]
pub fn chain_ok<T>(val: T) -> ResultChain<T> {
    ResultChain::Ok(val)
}

#[allow(dead_code)]
pub fn chain_err<T>(msg: &str) -> ResultChain<T> {
    ResultChain::Err(ChainError {
        message: msg.to_string(),
    })
}

#[allow(dead_code)]
pub fn chain_map<T, U, F: FnOnce(T) -> U>(rc: ResultChain<T>, f: F) -> ResultChain<U> {
    match rc {
        ResultChain::Ok(v) => ResultChain::Ok(f(v)),
        ResultChain::Err(e) => ResultChain::Err(e),
    }
}

#[allow(dead_code)]
pub fn chain_and_then<T, U, F: FnOnce(T) -> ResultChain<U>>(
    rc: ResultChain<T>,
    f: F,
) -> ResultChain<U> {
    match rc {
        ResultChain::Ok(v) => f(v),
        ResultChain::Err(e) => ResultChain::Err(e),
    }
}

#[allow(dead_code)]
pub fn chain_unwrap_or<T>(rc: ResultChain<T>, default: T) -> T {
    match rc {
        ResultChain::Ok(v) => v,
        ResultChain::Err(_) => default,
    }
}

#[allow(dead_code)]
pub fn chain_is_ok<T>(rc: &ResultChain<T>) -> bool {
    matches!(rc, ResultChain::Ok(_))
}

#[allow(dead_code)]
pub fn chain_error_message<T>(rc: &ResultChain<T>) -> Option<&str> {
    match rc {
        ResultChain::Err(e) => Some(&e.message),
        ResultChain::Ok(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ok() {
        let r = chain_ok::<i32>(42);
        assert!(chain_is_ok(&r));
    }

    #[test]
    fn test_err() {
        let r: ResultChain<i32> = chain_err("fail");
        assert!(!chain_is_ok(&r));
    }

    #[test]
    fn test_error_message() {
        let r: ResultChain<i32> = chain_err("oops");
        assert_eq!(chain_error_message(&r), Some("oops"));
    }

    #[test]
    fn test_ok_no_error() {
        let r = chain_ok(1);
        assert_eq!(chain_error_message(&r), None);
    }

    #[test]
    fn test_map() {
        let r = chain_ok(10);
        let r2 = chain_map(r, |x| x * 2);
        assert_eq!(chain_unwrap_or(r2, 0), 20);
    }

    #[test]
    fn test_map_err() {
        let r: ResultChain<i32> = chain_err("bad");
        let r2 = chain_map(r, |x| x * 2);
        assert!(!chain_is_ok(&r2));
    }

    #[test]
    fn test_and_then() {
        let r = chain_ok(5);
        let r2 = chain_and_then(r, |x| chain_ok(x + 1));
        assert_eq!(chain_unwrap_or(r2, 0), 6);
    }

    #[test]
    fn test_and_then_err() {
        let r = chain_ok(5);
        let r2 = chain_and_then(r, |_| chain_err::<i32>("nope"));
        assert!(!chain_is_ok(&r2));
    }

    #[test]
    fn test_unwrap_or_default() {
        let r: ResultChain<i32> = chain_err("err");
        assert_eq!(chain_unwrap_or(r, 99), 99);
    }

    #[test]
    fn test_new_result_chain() {
        let r = new_result_chain(7);
        assert!(chain_is_ok(&r));
        assert_eq!(chain_unwrap_or(r, 0), 7);
    }
}
