// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Resource quota manager.

use std::collections::HashMap;

/// A quota for a named resource.
#[derive(Debug, Clone)]
pub struct QuotaEntry {
    pub name: String,
    pub limit: u64,
    pub used: u64,
}

impl QuotaEntry {
    pub fn available(&self) -> u64 {
        self.limit.saturating_sub(self.used)
    }

    pub fn utilization(&self) -> f64 {
        if self.limit == 0 {
            1.0
        } else {
            self.used as f64 / self.limit as f64
        }
    }

    pub fn is_exceeded(&self) -> bool {
        self.used > self.limit
    }
}

/// Manager for multiple resource quotas.
#[derive(Debug, Default)]
pub struct QuotaManager {
    quotas: HashMap<String, QuotaEntry>,
}

impl QuotaManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_quota(&mut self, name: &str, limit: u64) {
        self.quotas
            .entry(name.to_string())
            .and_modify(|q| q.limit = limit)
            .or_insert(QuotaEntry {
                name: name.to_string(),
                limit,
                used: 0,
            });
    }

    pub fn consume(&mut self, name: &str, amount: u64) -> bool {
        if let Some(q) = self.quotas.get_mut(name) {
            if q.used + amount <= q.limit {
                q.used += amount;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn release(&mut self, name: &str, amount: u64) {
        if let Some(q) = self.quotas.get_mut(name) {
            q.used = q.used.saturating_sub(amount);
        }
    }

    pub fn available(&self, name: &str) -> u64 {
        self.quotas.get(name).map(|q| q.available()).unwrap_or(0)
    }

    pub fn exceeded_quotas(&self) -> Vec<&str> {
        self.quotas
            .values()
            .filter(|q| q.is_exceeded())
            .map(|q| q.name.as_str())
            .collect()
    }

    pub fn quota_count(&self) -> usize {
        self.quotas.len()
    }

    pub fn utilization(&self, name: &str) -> f64 {
        self.quotas
            .get(name)
            .map(|q| q.utilization())
            .unwrap_or(0.0)
    }
}

pub fn new_quota_manager() -> QuotaManager {
    QuotaManager::new()
}

pub fn qm_set(qm: &mut QuotaManager, name: &str, limit: u64) {
    qm.set_quota(name, limit);
}

pub fn qm_consume(qm: &mut QuotaManager, name: &str, amount: u64) -> bool {
    qm.consume(name, amount)
}

pub fn qm_release(qm: &mut QuotaManager, name: &str, amount: u64) {
    qm.release(name, amount);
}

pub fn qm_available(qm: &QuotaManager, name: &str) -> u64 {
    qm.available(name)
}

pub fn qm_utilization(qm: &QuotaManager, name: &str) -> f64 {
    qm.utilization(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_available() {
        let mut qm = new_quota_manager();
        qm_set(&mut qm, "cpu", 100);
        assert_eq!(qm_available(&qm, "cpu"), 100);
    }

    #[test]
    fn test_consume_within_limit() {
        let mut qm = new_quota_manager();
        qm_set(&mut qm, "mem", 1000);
        assert!(qm_consume(&mut qm, "mem", 500));
        assert_eq!(qm_available(&qm, "mem"), 500);
    }

    #[test]
    fn test_consume_over_limit_denied() {
        let mut qm = new_quota_manager();
        qm_set(&mut qm, "net", 100);
        assert!(!qm_consume(&mut qm, "net", 200));
    }

    #[test]
    fn test_release() {
        let mut qm = new_quota_manager();
        qm_set(&mut qm, "disk", 500);
        qm_consume(&mut qm, "disk", 300);
        qm_release(&mut qm, "disk", 100);
        assert_eq!(qm_available(&qm, "disk"), 300);
    }

    #[test]
    fn test_release_below_zero_saturates() {
        let mut qm = new_quota_manager();
        qm_set(&mut qm, "x", 100);
        qm_release(&mut qm, "x", 1000);
        assert_eq!(qm_available(&qm, "x"), 100);
    }

    #[test]
    fn test_utilization() {
        let mut qm = new_quota_manager();
        qm_set(&mut qm, "r", 100);
        qm_consume(&mut qm, "r", 25);
        let u = qm_utilization(&qm, "r");
        assert!((u - 0.25).abs() < 1e-10);
    }

    #[test]
    fn test_unknown_quota_available_zero() {
        let qm = new_quota_manager();
        assert_eq!(qm_available(&qm, "missing"), 0);
    }

    #[test]
    fn test_quota_count() {
        let mut qm = new_quota_manager();
        qm_set(&mut qm, "a", 10);
        qm_set(&mut qm, "b", 20);
        assert_eq!(qm.quota_count(), 2);
    }

    #[test]
    fn test_exact_limit_consume_succeeds() {
        let mut qm = new_quota_manager();
        qm_set(&mut qm, "z", 50);
        assert!(qm_consume(&mut qm, "z", 50));
        assert_eq!(qm_available(&qm, "z"), 0);
    }
}
