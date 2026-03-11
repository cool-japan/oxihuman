// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Timestamped sensor reading export (CSV-like format).

/// A single sensor reading.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SensorReading {
    pub timestamp_ms: u64,
    pub sensor_id: String,
    pub value: f64,
    pub unit: String,
}

/// A sensor log containing multiple readings.
#[allow(dead_code)]
pub struct SensorLog {
    pub readings: Vec<SensorReading>,
}

impl SensorLog {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            readings: Vec::new(),
        }
    }
}

impl Default for SensorLog {
    fn default() -> Self {
        Self::new()
    }
}

/// Add a reading to the log.
#[allow(dead_code)]
pub fn add_reading(
    log: &mut SensorLog,
    timestamp_ms: u64,
    sensor_id: &str,
    value: f64,
    unit: &str,
) {
    log.readings.push(SensorReading {
        timestamp_ms,
        sensor_id: sensor_id.to_string(),
        value,
        unit: unit.to_string(),
    });
}

/// Export sensor log to CSV string.
#[allow(dead_code)]
pub fn export_sensor_log_csv(log: &SensorLog) -> String {
    let mut out = String::from("timestamp_ms,sensor_id,value,unit\n");
    for r in &log.readings {
        out.push_str(&format!(
            "{},{},{},{}\n",
            r.timestamp_ms, r.sensor_id, r.value, r.unit
        ));
    }
    out
}

/// Reading count.
#[allow(dead_code)]
pub fn reading_count(log: &SensorLog) -> usize {
    log.readings.len()
}

/// Duration covered by the log (max - min timestamp).
#[allow(dead_code)]
pub fn log_duration_ms(log: &SensorLog) -> u64 {
    if log.readings.is_empty() {
        return 0;
    }
    let min_t = log
        .readings
        .iter()
        .map(|r| r.timestamp_ms)
        .min()
        .unwrap_or(0);
    let max_t = log
        .readings
        .iter()
        .map(|r| r.timestamp_ms)
        .max()
        .unwrap_or(0);
    max_t.saturating_sub(min_t)
}

/// Average value for a given sensor ID.
#[allow(dead_code)]
pub fn average_sensor_value(log: &SensorLog, sensor_id: &str) -> f64 {
    let values: Vec<f64> = log
        .readings
        .iter()
        .filter(|r| r.sensor_id == sensor_id)
        .map(|r| r.value)
        .collect();
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Peak (maximum absolute) value for a sensor.
#[allow(dead_code)]
pub fn peak_sensor_value(log: &SensorLog, sensor_id: &str) -> f64 {
    log.readings
        .iter()
        .filter(|r| r.sensor_id == sensor_id)
        .map(|r| r.value.abs())
        .fold(f64::NEG_INFINITY, f64::max)
}

/// List unique sensor IDs.
#[allow(dead_code)]
pub fn unique_sensor_ids(log: &SensorLog) -> Vec<String> {
    let mut ids: Vec<String> = log.readings.iter().map(|r| r.sensor_id.clone()).collect();
    ids.sort();
    ids.dedup();
    ids
}

/// Filter readings by sensor ID.
#[allow(dead_code)]
pub fn filter_by_sensor(log: &SensorLog, sensor_id: &str) -> Vec<SensorReading> {
    log.readings
        .iter()
        .filter(|r| r.sensor_id == sensor_id)
        .cloned()
        .collect()
}

/// Sort readings by timestamp.
#[allow(dead_code)]
pub fn sort_by_timestamp(log: &mut SensorLog) {
    log.readings.sort_by_key(|r| r.timestamp_ms);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_log() -> SensorLog {
        let mut log = SensorLog::new();
        add_reading(&mut log, 0, "temp", 20.0, "C");
        add_reading(&mut log, 100, "temp", 21.5, "C");
        add_reading(&mut log, 200, "pressure", 1013.0, "hPa");
        log
    }

    #[test]
    fn reading_count_correct() {
        let log = sample_log();
        assert_eq!(reading_count(&log), 3);
    }

    #[test]
    fn log_duration_200ms() {
        let log = sample_log();
        assert_eq!(log_duration_ms(&log), 200);
    }

    #[test]
    fn average_temp_correct() {
        let log = sample_log();
        let avg = average_sensor_value(&log, "temp");
        assert!((avg - 20.75).abs() < 1e-5);
    }

    #[test]
    fn csv_header_present() {
        let log = sample_log();
        let csv = export_sensor_log_csv(&log);
        assert!(csv.starts_with("timestamp_ms,sensor_id,value,unit"));
    }

    #[test]
    fn csv_line_count() {
        let log = sample_log();
        let csv = export_sensor_log_csv(&log);
        let lines: Vec<&str> = csv.trim().split('\n').collect();
        assert_eq!(lines.len(), 4);
    }

    #[test]
    fn unique_sensor_ids_correct() {
        let log = sample_log();
        let ids = unique_sensor_ids(&log);
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&String::from("pressure")));
    }

    #[test]
    fn filter_by_sensor_correct() {
        let log = sample_log();
        let temp_readings = filter_by_sensor(&log, "temp");
        assert_eq!(temp_readings.len(), 2);
    }

    #[test]
    fn peak_sensor_value_correct() {
        let log = sample_log();
        let peak = peak_sensor_value(&log, "temp");
        assert!((peak - 21.5).abs() < 1e-5);
    }

    #[test]
    fn sort_by_timestamp_ordered() {
        let mut log = SensorLog::new();
        add_reading(&mut log, 300, "x", 1.0, "m");
        add_reading(&mut log, 100, "x", 2.0, "m");
        add_reading(&mut log, 200, "x", 3.0, "m");
        sort_by_timestamp(&mut log);
        assert_eq!(log.readings[0].timestamp_ms, 100);
    }

    #[test]
    fn empty_log_duration_zero() {
        let log = SensorLog::new();
        assert_eq!(log_duration_ms(&log), 0);
    }
}
