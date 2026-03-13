// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct HealthMonitor {
    pub checks: Vec<(String, bool)>,
}

impl HealthMonitor {
    pub fn new() -> Self {
        HealthMonitor { checks: Vec::new() }
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_health_monitor() -> HealthMonitor {
    HealthMonitor::new()
}

pub fn health_register(m: &mut HealthMonitor, name: &str, healthy: bool) {
    m.checks.push((name.to_string(), healthy));
}

/// Update an existing check. Returns true if the check was found.
pub fn health_update(m: &mut HealthMonitor, name: &str, healthy: bool) -> bool {
    for (n, h) in &mut m.checks {
        if n == name {
            *h = healthy;
            return true;
        }
    }
    false
}

pub fn health_all_ok(m: &HealthMonitor) -> bool {
    m.checks.iter().all(|(_, h)| *h)
}

pub fn health_count(m: &HealthMonitor) -> usize {
    m.checks.len()
}

pub fn health_failing_count(m: &HealthMonitor) -> usize {
    m.checks.iter().filter(|(_, h)| !h).count()
}

pub fn health_summary(m: &HealthMonitor) -> String {
    let total = m.checks.len();
    let failing = health_failing_count(m);
    format!("checks={} failing={}", total, failing)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        /* new monitor has no checks */
        let m = new_health_monitor();
        assert_eq!(health_count(&m), 0);
    }

    #[test]
    fn test_register_healthy() {
        /* register adds a healthy check */
        let mut m = new_health_monitor();
        health_register(&mut m, "db", true);
        assert!(health_all_ok(&m));
    }

    #[test]
    fn test_register_failing() {
        /* register adds a failing check */
        let mut m = new_health_monitor();
        health_register(&mut m, "db", false);
        assert!(!health_all_ok(&m));
        assert_eq!(health_failing_count(&m), 1);
    }

    #[test]
    fn test_update_existing() {
        /* update changes check status */
        let mut m = new_health_monitor();
        health_register(&mut m, "svc", false);
        assert!(health_update(&mut m, "svc", true));
        assert!(health_all_ok(&m));
    }

    #[test]
    fn test_update_missing() {
        /* update returns false for unknown check */
        let mut m = new_health_monitor();
        assert!(!health_update(&mut m, "ghost", true));
    }

    #[test]
    fn test_failing_count() {
        /* failing count is correct */
        let mut m = new_health_monitor();
        health_register(&mut m, "a", true);
        health_register(&mut m, "b", false);
        health_register(&mut m, "c", false);
        assert_eq!(health_failing_count(&m), 2);
    }

    #[test]
    fn test_summary_format() {
        /* summary contains expected fields */
        let mut m = new_health_monitor();
        health_register(&mut m, "x", true);
        health_register(&mut m, "y", false);
        let s = health_summary(&m);
        assert!(s.contains("checks=2"));
        assert!(s.contains("failing=1"));
    }
}
