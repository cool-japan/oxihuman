//! Thread worker pool abstraction (stub — no real threads, uses fn pointers and task queue).

/// Configuration for the worker pool.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Number of logical workers to simulate.
    pub worker_count: usize,
    /// Maximum number of tasks that can be pending at once (0 = unlimited).
    pub max_pending: usize,
    /// Maximum tasks processed per `process_pending_tasks` call (0 = all).
    pub batch_size: usize,
}

/// A task stored in the pool.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WorkerTask {
    /// Application-defined task identifier.
    pub task_id: u64,
    /// Priority (higher values are processed first).
    pub priority: u8,
    /// Whether this task has been cancelled.
    pub cancelled: bool,
}

/// Snapshot statistics for the pool.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WorkerPoolStats {
    /// Tasks waiting to be processed.
    pub pending: usize,
    /// Tasks that have been successfully processed.
    pub completed: usize,
    /// Tasks that were cancelled.
    pub cancelled: usize,
    /// Number of logical workers.
    pub workers: usize,
}

/// Stub worker pool (single-threaded simulation).
#[allow(dead_code)]
pub struct WorkerPool {
    /// Active configuration.
    pub config: WorkerConfig,
    /// Tasks waiting to be processed, sorted high-priority first.
    pending: Vec<WorkerTask>,
    /// Count of completed tasks.
    completed: usize,
    /// Count of cancelled tasks.
    cancelled: usize,
}

// ── public API ───────────────────────────────────────────────────────────────

/// Return a sensible default `WorkerConfig`.
#[allow(dead_code)]
pub fn default_worker_config() -> WorkerConfig {
    WorkerConfig {
        worker_count: 4,
        max_pending: 0,
        batch_size: 0,
    }
}

/// Construct a new, empty `WorkerPool`.
#[allow(dead_code)]
pub fn new_worker_pool(cfg: &WorkerConfig) -> WorkerPool {
    WorkerPool {
        config: cfg.clone(),
        pending: Vec::new(),
        completed: 0,
        cancelled: 0,
    }
}

/// Submit a task to the pool.
/// Tasks are kept sorted by priority (descending); equal-priority tasks are FIFO.
#[allow(dead_code)]
pub fn submit_task(pool: &mut WorkerPool, task_id: u64, priority: u8) {
    // Enforce max_pending if configured.
    if pool.config.max_pending > 0 && pool.pending.len() >= pool.config.max_pending {
        return;
    }
    let task = WorkerTask { task_id, priority, cancelled: false };
    // Insert in priority-descending order (stable: new task goes after equal-priority ones).
    let pos = pool.pending.partition_point(|t| t.priority > priority);
    pool.pending.insert(pos, task);
}

/// Process pending tasks, simulating completion.
/// Returns the number of tasks processed this call.
#[allow(dead_code)]
pub fn process_pending_tasks(pool: &mut WorkerPool) -> usize {
    let limit = if pool.config.batch_size == 0 {
        pool.pending.len()
    } else {
        pool.config.batch_size.min(pool.pending.len())
    };

    let mut processed = 0;
    // Take `limit` tasks from the front (highest priority).
    for task in pool.pending.drain(..limit) {
        if task.cancelled {
            pool.cancelled += 1;
        } else {
            pool.completed += 1;
        }
        processed += 1;
    }
    processed
}

/// Return a statistics snapshot.
#[allow(dead_code)]
pub fn worker_pool_stats(pool: &WorkerPool) -> WorkerPoolStats {
    WorkerPoolStats {
        pending: pool.pending.len(),
        completed: pool.completed,
        cancelled: pool.cancelled,
        workers: pool.config.worker_count,
    }
}

/// Return the number of tasks currently pending.
#[allow(dead_code)]
pub fn pool_pending_count(pool: &WorkerPool) -> usize {
    pool.pending.len()
}

/// Return the total number of tasks completed so far.
#[allow(dead_code)]
pub fn pool_completed_count(pool: &WorkerPool) -> usize {
    pool.completed
}

/// Cancel a pending task by `task_id`.  Returns `true` if the task was found.
#[allow(dead_code)]
pub fn cancel_task(pool: &mut WorkerPool, task_id: u64) -> bool {
    for task in &mut pool.pending {
        if task.task_id == task_id && !task.cancelled {
            task.cancelled = true;
            return true;
        }
    }
    false
}

/// Reset the pool: clear pending tasks and zero the statistics counters.
#[allow(dead_code)]
pub fn reset_pool(pool: &mut WorkerPool) {
    pool.pending.clear();
    pool.completed = 0;
    pool.cancelled = 0;
}

/// Return the number of logical workers.
#[allow(dead_code)]
pub fn pool_worker_count(pool: &WorkerPool) -> usize {
    pool.config.worker_count
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pool_empty() {
        let cfg = default_worker_config();
        let pool = new_worker_pool(&cfg);
        assert_eq!(pool_pending_count(&pool), 0);
        assert_eq!(pool_completed_count(&pool), 0);
    }

    #[test]
    fn test_submit_and_process() {
        let cfg = default_worker_config();
        let mut pool = new_worker_pool(&cfg);
        submit_task(&mut pool, 1, 5);
        submit_task(&mut pool, 2, 10);
        assert_eq!(pool_pending_count(&pool), 2);
        let done = process_pending_tasks(&mut pool);
        assert_eq!(done, 2);
        assert_eq!(pool_completed_count(&pool), 2);
        assert_eq!(pool_pending_count(&pool), 0);
    }

    #[test]
    fn test_priority_ordering() {
        let cfg = default_worker_config();
        let mut pool = new_worker_pool(&cfg);
        submit_task(&mut pool, 1, 1);
        submit_task(&mut pool, 2, 255);
        submit_task(&mut pool, 3, 128);
        // Highest priority should be at front.
        assert_eq!(pool.pending[0].priority, 255);
    }

    #[test]
    fn test_cancel_task() {
        let cfg = default_worker_config();
        let mut pool = new_worker_pool(&cfg);
        submit_task(&mut pool, 42, 5);
        let cancelled = cancel_task(&mut pool, 42);
        assert!(cancelled);
        // Cancelled task is still pending until processed.
        process_pending_tasks(&mut pool);
        let stats = worker_pool_stats(&pool);
        assert_eq!(stats.cancelled, 1);
        assert_eq!(stats.completed, 0);
    }

    #[test]
    fn test_cancel_nonexistent_task() {
        let cfg = default_worker_config();
        let mut pool = new_worker_pool(&cfg);
        assert!(!cancel_task(&mut pool, 999));
    }

    #[test]
    fn test_reset_pool() {
        let cfg = default_worker_config();
        let mut pool = new_worker_pool(&cfg);
        submit_task(&mut pool, 1, 1);
        process_pending_tasks(&mut pool);
        reset_pool(&mut pool);
        assert_eq!(pool_pending_count(&pool), 0);
        assert_eq!(pool_completed_count(&pool), 0);
    }

    #[test]
    fn test_batch_size_limits_processing() {
        let cfg = WorkerConfig { worker_count: 2, max_pending: 0, batch_size: 2 };
        let mut pool = new_worker_pool(&cfg);
        for i in 0..5 {
            submit_task(&mut pool, i, 1);
        }
        let done = process_pending_tasks(&mut pool);
        assert_eq!(done, 2);
        assert_eq!(pool_pending_count(&pool), 3);
    }

    #[test]
    fn test_pool_worker_count() {
        let cfg = WorkerConfig { worker_count: 8, max_pending: 0, batch_size: 0 };
        let pool = new_worker_pool(&cfg);
        assert_eq!(pool_worker_count(&pool), 8);
    }
}
