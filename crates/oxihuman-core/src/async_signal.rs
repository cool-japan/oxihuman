// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Async signal primitive for cooperative, thread-safe signaling.
//!
//! [`AsyncSignal`] wraps an `Arc<(Mutex<SignalInner>, Condvar)>` so that multiple
//! threads can share the same signal instance by cloning it, while still being able
//! to block-wait on `signal_wait` / `signal_wait_timeout`.

use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// Internal state protected by the mutex.
#[derive(Debug)]
struct SignalInner {
    is_set: bool,
    count: usize,
}

impl SignalInner {
    fn new() -> Self {
        Self {
            is_set: false,
            count: 0,
        }
    }
}

/// Thread-safe signal primitive.
///
/// Cloning an [`AsyncSignal`] produces a handle that shares the same underlying
/// state — both the original and every clone observe the same `is_set` value and
/// wait on the same condition variable.
#[derive(Debug, Clone)]
pub struct AsyncSignal {
    name: String,
    inner: Arc<(Mutex<SignalInner>, Condvar)>,
}

// ──────────────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Acquire the mutex, recovering from poisoning by taking the inner value.
#[inline]
fn lock_inner(pair: &Arc<(Mutex<SignalInner>, Condvar)>) -> std::sync::MutexGuard<'_, SignalInner> {
    pair.0
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

// ──────────────────────────────────────────────────────────────────────────────
// Public API
// ──────────────────────────────────────────────────────────────────────────────

/// Construct a new, unset [`AsyncSignal`] with the given name.
pub fn new_async_signal(name: &str) -> AsyncSignal {
    AsyncSignal {
        name: name.to_string(),
        inner: Arc::new((Mutex::new(SignalInner::new()), Condvar::new())),
    }
}

/// Set the signal, increment the count, and wake every thread blocked in
/// `signal_wait` or `signal_wait_timeout`.
pub fn signal_set(sig: &AsyncSignal) {
    let (mutex, condvar) = sig.inner.as_ref();
    let mut guard = mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    guard.is_set = true;
    guard.count += 1;
    drop(guard);
    condvar.notify_all();
}

/// Clear the signal.  Threads that are already blocked in `signal_wait` will
/// continue to block until the signal is set again.
pub fn signal_reset(sig: &AsyncSignal) {
    let mut guard = lock_inner(&sig.inner);
    guard.is_set = false;
}

/// Block the calling thread until `sig` is set.
///
/// Returns `true` once the signal has been observed as set.  If the signal is
/// already set when this function is called it returns immediately.
pub fn signal_wait(sig: &AsyncSignal) -> bool {
    let (mutex, condvar) = sig.inner.as_ref();
    let mut guard = mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    while !guard.is_set {
        guard = condvar
            .wait(guard)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
    }
    true
}

/// Block the calling thread for at most `timeout_ms` milliseconds waiting for
/// `sig` to be set.
///
/// Returns `true` if the signal was observed as set before the timeout expired,
/// or `false` if the timeout elapsed first.
pub fn signal_wait_timeout(sig: &AsyncSignal, timeout_ms: u64) -> bool {
    let (mutex, condvar) = sig.inner.as_ref();
    let timeout = Duration::from_millis(timeout_ms);
    let mut guard = mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    // Short-circuit if already set.
    if guard.is_set {
        return true;
    }
    let result = condvar
        .wait_timeout_while(guard, timeout, |inner| !inner.is_set)
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    // result.0 is the MutexGuard, result.1.timed_out() is true when we hit the deadline.
    guard = result.0;
    guard.is_set
}

/// Non-blocking poll — returns the current `is_set` value without blocking.
pub fn signal_is_set(sig: &AsyncSignal) -> bool {
    lock_inner(&sig.inner).is_set
}

/// Compatibility shim (non-blocking poll).  Prefer [`signal_is_set`].
#[deprecated(
    since = "0.1.3",
    note = "use signal_is_set for non-blocking poll or signal_wait to block"
)]
pub fn signal_wait_stub(sig: &AsyncSignal) -> bool {
    signal_is_set(sig)
}

/// Return a clone of the signal's name.
pub fn signal_name_as(sig: &AsyncSignal) -> String {
    sig.name.clone()
}

/// Return the number of times the signal has been set.
pub fn signal_count_as(sig: &AsyncSignal) -> usize {
    lock_inner(&sig.inner).count
}

/// Serialize the signal state to a minimal JSON string.
pub fn signal_to_json(sig: &AsyncSignal) -> String {
    let guard = lock_inner(&sig.inner);
    format!(
        r#"{{"name":"{}","is_set":{},"count":{}}}"#,
        sig.name, guard.is_set, guard.count
    )
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;
    use std::time::Instant;

    // ── existing tests updated for new API ────────────────────────────────────

    #[test]
    fn test_new_async_signal() {
        let s = new_async_signal("test");
        assert_eq!(signal_name_as(&s), "test");
        assert!(!signal_is_set(&s));
    }

    #[test]
    fn test_signal_set_and_check() {
        let s = new_async_signal("sig");
        signal_set(&s);
        assert!(signal_is_set(&s));
    }

    #[test]
    fn test_signal_reset() {
        let s = new_async_signal("sig");
        signal_set(&s);
        signal_reset(&s);
        assert!(!signal_is_set(&s));
    }

    #[test]
    #[allow(deprecated)]
    fn test_signal_wait_stub_deprecated() {
        let s = new_async_signal("sig");
        assert!(!signal_wait_stub(&s));
        signal_set(&s);
        assert!(signal_wait_stub(&s));
    }

    #[test]
    fn test_signal_count() {
        let s = new_async_signal("sig");
        signal_set(&s);
        signal_set(&s);
        assert_eq!(signal_count_as(&s), 2);
    }

    #[test]
    fn test_signal_name() {
        let s = new_async_signal("my_signal");
        assert_eq!(signal_name_as(&s), "my_signal");
    }

    #[test]
    fn test_signal_to_json() {
        let s = new_async_signal("j");
        let json = signal_to_json(&s);
        assert!(json.contains("\"name\":\"j\""));
        assert!(json.contains("\"is_set\":false"));
    }

    #[test]
    fn test_signal_to_json_after_set() {
        let s = new_async_signal("j");
        signal_set(&s);
        let json = signal_to_json(&s);
        assert!(json.contains("\"is_set\":true"));
        assert!(json.contains("\"count\":1"));
    }

    #[test]
    fn test_multiple_reset_cycles() {
        let s = new_async_signal("cycle");
        signal_set(&s);
        signal_reset(&s);
        signal_set(&s);
        assert!(signal_is_set(&s));
        assert_eq!(signal_count_as(&s), 2);
    }

    #[test]
    fn test_initial_count_zero() {
        let s = new_async_signal("z");
        assert_eq!(signal_count_as(&s), 0);
    }

    // ── new blocking-wait tests ───────────────────────────────────────────────

    /// Spawns a thread that calls `signal_set` after a short delay.
    /// `signal_wait` must block until that set happens and then return `true`.
    #[test]
    fn test_signal_wait_blocks_until_set() {
        let sig = new_async_signal("blocking");
        let sig_thread = sig.clone();

        // Flipped to `true` by the setter thread immediately before it calls
        // `signal_set`. Because `signal_set` and `signal_wait` synchronise
        // through the same mutex, observing the signal as set establishes a
        // happens-before edge: once `signal_wait` returns we are guaranteed to
        // read this as `true`. That makes the "did not return early" check
        // deterministic, instead of relying on a racy elapsed-time lower bound
        // (which flaked under heavy parallel load when the main thread was
        // descheduled before it could record the start instant).
        let setter_fired = Arc::new(AtomicBool::new(false));
        let setter_fired_thread = Arc::clone(&setter_fired);

        let handle = thread::spawn(move || {
            // Brief delay so the waiter is very likely parked in `signal_wait`
            // first — exercises the blocking path, but is not required for the
            // correctness of the assertions below.
            thread::sleep(Duration::from_millis(10));
            setter_fired_thread.store(true, Ordering::Release);
            signal_set(&sig_thread);
        });

        let t0 = Instant::now();
        let result = signal_wait(&sig);
        let elapsed = t0.elapsed();

        handle.join().expect("thread panicked");

        assert!(result, "signal_wait must return true");
        // Deterministic ordering check: `signal_wait` cannot have returned
        // before the setter ran `signal_set`, so the store above is visible.
        assert!(
            setter_fired.load(Ordering::Acquire),
            "signal_wait returned before the setter called signal_set"
        );
        // Liveness guard: must wake up, not deadlock. Generous bound so it
        // never fires from mere scheduler jitter under heavy parallel load.
        assert!(
            elapsed < Duration::from_secs(5),
            "signal_wait took too long: {elapsed:?}"
        );
    }

    /// `signal_wait_timeout` on a permanently-unset signal must return `false`.
    #[test]
    fn test_signal_wait_timeout_fires() {
        let sig = new_async_signal("timeout_miss");
        let t0 = Instant::now();
        let result = signal_wait_timeout(&sig, 50);
        let elapsed = t0.elapsed();

        assert!(!result, "must return false on timeout");
        // Must have waited at least ~50 ms (give 20 ms of scheduler slack).
        assert!(
            elapsed >= Duration::from_millis(30),
            "returned too early: {elapsed:?}"
        );
        // Must not have spun for an excessive amount of time.
        assert!(
            elapsed < Duration::from_millis(2_000),
            "took too long: {elapsed:?}"
        );
    }

    /// `signal_wait_timeout` returns `true` when the signal is set before the
    /// deadline.
    #[test]
    fn test_signal_wait_timeout_succeeds() {
        let sig = new_async_signal("timeout_hit");
        let sig_thread = sig.clone();

        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(20));
            signal_set(&sig_thread);
        });

        // 5 s deadline — returns as soon as the 20 ms set fires; huge margin avoids load-induced flakiness.
        let result = signal_wait_timeout(&sig, 5_000);
        handle.join().expect("thread panicked");

        assert!(result, "must return true when signaled before timeout");
    }

    /// Setting on a clone must be visible from the original, and vice-versa.
    #[test]
    fn test_signal_clone_shared() {
        let original = new_async_signal("shared");
        let clone = original.clone();

        // Set via the clone — original must see it immediately.
        signal_set(&clone);
        assert!(signal_is_set(&original), "original must see set via clone");
        assert_eq!(signal_count_as(&original), 1);

        // Reset via the original — clone must see it.
        signal_reset(&original);
        assert!(!signal_is_set(&clone), "clone must see reset via original");

        // Set via original — clone must see it.
        signal_set(&original);
        assert!(signal_is_set(&clone), "clone must see set via original");
        assert_eq!(signal_count_as(&clone), 2);
    }

    /// Multiple waiters should all unblock when the signal is set.
    #[test]
    fn test_signal_wait_multiple_waiters() {
        let sig = new_async_signal("multi");
        let n_threads = 4_usize;

        let handles: Vec<_> = (0..n_threads)
            .map(|_| {
                let s = sig.clone();
                thread::spawn(move || signal_wait(&s))
            })
            .collect();

        // Let all threads reach their wait before we fire.
        thread::sleep(Duration::from_millis(20));
        signal_set(&sig);

        let results: Vec<bool> = handles
            .into_iter()
            .map(|h| h.join().expect("thread panicked"))
            .collect();

        assert!(
            results.iter().all(|&r| r),
            "all waiters must return true: {results:?}"
        );
    }
}
