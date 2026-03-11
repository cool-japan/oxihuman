//! Frame-time and CPU/GPU performance monitoring utility.

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct PerfConfig {
    pub sample_count: usize,
    pub target_fps: f32,
    pub warn_threshold_ms: f32,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct PerfSample {
    pub frame_ms: f32,
    pub cpu_ms: f32,
    pub gpu_ms: f32,
    pub frame_index: u64,
}

#[allow(dead_code)]
pub struct PerfMonitor {
    pub config: PerfConfig,
    pub samples: Vec<PerfSample>,
    pub frame_index: u64,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct PerfReport {
    pub avg_frame_ms: f32,
    pub min_frame_ms: f32,
    pub max_frame_ms: f32,
    pub fps: f32,
    pub sample_count: usize,
    pub over_budget_count: usize,
}

#[allow(dead_code)]
pub fn default_perf_config() -> PerfConfig {
    PerfConfig {
        sample_count: 60,
        target_fps: 60.0,
        warn_threshold_ms: 16.667,
    }
}

#[allow(dead_code)]
pub fn new_perf_monitor(cfg: PerfConfig) -> PerfMonitor {
    PerfMonitor {
        config: cfg,
        samples: Vec::new(),
        frame_index: 0,
    }
}

#[allow(dead_code)]
pub fn record_frame(mon: &mut PerfMonitor, frame_ms: f32, cpu_ms: f32, gpu_ms: f32) {
    mon.frame_index += 1;
    let sample = PerfSample {
        frame_ms,
        cpu_ms,
        gpu_ms,
        frame_index: mon.frame_index,
    };
    mon.samples.push(sample);
    let max = mon.config.sample_count;
    if max > 0 {
        while mon.samples.len() > max {
            mon.samples.remove(0);
        }
    }
}

#[allow(dead_code)]
pub fn generate_report(mon: &PerfMonitor) -> PerfReport {
    if mon.samples.is_empty() {
        return PerfReport {
            avg_frame_ms: 0.0,
            min_frame_ms: 0.0,
            max_frame_ms: 0.0,
            fps: 0.0,
            sample_count: 0,
            over_budget_count: 0,
        };
    }
    let count = mon.samples.len();
    let sum: f32 = mon.samples.iter().map(|s| s.frame_ms).sum();
    let min = mon
        .samples
        .iter()
        .map(|s| s.frame_ms)
        .fold(f32::INFINITY, f32::min);
    let max = mon
        .samples
        .iter()
        .map(|s| s.frame_ms)
        .fold(f32::NEG_INFINITY, f32::max);
    let avg = sum / count as f32;
    let fps = if avg > 0.0 { 1000.0 / avg } else { 0.0 };
    let threshold = mon.config.warn_threshold_ms;
    let over_budget = mon
        .samples
        .iter()
        .filter(|s| s.frame_ms > threshold)
        .count();
    PerfReport {
        avg_frame_ms: avg,
        min_frame_ms: min,
        max_frame_ms: max,
        fps,
        sample_count: count,
        over_budget_count: over_budget,
    }
}

#[allow(dead_code)]
pub fn current_fps(mon: &PerfMonitor) -> f32 {
    let avg = average_frame_time(mon);
    if avg > 0.0 {
        1000.0 / avg
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn average_frame_time(mon: &PerfMonitor) -> f32 {
    if mon.samples.is_empty() {
        return 0.0;
    }
    let sum: f32 = mon.samples.iter().map(|s| s.frame_ms).sum();
    sum / mon.samples.len() as f32
}

#[allow(dead_code)]
pub fn is_over_budget(mon: &PerfMonitor) -> bool {
    let avg = average_frame_time(mon);
    avg > mon.config.warn_threshold_ms
}

#[allow(dead_code)]
pub fn clear_samples(mon: &mut PerfMonitor) {
    mon.samples.clear();
}

#[allow(dead_code)]
pub fn sample_count_perf(mon: &PerfMonitor) -> usize {
    mon.samples.len()
}

#[allow(dead_code)]
pub fn perf_report_to_json(r: &PerfReport) -> String {
    format!(
        "{{\"avg_frame_ms\":{},\"min_frame_ms\":{},\"max_frame_ms\":{},\"fps\":{},\"sample_count\":{},\"over_budget_count\":{}}}",
        r.avg_frame_ms,
        r.min_frame_ms,
        r.max_frame_ms,
        r.fps,
        r.sample_count,
        r.over_budget_count,
    )
}

#[allow(dead_code)]
pub fn perf_monitor_to_json(mon: &PerfMonitor) -> String {
    let samples_json: Vec<String> = mon
        .samples
        .iter()
        .map(|s| {
            format!(
                "{{\"frame_ms\":{},\"cpu_ms\":{},\"gpu_ms\":{},\"frame_index\":{}}}",
                s.frame_ms, s.cpu_ms, s.gpu_ms, s.frame_index
            )
        })
        .collect();
    format!(
        "{{\"frame_index\":{},\"sample_count\":{},\"samples\":[{}]}}",
        mon.frame_index,
        mon.samples.len(),
        samples_json.join(",")
    )
}

#[allow(dead_code)]
pub fn worst_frame(mon: &PerfMonitor) -> Option<&PerfSample> {
    mon.samples.iter().max_by(|a, b| {
        a.frame_ms
            .partial_cmp(&b.frame_ms)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_perf_config() {
        let cfg = default_perf_config();
        assert_eq!(cfg.sample_count, 60);
        assert!((cfg.target_fps - 60.0).abs() < 1e-4);
    }

    #[test]
    fn test_new_perf_monitor_empty() {
        let cfg = default_perf_config();
        let mon = new_perf_monitor(cfg);
        assert!(mon.samples.is_empty());
        assert_eq!(mon.frame_index, 0);
    }

    #[test]
    fn test_record_frame_increments_index() {
        let cfg = default_perf_config();
        let mut mon = new_perf_monitor(cfg);
        record_frame(&mut mon, 16.0, 10.0, 6.0);
        assert_eq!(mon.frame_index, 1);
        assert_eq!(mon.samples.len(), 1);
    }

    #[test]
    fn test_sample_count_perf() {
        let cfg = default_perf_config();
        let mut mon = new_perf_monitor(cfg);
        record_frame(&mut mon, 10.0, 5.0, 5.0);
        record_frame(&mut mon, 12.0, 6.0, 6.0);
        assert_eq!(sample_count_perf(&mon), 2);
    }

    #[test]
    fn test_average_frame_time() {
        let cfg = default_perf_config();
        let mut mon = new_perf_monitor(cfg);
        record_frame(&mut mon, 10.0, 5.0, 5.0);
        record_frame(&mut mon, 20.0, 10.0, 10.0);
        let avg = average_frame_time(&mon);
        assert!((avg - 15.0).abs() < 1e-4);
    }

    #[test]
    fn test_current_fps() {
        let cfg = default_perf_config();
        let mut mon = new_perf_monitor(cfg);
        record_frame(&mut mon, 16.667, 8.0, 8.667);
        let fps = current_fps(&mon);
        assert!(fps > 59.0 && fps < 61.0);
    }

    #[test]
    fn test_is_over_budget_true() {
        let cfg = default_perf_config();
        let mut mon = new_perf_monitor(cfg);
        record_frame(&mut mon, 33.0, 20.0, 13.0);
        assert!(is_over_budget(&mon));
    }

    #[test]
    fn test_is_over_budget_false() {
        let cfg = default_perf_config();
        let mut mon = new_perf_monitor(cfg);
        record_frame(&mut mon, 10.0, 5.0, 5.0);
        assert!(!is_over_budget(&mon));
    }

    #[test]
    fn test_clear_samples() {
        let cfg = default_perf_config();
        let mut mon = new_perf_monitor(cfg);
        record_frame(&mut mon, 10.0, 5.0, 5.0);
        clear_samples(&mut mon);
        assert!(mon.samples.is_empty());
    }

    #[test]
    fn test_generate_report_empty() {
        let cfg = default_perf_config();
        let mon = new_perf_monitor(cfg);
        let report = generate_report(&mon);
        assert_eq!(report.sample_count, 0);
        assert!((report.fps).abs() < 1e-6);
    }

    #[test]
    fn test_generate_report_over_budget_count() {
        let cfg = default_perf_config();
        let mut mon = new_perf_monitor(cfg);
        record_frame(&mut mon, 10.0, 5.0, 5.0);
        record_frame(&mut mon, 33.0, 20.0, 13.0);
        record_frame(&mut mon, 40.0, 25.0, 15.0);
        let report = generate_report(&mon);
        assert_eq!(report.over_budget_count, 2);
    }

    #[test]
    fn test_worst_frame() {
        let cfg = default_perf_config();
        let mut mon = new_perf_monitor(cfg);
        record_frame(&mut mon, 10.0, 5.0, 5.0);
        record_frame(&mut mon, 50.0, 30.0, 20.0);
        record_frame(&mut mon, 20.0, 10.0, 10.0);
        let worst = worst_frame(&mon).unwrap();
        assert!((worst.frame_ms - 50.0).abs() < 1e-4);
    }

    #[test]
    fn test_worst_frame_empty() {
        let cfg = default_perf_config();
        let mon = new_perf_monitor(cfg);
        assert!(worst_frame(&mon).is_none());
    }

    #[test]
    fn test_perf_report_to_json() {
        let cfg = default_perf_config();
        let mut mon = new_perf_monitor(cfg);
        record_frame(&mut mon, 16.0, 8.0, 8.0);
        let report = generate_report(&mon);
        let json = perf_report_to_json(&report);
        assert!(json.contains("avg_frame_ms"));
        assert!(json.contains("fps"));
    }

    #[test]
    fn test_perf_monitor_to_json() {
        let cfg = default_perf_config();
        let mut mon = new_perf_monitor(cfg);
        record_frame(&mut mon, 16.0, 8.0, 8.0);
        let json = perf_monitor_to_json(&mon);
        assert!(json.contains("frame_index"));
        assert!(json.contains("sample_count"));
    }

    #[test]
    fn test_sample_count_capped() {
        let cfg = PerfConfig {
            sample_count: 3,
            target_fps: 60.0,
            warn_threshold_ms: 16.667,
        };
        let mut mon = new_perf_monitor(cfg);
        for i in 0..6 {
            record_frame(&mut mon, i as f32 * 2.0, 1.0, 1.0);
        }
        assert!(mon.samples.len() <= 3);
    }
}
