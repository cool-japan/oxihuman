// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export audio-sync metadata (phoneme timings, beat markers) as JSON.

#![allow(dead_code)]

/// Configuration for audio-sync export.
#[derive(Debug, Clone)]
pub struct AudioSyncExportConfig {
    /// Pretty-print JSON output.
    pub pretty: bool,
    /// Sample rate in Hz for the associated audio clip.
    pub sample_rate: u32,
    /// Human-readable name for the audio clip.
    pub clip_name: String,
}

/// A single synchronisation marker (phoneme, beat, cue, etc.).
#[derive(Debug, Clone)]
pub struct SyncMarker {
    /// Marker type tag (e.g. "phoneme", "beat", "cue").
    pub marker_type: String,
    /// Start time in seconds.
    pub time_s: f64,
    /// Duration in seconds (0 for instantaneous markers).
    pub duration_s: f64,
    /// Payload label (e.g. phoneme symbol "AA", beat label "1").
    pub label: String,
}

/// Result container for an audio-sync export session.
#[derive(Debug, Clone)]
pub struct AudioSyncExportResult {
    /// All recorded sync markers, ordered by insertion.
    pub markers: Vec<SyncMarker>,
    /// Total duration inferred from the latest marker end time.
    pub duration_s: f64,
    /// Total byte size of the last serialised output.
    pub total_bytes: usize,
}

/// Returns the default [`AudioSyncExportConfig`].
pub fn default_audio_sync_config() -> AudioSyncExportConfig {
    AudioSyncExportConfig {
        pretty: true,
        sample_rate: 44100,
        clip_name: "untitled".to_string(),
    }
}

/// Creates a new, empty [`AudioSyncExportResult`].
pub fn new_audio_sync_export() -> AudioSyncExportResult {
    AudioSyncExportResult {
        markers: Vec::new(),
        duration_s: 0.0,
        total_bytes: 0,
    }
}

/// Adds a sync marker and updates the tracked duration.
pub fn audio_sync_add_marker(result: &mut AudioSyncExportResult, marker: SyncMarker) {
    let end = marker.time_s + marker.duration_s;
    if end > result.duration_s {
        result.duration_s = end;
    }
    result.markers.push(marker);
}

/// Serialises the marker list as JSON.
pub fn audio_sync_to_json(
    result: &mut AudioSyncExportResult,
    cfg: &AudioSyncExportConfig,
) -> String {
    let indent = if cfg.pretty { "  " } else { "" };
    let nl = if cfg.pretty { "\n" } else { "" };
    let mut out = format!(
        "{{{nl}{indent}\"clip\":\"{}\",{nl}{indent}\"sample_rate\":{},{nl}{indent}\"duration_s\":{:.6},{nl}{indent}\"markers\":[{nl}",
        cfg.clip_name, cfg.sample_rate, result.duration_s
    );
    for (i, m) in result.markers.iter().enumerate() {
        let comma = if i + 1 < result.markers.len() { "," } else { "" };
        out.push_str(&format!(
            "{indent}{indent}{{\"type\":\"{}\",\"time_s\":{:.6},\"duration_s\":{:.6},\"label\":\"{}\"}}{}{nl}",
            m.marker_type, m.time_s, m.duration_s, m.label, comma
        ));
    }
    out.push_str(&format!("{indent}]{nl}}}"));
    result.total_bytes = out.len();
    out
}

/// Returns the number of markers currently recorded.
pub fn audio_sync_marker_count(result: &AudioSyncExportResult) -> usize {
    result.markers.len()
}

/// Returns the total duration inferred from marker extents.
pub fn audio_sync_duration(result: &AudioSyncExportResult) -> f64 {
    result.duration_s
}

/// Writes JSON to a file path (stub — returns byte count).
pub fn audio_sync_write_to_file(
    result: &mut AudioSyncExportResult,
    cfg: &AudioSyncExportConfig,
    _path: &str,
) -> usize {
    let json = audio_sync_to_json(result, cfg);
    result.total_bytes = json.len();
    result.total_bytes
}

/// Clears all markers and resets state.
pub fn audio_sync_clear(result: &mut AudioSyncExportResult) {
    result.markers.clear();
    result.duration_s = 0.0;
    result.total_bytes = 0;
}

/// Returns the byte count of the last serialised output.
pub fn audio_sync_total_bytes(result: &AudioSyncExportResult) -> usize {
    result.total_bytes
}

/// Returns the number of markers with the given `marker_type`.
pub fn audio_sync_marker_type_count(result: &AudioSyncExportResult, marker_type: &str) -> usize {
    result
        .markers
        .iter()
        .filter(|m| m.marker_type == marker_type)
        .count()
}

// ── internal helpers ───────────────────────────────────────────────────────────

fn make_marker(marker_type: &str, time_s: f64, duration_s: f64, label: &str) -> SyncMarker {
    SyncMarker {
        marker_type: marker_type.to_string(),
        time_s,
        duration_s,
        label: label.to_string(),
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_audio_sync_config();
        assert!(cfg.pretty);
        assert_eq!(cfg.sample_rate, 44100);
        assert_eq!(cfg.clip_name, "untitled");
    }

    #[test]
    fn new_export_is_empty() {
        let r = new_audio_sync_export();
        assert_eq!(audio_sync_marker_count(&r), 0);
        assert_eq!(audio_sync_total_bytes(&r), 0);
    }

    #[test]
    fn add_marker_increments_count() {
        let mut r = new_audio_sync_export();
        audio_sync_add_marker(&mut r, make_marker("beat", 0.5, 0.0, "1"));
        assert_eq!(audio_sync_marker_count(&r), 1);
    }

    #[test]
    fn duration_tracks_latest_end() {
        let mut r = new_audio_sync_export();
        audio_sync_add_marker(&mut r, make_marker("phoneme", 1.0, 0.3, "AA"));
        audio_sync_add_marker(&mut r, make_marker("phoneme", 2.0, 0.5, "IH"));
        let dur = audio_sync_duration(&r);
        assert!((dur - 2.5).abs() < 1e-9);
    }

    #[test]
    fn json_contains_clip_and_markers() {
        let mut r = new_audio_sync_export();
        audio_sync_add_marker(&mut r, make_marker("beat", 0.0, 0.0, "1"));
        let cfg = AudioSyncExportConfig {
            pretty: true,
            sample_rate: 48000,
            clip_name: "speech01".to_string(),
        };
        let json = audio_sync_to_json(&mut r, &cfg);
        assert!(json.contains("\"clip\":\"speech01\""));
        assert!(json.contains("\"markers\""));
        assert!(json.contains("beat"));
    }

    #[test]
    fn marker_type_count_filters() {
        let mut r = new_audio_sync_export();
        audio_sync_add_marker(&mut r, make_marker("beat", 0.0, 0.0, "1"));
        audio_sync_add_marker(&mut r, make_marker("phoneme", 0.5, 0.1, "AA"));
        audio_sync_add_marker(&mut r, make_marker("beat", 1.0, 0.0, "2"));
        assert_eq!(audio_sync_marker_type_count(&r, "beat"), 2);
        assert_eq!(audio_sync_marker_type_count(&r, "phoneme"), 1);
        assert_eq!(audio_sync_marker_type_count(&r, "cue"), 0);
    }

    #[test]
    fn write_to_file_sets_total_bytes() {
        let mut r = new_audio_sync_export();
        audio_sync_add_marker(&mut r, make_marker("beat", 0.0, 0.0, "1"));
        let cfg = default_audio_sync_config();
        let n = audio_sync_write_to_file(&mut r, &cfg, "/tmp/sync.json");
        assert!(n > 0);
        assert_eq!(audio_sync_total_bytes(&r), n);
    }

    #[test]
    fn clear_resets_state() {
        let mut r = new_audio_sync_export();
        audio_sync_add_marker(&mut r, make_marker("beat", 0.0, 0.0, "1"));
        let cfg = default_audio_sync_config();
        audio_sync_write_to_file(&mut r, &cfg, "/tmp/sync.json");
        audio_sync_clear(&mut r);
        assert_eq!(audio_sync_marker_count(&r), 0);
        assert_eq!(audio_sync_total_bytes(&r), 0);
        assert!((audio_sync_duration(&r)).abs() < 1e-12);
    }

    #[test]
    fn sample_rate_appears_in_json() {
        let mut r = new_audio_sync_export();
        let cfg = AudioSyncExportConfig {
            pretty: false,
            sample_rate: 22050,
            clip_name: "lo_fi".to_string(),
        };
        let json = audio_sync_to_json(&mut r, &cfg);
        assert!(json.contains("22050"));
    }
}
