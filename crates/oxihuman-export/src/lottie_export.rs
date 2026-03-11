// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Lottie animation JSON export stub.

/// A Lottie layer.
#[derive(Debug, Clone)]
pub struct LottieLayer {
    pub name: String,
    pub layer_type: u8,
    pub in_frame: f32,
    pub out_frame: f32,
}

/// A Lottie animation document.
#[derive(Debug, Clone)]
pub struct LottieExport {
    pub name: String,
    pub frame_rate: f32,
    pub width: u32,
    pub height: u32,
    pub total_frames: u32,
    pub layers: Vec<LottieLayer>,
}

impl LottieExport {
    /// Create a new Lottie export.
    pub fn new(name: &str, frame_rate: f32, width: u32, height: u32, total_frames: u32) -> Self {
        Self {
            name: name.to_string(),
            frame_rate,
            width,
            height,
            total_frames,
            layers: Vec::new(),
        }
    }

    /// Add a layer.
    pub fn add_layer(&mut self, layer: LottieLayer) {
        self.layers.push(layer);
    }

    /// Return layer count.
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Animation duration in seconds.
    pub fn duration_secs(&self) -> f32 {
        if self.frame_rate < 1e-6 {
            return 0.0;
        }
        self.total_frames as f32 / self.frame_rate
    }
}

/// Serialize to Lottie JSON string (stub).
pub fn export_lottie_json(doc: &LottieExport) -> String {
    format!(
        "{{\"nm\":\"{}\",\"fr\":{},\"w\":{},\"h\":{},\"op\":{},\"layers_count\":{}}}",
        doc.name,
        doc.frame_rate,
        doc.width,
        doc.height,
        doc.total_frames,
        doc.layer_count()
    )
}

/// Validate that all layer frames are within animation range.
pub fn validate_lottie(doc: &LottieExport) -> bool {
    let max_frame = doc.total_frames as f32;
    doc.layers
        .iter()
        .all(|l| l.in_frame >= 0.0 && l.out_frame <= max_frame)
}

/// Find layer by name.
pub fn find_lottie_layer<'a>(doc: &'a LottieExport, name: &str) -> Option<&'a LottieLayer> {
    doc.layers.iter().find(|l| l.name == name)
}

/// Total active frame range (min in to max out).
pub fn active_frame_range(doc: &LottieExport) -> (f32, f32) {
    if doc.layers.is_empty() {
        return (0.0, 0.0);
    }
    let min_in = doc
        .layers
        .iter()
        .map(|l| l.in_frame)
        .fold(f32::MAX, f32::min);
    let max_out = doc
        .layers
        .iter()
        .map(|l| l.out_frame)
        .fold(f32::MIN, f32::max);
    (min_in, max_out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_doc() -> LottieExport {
        let mut doc = LottieExport::new("anim", 24.0, 1920, 1080, 120);
        doc.add_layer(LottieLayer {
            name: "bg".into(),
            layer_type: 1,
            in_frame: 0.0,
            out_frame: 120.0,
        });
        doc.add_layer(LottieLayer {
            name: "fg".into(),
            layer_type: 4,
            in_frame: 10.0,
            out_frame: 100.0,
        });
        doc
    }

    #[test]
    fn test_layer_count() {
        /* document has correct layer count */
        let d = sample_doc();
        assert_eq!(d.layer_count(), 2);
    }

    #[test]
    fn test_duration_secs() {
        /* duration is total_frames / frame_rate */
        let d = sample_doc();
        assert!((d.duration_secs() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_export_json_not_empty() {
        /* JSON export is non-empty */
        let d = sample_doc();
        assert!(!export_lottie_json(&d).is_empty());
    }

    #[test]
    fn test_export_json_contains_name() {
        /* JSON contains animation name */
        let d = sample_doc();
        assert!(export_lottie_json(&d).contains("anim"));
    }

    #[test]
    fn test_validate_lottie_valid() {
        /* valid layers pass validation */
        let d = sample_doc();
        assert!(validate_lottie(&d));
    }

    #[test]
    fn test_find_lottie_layer() {
        /* find layer by name works */
        let d = sample_doc();
        assert!(find_lottie_layer(&d, "fg").is_some());
        assert!(find_lottie_layer(&d, "none").is_none());
    }

    #[test]
    fn test_active_frame_range() {
        /* active frame range spans all layers */
        let d = sample_doc();
        let (lo, hi) = active_frame_range(&d);
        assert!((lo - 0.0).abs() < 1e-5);
        assert!((hi - 120.0).abs() < 1e-5);
    }

    #[test]
    fn test_empty_doc_duration() {
        /* empty document has zero duration */
        let d = LottieExport::new("empty", 24.0, 100, 100, 0);
        assert!((d.duration_secs() - 0.0).abs() < 1e-6);
    }
}
