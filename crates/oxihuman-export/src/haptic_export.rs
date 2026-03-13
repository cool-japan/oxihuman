//! Haptic feedback force profile export.

#[allow(dead_code)]
#[derive(Clone)]
pub struct HapticSample {
    pub time_ms: f32,
    pub intensity: f32, // 0..1
    pub frequency: f32, // Hz
    pub duration_ms: f32,
}

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum HapticActuator {
    WholeHand,
    Fingertip,
    Palm,
    Wrist,
    Forearm,
}

#[allow(dead_code)]
pub struct HapticTrack {
    pub actuator: HapticActuator,
    pub samples: Vec<HapticSample>,
    pub name: String,
}

#[allow(dead_code)]
pub struct HapticExport {
    pub tracks: Vec<HapticTrack>,
    pub duration_ms: f32,
    pub sample_rate_hz: f32,
    pub device_id: String,
}

#[allow(dead_code)]
pub fn new_haptic_export(sample_rate: f32) -> HapticExport {
    HapticExport {
        tracks: Vec::new(),
        duration_ms: 0.0,
        sample_rate_hz: sample_rate,
        device_id: String::new(),
    }
}

#[allow(dead_code)]
pub fn add_haptic_track(export: &mut HapticExport, actuator: HapticActuator, name: &str) {
    export.tracks.push(HapticTrack {
        actuator,
        samples: Vec::new(),
        name: name.to_string(),
    });
}

#[allow(dead_code)]
pub fn add_haptic_sample(
    export: &mut HapticExport,
    track_idx: usize,
    sample: HapticSample,
) -> bool {
    if track_idx >= export.tracks.len() {
        return false;
    }
    let end = sample.time_ms + sample.duration_ms;
    if end > export.duration_ms {
        export.duration_ms = end;
    }
    export.tracks[track_idx].samples.push(sample);
    true
}

#[allow(dead_code)]
pub fn haptic_track_count(export: &HapticExport) -> usize {
    export.tracks.len()
}

#[allow(dead_code)]
pub fn haptic_sample_count(export: &HapticExport, track_idx: usize) -> usize {
    if track_idx >= export.tracks.len() {
        return 0;
    }
    export.tracks[track_idx].samples.len()
}

#[allow(dead_code)]
pub fn export_haptic_json(export: &HapticExport) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!(
        "  \"sample_rate_hz\": {},\n  \"duration_ms\": {},\n  \"device_id\": \"{}\",\n  \"tracks\": [\n",
        export.sample_rate_hz, export.duration_ms, export.device_id
    ));
    for (ti, track) in export.tracks.iter().enumerate() {
        out.push_str(&format!(
            "    {{ \"name\": \"{}\", \"actuator\": \"{:?}\", \"samples\": [",
            track.name, track.actuator
        ));
        for (si, s) in track.samples.iter().enumerate() {
            if si > 0 {
                out.push(',');
            }
            out.push_str(&format!(
                "{{\"time_ms\":{},\"intensity\":{},\"frequency\":{},\"duration_ms\":{}}}",
                s.time_ms, s.intensity, s.frequency, s.duration_ms
            ));
        }
        out.push(']');
        out.push('}');
        if ti + 1 < export.tracks.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ]\n}\n");
    out
}

#[allow(dead_code)]
pub fn export_haptic_csv(export: &HapticExport, track_idx: usize) -> String {
    if track_idx >= export.tracks.len() {
        return String::from("time_ms,intensity,frequency,duration_ms\n");
    }
    let track = &export.tracks[track_idx];
    let mut out = String::from("time_ms,intensity,frequency,duration_ms\n");
    for s in &track.samples {
        out.push_str(&format!(
            "{},{},{},{}\n",
            s.time_ms, s.intensity, s.frequency, s.duration_ms
        ));
    }
    out
}

#[allow(dead_code)]
pub fn evaluate_haptic_at(
    export: &HapticExport,
    track_idx: usize,
    time_ms: f32,
) -> Option<HapticSample> {
    if track_idx >= export.tracks.len() {
        return None;
    }
    let samples = &export.tracks[track_idx].samples;
    if samples.is_empty() {
        return None;
    }
    let mut best_idx = 0;
    let mut best_dist = (samples[0].time_ms - time_ms).abs();
    for (i, s) in samples.iter().enumerate() {
        let d = (s.time_ms - time_ms).abs();
        if d < best_dist {
            best_dist = d;
            best_idx = i;
        }
    }
    Some(samples[best_idx].clone())
}

#[allow(dead_code)]
pub fn peak_intensity(export: &HapticExport, track_idx: usize) -> f32 {
    if track_idx >= export.tracks.len() {
        return 0.0;
    }
    export.tracks[track_idx]
        .samples
        .iter()
        .map(|s| s.intensity)
        .fold(0.0_f32, f32::max)
}

#[allow(dead_code)]
pub fn average_intensity(export: &HapticExport, track_idx: usize) -> f32 {
    if track_idx >= export.tracks.len() {
        return 0.0;
    }
    let samples = &export.tracks[track_idx].samples;
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f32 = samples.iter().map(|s| s.intensity).sum();
    sum / samples.len() as f32
}

#[allow(dead_code)]
pub fn resample_haptic_track(
    export: &HapticExport,
    track_idx: usize,
    new_rate_hz: f32,
) -> Vec<HapticSample> {
    if track_idx >= export.tracks.len() || new_rate_hz <= 0.0 {
        return Vec::new();
    }
    let duration = export.duration_ms;
    if duration <= 0.0 {
        return Vec::new();
    }
    let step_ms = 1000.0 / new_rate_hz;
    let count = (duration / step_ms).ceil() as usize;
    let mut result = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f32 * step_ms;
        if let Some(s) = evaluate_haptic_at(export, track_idx, t) {
            result.push(HapticSample {
                time_ms: t,
                intensity: s.intensity,
                frequency: s.frequency,
                duration_ms: step_ms,
            });
        }
    }
    result
}

#[allow(dead_code)]
pub fn clamp_haptic_intensities(export: &mut HapticExport) {
    for track in &mut export.tracks {
        for s in &mut track.samples {
            s.intensity = s.intensity.clamp(0.0, 1.0);
        }
    }
}

#[allow(dead_code)]
pub fn haptic_export_duration(export: &HapticExport) -> f32 {
    export.duration_ms
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sample(time_ms: f32, intensity: f32) -> HapticSample {
        HapticSample {
            time_ms,
            intensity,
            frequency: 100.0,
            duration_ms: 10.0,
        }
    }

    #[test]
    fn test_new_export() {
        let e = new_haptic_export(1000.0);
        assert_eq!(e.sample_rate_hz, 1000.0);
        assert_eq!(e.tracks.len(), 0);
    }

    #[test]
    fn test_add_track() {
        let mut e = new_haptic_export(1000.0);
        add_haptic_track(&mut e, HapticActuator::Palm, "palm");
        assert_eq!(haptic_track_count(&e), 1);
        assert_eq!(e.tracks[0].name, "palm");
        assert_eq!(e.tracks[0].actuator, HapticActuator::Palm);
    }

    #[test]
    fn test_add_sample_valid() {
        let mut e = new_haptic_export(1000.0);
        add_haptic_track(&mut e, HapticActuator::Wrist, "wrist");
        let ok = add_haptic_sample(&mut e, 0, make_sample(0.0, 0.5));
        assert!(ok);
    }

    #[test]
    fn test_add_sample_invalid_track() {
        let mut e = new_haptic_export(1000.0);
        let ok = add_haptic_sample(&mut e, 99, make_sample(0.0, 0.5));
        assert!(!ok);
    }

    #[test]
    fn test_sample_count() {
        let mut e = new_haptic_export(1000.0);
        add_haptic_track(&mut e, HapticActuator::Fingertip, "tip");
        add_haptic_sample(&mut e, 0, make_sample(0.0, 0.3));
        add_haptic_sample(&mut e, 0, make_sample(10.0, 0.6));
        assert_eq!(haptic_sample_count(&e, 0), 2);
    }

    #[test]
    fn test_sample_count_invalid() {
        let e = new_haptic_export(1000.0);
        assert_eq!(haptic_sample_count(&e, 5), 0);
    }

    #[test]
    fn test_json_non_empty() {
        let mut e = new_haptic_export(500.0);
        add_haptic_track(&mut e, HapticActuator::WholeHand, "all");
        add_haptic_sample(&mut e, 0, make_sample(0.0, 1.0));
        let json = export_haptic_json(&e);
        assert!(!json.is_empty());
        assert!(json.contains("sample_rate_hz"));
        assert!(json.contains("tracks"));
    }

    #[test]
    fn test_csv_has_commas() {
        let mut e = new_haptic_export(1000.0);
        add_haptic_track(&mut e, HapticActuator::Forearm, "forearm");
        add_haptic_sample(&mut e, 0, make_sample(0.0, 0.8));
        let csv = export_haptic_csv(&e, 0);
        assert!(csv.contains(','));
    }

    #[test]
    fn test_peak_intensity() {
        let mut e = new_haptic_export(1000.0);
        add_haptic_track(&mut e, HapticActuator::Palm, "p");
        add_haptic_sample(&mut e, 0, make_sample(0.0, 0.2));
        add_haptic_sample(&mut e, 0, make_sample(10.0, 0.9));
        add_haptic_sample(&mut e, 0, make_sample(20.0, 0.5));
        assert!((peak_intensity(&e, 0) - 0.9).abs() < 1e-5);
    }

    #[test]
    fn test_average_intensity() {
        let mut e = new_haptic_export(1000.0);
        add_haptic_track(&mut e, HapticActuator::Palm, "p");
        add_haptic_sample(&mut e, 0, make_sample(0.0, 0.0));
        add_haptic_sample(&mut e, 0, make_sample(10.0, 1.0));
        let avg = average_intensity(&e, 0);
        assert!((avg - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_clamp() {
        let mut e = new_haptic_export(1000.0);
        add_haptic_track(&mut e, HapticActuator::Wrist, "w");
        add_haptic_sample(
            &mut e,
            0,
            HapticSample {
                time_ms: 0.0,
                intensity: 1.5,
                frequency: 100.0,
                duration_ms: 10.0,
            },
        );
        add_haptic_sample(
            &mut e,
            0,
            HapticSample {
                time_ms: 10.0,
                intensity: -0.3,
                frequency: 100.0,
                duration_ms: 10.0,
            },
        );
        clamp_haptic_intensities(&mut e);
        assert!((e.tracks[0].samples[0].intensity - 1.0).abs() < 1e-6);
        assert!((e.tracks[0].samples[1].intensity - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_haptic_at() {
        let mut e = new_haptic_export(1000.0);
        add_haptic_track(&mut e, HapticActuator::Palm, "p");
        add_haptic_sample(&mut e, 0, make_sample(0.0, 0.1));
        add_haptic_sample(&mut e, 0, make_sample(100.0, 0.9));
        let s = evaluate_haptic_at(&e, 0, 95.0).expect("should succeed");
        assert!((s.time_ms - 100.0).abs() < 1e-5);
    }

    #[test]
    fn test_duration_updated() {
        let mut e = new_haptic_export(1000.0);
        add_haptic_track(&mut e, HapticActuator::Fingertip, "f");
        add_haptic_sample(
            &mut e,
            0,
            HapticSample {
                time_ms: 50.0,
                intensity: 0.5,
                frequency: 100.0,
                duration_ms: 30.0,
            },
        );
        assert!((haptic_export_duration(&e) - 80.0).abs() < 1e-5);
    }

    #[test]
    fn test_resample() {
        let mut e = new_haptic_export(1000.0);
        add_haptic_track(&mut e, HapticActuator::Palm, "p");
        add_haptic_sample(&mut e, 0, make_sample(0.0, 0.5));
        add_haptic_sample(&mut e, 0, make_sample(50.0, 0.8));
        let resampled = resample_haptic_track(&e, 0, 10.0); // 10 Hz
        assert!(!resampled.is_empty());
    }

    #[test]
    fn test_track_count_multiple() {
        let mut e = new_haptic_export(1000.0);
        add_haptic_track(&mut e, HapticActuator::Palm, "p1");
        add_haptic_track(&mut e, HapticActuator::Wrist, "w1");
        add_haptic_track(&mut e, HapticActuator::Forearm, "fa1");
        assert_eq!(haptic_track_count(&e), 3);
    }

    #[test]
    fn test_peak_empty_track() {
        let e = new_haptic_export(1000.0);
        assert_eq!(peak_intensity(&e, 0), 0.0);
    }
}
