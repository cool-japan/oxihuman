// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Sequential job queue for managing and executing multiple export operations,
//! with progress tracking and error collection.

use std::path::PathBuf;

use oxihuman_mesh::MeshBuffers;

use crate::{
    export_glb, export_json_mesh_to_file, export_obj, export_ply, export_stl_binary, PlyFormat,
};

// ── Internal job type ────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum JobType {
    Glb,
    Obj,
    Stl,
    Ply,
    Json,
}

// ── Job status ───────────────────────────────────────────────────────────────

/// Status of a single export job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    /// Contains the error message if the job failed.
    Failed(String),
}

// ── Export job ───────────────────────────────────────────────────────────────

/// A single export job descriptor.
#[derive(Debug, Clone)]
pub struct ExportJob {
    pub id: usize,
    pub name: String,
    pub output_path: PathBuf,
    pub status: JobStatus,
    job_type: JobType,
}

impl ExportJob {
    /// Create a new export job in `Pending` state.
    pub fn new(id: usize, name: impl Into<String>, output_path: PathBuf) -> Self {
        Self {
            id,
            name: name.into(),
            output_path,
            status: JobStatus::Pending,
            job_type: JobType::Glb,
        }
    }

    /// Returns `true` if the job has finished (completed or failed).
    pub fn is_done(&self) -> bool {
        matches!(self.status, JobStatus::Completed | JobStatus::Failed(_))
    }

    /// Returns `true` if the job ended in failure.
    pub fn is_failed(&self) -> bool {
        matches!(self.status, JobStatus::Failed(_))
    }
}

// ── Queue result ─────────────────────────────────────────────────────────────

/// Result summary for a completed job queue run.
#[derive(Debug, Clone)]
pub struct QueueResult {
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    /// `(job_name, error_message)` pairs for each failure.
    pub errors: Vec<(String, String)>,
}

impl QueueResult {
    /// Fraction of jobs that succeeded, in `[0.0, 1.0]`.
    /// Returns `1.0` when `total == 0`.
    pub fn success_rate(&self) -> f32 {
        if self.total == 0 {
            return 1.0;
        }
        self.completed as f32 / self.total as f32
    }

    /// Returns `true` when every job completed without error.
    pub fn all_succeeded(&self) -> bool {
        self.failed == 0
    }

    /// Returns `true` when at least one job failed.
    pub fn has_failures(&self) -> bool {
        self.failed > 0
    }
}

// ── Job queue ────────────────────────────────────────────────────────────────

/// A sequential queue of export jobs.
pub struct ExportJobQueue {
    jobs: Vec<ExportJob>,
    next_id: usize,
}

impl ExportJobQueue {
    /// Create an empty job queue.
    pub fn new() -> Self {
        Self {
            jobs: Vec::new(),
            next_id: 0,
        }
    }

    // -- builders ------------------------------------------------------------

    fn add_job(&mut self, name: impl Into<String>, path: PathBuf, job_type: JobType) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let mut job = ExportJob::new(id, name, path);
        job.job_type = job_type;
        self.jobs.push(job);
        id
    }

    /// Add a GLB export job.
    pub fn add_glb(&mut self, name: impl Into<String>, path: PathBuf) -> usize {
        self.add_job(name, path, JobType::Glb)
    }

    /// Add an OBJ export job.
    pub fn add_obj(&mut self, name: impl Into<String>, path: PathBuf) -> usize {
        self.add_job(name, path, JobType::Obj)
    }

    /// Add an STL export job.
    pub fn add_stl(&mut self, name: impl Into<String>, path: PathBuf) -> usize {
        self.add_job(name, path, JobType::Stl)
    }

    /// Add a PLY export job.
    pub fn add_ply(&mut self, name: impl Into<String>, path: PathBuf) -> usize {
        self.add_job(name, path, JobType::Ply)
    }

    /// Add a JSON mesh export job.
    pub fn add_json(&mut self, name: impl Into<String>, path: PathBuf) -> usize {
        self.add_job(name, path, JobType::Json)
    }

    // -- introspection -------------------------------------------------------

    /// Total number of jobs (pending, running, done, or failed).
    pub fn job_count(&self) -> usize {
        self.jobs.len()
    }

    /// Number of jobs still waiting to run.
    pub fn pending_count(&self) -> usize {
        self.jobs
            .iter()
            .filter(|j| j.status == JobStatus::Pending)
            .count()
    }

    /// Returns `true` when the queue has no jobs at all.
    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }

    /// Look up a job by its ID.
    pub fn get_job(&self, id: usize) -> Option<&ExportJob> {
        self.jobs.iter().find(|j| j.id == id)
    }

    /// Collect references to all failed jobs.
    pub fn failed_jobs(&self) -> Vec<&ExportJob> {
        self.jobs.iter().filter(|j| j.is_failed()).collect()
    }

    // -- execution -----------------------------------------------------------

    /// Execute all pending jobs sequentially using `mesh`.
    ///
    /// `progress` is called with `(completed_so_far, total_pending)` after
    /// every job, whether it succeeded or failed.
    pub fn run(
        &mut self,
        mesh: &MeshBuffers,
        mut progress: impl FnMut(usize, usize),
    ) -> QueueResult {
        let pending_ids: Vec<usize> = self
            .jobs
            .iter()
            .filter(|j| j.status == JobStatus::Pending)
            .map(|j| j.id)
            .collect();

        let total = pending_ids.len();
        let mut completed = 0usize;
        let mut failed = 0usize;
        let mut errors: Vec<(String, String)> = Vec::new();

        for (step, id) in pending_ids.into_iter().enumerate() {
            // Mark as Running.
            if let Some(job) = self.jobs.iter_mut().find(|j| j.id == id) {
                job.status = JobStatus::Running;
            }

            // Snapshot what we need before the borrow ends.
            let (job_type, output_path, job_name) = {
                let job = match self.jobs.iter().find(|j| j.id == id) {
                    Some(j) => j,
                    None => continue,
                };
                (
                    job.job_type.clone(),
                    job.output_path.clone(),
                    job.name.clone(),
                )
            };

            let result = dispatch_export(mesh, &job_type, &output_path);

            // Update status.
            if let Some(job) = self.jobs.iter_mut().find(|j| j.id == id) {
                match result {
                    Ok(()) => {
                        job.status = JobStatus::Completed;
                        completed += 1;
                    }
                    Err(e) => {
                        let msg = e.to_string();
                        errors.push((job_name, msg.clone()));
                        job.status = JobStatus::Failed(msg);
                        failed += 1;
                    }
                }
            }

            progress(step + 1, total);
        }

        QueueResult {
            total,
            completed,
            failed,
            errors,
        }
    }

    /// Retry all failed jobs using `mesh`.
    pub fn retry_failed(&mut self, mesh: &MeshBuffers) -> QueueResult {
        // Reset failed jobs back to Pending.
        for job in &mut self.jobs {
            if job.is_failed() {
                job.status = JobStatus::Pending;
            }
        }
        self.run(mesh, |_, _| {})
    }

    // -- mutation ------------------------------------------------------------

    /// Remove all jobs from the queue.
    pub fn clear(&mut self) {
        self.jobs.clear();
    }

    /// Remove every job whose status is `Completed`.
    pub fn remove_completed(&mut self) {
        self.jobs.retain(|j| j.status != JobStatus::Completed);
    }
}

impl Default for ExportJobQueue {
    fn default() -> Self {
        Self::new()
    }
}

// ── Internal dispatch ────────────────────────────────────────────────────────

fn dispatch_export(
    mesh: &MeshBuffers,
    job_type: &JobType,
    output_path: &std::path::Path,
) -> anyhow::Result<()> {
    match job_type {
        JobType::Glb => {
            let mut m = mesh.clone();
            m.has_suit = true;
            export_glb(&m, output_path)
        }
        JobType::Obj => export_obj(mesh, output_path),
        JobType::Stl => export_stl_binary(mesh, output_path),
        JobType::Ply => export_ply(mesh, output_path, PlyFormat::BinaryLittleEndian),
        JobType::Json => export_json_mesh_to_file(mesh, output_path),
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mesh() -> MeshBuffers {
        MeshBuffers {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            colors: None,
            has_suit: false,
        }
    }

    #[test]
    fn queue_new_is_empty() {
        let q = ExportJobQueue::new();
        assert!(q.is_empty());
        assert_eq!(q.job_count(), 0);
        assert_eq!(q.pending_count(), 0);
    }

    #[test]
    fn add_glb_job_increases_count() {
        let mut q = ExportJobQueue::new();
        q.add_glb("test", PathBuf::from("/tmp/test_job_queue_x.glb"));
        assert_eq!(q.job_count(), 1);
        assert_eq!(q.pending_count(), 1);
        assert!(!q.is_empty());
    }

    #[test]
    fn add_multiple_jobs() {
        let mut q = ExportJobQueue::new();
        q.add_glb("a", PathBuf::from("/tmp/test_job_queue_a.glb"));
        q.add_obj("b", PathBuf::from("/tmp/test_job_queue_b.obj"));
        q.add_stl("c", PathBuf::from("/tmp/test_job_queue_c.stl"));
        q.add_ply("d", PathBuf::from("/tmp/test_job_queue_d.ply"));
        q.add_json("e", PathBuf::from("/tmp/test_job_queue_e.json"));
        assert_eq!(q.job_count(), 5);
        assert_eq!(q.pending_count(), 5);
    }

    #[test]
    fn job_status_starts_pending() {
        let mut q = ExportJobQueue::new();
        let id = q.add_glb("pending", PathBuf::from("/tmp/test_job_queue_p.glb"));
        let job = q.get_job(id).unwrap();
        assert_eq!(job.status, JobStatus::Pending);
        assert!(!job.is_done());
        assert!(!job.is_failed());
    }

    #[test]
    fn run_empty_queue_succeeds() {
        let mut q = ExportJobQueue::new();
        let mesh = make_mesh();
        let result = q.run(&mesh, |_, _| {});
        assert_eq!(result.total, 0);
        assert_eq!(result.completed, 0);
        assert_eq!(result.failed, 0);
        assert!(result.all_succeeded());
        assert!(!result.has_failures());
        assert!((result.success_rate() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn run_single_glb_job() {
        let mut q = ExportJobQueue::new();
        let path = PathBuf::from("/tmp/test_job_queue_single.glb");
        q.add_glb("single-glb", path.clone());
        let mesh = make_mesh();
        let result = q.run(&mesh, |_, _| {});
        assert_eq!(result.total, 1);
        assert_eq!(result.completed, 1);
        assert_eq!(result.failed, 0);
        assert!(result.all_succeeded());
        assert!(path.exists(), "GLB file should have been created");
    }

    #[test]
    fn run_single_obj_job() {
        let mut q = ExportJobQueue::new();
        let path = PathBuf::from("/tmp/test_job_queue_single.obj");
        q.add_obj("single-obj", path.clone());
        let mesh = make_mesh();
        let result = q.run(&mesh, |_, _| {});
        assert_eq!(result.total, 1);
        assert_eq!(result.completed, 1);
        assert!(result.all_succeeded());
        assert!(path.exists(), "OBJ file should have been created");
    }

    #[test]
    fn run_multiple_jobs_all_complete() {
        let mut q = ExportJobQueue::new();
        q.add_glb("glb", PathBuf::from("/tmp/test_job_queue_multi.glb"));
        q.add_obj("obj", PathBuf::from("/tmp/test_job_queue_multi.obj"));
        q.add_stl("stl", PathBuf::from("/tmp/test_job_queue_multi.stl"));
        q.add_ply("ply", PathBuf::from("/tmp/test_job_queue_multi.ply"));
        q.add_json("json", PathBuf::from("/tmp/test_job_queue_multi.json"));

        let mesh = make_mesh();
        let result = q.run(&mesh, |_, _| {});
        assert_eq!(result.total, 5);
        assert_eq!(result.completed, 5);
        assert_eq!(result.failed, 0);
        assert!(result.all_succeeded());
    }

    #[test]
    fn queue_result_success_rate() {
        let result = QueueResult {
            total: 4,
            completed: 3,
            failed: 1,
            errors: vec![("job".into(), "oops".into())],
        };
        let rate = result.success_rate();
        assert!((rate - 0.75).abs() < 1e-5, "expected 0.75, got {rate}");
        assert!(result.has_failures());
        assert!(!result.all_succeeded());
    }

    #[test]
    fn failed_jobs_empty_after_success() {
        let mut q = ExportJobQueue::new();
        q.add_obj("obj", PathBuf::from("/tmp/test_job_queue_fj.obj"));
        let mesh = make_mesh();
        q.run(&mesh, |_, _| {});
        assert!(q.failed_jobs().is_empty());
    }

    #[test]
    fn clear_removes_all_jobs() {
        let mut q = ExportJobQueue::new();
        q.add_glb("a", PathBuf::from("/tmp/test_job_queue_ca.glb"));
        q.add_obj("b", PathBuf::from("/tmp/test_job_queue_cb.obj"));
        assert_eq!(q.job_count(), 2);
        q.clear();
        assert!(q.is_empty());
        assert_eq!(q.job_count(), 0);
    }

    #[test]
    fn remove_completed_keeps_failed() {
        let mut q = ExportJobQueue::new();
        // Add a good job and a job with a bad (non-writable) path.
        q.add_obj("good", PathBuf::from("/tmp/test_job_queue_rc_good.obj"));
        q.add_glb("bad", PathBuf::from("/no_such_dir/impossible.glb"));

        let mesh = make_mesh();
        q.run(&mesh, |_, _| {});

        q.remove_completed();

        // The failed job must survive; the completed job must be gone.
        assert_eq!(q.job_count(), 1);
        assert_eq!(q.failed_jobs().len(), 1);
    }

    #[test]
    fn progress_callback_called() {
        let mut q = ExportJobQueue::new();
        q.add_obj("p1", PathBuf::from("/tmp/test_job_queue_prog1.obj"));
        q.add_obj("p2", PathBuf::from("/tmp/test_job_queue_prog2.obj"));
        q.add_obj("p3", PathBuf::from("/tmp/test_job_queue_prog3.obj"));

        let mesh = make_mesh();
        let mut calls: Vec<(usize, usize)> = Vec::new();
        q.run(&mesh, |done, total| calls.push((done, total)));

        assert_eq!(calls.len(), 3);
        assert_eq!(calls[0], (1, 3));
        assert_eq!(calls[1], (2, 3));
        assert_eq!(calls[2], (3, 3));
    }
}
