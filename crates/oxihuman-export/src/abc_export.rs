// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Alembic (.abc) stub export for VFX pipeline integration.
//!
//! This module provides a lightweight, dependency-free representation of
//! Alembic data suitable for round-tripping through JSON or binary stubs.

// ── Types ────────────────────────────────────────────────────────────────────

/// Configuration for an Alembic export operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AbcExportConfig {
    /// Frames per second for the time-sampling.
    pub fps: f32,
    /// First frame number.
    pub start_frame: i32,
    /// Last frame number (inclusive).
    pub end_frame: i32,
    /// Whether to include normals in each sample.
    pub include_normals: bool,
    /// Whether to include UV coordinates in each sample.
    pub include_uvs: bool,
}

/// Time-sampling descriptor for an Alembic archive.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct AbcTimeSampling {
    /// Frames per second.
    pub fps: f32,
    /// Time of the first sample in seconds.
    pub start_time: f32,
    /// Time of the last sample in seconds.
    pub end_time: f32,
}

/// A single time-sample of Alembic mesh data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AbcSample {
    /// Time in seconds.
    pub time: f32,
    /// Vertex positions.
    pub positions: Vec<[f32; 3]>,
    /// Per-vertex normals (may be empty if not exported).
    pub normals: Vec<[f32; 3]>,
    /// Per-vertex UV coordinates (may be empty if not exported).
    pub uvs: Vec<[f32; 2]>,
    /// Face vertex counts (3 for triangles, 4 for quads, etc.).
    pub face_counts: Vec<u32>,
    /// Face vertex indices into the positions array.
    pub face_indices: Vec<u32>,
}

// ── Type aliases ─────────────────────────────────────────────────────────────

/// Result type returned by validation helpers.
pub type AbcValidationResult = Vec<String>;

// ── Config helpers ────────────────────────────────────────────────────────────

/// Create a default [`AbcExportConfig`] targeting 24 fps, frames 0–240.
#[allow(dead_code)]
pub fn default_abc_config() -> AbcExportConfig {
    AbcExportConfig {
        fps: 24.0,
        start_frame: 0,
        end_frame: 240,
        include_normals: true,
        include_uvs: true,
    }
}

/// Return the frames-per-second value from a config.
#[allow(dead_code)]
pub fn abc_fps(cfg: &AbcExportConfig) -> f32 {
    cfg.fps
}

/// Update the fps in a config, clamping to a minimum of 1.0.
#[allow(dead_code)]
pub fn set_abc_fps(cfg: &mut AbcExportConfig, fps: f32) {
    cfg.fps = fps.max(1.0);
}

/// Return `(start_frame, end_frame)` from a config.
#[allow(dead_code)]
pub fn abc_frame_range(cfg: &AbcExportConfig) -> (i32, i32) {
    (cfg.start_frame, cfg.end_frame)
}

// ── Sample construction ───────────────────────────────────────────────────────

/// Create a new empty [`AbcSample`] at the given time.
#[allow(dead_code)]
pub fn new_abc_sample(time: f32) -> AbcSample {
    AbcSample {
        time,
        positions: Vec::new(),
        normals: Vec::new(),
        uvs: Vec::new(),
        face_counts: Vec::new(),
        face_indices: Vec::new(),
    }
}

/// Append position data to an existing sample (replaces any existing positions).
#[allow(dead_code)]
pub fn add_position_sample(sample: &mut AbcSample, positions: &[[f32; 3]]) {
    sample.positions = positions.to_vec();
}

/// Append normal data to an existing sample.
#[allow(dead_code)]
pub fn add_normal_sample(sample: &mut AbcSample, normals: &[[f32; 3]]) {
    sample.normals = normals.to_vec();
}

/// Append UV data to an existing sample.
#[allow(dead_code)]
pub fn add_uv_sample(sample: &mut AbcSample, uvs: &[[f32; 2]]) {
    sample.uvs = uvs.to_vec();
}

// ── Sample queries ─────────────────────────────────────────────────────────────

/// Return the number of samples in a slice.
#[allow(dead_code)]
pub fn sample_count(samples: &[AbcSample]) -> usize {
    samples.len()
}

/// Return `(min_time, max_time)` over a slice of samples, or `(0.0, 0.0)` if empty.
#[allow(dead_code)]
pub fn time_range(samples: &[AbcSample]) -> (f32, f32) {
    if samples.is_empty() {
        return (0.0, 0.0);
    }
    let min = samples.iter().map(|s| s.time).fold(f32::INFINITY, f32::min);
    let max = samples
        .iter()
        .map(|s| s.time)
        .fold(f32::NEG_INFINITY, f32::max);
    (min, max)
}

/// Find the sample whose time is closest to `t`.
///
/// Returns `None` if the slice is empty.
#[allow(dead_code)]
pub fn sample_at_time(samples: &[AbcSample], t: f32) -> Option<&AbcSample> {
    samples
        .iter()
        .min_by(|a, b| (a.time - t).abs().partial_cmp(&(b.time - t).abs()).unwrap_or(std::cmp::Ordering::Equal))
}

/// Approximate byte size of a single sample (positions + normals + uvs + indices).
#[allow(dead_code)]
pub fn abc_sample_size_bytes(sample: &AbcSample) -> usize {
    sample.positions.len() * 12
        + sample.normals.len() * 12
        + sample.uvs.len() * 8
        + sample.face_counts.len() * 4
        + sample.face_indices.len() * 4
        + 4 // time f32
}

// ── Validation ────────────────────────────────────────────────────────────────

/// Validate a slice of [`AbcSample`]s and return a list of error strings.
///
/// Checks include:
/// - At least one sample present
/// - Times are monotonically non-decreasing
/// - face_indices count matches the sum of face_counts
#[allow(dead_code)]
pub fn validate_abc_samples(samples: &[AbcSample]) -> AbcValidationResult {
    let mut errors = Vec::new();
    if samples.is_empty() {
        errors.push("no samples provided".into());
        return errors;
    }
    let mut prev_time = f32::NEG_INFINITY;
    for (i, s) in samples.iter().enumerate() {
        if s.time < prev_time {
            errors.push(format!("sample {i}: time not monotonically increasing"));
        }
        prev_time = s.time;
        let expected_indices: usize = s.face_counts.iter().map(|&c| c as usize).sum();
        if !s.face_indices.is_empty() && expected_indices != s.face_indices.len() {
            errors.push(format!(
                "sample {i}: face_indices length {} != expected {expected_indices}",
                s.face_indices.len()
            ));
        }
        if !s.normals.is_empty() && s.normals.len() != s.positions.len() {
            errors.push(format!(
                "sample {i}: normals length {} != positions length {}",
                s.normals.len(),
                s.positions.len()
            ));
        }
        if !s.uvs.is_empty() && s.uvs.len() != s.positions.len() {
            errors.push(format!(
                "sample {i}: uvs length {} != positions length {}",
                s.uvs.len(),
                s.positions.len()
            ));
        }
    }
    errors
}

// ── JSON stub serialisation ───────────────────────────────────────────────────

/// Serialise a slice of samples to a compact JSON string (no external deps).
#[allow(dead_code)]
pub fn abc_to_json_stub(samples: &[AbcSample], cfg: &AbcExportConfig) -> String {
    let sample_jsons: Vec<String> = samples
        .iter()
        .map(|s| {
            let pos: Vec<String> = s
                .positions
                .iter()
                .map(|p| format!("[{:.6},{:.6},{:.6}]", p[0], p[1], p[2]))
                .collect();
            let nrm: Vec<String> = s
                .normals
                .iter()
                .map(|n| format!("[{:.6},{:.6},{:.6}]", n[0], n[1], n[2]))
                .collect();
            let uv: Vec<String> = s
                .uvs
                .iter()
                .map(|u| format!("[{:.6},{:.6}]", u[0], u[1]))
                .collect();
            let fc: Vec<String> = s.face_counts.iter().map(|c| c.to_string()).collect();
            let fi: Vec<String> = s.face_indices.iter().map(|i| i.to_string()).collect();
            format!(
                r#"{{"time":{:.6},"positions":[{}],"normals":[{}],"uvs":[{}],"face_counts":[{}],"face_indices":[{}]}}"#,
                s.time,
                pos.join(","),
                nrm.join(","),
                uv.join(","),
                fc.join(","),
                fi.join(","),
            )
        })
        .collect();
    format!(
        r#"{{"fps":{:.6},"start_frame":{},"end_frame":{},"samples":[{}]}}"#,
        cfg.fps,
        cfg.start_frame,
        cfg.end_frame,
        sample_jsons.join(","),
    )
}

// ── TimeSampling helpers ──────────────────────────────────────────────────────

/// Build an [`AbcTimeSampling`] from a config and a sample count.
#[allow(dead_code)]
pub fn time_sampling_from_config(cfg: &AbcExportConfig, n_samples: usize) -> AbcTimeSampling {
    let start = cfg.start_frame as f32 / cfg.fps.max(1.0);
    let end = if n_samples == 0 {
        start
    } else {
        cfg.start_frame as f32 / cfg.fps.max(1.0)
            + (n_samples.saturating_sub(1)) as f32 / cfg.fps.max(1.0)
    };
    AbcTimeSampling {
        fps: cfg.fps,
        start_time: start,
        end_time: end,
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sample(t: f32, n: usize) -> AbcSample {
        let positions: Vec<[f32; 3]> = (0..n).map(|i| [i as f32, 0.0, 0.0]).collect();
        let face_counts = if n >= 3 { vec![3u32] } else { vec![] };
        let face_indices = if n >= 3 { vec![0u32, 1, 2] } else { vec![] };
        AbcSample {
            time: t,
            positions,
            normals: Vec::new(),
            uvs: Vec::new(),
            face_counts,
            face_indices,
        }
    }

    #[test]
    fn default_config_fps_is_24() {
        let cfg = default_abc_config();
        assert!((cfg.fps - 24.0).abs() < 1e-6);
    }

    #[test]
    fn default_config_frame_range() {
        let cfg = default_abc_config();
        assert_eq!(cfg.start_frame, 0);
        assert_eq!(cfg.end_frame, 240);
    }

    #[test]
    fn abc_fps_returns_fps() {
        let cfg = default_abc_config();
        assert!((abc_fps(&cfg) - 24.0).abs() < 1e-6);
    }

    #[test]
    fn set_abc_fps_updates() {
        let mut cfg = default_abc_config();
        set_abc_fps(&mut cfg, 30.0);
        assert!((cfg.fps - 30.0).abs() < 1e-6);
    }

    #[test]
    fn set_abc_fps_clamps_to_one() {
        let mut cfg = default_abc_config();
        set_abc_fps(&mut cfg, 0.0);
        assert!(cfg.fps >= 1.0);
    }

    #[test]
    fn abc_frame_range_returns_pair() {
        let cfg = default_abc_config();
        assert_eq!(abc_frame_range(&cfg), (0, 240));
    }

    #[test]
    fn new_abc_sample_empty() {
        let s = new_abc_sample(1.0);
        assert!((s.time - 1.0).abs() < 1e-6);
        assert!(s.positions.is_empty());
        assert!(s.normals.is_empty());
        assert!(s.uvs.is_empty());
    }

    #[test]
    fn add_position_sample_sets_positions() {
        let mut s = new_abc_sample(0.0);
        let pts = [[1.0f32, 2.0, 3.0], [4.0, 5.0, 6.0]];
        add_position_sample(&mut s, &pts);
        assert_eq!(s.positions.len(), 2);
        assert_eq!(s.positions[0], [1.0, 2.0, 3.0]);
    }

    #[test]
    fn add_normal_sample_sets_normals() {
        let mut s = new_abc_sample(0.0);
        let nrm = [[0.0f32, 1.0, 0.0]];
        add_normal_sample(&mut s, &nrm);
        assert_eq!(s.normals.len(), 1);
    }

    #[test]
    fn add_uv_sample_sets_uvs() {
        let mut s = new_abc_sample(0.0);
        let uvs = [[0.5f32, 0.5]];
        add_uv_sample(&mut s, &uvs);
        assert_eq!(s.uvs.len(), 1);
    }

    #[test]
    fn sample_count_empty() {
        assert_eq!(sample_count(&[]), 0);
    }

    #[test]
    fn sample_count_nonempty() {
        let samples = vec![make_sample(0.0, 3), make_sample(1.0, 3)];
        assert_eq!(sample_count(&samples), 2);
    }

    #[test]
    fn time_range_empty_returns_zero() {
        assert_eq!(time_range(&[]), (0.0, 0.0));
    }

    #[test]
    fn time_range_correct() {
        let samples = vec![make_sample(0.0, 3), make_sample(0.5, 3), make_sample(1.0, 3)];
        let (mn, mx) = time_range(&samples);
        assert!((mn - 0.0).abs() < 1e-6);
        assert!((mx - 1.0).abs() < 1e-6);
    }

    #[test]
    fn sample_at_time_finds_closest() {
        let samples = vec![make_sample(0.0, 3), make_sample(1.0, 3), make_sample(2.0, 3)];
        let s = sample_at_time(&samples, 0.9).expect("should succeed");
        assert!((s.time - 1.0).abs() < 1e-6);
    }

    #[test]
    fn sample_at_time_empty_returns_none() {
        assert!(sample_at_time(&[], 0.5).is_none());
    }

    #[test]
    fn abc_sample_size_bytes_grows_with_data() {
        let s1 = make_sample(0.0, 3);
        let s2 = make_sample(0.0, 6);
        assert!(abc_sample_size_bytes(&s2) > abc_sample_size_bytes(&s1));
    }

    #[test]
    fn validate_abc_samples_valid() {
        let samples = vec![make_sample(0.0, 3), make_sample(1.0, 3)];
        let errs = validate_abc_samples(&samples);
        assert!(errs.is_empty(), "unexpected errors: {errs:?}");
    }

    #[test]
    fn validate_abc_samples_empty_errors() {
        let errs = validate_abc_samples(&[]);
        assert!(!errs.is_empty());
    }

    #[test]
    fn validate_abc_samples_non_monotonic_errors() {
        let samples = vec![make_sample(1.0, 3), make_sample(0.0, 3)];
        let errs = validate_abc_samples(&samples);
        assert!(!errs.is_empty());
    }

    #[test]
    fn abc_to_json_stub_contains_fps() {
        let samples = vec![make_sample(0.0, 3)];
        let cfg = default_abc_config();
        let json = abc_to_json_stub(&samples, &cfg);
        assert!(json.contains("fps"));
        assert!(json.contains("samples"));
    }

    #[test]
    fn time_sampling_from_config_fps_matches() {
        let cfg = default_abc_config();
        let ts = time_sampling_from_config(&cfg, 10);
        assert!((ts.fps - 24.0).abs() < 1e-6);
    }
}
