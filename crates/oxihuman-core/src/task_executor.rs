// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Synchronous task executor stub for WASM-compatible task scheduling.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecTaskStatus {
    Queued,
    Running,
    Done,
    Cancelled,
    Failed,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExecConfig {
    pub max_tasks: usize,
    pub time_slice_ms: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExecTask {
    pub id: u64,
    pub name: std::string::String,
    pub status: ExecTaskStatus,
    pub priority: i32,
    pub created_ms: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TaskExecutor {
    pub config: ExecConfig,
    pub tasks: Vec<ExecTask>,
    pub next_id: u64,
    pub total_executed: u64,
}

#[allow(dead_code)]
pub fn default_exec_config() -> ExecConfig {
    ExecConfig {
        max_tasks: 64,
        time_slice_ms: 16,
    }
}

#[allow(dead_code)]
pub fn new_task_executor(cfg: ExecConfig) -> TaskExecutor {
    TaskExecutor {
        config: cfg,
        tasks: Vec::new(),
        next_id: 1,
        total_executed: 0,
    }
}

#[allow(dead_code)]
pub fn exec_submit(executor: &mut TaskExecutor, name: &str, priority: i32, ts: u64) -> u64 {
    let id = executor.next_id;
    executor.next_id += 1;
    executor.tasks.push(ExecTask {
        id,
        name: name.to_string(),
        status: ExecTaskStatus::Queued,
        priority,
        created_ms: ts,
    });
    id
}

#[allow(dead_code)]
pub fn exec_tick(executor: &mut TaskExecutor) {
    // Find the highest-priority Queued task (highest priority number = first processed)
    let best_idx = executor
        .tasks
        .iter()
        .enumerate()
        .filter(|(_, t)| t.status == ExecTaskStatus::Queued)
        .max_by_key(|(_, t)| t.priority)
        .map(|(i, _)| i);

    if let Some(idx) = best_idx {
        executor.tasks[idx].status = ExecTaskStatus::Done;
        executor.total_executed += 1;
    }
}

#[allow(dead_code)]
pub fn exec_cancel(executor: &mut TaskExecutor, id: u64) -> bool {
    for task in &mut executor.tasks {
        if task.id == id && task.status == ExecTaskStatus::Queued {
            task.status = ExecTaskStatus::Cancelled;
            return true;
        }
    }
    false
}

#[allow(dead_code)]
pub fn exec_task_status(executor: &TaskExecutor, id: u64) -> Option<&ExecTaskStatus> {
    executor.tasks.iter().find(|t| t.id == id).map(|t| &t.status)
}

#[allow(dead_code)]
pub fn exec_queued_count(executor: &TaskExecutor) -> usize {
    executor
        .tasks
        .iter()
        .filter(|t| t.status == ExecTaskStatus::Queued)
        .count()
}

#[allow(dead_code)]
pub fn exec_done_count(executor: &TaskExecutor) -> u64 {
    executor.total_executed
}

#[allow(dead_code)]
pub fn exec_clear_done(executor: &mut TaskExecutor) {
    executor.tasks.retain(|t| t.status != ExecTaskStatus::Done);
}

#[allow(dead_code)]
pub fn exec_to_json(executor: &TaskExecutor) -> std::string::String {
    let tasks_json: Vec<std::string::String> = executor
        .tasks
        .iter()
        .map(|t| {
            let status = match t.status {
                ExecTaskStatus::Queued => "queued",
                ExecTaskStatus::Running => "running",
                ExecTaskStatus::Done => "done",
                ExecTaskStatus::Cancelled => "cancelled",
                ExecTaskStatus::Failed => "failed",
            };
            format!(
                r#"{{"id":{},"name":{:?},"status":{:?},"priority":{},"created_ms":{}}}"#,
                t.id, t.name, status, t.priority, t.created_ms
            )
        })
        .collect();
    format!(
        r#"{{"max_tasks":{},"total_executed":{},"tasks":[{}]}}"#,
        executor.config.max_tasks,
        executor.total_executed,
        tasks_json.join(",")
    )
}

#[allow(dead_code)]
pub fn exec_task_count(executor: &TaskExecutor) -> usize {
    executor.tasks.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_exec_config() {
        let cfg = default_exec_config();
        assert_eq!(cfg.max_tasks, 64);
        assert_eq!(cfg.time_slice_ms, 16);
    }

    #[test]
    fn test_new_task_executor() {
        let cfg = default_exec_config();
        let ex = new_task_executor(cfg);
        assert_eq!(ex.tasks.len(), 0);
        assert_eq!(ex.total_executed, 0);
    }

    #[test]
    fn test_exec_submit() {
        let mut ex = new_task_executor(default_exec_config());
        let id = exec_submit(&mut ex, "task1", 0, 100);
        assert_eq!(id, 1);
        assert_eq!(exec_task_count(&ex), 1);
        assert_eq!(exec_queued_count(&ex), 1);
    }

    #[test]
    fn test_exec_tick_moves_highest_priority() {
        let mut ex = new_task_executor(default_exec_config());
        exec_submit(&mut ex, "low", 1, 0);
        exec_submit(&mut ex, "high", 10, 0);
        exec_tick(&mut ex);
        // "high" (priority=10) should be done
        let high_id = ex.tasks.iter().find(|t| t.name == "high").expect("should succeed").id;
        assert_eq!(exec_task_status(&ex, high_id), Some(&ExecTaskStatus::Done));
        assert_eq!(exec_done_count(&ex), 1);
    }

    #[test]
    fn test_exec_cancel() {
        let mut ex = new_task_executor(default_exec_config());
        let id = exec_submit(&mut ex, "task", 0, 0);
        assert!(exec_cancel(&mut ex, id));
        assert_eq!(exec_task_status(&ex, id), Some(&ExecTaskStatus::Cancelled));
        // Cancel again should fail
        assert!(!exec_cancel(&mut ex, id));
    }

    #[test]
    fn test_exec_clear_done() {
        let mut ex = new_task_executor(default_exec_config());
        exec_submit(&mut ex, "t1", 0, 0);
        exec_tick(&mut ex);
        assert_eq!(exec_task_count(&ex), 1);
        exec_clear_done(&mut ex);
        assert_eq!(exec_task_count(&ex), 0);
    }

    #[test]
    fn test_exec_to_json() {
        let mut ex = new_task_executor(default_exec_config());
        exec_submit(&mut ex, "myjob", 5, 42);
        let json = exec_to_json(&ex);
        assert!(json.contains("myjob"));
        assert!(json.contains("queued"));
        assert!(json.contains("max_tasks"));
    }

    #[test]
    fn test_exec_task_status_unknown() {
        let ex = new_task_executor(default_exec_config());
        assert_eq!(exec_task_status(&ex, 999), None);
    }

    #[test]
    fn test_exec_tick_empty() {
        let mut ex = new_task_executor(default_exec_config());
        exec_tick(&mut ex); // should not panic
        assert_eq!(exec_done_count(&ex), 0);
    }

    #[test]
    fn test_exec_queued_count_after_cancel() {
        let mut ex = new_task_executor(default_exec_config());
        let id = exec_submit(&mut ex, "t", 0, 0);
        assert_eq!(exec_queued_count(&ex), 1);
        exec_cancel(&mut ex, id);
        assert_eq!(exec_queued_count(&ex), 0);
    }
}
