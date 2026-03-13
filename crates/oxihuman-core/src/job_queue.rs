// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Job/task queue with priority, status tracking and cancellation.

#![allow(dead_code)]

// ── Types ─────────────────────────────────────────────────────────────────────

/// Status of a job in the queue.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Running,
    Done,
    Cancelled,
}

/// A single job entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Job {
    /// Unique job id.
    pub id: u32,
    /// Human-readable description.
    pub description: String,
    /// Priority (higher = more urgent).
    pub priority: i32,
    /// Current status.
    pub status: JobStatus,
    /// Submission timestamp (monotonic ms counter).
    pub submitted_at: u64,
    /// Optional tag/category.
    pub tag: Option<String>,
}

/// The job queue.
#[allow(dead_code)]
pub struct JobQueue {
    pub jobs: Vec<Job>,
    next_id: u32,
    time_ms: u64,
}

/// Alias for a slice of jobs filtered by status.
pub type JobSlice<'a> = Vec<&'a Job>;

// ── Constructors ──────────────────────────────────────────────────────────────

/// Create a new empty job queue.
#[allow(dead_code)]
pub fn new_job_queue() -> JobQueue {
    JobQueue {
        jobs: Vec::new(),
        next_id: 1,
        time_ms: 0,
    }
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Submit a new job. Returns the assigned job id.
#[allow(dead_code)]
pub fn submit_job(queue: &mut JobQueue, description: &str, priority: i32) -> u32 {
    let id = queue.next_id;
    queue.next_id += 1;
    queue.time_ms += 1;
    queue.jobs.push(Job {
        id,
        description: description.to_string(),
        priority,
        status: JobStatus::Pending,
        submitted_at: queue.time_ms,
        tag: None,
    });
    id
}

/// Cancel a job by id. Returns true if found and cancelled.
#[allow(dead_code)]
pub fn cancel_job(queue: &mut JobQueue, id: u32) -> bool {
    for job in &mut queue.jobs {
        if job.id == id && job.status == JobStatus::Pending {
            job.status = JobStatus::Cancelled;
            return true;
        }
    }
    false
}

/// Get the current status of a job. Returns `None` if not found.
#[allow(dead_code)]
pub fn job_status(queue: &JobQueue, id: u32) -> Option<JobStatus> {
    queue
        .jobs
        .iter()
        .find(|j| j.id == id)
        .map(|j| j.status.clone())
}

/// Count of pending jobs.
#[allow(dead_code)]
pub fn pending_job_count(queue: &JobQueue) -> usize {
    queue
        .jobs
        .iter()
        .filter(|j| j.status == JobStatus::Pending)
        .count()
}

/// Count of done jobs.
#[allow(dead_code)]
pub fn done_job_count(queue: &JobQueue) -> usize {
    queue
        .jobs
        .iter()
        .filter(|j| j.status == JobStatus::Done)
        .count()
}

/// Count of cancelled jobs.
#[allow(dead_code)]
pub fn cancelled_job_count(queue: &JobQueue) -> usize {
    queue
        .jobs
        .iter()
        .filter(|j| j.status == JobStatus::Cancelled)
        .count()
}

/// Total number of jobs (all statuses).
#[allow(dead_code)]
pub fn total_job_count(queue: &JobQueue) -> usize {
    queue.jobs.len()
}

/// Remove and return all done jobs.
#[allow(dead_code)]
pub fn drain_done_jobs(queue: &mut JobQueue) -> Vec<Job> {
    let mut done = Vec::new();
    queue.jobs.retain(|j| {
        if j.status == JobStatus::Done {
            done.push(j.clone());
            false
        } else {
            true
        }
    });
    done
}

/// Remove all jobs from the queue.
#[allow(dead_code)]
pub fn clear_job_queue(queue: &mut JobQueue) {
    queue.jobs.clear();
}

/// Mark a pending job as running, then done (simulated execution). Returns true if found.
#[allow(dead_code)]
pub fn requeue_job(queue: &mut JobQueue, id: u32) -> bool {
    for job in &mut queue.jobs {
        if job.id == id {
            job.status = JobStatus::Pending;
            return true;
        }
    }
    false
}

/// Serialize the queue to a simple JSON string.
#[allow(dead_code)]
pub fn job_queue_to_json(queue: &JobQueue) -> String {
    let entries: Vec<String> = queue
        .jobs
        .iter()
        .map(|j| {
            let status = match j.status {
                JobStatus::Pending => "pending",
                JobStatus::Running => "running",
                JobStatus::Done => "done",
                JobStatus::Cancelled => "cancelled",
            };
            format!(
                r#"{{"id":{},"status":"{}","priority":{},"description":"{}"}}"#,
                j.id, status, j.priority, j.description
            )
        })
        .collect();
    format!("[{}]", entries.join(","))
}

/// Return a reference to the oldest pending job (lowest `submitted_at`).
#[allow(dead_code)]
pub fn oldest_pending_job(queue: &JobQueue) -> Option<&Job> {
    queue
        .jobs
        .iter()
        .filter(|j| j.status == JobStatus::Pending)
        .min_by_key(|j| j.submitted_at)
}

/// Return references to all jobs with the given status, sorted by priority desc.
#[allow(dead_code)]
pub fn jobs_by_status<'a>(queue: &'a JobQueue, status: &JobStatus) -> JobSlice<'a> {
    let mut v: Vec<&Job> = queue.jobs.iter().filter(|j| &j.status == status).collect();
    v.sort_by(|a, b| b.priority.cmp(&a.priority));
    v
}

/// Mark a job as running. Returns true if found in Pending state.
#[allow(dead_code)]
pub fn start_job(queue: &mut JobQueue, id: u32) -> bool {
    for job in &mut queue.jobs {
        if job.id == id && job.status == JobStatus::Pending {
            job.status = JobStatus::Running;
            return true;
        }
    }
    false
}

/// Mark a running job as done. Returns true if found in Running state.
#[allow(dead_code)]
pub fn finish_job(queue: &mut JobQueue, id: u32) -> bool {
    for job in &mut queue.jobs {
        if job.id == id && job.status == JobStatus::Running {
            job.status = JobStatus::Done;
            return true;
        }
    }
    false
}

/// Set a tag on a job. Returns true if the job was found.
#[allow(dead_code)]
pub fn tag_job(queue: &mut JobQueue, id: u32, tag: &str) -> bool {
    for job in &mut queue.jobs {
        if job.id == id {
            job.tag = Some(tag.to_string());
            return true;
        }
    }
    false
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_queue() -> JobQueue {
        new_job_queue()
    }

    #[test]
    fn test_new_queue_empty() {
        let q = make_queue();
        assert_eq!(total_job_count(&q), 0);
    }

    #[test]
    fn test_submit_job_increments_count() {
        let mut q = make_queue();
        submit_job(&mut q, "task A", 1);
        submit_job(&mut q, "task B", 2);
        assert_eq!(total_job_count(&q), 2);
    }

    #[test]
    fn test_submit_returns_unique_ids() {
        let mut q = make_queue();
        let a = submit_job(&mut q, "a", 0);
        let b = submit_job(&mut q, "b", 0);
        assert_ne!(a, b);
    }

    #[test]
    fn test_cancel_job() {
        let mut q = make_queue();
        let id = submit_job(&mut q, "task", 1);
        assert!(cancel_job(&mut q, id));
        assert_eq!(job_status(&q, id), Some(JobStatus::Cancelled));
    }

    #[test]
    fn test_cancel_nonexistent() {
        let mut q = make_queue();
        assert!(!cancel_job(&mut q, 999));
    }

    #[test]
    fn test_job_status_pending() {
        let mut q = make_queue();
        let id = submit_job(&mut q, "x", 0);
        assert_eq!(job_status(&q, id), Some(JobStatus::Pending));
    }

    #[test]
    fn test_job_status_none() {
        let q = make_queue();
        assert!(job_status(&q, 42).is_none());
    }

    #[test]
    fn test_pending_job_count() {
        let mut q = make_queue();
        submit_job(&mut q, "a", 1);
        submit_job(&mut q, "b", 1);
        let id = submit_job(&mut q, "c", 1);
        cancel_job(&mut q, id);
        assert_eq!(pending_job_count(&q), 2);
    }

    #[test]
    fn test_done_job_count() {
        let mut q = make_queue();
        let id = submit_job(&mut q, "x", 1);
        start_job(&mut q, id);
        finish_job(&mut q, id);
        assert_eq!(done_job_count(&q), 1);
    }

    #[test]
    fn test_cancelled_job_count() {
        let mut q = make_queue();
        let a = submit_job(&mut q, "a", 1);
        let b = submit_job(&mut q, "b", 1);
        cancel_job(&mut q, a);
        cancel_job(&mut q, b);
        assert_eq!(cancelled_job_count(&q), 2);
    }

    #[test]
    fn test_total_job_count() {
        let mut q = make_queue();
        submit_job(&mut q, "a", 0);
        submit_job(&mut q, "b", 0);
        submit_job(&mut q, "c", 0);
        assert_eq!(total_job_count(&q), 3);
    }

    #[test]
    fn test_drain_done_jobs() {
        let mut q = make_queue();
        let a = submit_job(&mut q, "a", 1);
        let b = submit_job(&mut q, "b", 1);
        start_job(&mut q, a);
        finish_job(&mut q, a);
        let drained = drain_done_jobs(&mut q);
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].id, a);
        assert_eq!(total_job_count(&q), 1);
        let _ = b;
    }

    #[test]
    fn test_clear_job_queue() {
        let mut q = make_queue();
        submit_job(&mut q, "a", 0);
        submit_job(&mut q, "b", 0);
        clear_job_queue(&mut q);
        assert_eq!(total_job_count(&q), 0);
    }

    #[test]
    fn test_requeue_job() {
        let mut q = make_queue();
        let id = submit_job(&mut q, "x", 1);
        cancel_job(&mut q, id);
        assert!(requeue_job(&mut q, id));
        assert_eq!(job_status(&q, id), Some(JobStatus::Pending));
    }

    #[test]
    fn test_job_queue_to_json() {
        let mut q = make_queue();
        submit_job(&mut q, "render", 5);
        let json = job_queue_to_json(&q);
        assert!(json.contains("render"));
        assert!(json.contains("pending"));
    }

    #[test]
    fn test_oldest_pending_job() {
        let mut q = make_queue();
        let first = submit_job(&mut q, "first", 1);
        submit_job(&mut q, "second", 10);
        let oldest = oldest_pending_job(&q).expect("should succeed");
        assert_eq!(oldest.id, first);
    }

    #[test]
    fn test_jobs_by_status_sorted() {
        let mut q = make_queue();
        submit_job(&mut q, "low", 1);
        submit_job(&mut q, "high", 99);
        let v = jobs_by_status(&q, &JobStatus::Pending);
        assert_eq!(v.len(), 2);
        assert!(v[0].priority >= v[1].priority);
    }

    #[test]
    fn test_start_and_finish_job() {
        let mut q = make_queue();
        let id = submit_job(&mut q, "work", 1);
        assert!(start_job(&mut q, id));
        assert_eq!(job_status(&q, id), Some(JobStatus::Running));
        assert!(finish_job(&mut q, id));
        assert_eq!(job_status(&q, id), Some(JobStatus::Done));
    }

    #[test]
    fn test_tag_job() {
        let mut q = make_queue();
        let id = submit_job(&mut q, "tagged", 1);
        assert!(tag_job(&mut q, id, "render"));
        let job = q.jobs.iter().find(|j| j.id == id).expect("should succeed");
        assert_eq!(job.tag.as_deref(), Some("render"));
    }
}
