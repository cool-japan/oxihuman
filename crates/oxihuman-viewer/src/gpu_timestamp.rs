// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! GPU timestamp query management.

use std::f32::consts::FRAC_PI_4;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GpuTimestampEntry {
    pub label: String,
    pub frame: u64,
    pub start_ns: u64,
    pub end_ns: u64,
}

impl GpuTimestampEntry {
    #[allow(dead_code)]
    pub fn duration_ns(&self) -> u64 {
        self.end_ns.saturating_sub(self.start_ns)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct GpuTimestampManager {
    pub entries: Vec<GpuTimestampEntry>,
    pub current_frame: u64,
}

#[allow(dead_code)]
pub fn new_gpu_timestamp_manager() -> GpuTimestampManager {
    GpuTimestampManager::default()
}

#[allow(dead_code)]
pub fn gts_begin_frame(mgr: &mut GpuTimestampManager) {
    mgr.current_frame += 1;
}

#[allow(dead_code)]
pub fn gts_record(mgr: &mut GpuTimestampManager, label: &str, start_ns: u64, end_ns: u64) {
    mgr.entries.push(GpuTimestampEntry {
        label: label.to_string(),
        frame: mgr.current_frame,
        start_ns,
        end_ns,
    });
}

#[allow(dead_code)]
pub fn gts_clear(mgr: &mut GpuTimestampManager) {
    mgr.entries.clear();
}

#[allow(dead_code)]
pub fn gts_count(mgr: &GpuTimestampManager) -> usize {
    mgr.entries.len()
}

#[allow(dead_code)]
pub fn gts_total_ns(mgr: &GpuTimestampManager) -> u64 {
    mgr.entries.iter().map(|e| e.duration_ns()).sum()
}

#[allow(dead_code)]
pub fn gts_average_ns(mgr: &GpuTimestampManager) -> f64 {
    if mgr.entries.is_empty() {
        return 0.0;
    }
    gts_total_ns(mgr) as f64 / mgr.entries.len() as f64
}

#[allow(dead_code)]
pub fn gts_max_ns(mgr: &GpuTimestampManager) -> u64 {
    mgr.entries
        .iter()
        .map(|e| e.duration_ns())
        .max()
        .unwrap_or(0)
}

#[allow(dead_code)]
pub fn gts_slowest_pass(mgr: &GpuTimestampManager) -> Option<&str> {
    mgr.entries
        .iter()
        .max_by_key(|e| e.duration_ns())
        .map(|e| e.label.as_str())
}

#[allow(dead_code)]
pub fn gts_time_angle_rad(mgr: &GpuTimestampManager) -> f32 {
    let t = gts_average_ns(mgr) as f32;
    if t > 0.0 {
        (1.0 / t).atan().min(FRAC_PI_4)
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn gts_to_json(mgr: &GpuTimestampManager) -> String {
    format!(
        "{{\"count\":{},\"total_ns\":{}}}",
        gts_count(mgr),
        gts_total_ns(mgr)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_is_empty() {
        assert_eq!(gts_count(&new_gpu_timestamp_manager()), 0);
    }
    #[test]
    fn record_increments_count() {
        let mut m = new_gpu_timestamp_manager();
        gts_record(&mut m, "pass_a", 0, 1000);
        assert_eq!(gts_count(&m), 1);
    }
    #[test]
    fn clear_empties() {
        let mut m = new_gpu_timestamp_manager();
        gts_record(&mut m, "a", 0, 100);
        gts_clear(&mut m);
        assert_eq!(gts_count(&m), 0);
    }
    #[test]
    fn begin_frame_increments() {
        let mut m = new_gpu_timestamp_manager();
        gts_begin_frame(&mut m);
        assert_eq!(m.current_frame, 1);
    }
    #[test]
    fn total_ns_sums() {
        let mut m = new_gpu_timestamp_manager();
        gts_record(&mut m, "a", 0, 500);
        gts_record(&mut m, "b", 0, 300);
        assert_eq!(gts_total_ns(&m), 800);
    }
    #[test]
    fn average_ns_empty_zero() {
        assert!(gts_average_ns(&new_gpu_timestamp_manager()).abs() < 1e-9);
    }
    #[test]
    fn max_ns_correct() {
        let mut m = new_gpu_timestamp_manager();
        gts_record(&mut m, "a", 0, 100);
        gts_record(&mut m, "b", 0, 300);
        assert_eq!(gts_max_ns(&m), 300);
    }
    #[test]
    fn slowest_pass_correct() {
        let mut m = new_gpu_timestamp_manager();
        gts_record(&mut m, "fast", 0, 50);
        gts_record(&mut m, "slow", 0, 500);
        assert_eq!(gts_slowest_pass(&m), Some("slow"));
    }
    #[test]
    fn time_angle_nonneg() {
        assert!(gts_time_angle_rad(&new_gpu_timestamp_manager()) >= 0.0);
    }
    #[test]
    fn to_json_has_count() {
        assert!(gts_to_json(&new_gpu_timestamp_manager()).contains("\"count\""));
    }
}
