// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Experiment variant tracker.

use std::collections::HashMap;

/// A recorded experiment assignment.
#[derive(Debug, Clone)]
pub struct ExperimentAssignment {
    pub experiment_id: String,
    pub variant: String,
    pub user_id: String,
}

/// Tracks which experiment variant each user is assigned to.
#[derive(Debug, Default)]
pub struct ExperimentTracker {
    /// experiment_id -> (user_id -> variant)
    assignments: HashMap<String, HashMap<String, String>>,
}

impl ExperimentTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn assign(&mut self, experiment_id: &str, user_id: &str, variant: &str) {
        self.assignments
            .entry(experiment_id.to_string())
            .or_default()
            .insert(user_id.to_string(), variant.to_string());
    }

    pub fn get_variant(&self, experiment_id: &str, user_id: &str) -> Option<&str> {
        self.assignments
            .get(experiment_id)?
            .get(user_id)
            .map(String::as_str)
    }

    pub fn participant_count(&self, experiment_id: &str) -> usize {
        self.assignments
            .get(experiment_id)
            .map(|m| m.len())
            .unwrap_or(0)
    }

    pub fn variant_counts(&self, experiment_id: &str) -> HashMap<String, usize> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        if let Some(users) = self.assignments.get(experiment_id) {
            for v in users.values() {
                *counts.entry(v.clone()).or_insert(0) += 1;
            }
        }
        counts
    }

    pub fn experiment_count(&self) -> usize {
        self.assignments.len()
    }

    pub fn all_assignments(&self, experiment_id: &str) -> Vec<ExperimentAssignment> {
        let mut out = Vec::new();
        if let Some(users) = self.assignments.get(experiment_id) {
            for (user, variant) in users {
                out.push(ExperimentAssignment {
                    experiment_id: experiment_id.to_string(),
                    variant: variant.clone(),
                    user_id: user.clone(),
                });
            }
        }
        out
    }
}

pub fn new_experiment_tracker() -> ExperimentTracker {
    ExperimentTracker::new()
}

pub fn tracker_assign(tracker: &mut ExperimentTracker, exp: &str, user: &str, variant: &str) {
    tracker.assign(exp, user, variant);
}

pub fn tracker_get_variant<'a>(
    tracker: &'a ExperimentTracker,
    exp: &str,
    user: &str,
) -> Option<&'a str> {
    tracker.get_variant(exp, user)
}

pub fn tracker_participant_count(tracker: &ExperimentTracker, exp: &str) -> usize {
    tracker.participant_count(exp)
}

pub fn tracker_experiment_count(tracker: &ExperimentTracker) -> usize {
    tracker.experiment_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assign_and_get() {
        let mut t = new_experiment_tracker();
        tracker_assign(&mut t, "exp1", "user1", "control");
        assert_eq!(tracker_get_variant(&t, "exp1", "user1"), Some("control"));
    }

    #[test]
    fn test_unknown_user() {
        let t = new_experiment_tracker();
        assert_eq!(tracker_get_variant(&t, "exp", "nobody"), None);
    }

    #[test]
    fn test_participant_count() {
        let mut t = new_experiment_tracker();
        tracker_assign(&mut t, "exp", "u1", "a");
        tracker_assign(&mut t, "exp", "u2", "b");
        assert_eq!(tracker_participant_count(&t, "exp"), 2);
    }

    #[test]
    fn test_experiment_count() {
        let mut t = new_experiment_tracker();
        tracker_assign(&mut t, "exp1", "u1", "a");
        tracker_assign(&mut t, "exp2", "u2", "b");
        assert_eq!(tracker_experiment_count(&t), 2);
    }

    #[test]
    fn test_variant_counts() {
        let mut t = new_experiment_tracker();
        tracker_assign(&mut t, "e", "u1", "ctrl");
        tracker_assign(&mut t, "e", "u2", "ctrl");
        tracker_assign(&mut t, "e", "u3", "treat");
        let counts = t.variant_counts("e");
        assert_eq!(counts["ctrl"], 2);
        assert_eq!(counts["treat"], 1);
    }

    #[test]
    fn test_overwrite_assignment() {
        let mut t = new_experiment_tracker();
        tracker_assign(&mut t, "e", "u1", "a");
        tracker_assign(&mut t, "e", "u1", "b");
        assert_eq!(tracker_get_variant(&t, "e", "u1"), Some("b"));
    }

    #[test]
    fn test_all_assignments_count() {
        let mut t = new_experiment_tracker();
        tracker_assign(&mut t, "e", "u1", "a");
        tracker_assign(&mut t, "e", "u2", "b");
        assert_eq!(t.all_assignments("e").len(), 2);
    }

    #[test]
    fn test_zero_participants_unknown_exp() {
        let t = new_experiment_tracker();
        assert_eq!(tracker_participant_count(&t, "no_exp"), 0);
    }

    #[test]
    fn test_multiple_experiments_isolated() {
        /* user in exp1 is not visible in exp2 */
        let mut t = new_experiment_tracker();
        tracker_assign(&mut t, "exp1", "u1", "x");
        assert_eq!(tracker_get_variant(&t, "exp2", "u1"), None);
    }
}
