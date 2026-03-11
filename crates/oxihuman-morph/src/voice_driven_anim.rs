//! Voice/audio-driven animation (speech envelope → jaw, viseme blending).

#[allow(dead_code)]
pub struct AudioFrame {
    pub time: f32,
    pub amplitude: f32,
    pub frequency: f32,
    pub voiced: bool,
}

#[allow(dead_code)]
pub struct JawCurve {
    pub keys: Vec<(f32, f32)>,
}

#[allow(dead_code)]
pub struct VoiceAnimConfig {
    pub jaw_scale: f32,
    pub jaw_smooth: f32,
    pub min_amplitude: f32,
    pub viseme_blend_time: f32,
}

#[allow(dead_code)]
pub struct VoiceAnimResult {
    pub jaw_curve: JawCurve,
    pub viseme_weights: Vec<Vec<f32>>,
    pub frame_times: Vec<f32>,
}

#[allow(dead_code)]
pub fn default_voice_anim_config() -> VoiceAnimConfig {
    VoiceAnimConfig {
        jaw_scale: 0.8,
        jaw_smooth: 0.05,
        min_amplitude: 0.02,
        viseme_blend_time: 0.08,
    }
}

#[allow(dead_code)]
pub fn amplitude_to_jaw(amplitude: f32, cfg: &VoiceAnimConfig) -> f32 {
    let v = amplitude.clamp(0.0, 1.0) * cfg.jaw_scale;
    v.clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn smooth_jaw_curve(curve: &JawCurve, window: f32) -> JawCurve {
    if curve.keys.is_empty() {
        return JawCurve { keys: Vec::new() };
    }
    let half = window * 0.5;
    let keys: Vec<(f32, f32)> = curve
        .keys
        .iter()
        .map(|&(t, _)| {
            let mut sum = 0.0f32;
            let mut count = 0u32;
            for &(kt, kv) in &curve.keys {
                if (kt - t).abs() <= half {
                    sum += kv;
                    count += 1;
                }
            }
            (t, if count > 0 { sum / count as f32 } else { 0.0 })
        })
        .collect();
    JawCurve { keys }
}

#[allow(dead_code)]
pub fn audio_frames_to_jaw_curve(frames: &[AudioFrame], cfg: &VoiceAnimConfig) -> JawCurve {
    let keys: Vec<(f32, f32)> = frames
        .iter()
        .map(|f| {
            let jaw = if f.amplitude >= cfg.min_amplitude {
                amplitude_to_jaw(f.amplitude, cfg)
            } else {
                0.0
            };
            (f.time, jaw)
        })
        .collect();
    JawCurve { keys }
}

#[allow(dead_code)]
pub fn sample_jaw_curve(curve: &JawCurve, time: f32) -> f32 {
    if curve.keys.is_empty() {
        return 0.0;
    }
    if curve.keys.len() == 1 {
        return curve.keys[0].1;
    }
    let first = curve.keys[0];
    let last = curve.keys[curve.keys.len() - 1];
    if time <= first.0 {
        return first.1;
    }
    if time >= last.0 {
        return last.1;
    }
    for i in 0..curve.keys.len() - 1 {
        let (t0, v0) = curve.keys[i];
        let (t1, v1) = curve.keys[i + 1];
        if time >= t0 && time <= t1 {
            let span = t1 - t0;
            if span < 1e-9 {
                return v0;
            }
            let alpha = (time - t0) / span;
            return v0 + (v1 - v0) * alpha;
        }
    }
    last.1
}

#[allow(dead_code)]
pub fn voiced_segments(frames: &[AudioFrame], min_amplitude: f32) -> Vec<(f32, f32)> {
    let mut segments: Vec<(f32, f32)> = Vec::new();
    let mut in_segment = false;
    let mut seg_start = 0.0f32;

    for frame in frames {
        let active = frame.voiced && frame.amplitude >= min_amplitude;
        if active && !in_segment {
            in_segment = true;
            seg_start = frame.time;
        } else if !active && in_segment {
            in_segment = false;
            segments.push((seg_start, frame.time));
        }
    }
    if in_segment {
        if let Some(last) = frames.last() {
            segments.push((seg_start, last.time));
        }
    }
    segments
}

/// Map dominant frequency to one of 14 viseme indices.
/// Rough mapping based on formant frequency ranges.
#[allow(dead_code)]
pub fn frequency_to_viseme_index(frequency: f32) -> usize {
    // 14 viseme buckets across 80..3400 Hz range
    let range_min = 80.0f32;
    let range_max = 3400.0f32;
    let clamped = frequency.clamp(range_min, range_max);
    let normalized = (clamped - range_min) / (range_max - range_min);
    let idx = (normalized * 13.9) as usize;
    idx.min(13)
}

#[allow(dead_code)]
pub fn frames_to_viseme_weights(frames: &[AudioFrame]) -> Vec<Vec<f32>> {
    frames
        .iter()
        .map(|f| {
            let mut weights = vec![0.0f32; 14];
            let idx = frequency_to_viseme_index(f.frequency);
            weights[idx] = f.amplitude.clamp(0.0, 1.0);
            weights
        })
        .collect()
}

#[allow(dead_code)]
pub fn jaw_curve_duration(curve: &JawCurve) -> f32 {
    if curve.keys.len() < 2 {
        return 0.0;
    }
    curve.keys.last().map(|k| k.0).unwrap_or(0.0) - curve.keys.first().map(|k| k.0).unwrap_or(0.0)
}

#[allow(dead_code)]
pub fn jaw_curve_max(curve: &JawCurve) -> f32 {
    curve.keys.iter().map(|k| k.1).fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn blend_jaw_curves(a: &JawCurve, b: &JawCurve, t: f32) -> JawCurve {
    let t = t.clamp(0.0, 1.0);
    // Use keys from a, sample b at same times
    let keys: Vec<(f32, f32)> = a
        .keys
        .iter()
        .map(|&(time, va)| {
            let vb = sample_jaw_curve(b, time);
            (time, va + (vb - va) * t)
        })
        .collect();
    JawCurve { keys }
}

#[allow(dead_code)]
pub fn voice_anim_from_frames(frames: &[AudioFrame], cfg: &VoiceAnimConfig) -> VoiceAnimResult {
    let jaw_curve_raw = audio_frames_to_jaw_curve(frames, cfg);
    let jaw_curve = smooth_jaw_curve(&jaw_curve_raw, cfg.jaw_smooth);
    let viseme_weights = frames_to_viseme_weights(frames);
    let frame_times: Vec<f32> = frames.iter().map(|f| f.time).collect();
    VoiceAnimResult {
        jaw_curve,
        viseme_weights,
        frame_times,
    }
}

#[allow(dead_code)]
pub fn silence_duration(frames: &[AudioFrame], cfg: &VoiceAnimConfig) -> f32 {
    frames
        .iter()
        .filter(|f| f.amplitude < cfg.min_amplitude)
        .count() as f32
        / frames.len().max(1) as f32
        * jaw_curve_duration(&JawCurve {
            keys: frames.iter().map(|f| (f.time, 0.0)).collect(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_frames(n: usize) -> Vec<AudioFrame> {
        (0..n)
            .map(|i| AudioFrame {
                time: i as f32 * 0.033,
                amplitude: if i % 3 == 0 { 0.01 } else { 0.5 },
                frequency: 200.0 + i as f32 * 50.0,
                voiced: i % 3 != 0,
            })
            .collect()
    }

    #[test]
    fn test_amplitude_to_jaw_in_range() {
        let cfg = default_voice_anim_config();
        let v = amplitude_to_jaw(0.5, &cfg);
        assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn test_amplitude_to_jaw_zero() {
        let cfg = default_voice_anim_config();
        assert_eq!(amplitude_to_jaw(0.0, &cfg), 0.0);
    }

    #[test]
    fn test_amplitude_to_jaw_max() {
        let cfg = default_voice_anim_config();
        let v = amplitude_to_jaw(1.0, &cfg);
        assert!(v <= 1.0);
    }

    #[test]
    fn test_audio_frames_to_jaw_curve_length() {
        let cfg = default_voice_anim_config();
        let frames = make_frames(10);
        let curve = audio_frames_to_jaw_curve(&frames, &cfg);
        assert_eq!(curve.keys.len(), 10);
    }

    #[test]
    fn test_sample_at_t0() {
        let curve = JawCurve {
            keys: vec![(0.0, 0.3), (1.0, 0.8)],
        };
        let v = sample_jaw_curve(&curve, 0.0);
        assert!((v - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_sample_interpolation() {
        let curve = JawCurve {
            keys: vec![(0.0, 0.0), (1.0, 1.0)],
        };
        let v = sample_jaw_curve(&curve, 0.5);
        assert!((v - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_voiced_segments_count() {
        let frames = vec![
            AudioFrame {
                time: 0.0,
                amplitude: 0.5,
                frequency: 200.0,
                voiced: true,
            },
            AudioFrame {
                time: 0.1,
                amplitude: 0.5,
                frequency: 200.0,
                voiced: true,
            },
            AudioFrame {
                time: 0.2,
                amplitude: 0.01,
                frequency: 200.0,
                voiced: false,
            },
            AudioFrame {
                time: 0.3,
                amplitude: 0.5,
                frequency: 200.0,
                voiced: true,
            },
            AudioFrame {
                time: 0.4,
                amplitude: 0.5,
                frequency: 200.0,
                voiced: true,
            },
        ];
        let segs = voiced_segments(&frames, 0.02);
        assert_eq!(segs.len(), 2);
    }

    #[test]
    fn test_frequency_to_viseme_index_valid() {
        for freq in [100.0f32, 200.0, 500.0, 1000.0, 2000.0, 3000.0] {
            let idx = frequency_to_viseme_index(freq);
            assert!(idx < 14, "viseme index out of range for freq {}", freq);
        }
    }

    #[test]
    fn test_jaw_curve_max() {
        let curve = JawCurve {
            keys: vec![(0.0, 0.2), (0.5, 0.9), (1.0, 0.4)],
        };
        let m = jaw_curve_max(&curve);
        assert!((m - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_smooth_does_not_change_length() {
        let cfg = default_voice_anim_config();
        let frames = make_frames(20);
        let curve = audio_frames_to_jaw_curve(&frames, &cfg);
        let smoothed = smooth_jaw_curve(&curve, 0.05);
        assert_eq!(smoothed.keys.len(), curve.keys.len());
    }

    #[test]
    fn test_silence_duration_all_silent() {
        let cfg = default_voice_anim_config();
        let frames: Vec<AudioFrame> = (0..10)
            .map(|i| AudioFrame {
                time: i as f32 * 0.1,
                amplitude: 0.0,
                frequency: 200.0,
                voiced: false,
            })
            .collect();
        let sd = silence_duration(&frames, &cfg);
        assert!(sd >= 0.0);
    }

    #[test]
    fn test_frames_to_viseme_weights_length() {
        let frames = make_frames(5);
        let w = frames_to_viseme_weights(&frames);
        assert_eq!(w.len(), 5);
        for row in &w {
            assert_eq!(row.len(), 14);
        }
    }

    #[test]
    fn test_jaw_curve_duration() {
        let curve = JawCurve {
            keys: vec![(0.0, 0.0), (0.5, 0.5), (1.5, 0.3)],
        };
        let d = jaw_curve_duration(&curve);
        assert!((d - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_jaw_curves() {
        let a = JawCurve {
            keys: vec![(0.0, 0.0), (1.0, 1.0)],
        };
        let b = JawCurve {
            keys: vec![(0.0, 1.0), (1.0, 0.0)],
        };
        let blended = blend_jaw_curves(&a, &b, 0.5);
        assert!((blended.keys[0].1 - 0.5).abs() < 1e-6);
        assert!((blended.keys[1].1 - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_voice_anim_from_frames() {
        let cfg = default_voice_anim_config();
        let frames = make_frames(8);
        let result = voice_anim_from_frames(&frames, &cfg);
        assert_eq!(result.jaw_curve.keys.len(), 8);
        assert_eq!(result.viseme_weights.len(), 8);
        assert_eq!(result.frame_times.len(), 8);
    }
}
