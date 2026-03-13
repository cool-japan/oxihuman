#![allow(dead_code)]
//! Export keyframe animation data.

/// Keyframe export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct KeyframeExport {
    pub times: Vec<f32>,
    pub values: Vec<f32>,
    pub interpolation: String,
}

/// Export keyframes.
#[allow(dead_code)]
pub fn export_keyframes(times: &[f32], values: &[f32], interpolation: &str) -> KeyframeExport {
    KeyframeExport {
        times: times.to_vec(),
        values: values.to_vec(),
        interpolation: interpolation.to_string(),
    }
}

/// Return the keyframe count.
#[allow(dead_code)]
pub fn keyframe_count(export: &KeyframeExport) -> usize {
    export.times.len()
}

/// Get the time at a given keyframe index.
#[allow(dead_code)]
pub fn keyframe_time_at(export: &KeyframeExport, index: usize) -> Option<f32> {
    export.times.get(index).copied()
}

/// Get the value at a given keyframe index.
#[allow(dead_code)]
pub fn keyframe_value_at(export: &KeyframeExport, index: usize) -> Option<f32> {
    export.values.get(index).copied()
}

/// Get the interpolation type.
#[allow(dead_code)]
pub fn keyframe_interpolation(export: &KeyframeExport) -> &str {
    &export.interpolation
}

/// Convert keyframes to JSON.
#[allow(dead_code)]
pub fn keyframe_to_json(export: &KeyframeExport) -> String {
    let times_str: Vec<String> = export.times.iter().map(|t| format!("{:.4}", t)).collect();
    let vals_str: Vec<String> = export.values.iter().map(|v| format!("{:.4}", v)).collect();
    format!(
        "{{\"keyframe_count\":{},\"times\":[{}],\"values\":[{}],\"interpolation\":\"{}\"}}",
        export.times.len(),
        times_str.join(","),
        vals_str.join(","),
        export.interpolation
    )
}

/// Compute total duration (last time - first time).
#[allow(dead_code)]
pub fn keyframe_duration(export: &KeyframeExport) -> f32 {
    if export.times.len() < 2 {
        return 0.0;
    }
    export.times.last().unwrap_or(&0.0) - export.times.first().unwrap_or(&0.0)
}

/// Validate keyframe data (times must be sorted, same count as values).
#[allow(dead_code)]
pub fn validate_keyframes(export: &KeyframeExport) -> bool {
    if export.times.len() != export.values.len() {
        return false;
    }
    for w in export.times.windows(2) {
        if w[1] < w[0] {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> KeyframeExport {
        export_keyframes(&[0.0, 0.5, 1.0], &[0.0, 0.5, 1.0], "LINEAR")
    }

    #[test]
    fn test_export_keyframes() {
        let kf = sample();
        assert_eq!(keyframe_count(&kf), 3);
    }

    #[test]
    fn test_keyframe_time_at() {
        let kf = sample();
        assert!((keyframe_time_at(&kf, 1).expect("should succeed") - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_keyframe_time_at_oob() {
        let kf = sample();
        assert!(keyframe_time_at(&kf, 10).is_none());
    }

    #[test]
    fn test_keyframe_value_at() {
        let kf = sample();
        assert!((keyframe_value_at(&kf, 2).expect("should succeed") - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_keyframe_interpolation() {
        let kf = sample();
        assert_eq!(keyframe_interpolation(&kf), "LINEAR");
    }

    #[test]
    fn test_keyframe_to_json() {
        let kf = sample();
        let j = keyframe_to_json(&kf);
        assert!(j.contains("keyframe_count"));
    }

    #[test]
    fn test_keyframe_duration() {
        let kf = sample();
        assert!((keyframe_duration(&kf) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_keyframe_duration_single() {
        let kf = export_keyframes(&[0.0], &[1.0], "STEP");
        assert!((keyframe_duration(&kf)).abs() < 1e-6);
    }

    #[test]
    fn test_validate_keyframes() {
        let kf = sample();
        assert!(validate_keyframes(&kf));
    }

    #[test]
    fn test_validate_keyframes_bad() {
        let kf = export_keyframes(&[1.0, 0.0], &[0.0, 1.0], "LINEAR");
        assert!(!validate_keyframes(&kf));
    }
}
