//! Task scheduler with priorities and time-based execution.

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct ScheduledTask {
    pub id: u64,
    pub name: String,
    pub priority: TaskPriority,
    /// Absolute scheduled time in seconds.
    pub scheduled_time: f64,
    /// None = one-shot, Some = repeating interval.
    pub interval: Option<f64>,
    pub enabled: bool,
    pub run_count: u32,
}

#[allow(dead_code)]
pub struct Scheduler {
    pub tasks: Vec<ScheduledTask>,
    pub current_time: f64,
    pub next_id: u64,
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn new_scheduler() -> Scheduler {
    Scheduler {
        tasks: Vec::new(),
        current_time: 0.0,
        next_id: 1,
    }
}

// ---------------------------------------------------------------------------
// Scheduling
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn schedule_once(sched: &mut Scheduler, name: &str, delay: f64, priority: TaskPriority) -> u64 {
    let id = sched.next_id;
    sched.next_id += 1;
    sched.tasks.push(ScheduledTask {
        id,
        name: name.to_string(),
        priority,
        scheduled_time: sched.current_time + delay,
        interval: None,
        enabled: true,
        run_count: 0,
    });
    id
}

#[allow(dead_code)]
pub fn schedule_repeating(
    sched: &mut Scheduler,
    name: &str,
    interval: f64,
    priority: TaskPriority,
) -> u64 {
    let id = sched.next_id;
    sched.next_id += 1;
    sched.tasks.push(ScheduledTask {
        id,
        name: name.to_string(),
        priority,
        scheduled_time: sched.current_time + interval,
        interval: Some(interval),
        enabled: true,
        run_count: 0,
    });
    id
}

#[allow(dead_code)]
pub fn cancel_task(sched: &mut Scheduler, id: u64) -> bool {
    let before = sched.tasks.len();
    sched.tasks.retain(|t| t.id != id);
    sched.tasks.len() < before
}

// ---------------------------------------------------------------------------
// Time advancement
// ---------------------------------------------------------------------------

/// Advance scheduler time by `dt`. Returns all tasks that fired during this step.
#[allow(dead_code)]
pub fn advance_time(sched: &mut Scheduler, dt: f64) -> Vec<ScheduledTask> {
    sched.current_time += dt;
    let current = sched.current_time;

    let mut fired: Vec<ScheduledTask> = Vec::new();

    for task in sched.tasks.iter_mut() {
        if !task.enabled {
            continue;
        }
        if task.scheduled_time <= current {
            let snapshot = task.clone();
            fired.push(snapshot);
            task.run_count += 1;
            if let Some(interval) = task.interval {
                // Reschedule repeating task.
                task.scheduled_time += interval;
            }
        }
    }

    // Sort fired by priority (highest first), then by scheduled_time.
    fired.sort_by(|a, b| {
        b.priority.cmp(&a.priority).then(
            a.scheduled_time
                .partial_cmp(&b.scheduled_time)
                .unwrap_or(std::cmp::Ordering::Equal),
        )
    });

    fired
}

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn due_tasks(sched: &Scheduler) -> Vec<&ScheduledTask> {
    sched
        .tasks
        .iter()
        .filter(|t| t.enabled && t.scheduled_time <= sched.current_time)
        .collect()
}

#[allow(dead_code)]
pub fn task_count(sched: &Scheduler) -> usize {
    sched.tasks.len()
}

#[allow(dead_code)]
pub fn enabled_task_count(sched: &Scheduler) -> usize {
    sched.tasks.iter().filter(|t| t.enabled).count()
}

#[allow(dead_code)]
pub fn get_scheduled_task(sched: &Scheduler, id: u64) -> Option<&ScheduledTask> {
    sched.tasks.iter().find(|t| t.id == id)
}

#[allow(dead_code)]
pub fn set_task_enabled(sched: &mut Scheduler, id: u64, enabled: bool) {
    if let Some(task) = sched.tasks.iter_mut().find(|t| t.id == id) {
        task.enabled = enabled;
    }
}

/// Returns tasks sorted by priority (highest first, then by scheduled_time).
#[allow(dead_code)]
pub fn tasks_by_priority(sched: &Scheduler) -> Vec<&ScheduledTask> {
    let mut sorted: Vec<&ScheduledTask> = sched.tasks.iter().collect();
    sorted.sort_by(|a, b| {
        b.priority.cmp(&a.priority).then(
            a.scheduled_time
                .partial_cmp(&b.scheduled_time)
                .unwrap_or(std::cmp::Ordering::Equal),
        )
    });
    sorted
}

#[allow(dead_code)]
pub fn next_due_time(sched: &Scheduler) -> Option<f64> {
    sched
        .tasks
        .iter()
        .filter(|t| t.enabled)
        .map(|t| t.scheduled_time)
        .reduce(f64::min)
}

#[allow(dead_code)]
pub fn clear_completed_tasks(sched: &mut Scheduler) {
    // Remove one-shot tasks that have been run at least once.
    sched
        .tasks
        .retain(|t| !(t.interval.is_none() && t.run_count > 0));
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_scheduler() {
        let s = new_scheduler();
        assert_eq!(task_count(&s), 0);
        assert!((s.current_time).abs() < 1e-9);
    }

    #[test]
    fn test_schedule_once() {
        let mut s = new_scheduler();
        let id = schedule_once(&mut s, "task_a", 1.0, TaskPriority::Normal);
        assert_eq!(task_count(&s), 1);
        assert!(get_scheduled_task(&s, id).is_some());
    }

    #[test]
    fn test_schedule_repeating() {
        let mut s = new_scheduler();
        let id = schedule_repeating(&mut s, "rep", 0.5, TaskPriority::High);
        let task = get_scheduled_task(&s, id).expect("should succeed");
        assert!(task.interval.is_some());
    }

    #[test]
    fn test_cancel_task() {
        let mut s = new_scheduler();
        let id = schedule_once(&mut s, "tmp", 1.0, TaskPriority::Low);
        assert!(cancel_task(&mut s, id));
        assert_eq!(task_count(&s), 0);
        assert!(!cancel_task(&mut s, id));
    }

    #[test]
    fn test_advance_time_fires_task() {
        let mut s = new_scheduler();
        schedule_once(&mut s, "go", 1.0, TaskPriority::Normal);
        let fired = advance_time(&mut s, 1.5);
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].name, "go");
    }

    #[test]
    fn test_advance_time_no_fire_early() {
        let mut s = new_scheduler();
        schedule_once(&mut s, "future", 5.0, TaskPriority::Normal);
        let fired = advance_time(&mut s, 1.0);
        assert!(fired.is_empty());
    }

    #[test]
    fn test_repeating_reschedules() {
        let mut s = new_scheduler();
        let id = schedule_repeating(&mut s, "rep", 1.0, TaskPriority::Normal);
        let fired1 = advance_time(&mut s, 1.1);
        assert_eq!(fired1.len(), 1);
        // Task should still be in the scheduler (rescheduled).
        assert!(get_scheduled_task(&s, id).is_some());
        let task = get_scheduled_task(&s, id).expect("should succeed");
        assert!(task.scheduled_time > s.current_time - 0.001);
    }

    #[test]
    fn test_task_count() {
        let mut s = new_scheduler();
        schedule_once(&mut s, "a", 1.0, TaskPriority::Low);
        schedule_once(&mut s, "b", 2.0, TaskPriority::High);
        assert_eq!(task_count(&s), 2);
    }

    #[test]
    fn test_due_tasks() {
        let mut s = new_scheduler();
        schedule_once(&mut s, "now", 0.0, TaskPriority::Normal);
        schedule_once(&mut s, "later", 10.0, TaskPriority::Normal);
        let due = due_tasks(&s);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].name, "now");
    }

    #[test]
    fn test_tasks_by_priority_order() {
        let mut s = new_scheduler();
        schedule_once(&mut s, "low", 1.0, TaskPriority::Low);
        schedule_once(&mut s, "crit", 1.0, TaskPriority::Critical);
        schedule_once(&mut s, "norm", 1.0, TaskPriority::Normal);
        let sorted = tasks_by_priority(&s);
        assert_eq!(sorted[0].priority, TaskPriority::Critical);
        assert_eq!(sorted[sorted.len() - 1].priority, TaskPriority::Low);
    }

    #[test]
    fn test_next_due_time() {
        let mut s = new_scheduler();
        schedule_once(&mut s, "a", 3.0, TaskPriority::Normal);
        schedule_once(&mut s, "b", 1.0, TaskPriority::Normal);
        let next = next_due_time(&s).expect("should succeed");
        assert!((next - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_next_due_time_empty() {
        let s = new_scheduler();
        assert!(next_due_time(&s).is_none());
    }

    #[test]
    fn test_clear_completed_tasks() {
        let mut s = new_scheduler();
        schedule_once(&mut s, "one_shot", 0.5, TaskPriority::Normal);
        schedule_repeating(&mut s, "rep", 1.0, TaskPriority::Normal);
        advance_time(&mut s, 1.5);
        clear_completed_tasks(&mut s);
        // One-shot should be removed; repeating should remain.
        assert_eq!(task_count(&s), 1);
        assert_eq!(s.tasks[0].name, "rep");
    }

    #[test]
    fn test_enabled_task_count() {
        let mut s = new_scheduler();
        let id1 = schedule_once(&mut s, "a", 1.0, TaskPriority::Normal);
        schedule_once(&mut s, "b", 1.0, TaskPriority::Normal);
        set_task_enabled(&mut s, id1, false);
        assert_eq!(enabled_task_count(&s), 1);
    }

    #[test]
    fn test_set_task_enabled() {
        let mut s = new_scheduler();
        let id = schedule_once(&mut s, "x", 0.0, TaskPriority::Normal);
        set_task_enabled(&mut s, id, false);
        let fired = advance_time(&mut s, 1.0);
        // Disabled task should not fire.
        assert!(fired.is_empty());
    }
}
