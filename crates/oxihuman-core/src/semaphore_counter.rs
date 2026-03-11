//! Simple counting semaphore for controlling concurrent access to a limited resource.
//!
//! A deterministic, single-threaded counting semaphore useful for tracking
//! how many concurrent slots are in use without OS primitives.

#![allow(dead_code)]

/// Configuration for a `SemaphoreCounter`.
#[derive(Debug, Clone)]
pub struct SemaphoreConfig {
    /// Maximum number of simultaneous holders.
    pub max_count: usize,
}

/// A counting semaphore.
#[derive(Debug, Clone)]
pub struct SemaphoreCounter {
    config: SemaphoreConfig,
    current: usize,
}

/// Build a default `SemaphoreConfig` (max_count = 1, i.e. mutex-like).
#[allow(dead_code)]
pub fn default_semaphore_config() -> SemaphoreConfig {
    SemaphoreConfig { max_count: 1 }
}

/// Create a new `SemaphoreCounter` with all slots available.
#[allow(dead_code)]
pub fn new_semaphore_counter(config: SemaphoreConfig) -> SemaphoreCounter {
    SemaphoreCounter { config, current: 0 }
}

/// Acquire one slot. Returns `true` on success, `false` if already full.
#[allow(dead_code)]
pub fn sem_acquire(sem: &mut SemaphoreCounter) -> bool {
    if sem.current < sem.config.max_count {
        sem.current += 1;
        true
    } else {
        false
    }
}

/// Release one slot. Returns `true` on success, `false` if already at zero.
#[allow(dead_code)]
pub fn sem_release(sem: &mut SemaphoreCounter) -> bool {
    if sem.current > 0 {
        sem.current -= 1;
        true
    } else {
        false
    }
}

/// Attempt to acquire without blocking — same as `sem_acquire` in this model.
#[allow(dead_code)]
pub fn sem_try_acquire(sem: &mut SemaphoreCounter) -> bool {
    sem_acquire(sem)
}

/// Return the number of available (free) slots.
#[allow(dead_code)]
pub fn sem_available(sem: &SemaphoreCounter) -> usize {
    sem.config.max_count.saturating_sub(sem.current)
}

/// Return the configured maximum count.
#[allow(dead_code)]
pub fn sem_max_count(sem: &SemaphoreCounter) -> usize {
    sem.config.max_count
}

/// Return `true` if no slots are available.
#[allow(dead_code)]
pub fn sem_is_full(sem: &SemaphoreCounter) -> bool {
    sem.current >= sem.config.max_count
}

/// Serialize the semaphore state to a JSON string.
#[allow(dead_code)]
pub fn sem_to_json(sem: &SemaphoreCounter) -> String {
    format!(
        "{{\"max_count\":{},\"current\":{},\"available\":{}}}",
        sem.config.max_count,
        sem.current,
        sem_available(sem)
    )
}

/// Reset the semaphore to the fully-available state.
#[allow(dead_code)]
pub fn sem_reset(sem: &mut SemaphoreCounter) {
    sem.current = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sem(max: usize) -> SemaphoreCounter {
        new_semaphore_counter(SemaphoreConfig { max_count: max })
    }

    #[test]
    fn test_initial_available() {
        let sem = make_sem(3);
        assert_eq!(sem_available(&sem), 3);
    }

    #[test]
    fn test_acquire_reduces_available() {
        let mut sem = make_sem(3);
        assert!(sem_acquire(&mut sem));
        assert_eq!(sem_available(&sem), 2);
    }

    #[test]
    fn test_acquire_fails_when_full() {
        let mut sem = make_sem(1);
        assert!(sem_acquire(&mut sem));
        assert!(!sem_acquire(&mut sem));
    }

    #[test]
    fn test_release_increases_available() {
        let mut sem = make_sem(2);
        sem_acquire(&mut sem);
        sem_release(&mut sem);
        assert_eq!(sem_available(&sem), 2);
    }

    #[test]
    fn test_release_below_zero_fails() {
        let mut sem = make_sem(1);
        assert!(!sem_release(&mut sem));
    }

    #[test]
    fn test_is_full() {
        let mut sem = make_sem(1);
        assert!(!sem_is_full(&sem));
        sem_acquire(&mut sem);
        assert!(sem_is_full(&sem));
    }

    #[test]
    fn test_reset() {
        let mut sem = make_sem(2);
        sem_acquire(&mut sem);
        sem_acquire(&mut sem);
        sem_reset(&mut sem);
        assert_eq!(sem_available(&sem), 2);
    }

    #[test]
    fn test_max_count() {
        let sem = make_sem(5);
        assert_eq!(sem_max_count(&sem), 5);
    }

    #[test]
    fn test_to_json_contains_max_count() {
        let sem = make_sem(4);
        let json = sem_to_json(&sem);
        assert!(json.contains("\"max_count\":4"));
    }

    #[test]
    fn test_try_acquire_same_as_acquire() {
        let mut sem = make_sem(1);
        assert!(sem_try_acquire(&mut sem));
        assert!(!sem_try_acquire(&mut sem));
    }
}
