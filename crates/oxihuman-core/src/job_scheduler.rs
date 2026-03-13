#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Simple job scheduler with tick-based progression.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ScheduledJob {
    pub id: usize,
    pub name: String,
    pub delay_ticks: usize,
    pub remaining: usize,
    pub running: bool,
    pub done: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JobScheduler {
    jobs: Vec<ScheduledJob>,
    next_id: usize,
}

#[allow(dead_code)]
pub fn new_job_scheduler() -> JobScheduler {
    JobScheduler {
        jobs: Vec::new(),
        next_id: 0,
    }
}

#[allow(dead_code)]
pub fn schedule_job(sched: &mut JobScheduler, name: &str, delay_ticks: usize) -> usize {
    let id = sched.next_id;
    sched.next_id += 1;
    sched.jobs.push(ScheduledJob {
        id,
        name: name.to_string(),
        delay_ticks,
        remaining: delay_ticks,
        running: false,
        done: false,
    });
    id
}

#[allow(dead_code)]
pub fn cancel_job(sched: &mut JobScheduler, id: usize) -> bool {
    if let Some(job) = sched.jobs.iter_mut().find(|j| j.id == id && !j.done) {
        job.done = true;
        job.running = false;
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn pending_jobs(sched: &JobScheduler) -> usize {
    sched.jobs.iter().filter(|j| !j.done && !j.running).count()
}

#[allow(dead_code)]
pub fn running_jobs(sched: &JobScheduler) -> usize {
    sched.jobs.iter().filter(|j| j.running).count()
}

#[allow(dead_code)]
pub fn job_count_js(sched: &JobScheduler) -> usize {
    sched.jobs.len()
}

#[allow(dead_code)]
pub fn scheduler_tick(sched: &mut JobScheduler) {
    for job in &mut sched.jobs {
        if job.done {
            continue;
        }
        if job.remaining > 0 {
            job.remaining -= 1;
            if job.remaining == 0 {
                job.running = true;
            }
        } else if job.running {
            job.running = false;
            job.done = true;
        }
    }
}

#[allow(dead_code)]
pub fn scheduler_to_json(sched: &JobScheduler) -> String {
    format!(
        r#"{{"total":{},"pending":{},"running":{}}}"#,
        sched.jobs.len(),
        pending_jobs(sched),
        running_jobs(sched)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_scheduler() {
        let s = new_job_scheduler();
        assert_eq!(job_count_js(&s), 0);
    }

    #[test]
    fn test_schedule_job() {
        let mut s = new_job_scheduler();
        let id = schedule_job(&mut s, "task", 3);
        assert_eq!(id, 0);
        assert_eq!(job_count_js(&s), 1);
    }

    #[test]
    fn test_pending_jobs() {
        let mut s = new_job_scheduler();
        schedule_job(&mut s, "task", 3);
        assert_eq!(pending_jobs(&s), 1);
    }

    #[test]
    fn test_tick_progression() {
        let mut s = new_job_scheduler();
        schedule_job(&mut s, "task", 2);
        scheduler_tick(&mut s);
        assert_eq!(pending_jobs(&s), 1);
        scheduler_tick(&mut s);
        assert_eq!(running_jobs(&s), 1);
    }

    #[test]
    fn test_job_completion() {
        let mut s = new_job_scheduler();
        schedule_job(&mut s, "task", 1);
        scheduler_tick(&mut s); // remaining=0, running=true
        scheduler_tick(&mut s); // running=false, done=true
        assert_eq!(running_jobs(&s), 0);
        assert_eq!(pending_jobs(&s), 0);
    }

    #[test]
    fn test_cancel_job() {
        let mut s = new_job_scheduler();
        let id = schedule_job(&mut s, "task", 5);
        assert!(cancel_job(&mut s, id));
        assert_eq!(pending_jobs(&s), 0);
    }

    #[test]
    fn test_cancel_invalid() {
        let mut s = new_job_scheduler();
        assert!(!cancel_job(&mut s, 99));
    }

    #[test]
    fn test_scheduler_to_json() {
        let s = new_job_scheduler();
        let json = scheduler_to_json(&s);
        assert!(json.contains("\"total\":0"));
    }

    #[test]
    fn test_immediate_job() {
        let mut s = new_job_scheduler();
        schedule_job(&mut s, "now", 0);
        assert_eq!(running_jobs(&s), 0);
        scheduler_tick(&mut s);
        assert!(s.jobs[0].done);
    }

    #[test]
    fn test_multiple_jobs() {
        let mut s = new_job_scheduler();
        schedule_job(&mut s, "a", 1);
        schedule_job(&mut s, "b", 2);
        assert_eq!(job_count_js(&s), 2);
    }
}
