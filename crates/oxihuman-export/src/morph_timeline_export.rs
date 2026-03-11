//! Morph target animation timeline export.

// ── Structs ───────────────────────────────────────────────────────────────────

/// Configuration for morph timeline export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphTimelineConfig {
    pub frame_rate: f32,
    pub start_frame: i32,
    pub end_frame: i32,
    pub quantize: bool,
}

/// A single frame of morph weights.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphFrame {
    pub frame: i32,
    pub weights: Vec<(String, f32)>,
}

/// A morph target timeline (sequence of frames for one target group).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphTimeline {
    pub target_name: String,
    pub frames: Vec<MorphFrame>,
}

/// Result of exporting one or more morph timelines.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphTimelineExportResult {
    pub timelines: Vec<String>,
    pub frame_count: usize,
    pub target_count: usize,
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Returns a default `MorphTimelineConfig`.
#[allow(dead_code)]
pub fn default_morph_timeline_config() -> MorphTimelineConfig {
    MorphTimelineConfig {
        frame_rate: 24.0,
        start_frame: 0,
        end_frame: 100,
        quantize: false,
    }
}

/// Creates a new `MorphTimeline` for the given target name.
#[allow(dead_code)]
pub fn new_morph_timeline(target: &str) -> MorphTimeline {
    MorphTimeline {
        target_name: target.to_string(),
        frames: Vec::new(),
    }
}

/// Appends a frame to a timeline.
#[allow(dead_code)]
pub fn add_morph_frame(tl: &mut MorphTimeline, frame: MorphFrame) {
    tl.frames.push(frame);
}

/// Creates a new empty `MorphFrame` at the given frame index.
#[allow(dead_code)]
pub fn new_morph_frame(frame: i32) -> MorphFrame {
    MorphFrame {
        frame,
        weights: Vec::new(),
    }
}

/// Adds a named weight to a frame.
#[allow(dead_code)]
pub fn add_weight_to_frame(frame: &mut MorphFrame, name: &str, weight: f32) {
    frame.weights.push((name.to_string(), weight));
}

/// Exports a single timeline to a string representation.
#[allow(dead_code)]
pub fn export_morph_timeline(tl: &MorphTimeline, cfg: &MorphTimelineConfig) -> String {
    let frames_json: Vec<String> = tl
        .frames
        .iter()
        .map(|f| {
            let ws: Vec<String> = f
                .weights
                .iter()
                .map(|(n, w)| {
                    let val = if cfg.quantize {
                        (w * 255.0).round() / 255.0
                    } else {
                        *w
                    };
                    format!("\"{}\":{:.6}", n, val)
                })
                .collect();
            format!("{{\"frame\":{},\"weights\":{{{}}}}}", f.frame, ws.join(","))
        })
        .collect();
    format!(
        "{{\"target\":\"{}\",\"frame_rate\":{:.2},\"frames\":[{}]}}",
        tl.target_name,
        cfg.frame_rate,
        frames_json.join(","),
    )
}

/// Exports multiple timelines and collects a result.
#[allow(dead_code)]
pub fn export_morph_timelines(
    tls: &[MorphTimeline],
    cfg: &MorphTimelineConfig,
) -> MorphTimelineExportResult {
    let mut max_frames = 0usize;
    let mut serialised = Vec::with_capacity(tls.len());
    for tl in tls {
        let fc = tl.frames.len();
        if fc > max_frames {
            max_frames = fc;
        }
        serialised.push(export_morph_timeline(tl, cfg));
    }
    MorphTimelineExportResult {
        timelines: serialised,
        frame_count: max_frames,
        target_count: tls.len(),
    }
}

/// Returns the number of frames in a timeline.
#[allow(dead_code)]
pub fn timeline_frame_count(tl: &MorphTimeline) -> usize {
    tl.frames.len()
}

/// Returns the duration in seconds for a timeline given a config.
#[allow(dead_code)]
pub fn timeline_duration_sec(tl: &MorphTimeline, cfg: &MorphTimelineConfig) -> f32 {
    if tl.frames.is_empty() || cfg.frame_rate <= 0.0 {
        return 0.0;
    }
    let range = cfg.end_frame - cfg.start_frame;
    (range.max(0) as f32) / cfg.frame_rate
}

/// Serialises a `MorphTimelineExportResult` to JSON.
#[allow(dead_code)]
pub fn morph_timeline_result_to_json(r: &MorphTimelineExportResult) -> String {
    format!(
        "{{\"target_count\":{},\"frame_count\":{},\"timelines\":{}}}",
        r.target_count,
        r.frame_count,
        r.timelines.len(),
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_morph_timeline_config();
        assert!((cfg.frame_rate - 24.0).abs() < 1e-6);
        assert_eq!(cfg.start_frame, 0);
        assert_eq!(cfg.end_frame, 100);
        assert!(!cfg.quantize);
    }

    #[test]
    fn new_timeline_is_empty() {
        let tl = new_morph_timeline("face");
        assert_eq!(tl.target_name, "face");
        assert!(tl.frames.is_empty());
    }

    #[test]
    fn add_frame_increases_count() {
        let mut tl = new_morph_timeline("brow");
        let f = new_morph_frame(0);
        add_morph_frame(&mut tl, f);
        assert_eq!(timeline_frame_count(&tl), 1);
    }

    #[test]
    fn add_weight_to_frame_works() {
        let mut f = new_morph_frame(5);
        add_weight_to_frame(&mut f, "smile", 0.8);
        assert_eq!(f.weights.len(), 1);
        assert!((f.weights[0].1 - 0.8).abs() < 1e-6);
    }

    #[test]
    fn export_timeline_contains_target_name() {
        let cfg = default_morph_timeline_config();
        let tl = new_morph_timeline("jaw");
        let s = export_morph_timeline(&tl, &cfg);
        assert!(s.contains("jaw"));
    }

    #[test]
    fn export_multiple_timelines_result() {
        let cfg = default_morph_timeline_config();
        let mut tl1 = new_morph_timeline("eye_l");
        add_morph_frame(&mut tl1, new_morph_frame(0));
        add_morph_frame(&mut tl1, new_morph_frame(1));
        let tl2 = new_morph_timeline("eye_r");
        let r = export_morph_timelines(&[tl1, tl2], &cfg);
        assert_eq!(r.target_count, 2);
        assert_eq!(r.frame_count, 2);
    }

    #[test]
    fn timeline_duration_sec_basic() {
        let cfg = default_morph_timeline_config(); // 0..100 @ 24fps
        let mut tl = new_morph_timeline("t");
        add_morph_frame(&mut tl, new_morph_frame(0));
        let d = timeline_duration_sec(&tl, &cfg);
        assert!((d - 100.0 / 24.0).abs() < 1e-4);
    }

    #[test]
    fn result_to_json_contains_counts() {
        let r = MorphTimelineExportResult {
            timelines: vec!["a".to_string()],
            frame_count: 50,
            target_count: 3,
        };
        let j = morph_timeline_result_to_json(&r);
        assert!(j.contains("\"target_count\":3"));
        assert!(j.contains("\"frame_count\":50"));
    }
}
