// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Simple sequential plan executor with named steps.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum StepState {
    Pending,
    Running,
    Done,
    Failed(String),
    Skipped,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PlanStep {
    pub name: String,
    pub state: StepState,
    pub duration_ms: u32,
}

#[allow(dead_code)]
pub struct PlanExecutor {
    steps: Vec<PlanStep>,
    current: usize,
    abort_on_failure: bool,
    aborted: bool,
}

#[allow(dead_code)]
impl PlanExecutor {
    pub fn new(abort_on_failure: bool) -> Self {
        Self {
            steps: Vec::new(),
            current: 0,
            abort_on_failure,
            aborted: false,
        }
    }
    pub fn add_step(&mut self, name: &str) {
        self.steps.push(PlanStep {
            name: name.to_string(),
            state: StepState::Pending,
            duration_ms: 0,
        });
    }
    pub fn complete_current(&mut self, duration_ms: u32) -> bool {
        if self.current >= self.steps.len() {
            return false;
        }
        self.steps[self.current].state = StepState::Done;
        self.steps[self.current].duration_ms = duration_ms;
        self.current += 1;
        true
    }
    pub fn fail_current(&mut self, reason: &str) -> bool {
        if self.current >= self.steps.len() {
            return false;
        }
        self.steps[self.current].state = StepState::Failed(reason.to_string());
        self.current += 1;
        if self.abort_on_failure {
            self.aborted = true;
        }
        true
    }
    pub fn skip_current(&mut self) -> bool {
        if self.current >= self.steps.len() {
            return false;
        }
        self.steps[self.current].state = StepState::Skipped;
        self.current += 1;
        true
    }
    pub fn is_complete(&self) -> bool {
        !self.aborted && self.current >= self.steps.len()
    }
    pub fn is_aborted(&self) -> bool {
        self.aborted
    }
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }
    pub fn done_count(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| s.state == StepState::Done)
            .count()
    }
    pub fn failed_count(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| matches!(s.state, StepState::Failed(_)))
            .count()
    }
    pub fn pending_count(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| s.state == StepState::Pending)
            .count()
    }
    pub fn current_step(&self) -> Option<&PlanStep> {
        self.steps.get(self.current)
    }
    pub fn steps(&self) -> &[PlanStep] {
        &self.steps
    }
    pub fn total_duration_ms(&self) -> u32 {
        self.steps.iter().map(|s| s.duration_ms).sum()
    }
    pub fn reset(&mut self) {
        for s in &mut self.steps {
            s.state = StepState::Pending;
            s.duration_ms = 0;
        }
        self.current = 0;
        self.aborted = false;
    }
}

#[allow(dead_code)]
pub fn new_plan_executor(abort_on_failure: bool) -> PlanExecutor {
    PlanExecutor::new(abort_on_failure)
}
#[allow(dead_code)]
pub fn pe_add_step(e: &mut PlanExecutor, name: &str) {
    e.add_step(name);
}
#[allow(dead_code)]
pub fn pe_complete(e: &mut PlanExecutor, ms: u32) -> bool {
    e.complete_current(ms)
}
#[allow(dead_code)]
pub fn pe_fail(e: &mut PlanExecutor, reason: &str) -> bool {
    e.fail_current(reason)
}
#[allow(dead_code)]
pub fn pe_skip(e: &mut PlanExecutor) -> bool {
    e.skip_current()
}
#[allow(dead_code)]
pub fn pe_is_complete(e: &PlanExecutor) -> bool {
    e.is_complete()
}
#[allow(dead_code)]
pub fn pe_is_aborted(e: &PlanExecutor) -> bool {
    e.is_aborted()
}
#[allow(dead_code)]
pub fn pe_done_count(e: &PlanExecutor) -> usize {
    e.done_count()
}
#[allow(dead_code)]
pub fn pe_failed_count(e: &PlanExecutor) -> usize {
    e.failed_count()
}
#[allow(dead_code)]
pub fn pe_step_count(e: &PlanExecutor) -> usize {
    e.step_count()
}
#[allow(dead_code)]
pub fn pe_total_ms(e: &PlanExecutor) -> u32 {
    e.total_duration_ms()
}
#[allow(dead_code)]
pub fn pe_reset(e: &mut PlanExecutor) {
    e.reset();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_complete_all() {
        let mut e = new_plan_executor(true);
        pe_add_step(&mut e, "init");
        pe_add_step(&mut e, "build");
        pe_complete(&mut e, 10);
        pe_complete(&mut e, 20);
        assert!(pe_is_complete(&e));
        assert_eq!(pe_done_count(&e), 2);
    }
    #[test]
    fn test_fail_aborts() {
        let mut e = new_plan_executor(true);
        pe_add_step(&mut e, "step1");
        pe_fail(&mut e, "error");
        assert!(pe_is_aborted(&e));
    }
    #[test]
    fn test_fail_no_abort() {
        let mut e = new_plan_executor(false);
        pe_add_step(&mut e, "step1");
        pe_fail(&mut e, "error");
        assert!(!pe_is_aborted(&e));
        assert_eq!(pe_failed_count(&e), 1);
    }
    #[test]
    fn test_skip() {
        let mut e = new_plan_executor(true);
        pe_add_step(&mut e, "s");
        pe_skip(&mut e);
        assert!(pe_is_complete(&e));
    }
    #[test]
    fn test_total_duration() {
        let mut e = new_plan_executor(true);
        pe_add_step(&mut e, "a");
        pe_add_step(&mut e, "b");
        pe_complete(&mut e, 100);
        pe_complete(&mut e, 200);
        assert_eq!(pe_total_ms(&e), 300);
    }
    #[test]
    fn test_current_step() {
        let mut e = new_plan_executor(true);
        pe_add_step(&mut e, "first");
        assert_eq!(e.current_step().map(|s| s.name.as_str()), Some("first"));
        pe_complete(&mut e, 0);
        assert!(e.current_step().is_none());
    }
    #[test]
    fn test_pending_count() {
        let mut e = new_plan_executor(true);
        pe_add_step(&mut e, "a");
        pe_add_step(&mut e, "b");
        pe_add_step(&mut e, "c");
        pe_complete(&mut e, 0);
        assert_eq!(e.pending_count(), 2);
    }
    #[test]
    fn test_reset() {
        let mut e = new_plan_executor(true);
        pe_add_step(&mut e, "x");
        pe_complete(&mut e, 5);
        pe_reset(&mut e);
        assert_eq!(pe_done_count(&e), 0);
        assert!(!pe_is_complete(&e));
    }
    #[test]
    fn test_step_count() {
        let mut e = new_plan_executor(false);
        pe_add_step(&mut e, "a");
        pe_add_step(&mut e, "b");
        assert_eq!(pe_step_count(&e), 2);
    }
    #[test]
    fn test_empty_complete_immediately() {
        let e = new_plan_executor(true);
        assert!(pe_is_complete(&e));
    }
}
