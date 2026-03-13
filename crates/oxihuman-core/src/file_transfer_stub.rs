// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! File transfer protocol stub.

/// Transfer state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransferState {
    Pending,
    InProgress { bytes_sent: u64, total_bytes: u64 },
    Completed,
    Failed(String),
}

impl TransferState {
    pub fn is_done(&self) -> bool {
        matches!(self, TransferState::Completed | TransferState::Failed(_))
    }

    pub fn progress_pct(&self) -> f32 {
        match self {
            TransferState::InProgress {
                bytes_sent,
                total_bytes,
            } => {
                if *total_bytes == 0 {
                    1.0
                } else {
                    *bytes_sent as f32 / *total_bytes as f32
                }
            }
            TransferState::Completed => 1.0,
            _ => 0.0,
        }
    }
}

/// A file transfer job.
#[derive(Debug, Clone)]
pub struct TransferJob {
    pub id: u64,
    pub source: String,
    pub destination: String,
    pub state: TransferState,
}

impl TransferJob {
    pub fn new(id: u64, source: &str, destination: &str, size_bytes: u64) -> Self {
        TransferJob {
            id,
            source: source.to_string(),
            destination: destination.to_string(),
            state: TransferState::InProgress {
                bytes_sent: 0,
                total_bytes: size_bytes,
            },
        }
    }

    pub fn mark_complete(&mut self) {
        self.state = TransferState::Completed;
    }

    pub fn mark_failed(&mut self, reason: &str) {
        self.state = TransferState::Failed(reason.to_string());
    }

    pub fn advance(&mut self, bytes: u64) {
        if let TransferState::InProgress {
            ref mut bytes_sent,
            total_bytes,
        } = self.state
        {
            *bytes_sent = (*bytes_sent + bytes).min(total_bytes);
            if *bytes_sent == total_bytes {
                self.state = TransferState::Completed;
            }
        }
    }
}

/// Transfer manager stub.
pub struct TransferManager {
    jobs: Vec<TransferJob>,
    next_id: u64,
}

impl TransferManager {
    pub fn new() -> Self {
        TransferManager {
            jobs: Vec::new(),
            next_id: 1,
        }
    }

    pub fn enqueue(&mut self, source: &str, destination: &str, size_bytes: u64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.jobs
            .push(TransferJob::new(id, source, destination, size_bytes));
        id
    }

    pub fn get_job(&self, id: u64) -> Option<&TransferJob> {
        self.jobs.iter().find(|j| j.id == id)
    }

    pub fn get_job_mut(&mut self, id: u64) -> Option<&mut TransferJob> {
        self.jobs.iter_mut().find(|j| j.id == id)
    }

    pub fn completed_count(&self) -> usize {
        self.jobs
            .iter()
            .filter(|j| j.state == TransferState::Completed)
            .count()
    }

    pub fn pending_count(&self) -> usize {
        self.jobs.iter().filter(|j| !j.state.is_done()).count()
    }
}

impl Default for TransferManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a transfer manager.
pub fn new_transfer_manager() -> TransferManager {
    TransferManager::new()
}

/// Advance all in-progress jobs by `chunk_size` bytes.
pub fn tick_all(mgr: &mut TransferManager, chunk_size: u64) {
    for job in &mut mgr.jobs {
        job.advance(chunk_size);
    }
}

/// Cancel a job by ID (mark as failed).
pub fn cancel_job(mgr: &mut TransferManager, id: u64) {
    if let Some(job) = mgr.get_job_mut(id) {
        job.mark_failed("cancelled");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enqueue_job() {
        let mut m = new_transfer_manager();
        let id = m.enqueue("/src", "/dst", 1000);
        assert_eq!(id, 1);
        assert!(m.get_job(id).is_some());
    }

    #[test]
    fn test_advance_completes() {
        let mut m = new_transfer_manager();
        let id = m.enqueue("/src", "/dst", 100);
        if let Some(job) = m.get_job_mut(id) {
            job.advance(100);
        }
        assert_eq!(m.completed_count(), 1);
    }

    #[test]
    fn test_progress_pct() {
        let state = TransferState::InProgress {
            bytes_sent: 50,
            total_bytes: 100,
        };
        assert!((state.progress_pct() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_is_done_completed() {
        assert!(TransferState::Completed.is_done());
    }

    #[test]
    fn test_is_done_failed() {
        assert!(TransferState::Failed("err".to_string()).is_done());
    }

    #[test]
    fn test_cancel_job() {
        let mut m = new_transfer_manager();
        let id = m.enqueue("/src", "/dst", 1000);
        cancel_job(&mut m, id);
        assert!(matches!(
            m.get_job(id).expect("should succeed").state,
            TransferState::Failed(_)
        ));
    }

    #[test]
    fn test_tick_all() {
        let mut m = new_transfer_manager();
        m.enqueue("/s", "/d", 100);
        tick_all(&mut m, 50);
        let j = m.get_job(1).expect("should succeed");
        assert!((j.state.progress_pct() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_pending_count() {
        let mut m = new_transfer_manager();
        m.enqueue("/s1", "/d1", 100);
        m.enqueue("/s2", "/d2", 200);
        assert_eq!(m.pending_count(), 2);
    }

    #[test]
    fn test_mark_failed() {
        let mut job = TransferJob::new(1, "/s", "/d", 100);
        job.mark_failed("network error");
        assert!(matches!(job.state, TransferState::Failed(_)));
    }

    #[test]
    fn test_mark_complete() {
        let mut job = TransferJob::new(1, "/s", "/d", 0);
        job.mark_complete();
        assert_eq!(job.state, TransferState::Completed);
    }
}
