// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Health check result aggregator — collects and summarizes component health.

/// Overall health status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// A single component's health check result.
#[derive(Clone, Debug)]
pub struct HealthCheckResult {
    pub component: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub latency_ms: u64,
}

/// Aggregated health report across all components.
#[derive(Clone, Debug)]
pub struct HealthReport {
    pub overall: HealthStatus,
    pub results: Vec<HealthCheckResult>,
}

/// Configuration for the health check aggregator.
#[derive(Clone, Debug)]
pub struct HealthCheckConfig {
    /// Maximum acceptable latency in ms before marking degraded.
    pub max_latency_ms: u64,
    /// Name of this service.
    pub service_name: String,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            max_latency_ms: 500,
            service_name: "default".into(),
        }
    }
}

/// A health check aggregator that collects component results.
pub struct HealthAggregator {
    pub config: HealthCheckConfig,
    results: Vec<HealthCheckResult>,
}

/// Creates a new health aggregator.
pub fn new_aggregator(config: HealthCheckConfig) -> HealthAggregator {
    HealthAggregator {
        config,
        results: Vec::new(),
    }
}

/// Adds a health check result for a component.
pub fn add_result(agg: &mut HealthAggregator, result: HealthCheckResult) {
    agg.results.push(result);
}

/// Computes the overall health from all recorded results.
pub fn aggregate_health(agg: &HealthAggregator) -> HealthReport {
    let overall = compute_overall(&agg.results, agg.config.max_latency_ms);
    HealthReport {
        overall,
        results: agg.results.clone(),
    }
}

fn compute_overall(results: &[HealthCheckResult], max_latency_ms: u64) -> HealthStatus {
    if results.is_empty() {
        return HealthStatus::Unknown;
    }
    let mut worst = HealthStatus::Healthy;
    for r in results {
        let effective = if r.latency_ms > max_latency_ms {
            HealthStatus::Degraded
        } else {
            r.status
        };
        worst = worse_of(worst, effective);
    }
    worst
}

fn worse_of(a: HealthStatus, b: HealthStatus) -> HealthStatus {
    match (a, b) {
        (HealthStatus::Unhealthy, _) | (_, HealthStatus::Unhealthy) => HealthStatus::Unhealthy,
        (HealthStatus::Degraded, _) | (_, HealthStatus::Degraded) => HealthStatus::Degraded,
        (HealthStatus::Unknown, _) | (_, HealthStatus::Unknown) => HealthStatus::Unknown,
        _ => HealthStatus::Healthy,
    }
}

/// Returns true if all components are healthy.
pub fn all_healthy(report: &HealthReport) -> bool {
    report.overall == HealthStatus::Healthy
}

/// Counts components by status.
pub fn count_by_status(report: &HealthReport, status: HealthStatus) -> usize {
    report.results.iter().filter(|r| r.status == status).count()
}

impl HealthAggregator {
    /// Creates a new aggregator with default config.
    pub fn new(config: HealthCheckConfig) -> Self {
        new_aggregator(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_agg() -> HealthAggregator {
        new_aggregator(HealthCheckConfig::default())
    }

    fn result(component: &str, status: HealthStatus, latency: u64) -> HealthCheckResult {
        HealthCheckResult {
            component: component.into(),
            status,
            message: None,
            latency_ms: latency,
        }
    }

    #[test]
    fn test_empty_aggregator_reports_unknown() {
        let agg = make_agg();
        let report = aggregate_health(&agg);
        assert_eq!(report.overall, HealthStatus::Unknown);
    }

    #[test]
    fn test_single_healthy_component() {
        let mut agg = make_agg();
        add_result(&mut agg, result("db", HealthStatus::Healthy, 10));
        let report = aggregate_health(&agg);
        assert_eq!(report.overall, HealthStatus::Healthy);
    }

    #[test]
    fn test_one_unhealthy_makes_overall_unhealthy() {
        let mut agg = make_agg();
        add_result(&mut agg, result("db", HealthStatus::Healthy, 10));
        add_result(&mut agg, result("cache", HealthStatus::Unhealthy, 10));
        let report = aggregate_health(&agg);
        assert_eq!(report.overall, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_high_latency_causes_degraded() {
        let mut agg = make_agg(); /* max_latency = 500ms */
        add_result(&mut agg, result("api", HealthStatus::Healthy, 600));
        let report = aggregate_health(&agg);
        assert_eq!(report.overall, HealthStatus::Degraded);
    }

    #[test]
    fn test_all_healthy_returns_true() {
        let mut agg = make_agg();
        add_result(&mut agg, result("a", HealthStatus::Healthy, 1));
        add_result(&mut agg, result("b", HealthStatus::Healthy, 2));
        let report = aggregate_health(&agg);
        assert!(all_healthy(&report));
    }

    #[test]
    fn test_count_by_status_works() {
        let mut agg = make_agg();
        add_result(&mut agg, result("a", HealthStatus::Healthy, 1));
        add_result(&mut agg, result("b", HealthStatus::Unhealthy, 1));
        add_result(&mut agg, result("c", HealthStatus::Healthy, 1));
        let report = aggregate_health(&agg);
        assert_eq!(count_by_status(&report, HealthStatus::Healthy), 2);
        assert_eq!(count_by_status(&report, HealthStatus::Unhealthy), 1);
    }

    #[test]
    fn test_degraded_overridden_by_unhealthy() {
        let mut agg = make_agg();
        add_result(&mut agg, result("a", HealthStatus::Degraded, 1));
        add_result(&mut agg, result("b", HealthStatus::Unhealthy, 1));
        let report = aggregate_health(&agg);
        assert_eq!(report.overall, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_report_contains_all_results() {
        let mut agg = make_agg();
        add_result(&mut agg, result("x", HealthStatus::Healthy, 5));
        add_result(&mut agg, result("y", HealthStatus::Healthy, 5));
        let report = aggregate_health(&agg);
        assert_eq!(report.results.len(), 2);
    }
}
