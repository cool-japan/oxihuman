// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Telemetry frame export with channel metadata.

/// A telemetry channel definition.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TelemetryChannel {
    pub id: u32,
    pub name: String,
    pub unit: String,
    pub data_type: String,
}

/// A single telemetry frame (one timestamp, multiple channel values).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TelemetryFrame {
    pub timestamp_us: u64,
    pub values: Vec<f64>,
}

/// A telemetry export session.
#[allow(dead_code)]
pub struct TelemetryExport {
    pub session_id: String,
    pub channels: Vec<TelemetryChannel>,
    pub frames: Vec<TelemetryFrame>,
}

impl TelemetryExport {
    #[allow(dead_code)]
    pub fn new(session_id: &str) -> Self {
        Self {
            session_id: session_id.to_string(),
            channels: Vec::new(),
            frames: Vec::new(),
        }
    }
}

/// Add a channel definition.
#[allow(dead_code)]
pub fn add_telemetry_channel(
    export: &mut TelemetryExport,
    name: &str,
    unit: &str,
    data_type: &str,
) -> u32 {
    let id = export.channels.len() as u32;
    export.channels.push(TelemetryChannel {
        id,
        name: name.to_string(),
        unit: unit.to_string(),
        data_type: data_type.to_string(),
    });
    id
}

/// Record a telemetry frame.
#[allow(dead_code)]
pub fn record_frame(export: &mut TelemetryExport, timestamp_us: u64, values: Vec<f64>) {
    export.frames.push(TelemetryFrame {
        timestamp_us,
        values,
    });
}

/// Export to CSV string.
#[allow(dead_code)]
pub fn export_telemetry_csv(export: &TelemetryExport) -> String {
    let mut out = String::new();
    out.push_str("timestamp_us");
    for ch in &export.channels {
        out.push_str(&format!(",{}", ch.name));
    }
    out.push('\n');
    for frame in &export.frames {
        out.push_str(&frame.timestamp_us.to_string());
        for v in &frame.values {
            out.push_str(&format!(",{v}"));
        }
        out.push('\n');
    }
    out
}

/// Export metadata as JSON-like string.
#[allow(dead_code)]
pub fn export_telemetry_meta(export: &TelemetryExport) -> String {
    let mut out = format!("{{\"session_id\":\"{}\",\"channels\":[", export.session_id);
    for (i, ch) in export.channels.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "{{\"id\":{},\"name\":\"{}\",\"unit\":\"{}\",\"type\":\"{}\"}}",
            ch.id, ch.name, ch.unit, ch.data_type
        ));
    }
    out.push_str("]}");
    out
}

/// Frame count.
#[allow(dead_code)]
pub fn frame_count_tl(export: &TelemetryExport) -> usize {
    export.frames.len()
}

/// Channel count.
#[allow(dead_code)]
pub fn channel_count_tl(export: &TelemetryExport) -> usize {
    export.channels.len()
}

/// Duration in microseconds.
#[allow(dead_code)]
pub fn session_duration_us(export: &TelemetryExport) -> u64 {
    if export.frames.is_empty() {
        return 0;
    }
    let min_t = export
        .frames
        .iter()
        .map(|f| f.timestamp_us)
        .min()
        .unwrap_or(0);
    let max_t = export
        .frames
        .iter()
        .map(|f| f.timestamp_us)
        .max()
        .unwrap_or(0);
    max_t.saturating_sub(min_t)
}

/// Average value of a channel across all frames.
#[allow(dead_code)]
pub fn channel_average(export: &TelemetryExport, channel_id: usize) -> f64 {
    let values: Vec<f64> = export
        .frames
        .iter()
        .filter_map(|f| f.values.get(channel_id).copied())
        .collect();
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

/// Find channel by name.
#[allow(dead_code)]
pub fn find_channel_by_name<'a>(
    export: &'a TelemetryExport,
    name: &str,
) -> Option<&'a TelemetryChannel> {
    export.channels.iter().find(|c| c.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_export() -> TelemetryExport {
        let mut e = TelemetryExport::new("test_session");
        add_telemetry_channel(&mut e, "velocity", "m/s", "f64");
        add_telemetry_channel(&mut e, "rpm", "rpm", "f64");
        record_frame(&mut e, 0, vec![1.0, 3000.0]);
        record_frame(&mut e, 1000, vec![2.0, 3200.0]);
        record_frame(&mut e, 2000, vec![1.5, 3100.0]);
        e
    }

    #[test]
    fn channel_count_correct() {
        let e = sample_export();
        assert_eq!(channel_count_tl(&e), 2);
    }

    #[test]
    fn frame_count_correct() {
        let e = sample_export();
        assert_eq!(frame_count_tl(&e), 3);
    }

    #[test]
    fn session_duration_2000us() {
        let e = sample_export();
        assert_eq!(session_duration_us(&e), 2000);
    }

    #[test]
    fn channel_average_velocity() {
        let e = sample_export();
        let avg = channel_average(&e, 0);
        assert!((avg - 1.5).abs() < 1e-5);
    }

    #[test]
    fn csv_has_header() {
        let e = sample_export();
        let csv = export_telemetry_csv(&e);
        assert!(csv.starts_with("timestamp_us,velocity,rpm"));
    }

    #[test]
    fn csv_line_count() {
        let e = sample_export();
        let csv = export_telemetry_csv(&e);
        let lines: Vec<&str> = csv.trim().split('\n').collect();
        assert_eq!(lines.len(), 4);
    }

    #[test]
    fn meta_contains_session_id() {
        let e = sample_export();
        let meta = export_telemetry_meta(&e);
        assert!(meta.contains("test_session"));
    }

    #[test]
    fn find_channel_by_name_some() {
        let e = sample_export();
        let ch = find_channel_by_name(&e, "velocity");
        assert!(ch.is_some());
    }

    #[test]
    fn find_channel_by_name_none() {
        let e = sample_export();
        let ch = find_channel_by_name(&e, "nonexistent");
        assert!(ch.is_none());
    }

    #[test]
    fn empty_session_duration_zero() {
        let e = TelemetryExport::new("empty");
        assert_eq!(session_duration_us(&e), 0);
    }
}
