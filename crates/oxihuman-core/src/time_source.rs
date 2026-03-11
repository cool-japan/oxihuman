#![allow(dead_code)]

/// A timestamp in milliseconds.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Timestamp {
    pub ms: u64,
}

/// A deterministic time source (no real clock dependency).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TimeSource {
    current_ms: u64,
    start_ms: u64,
}

/// Creates a new time source starting at the given ms.
#[allow(dead_code)]
pub fn new_time_source(start_ms: u64) -> TimeSource {
    TimeSource {
        current_ms: start_ms,
        start_ms,
    }
}

/// Returns the current time in ms.
#[allow(dead_code)]
pub fn current_time_ms(ts: &TimeSource) -> u64 {
    ts.current_ms
}

/// Returns elapsed ms since a given timestamp.
#[allow(dead_code)]
pub fn elapsed_since(ts: &TimeSource, since: &Timestamp) -> u64 {
    ts.current_ms.saturating_sub(since.ms)
}

/// Returns the difference in ms between two timestamps.
#[allow(dead_code)]
pub fn time_diff_ms(a: &Timestamp, b: &Timestamp) -> i64 {
    (a.ms as i64) - (b.ms as i64)
}

/// Converts a timestamp to a simple string representation.
#[allow(dead_code)]
pub fn timestamp_to_string(t: &Timestamp) -> String {
    let secs = t.ms / 1000;
    let millis = t.ms % 1000;
    format!("{secs}.{millis:03}s")
}

/// Returns true if timestamp a is after timestamp b.
#[allow(dead_code)]
pub fn timestamp_is_after(a: &Timestamp, b: &Timestamp) -> bool {
    a.ms > b.ms
}

/// Adds milliseconds to a timestamp.
#[allow(dead_code)]
pub fn timestamp_add_ms(t: &Timestamp, ms: u64) -> Timestamp {
    Timestamp { ms: t.ms + ms }
}

/// Resets the time source to its start value.
#[allow(dead_code)]
pub fn time_source_reset(ts: &mut TimeSource) {
    ts.current_ms = ts.start_ms;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_time_source() {
        let ts = new_time_source(1000);
        assert_eq!(current_time_ms(&ts), 1000);
    }

    #[test]
    fn test_elapsed_since() {
        let ts = new_time_source(5000);
        let stamp = Timestamp { ms: 3000 };
        assert_eq!(elapsed_since(&ts, &stamp), 2000);
    }

    #[test]
    fn test_time_diff_ms() {
        let a = Timestamp { ms: 5000 };
        let b = Timestamp { ms: 3000 };
        assert_eq!(time_diff_ms(&a, &b), 2000);
        assert_eq!(time_diff_ms(&b, &a), -2000);
    }

    #[test]
    fn test_timestamp_to_string() {
        let t = Timestamp { ms: 1500 };
        assert_eq!(timestamp_to_string(&t), "1.500s");
    }

    #[test]
    fn test_timestamp_is_after() {
        let a = Timestamp { ms: 200 };
        let b = Timestamp { ms: 100 };
        assert!(timestamp_is_after(&a, &b));
        assert!(!timestamp_is_after(&b, &a));
    }

    #[test]
    fn test_timestamp_add_ms() {
        let t = Timestamp { ms: 1000 };
        let t2 = timestamp_add_ms(&t, 500);
        assert_eq!(t2.ms, 1500);
    }

    #[test]
    fn test_time_source_reset() {
        let mut ts = new_time_source(100);
        ts.current_ms = 999;
        time_source_reset(&mut ts);
        assert_eq!(current_time_ms(&ts), 100);
    }

    #[test]
    fn test_elapsed_since_future() {
        let ts = new_time_source(100);
        let stamp = Timestamp { ms: 500 };
        assert_eq!(elapsed_since(&ts, &stamp), 0);
    }

    #[test]
    fn test_timestamp_equality() {
        let a = Timestamp { ms: 42 };
        let b = Timestamp { ms: 42 };
        assert_eq!(a, b);
    }

    #[test]
    fn test_timestamp_ordering() {
        let a = Timestamp { ms: 10 };
        let b = Timestamp { ms: 20 };
        assert!(a < b);
    }
}
