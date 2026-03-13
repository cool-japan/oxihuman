// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Capacity planning calculator stub.

/// A capacity planning resource specification.
#[derive(Debug, Clone)]
pub struct CapacitySpec {
    pub resource_name: String,
    pub current_capacity: f64,
    pub current_usage: f64,
    /// Expected growth rate per period (fraction, e.g. 0.1 = 10%).
    pub growth_rate: f64,
    /// Safety headroom factor (e.g. 1.2 = 20% buffer).
    pub headroom_factor: f64,
}

impl CapacitySpec {
    pub fn new(
        resource_name: &str,
        current_capacity: f64,
        current_usage: f64,
        growth_rate: f64,
        headroom_factor: f64,
    ) -> Self {
        Self {
            resource_name: resource_name.to_string(),
            current_capacity,
            current_usage,
            growth_rate,
            headroom_factor,
        }
    }

    pub fn utilization(&self) -> f64 {
        if self.current_capacity <= 0.0 {
            1.0
        } else {
            (self.current_usage / self.current_capacity).clamp(0.0, 1.0)
        }
    }

    /// Predicted usage after `periods` time periods with compound growth.
    pub fn predicted_usage(&self, periods: u32) -> f64 {
        self.current_usage * (1.0 + self.growth_rate).powi(periods as i32)
    }

    /// Required capacity to handle predicted usage with headroom.
    pub fn required_capacity(&self, periods: u32) -> f64 {
        self.predicted_usage(periods) * self.headroom_factor
    }

    /// Periods until capacity is breached (without headroom).
    pub fn periods_until_full(&self) -> Option<u32> {
        if self.growth_rate <= 0.0 || self.current_usage <= 0.0 {
            return None;
        }
        let mut usage = self.current_usage;
        for p in 0..10_000 {
            if usage >= self.current_capacity {
                return Some(p);
            }
            usage *= 1.0 + self.growth_rate;
        }
        None
    }
}

/// Registry of capacity specs.
#[derive(Debug, Default)]
pub struct CapacityPlanner {
    specs: Vec<CapacitySpec>,
}

impl CapacityPlanner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, spec: CapacitySpec) {
        self.specs.push(spec);
    }

    pub fn spec_count(&self) -> usize {
        self.specs.len()
    }

    pub fn most_utilized(&self) -> Option<&CapacitySpec> {
        self.specs.iter().max_by(|a, b| {
            a.utilization()
                .partial_cmp(&b.utilization())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn over_threshold(&self, threshold: f64) -> Vec<&CapacitySpec> {
        self.specs
            .iter()
            .filter(|s| s.utilization() > threshold)
            .collect()
    }

    pub fn specs(&self) -> &[CapacitySpec] {
        &self.specs
    }
}

pub fn new_capacity_planner() -> CapacityPlanner {
    CapacityPlanner::new()
}

pub fn cp_add(planner: &mut CapacityPlanner, spec: CapacitySpec) {
    planner.add(spec);
}

pub fn cp_spec_count(planner: &CapacityPlanner) -> usize {
    planner.spec_count()
}

pub fn cp_most_utilized(planner: &CapacityPlanner) -> Option<&CapacitySpec> {
    planner.most_utilized()
}

pub fn cp_over_threshold(planner: &CapacityPlanner, threshold: f64) -> Vec<&CapacitySpec> {
    planner.over_threshold(threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_spec(name: &str, cap: f64, usage: f64, growth: f64) -> CapacitySpec {
        CapacitySpec::new(name, cap, usage, growth, 1.2)
    }

    #[test]
    fn test_utilization() {
        let spec = make_spec("cpu", 100.0, 80.0, 0.05);
        assert!((spec.utilization() - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_predicted_usage_grows() {
        let spec = make_spec("mem", 1000.0, 500.0, 0.1);
        let pred = spec.predicted_usage(1);
        assert!(pred > 500.0);
    }

    #[test]
    fn test_required_capacity_includes_headroom() {
        let spec = CapacitySpec::new("disk", 100.0, 50.0, 0.0, 1.2);
        let req = spec.required_capacity(0);
        assert!((req - 60.0).abs() < 1e-5);
    }

    #[test]
    fn test_periods_until_full() {
        let spec = make_spec("net", 100.0, 50.0, 0.2);
        let p = spec.periods_until_full();
        assert!(p.is_some());
        assert!(p.expect("should succeed") > 0);
    }

    #[test]
    fn test_no_growth_no_full() {
        let spec = make_spec("x", 100.0, 50.0, 0.0);
        assert_eq!(spec.periods_until_full(), None);
    }

    #[test]
    fn test_add_and_count() {
        let mut planner = new_capacity_planner();
        cp_add(&mut planner, make_spec("cpu", 100.0, 40.0, 0.05));
        assert_eq!(cp_spec_count(&planner), 1);
    }

    #[test]
    fn test_most_utilized() {
        let mut planner = new_capacity_planner();
        cp_add(&mut planner, make_spec("cpu", 100.0, 80.0, 0.05));
        cp_add(&mut planner, make_spec("mem", 100.0, 30.0, 0.05));
        let top = cp_most_utilized(&planner).expect("should succeed");
        assert_eq!(top.resource_name, "cpu");
    }

    #[test]
    fn test_over_threshold() {
        let mut planner = new_capacity_planner();
        cp_add(&mut planner, make_spec("a", 100.0, 90.0, 0.0));
        cp_add(&mut planner, make_spec("b", 100.0, 50.0, 0.0));
        let over = cp_over_threshold(&planner, 0.8);
        assert_eq!(over.len(), 1);
    }

    #[test]
    fn test_compound_growth_5_periods() {
        let spec = make_spec("svc", 1000.0, 100.0, 0.1);
        let pred5 = spec.predicted_usage(5);
        /* 100 * 1.1^5 ≈ 161.05 */
        assert!(pred5 > 160.0 && pred5 < 162.0);
    }
}
