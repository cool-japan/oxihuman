#![allow(dead_code)]
//! Export animation tracks.

/// Animation track export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct AnimTrackExport {
    pub name: String,
    pub channels: u32,
    pub samples: Vec<Vec<f32>>,
    pub duration: f32,
}

/// Export an animation track.
#[allow(dead_code)]
pub fn export_animation_track(
    name: &str,
    channels: u32,
    samples: &[Vec<f32>],
    duration: f32,
) -> AnimTrackExport {
    AnimTrackExport {
        name: name.to_string(),
        channels,
        samples: samples.to_vec(),
        duration,
    }
}

/// Return track name.
#[allow(dead_code)]
pub fn track_name(exp: &AnimTrackExport) -> &str {
    &exp.name
}

/// Return channel count.
#[allow(dead_code)]
pub fn track_channel_count(exp: &AnimTrackExport) -> u32 {
    exp.channels
}

/// Return sample count.
#[allow(dead_code)]
pub fn track_sample_count_export(exp: &AnimTrackExport) -> usize {
    exp.samples.len()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn track_to_json(exp: &AnimTrackExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"channels\":{},\"samples\":{},\"duration\":{:.4}}}",
        exp.name, exp.channels, exp.samples.len(), exp.duration
    )
}

/// Return duration.
#[allow(dead_code)]
pub fn track_duration(exp: &AnimTrackExport) -> f32 {
    exp.duration
}

/// Compute export size.
#[allow(dead_code)]
pub fn track_export_size(exp: &AnimTrackExport) -> usize {
    exp.samples.iter().map(|s| s.len() * 4).sum()
}

/// Validate track.
#[allow(dead_code)]
pub fn validate_track(exp: &AnimTrackExport) -> bool {
    !exp.name.is_empty() && exp.duration >= 0.0 && exp.channels > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_animation_track() {
        let e = export_animation_track("walk", 3, &[vec![0.0, 1.0, 0.0]], 2.0);
        assert_eq!(track_name(&e), "walk");
    }

    #[test]
    fn test_track_channel_count() {
        let e = export_animation_track("t", 4, &[], 1.0);
        assert_eq!(track_channel_count(&e), 4);
    }

    #[test]
    fn test_track_sample_count() {
        let e = export_animation_track("t", 1, &[vec![0.0], vec![1.0]], 1.0);
        assert_eq!(track_sample_count_export(&e), 2);
    }

    #[test]
    fn test_track_to_json() {
        let e = export_animation_track("run", 3, &[], 1.5);
        let j = track_to_json(&e);
        assert!(j.contains("\"name\":\"run\""));
    }

    #[test]
    fn test_track_duration() {
        let e = export_animation_track("t", 1, &[], 3.5);
        assert!((track_duration(&e) - 3.5).abs() < 0.001);
    }

    #[test]
    fn test_track_export_size() {
        let e = export_animation_track("t", 3, &[vec![0.0, 1.0, 2.0]], 1.0);
        assert_eq!(track_export_size(&e), 12);
    }

    #[test]
    fn test_validate_track() {
        let e = export_animation_track("walk", 3, &[], 2.0);
        assert!(validate_track(&e));
    }

    #[test]
    fn test_validate_empty_name() {
        let e = export_animation_track("", 3, &[], 2.0);
        assert!(!validate_track(&e));
    }

    #[test]
    fn test_validate_zero_channels() {
        let e = export_animation_track("t", 0, &[], 1.0);
        assert!(!validate_track(&e));
    }

    #[test]
    fn test_empty_track() {
        let e = export_animation_track("t", 1, &[], 0.0);
        assert_eq!(track_sample_count_export(&e), 0);
        assert_eq!(track_export_size(&e), 0);
    }
}
