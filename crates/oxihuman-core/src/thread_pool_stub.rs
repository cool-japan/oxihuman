// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Stub thread pool for WASM-compatible async task scheduling.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum PoolTaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub worker_count: usize,
    pub queue_capacity: usize,
    pub stack_size_bytes: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoolTask {
    pub id: u64,
    pub name: String,
    pub status: PoolTaskStatus,
    pub priority: u32,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ThreadPoolStub {
    pub config: PoolConfig,
    pub tasks: Vec<PoolTask>,
    pub next_id: u64,
    pub completed_count: u64,
}

#[allow(dead_code)]
pub fn default_pool_config() -> PoolConfig {
    PoolConfig {
        worker_count: 4,
        queue_capacity: 128,
        stack_size_bytes: 2 * 1024 * 1024,
    }
}

#[allow(dead_code)]
pub fn new_thread_pool_stub(cfg: PoolConfig) -> ThreadPoolStub {
    ThreadPoolStub {
        config: cfg,
        tasks: Vec::new(),
        next_id: 1,
        completed_count: 0,
    }
}

#[allow(dead_code)]
pub fn submit_task(pool: &mut ThreadPoolStub, name: &str, priority: u32) -> u64 {
    let id = pool.next_id;
    pool.next_id += 1;
    pool.tasks.push(PoolTask {
        id,
        name: name.to_string(),
        status: PoolTaskStatus::Pending,
        priority,
    });
    id
}

/// Moves the first Pending task to Completed (stub implementation).
#[allow(dead_code)]
pub fn tick_pool(pool: &mut ThreadPoolStub) {
    if let Some(task) = pool
        .tasks
        .iter_mut()
        .find(|t| t.status == PoolTaskStatus::Pending)
    {
        task.status = PoolTaskStatus::Completed;
        pool.completed_count += 1;
    }
}

#[allow(dead_code)]
pub fn task_status(pool: &ThreadPoolStub, id: u64) -> Option<&PoolTaskStatus> {
    pool.tasks.iter().find(|t| t.id == id).map(|t| &t.status)
}

#[allow(dead_code)]
pub fn pending_task_count(pool: &ThreadPoolStub) -> usize {
    pool.tasks
        .iter()
        .filter(|t| t.status == PoolTaskStatus::Pending)
        .count()
}

#[allow(dead_code)]
pub fn completed_task_count(pool: &ThreadPoolStub) -> u64 {
    pool.completed_count
}

#[allow(dead_code)]
pub fn cancel_task(pool: &mut ThreadPoolStub, id: u64) -> bool {
    if let Some(task) = pool
        .tasks
        .iter_mut()
        .find(|t| t.id == id && t.status == PoolTaskStatus::Pending)
    {
        task.status = PoolTaskStatus::Failed;
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn pool_to_json(pool: &ThreadPoolStub) -> String {
    let tasks: Vec<String> = pool
        .tasks
        .iter()
        .map(|t| {
            let status = match t.status {
                PoolTaskStatus::Pending => "Pending",
                PoolTaskStatus::Running => "Running",
                PoolTaskStatus::Completed => "Completed",
                PoolTaskStatus::Failed => "Failed",
            };
            format!(
                r#"{{"id":{},"name":"{}","status":"{}","priority":{}}}"#,
                t.id, t.name, status, t.priority
            )
        })
        .collect();
    format!(
        r#"{{"worker_count":{},"queue_capacity":{},"next_id":{},"completed_count":{},"tasks":[{}]}}"#,
        pool.config.worker_count,
        pool.config.queue_capacity,
        pool.next_id,
        pool.completed_count,
        tasks.join(",")
    )
}

#[allow(dead_code)]
pub fn worker_count(pool: &ThreadPoolStub) -> usize {
    pool.config.worker_count
}

#[allow(dead_code)]
pub fn clear_completed(pool: &mut ThreadPoolStub) {
    pool.tasks.retain(|t| t.status != PoolTaskStatus::Completed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_pool_config();
        assert_eq!(cfg.worker_count, 4);
        assert_eq!(cfg.queue_capacity, 128);
    }

    #[test]
    fn test_submit_task_returns_id() {
        let mut pool = new_thread_pool_stub(default_pool_config());
        let id1 = submit_task(&mut pool, "task_a", 1);
        let id2 = submit_task(&mut pool, "task_b", 2);
        assert_ne!(id1, id2);
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn test_pending_count() {
        let mut pool = new_thread_pool_stub(default_pool_config());
        submit_task(&mut pool, "a", 1);
        submit_task(&mut pool, "b", 1);
        assert_eq!(pending_task_count(&pool), 2);
    }

    #[test]
    fn test_tick_completes_first_pending() {
        let mut pool = new_thread_pool_stub(default_pool_config());
        let id = submit_task(&mut pool, "job", 1);
        tick_pool(&mut pool);
        assert_eq!(task_status(&pool, id), Some(&PoolTaskStatus::Completed));
        assert_eq!(completed_task_count(&pool), 1);
    }

    #[test]
    fn test_task_status_not_found() {
        let pool = new_thread_pool_stub(default_pool_config());
        assert!(task_status(&pool, 999).is_none());
    }

    #[test]
    fn test_cancel_task() {
        let mut pool = new_thread_pool_stub(default_pool_config());
        let id = submit_task(&mut pool, "cancel_me", 1);
        assert!(cancel_task(&mut pool, id));
        assert_eq!(task_status(&pool, id), Some(&PoolTaskStatus::Failed));
    }

    #[test]
    fn test_cancel_nonexistent() {
        let mut pool = new_thread_pool_stub(default_pool_config());
        assert!(!cancel_task(&mut pool, 999));
    }

    #[test]
    fn test_cancel_completed_fails() {
        let mut pool = new_thread_pool_stub(default_pool_config());
        let id = submit_task(&mut pool, "done", 1);
        tick_pool(&mut pool);
        // Cannot cancel a completed task
        assert!(!cancel_task(&mut pool, id));
    }

    #[test]
    fn test_clear_completed() {
        let mut pool = new_thread_pool_stub(default_pool_config());
        submit_task(&mut pool, "a", 1);
        submit_task(&mut pool, "b", 1);
        tick_pool(&mut pool);
        clear_completed(&mut pool);
        assert_eq!(pool.tasks.len(), 1);
        assert_eq!(pool.tasks[0].name, "b");
    }

    #[test]
    fn test_worker_count() {
        let pool = new_thread_pool_stub(default_pool_config());
        assert_eq!(worker_count(&pool), 4);
    }

    #[test]
    fn test_pool_to_json() {
        let mut pool = new_thread_pool_stub(default_pool_config());
        submit_task(&mut pool, "my_task", 5);
        let j = pool_to_json(&pool);
        assert!(j.contains("\"my_task\""));
        assert!(j.contains("\"Pending\""));
        assert!(j.contains("\"worker_count\":4"));
    }
}
