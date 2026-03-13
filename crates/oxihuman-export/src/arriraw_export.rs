// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! ARRI RAW stub export.

/// ARRI camera model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArriModel {
    AlexaMini,
    AlexaMiniLf,
    Amira,
}

impl ArriModel {
    pub fn name(&self) -> &'static str {
        match self {
            ArriModel::AlexaMini => "ALEXA Mini",
            ArriModel::AlexaMiniLf => "ALEXA Mini LF",
            ArriModel::Amira => "AMIRA",
        }
    }
}

/// ARRI RAW frame metadata.
#[derive(Debug, Clone)]
pub struct ArriFrame {
    pub frame_index: u32,
    pub width: u32,
    pub height: u32,
    pub iso: u32,
    pub exposure_index: u32,
}

/// ARRI RAW export stub.
#[derive(Debug, Clone)]
pub struct ArriRawExport {
    pub model: ArriModel,
    pub fps: f32,
    pub color_space: String,
    pub frames: Vec<ArriFrame>,
}

/// Create a new ARRI RAW export.
pub fn new_arriraw_export(model: ArriModel, fps: f32) -> ArriRawExport {
    ArriRawExport {
        model,
        fps,
        color_space: "LogC".to_string(),
        frames: Vec::new(),
    }
}

/// Add a frame.
pub fn arriraw_add_frame(
    export: &mut ArriRawExport,
    frame_index: u32,
    width: u32,
    height: u32,
    iso: u32,
    ei: u32,
) {
    export.frames.push(ArriFrame {
        frame_index,
        width,
        height,
        iso,
        exposure_index: ei,
    });
}

/// Frame count.
pub fn arriraw_frame_count(export: &ArriRawExport) -> usize {
    export.frames.len()
}

/// Duration in seconds.
pub fn arriraw_duration(export: &ArriRawExport) -> f32 {
    if export.fps <= 0.0 {
        return 0.0;
    }
    export.frames.len() as f32 / export.fps
}

/// Validate the export.
pub fn validate_arriraw(export: &ArriRawExport) -> bool {
    export.fps > 0.0 && !export.color_space.is_empty()
}

/// Estimate the file size.
pub fn arriraw_size_estimate(export: &ArriRawExport) -> usize {
    /* ~12 bits per pixel per frame */
    export
        .frames
        .iter()
        .map(|f| {
            let pixels = f.width as usize * f.height as usize;
            pixels * 12 / 8
        })
        .sum()
}

/// Generate metadata string.
pub fn arriraw_metadata_string(export: &ArriRawExport) -> String {
    format!(
        "ARRI|{}|FPS:{}|ColorSpace:{}|Frames:{}",
        export.model.name(),
        export.fps,
        export.color_space,
        export.frames.len()
    )
}

/// Average ISO across frames.
pub fn arriraw_average_iso(export: &ArriRawExport) -> f32 {
    if export.frames.is_empty() {
        return 0.0;
    }
    export.frames.iter().map(|f| f.iso as f32).sum::<f32>() / export.frames.len() as f32
}

/// Resolution of the first frame.
pub fn arriraw_resolution(export: &ArriRawExport) -> Option<(u32, u32)> {
    export.frames.first().map(|f| (f.width, f.height))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ArriRawExport {
        let mut exp = new_arriraw_export(ArriModel::AlexaMini, 24.0);
        arriraw_add_frame(&mut exp, 0, 2880, 2160, 800, 800);
        arriraw_add_frame(&mut exp, 1, 2880, 2160, 1600, 1600);
        exp
    }

    #[test]
    fn test_frame_count() {
        assert_eq!(arriraw_frame_count(&sample()), 2);
    }

    #[test]
    fn test_duration() {
        assert!((arriraw_duration(&sample()) - 2.0 / 24.0).abs() < 1e-4);
    }

    #[test]
    fn test_validate() {
        assert!(validate_arriraw(&sample()));
    }

    #[test]
    fn test_size_estimate() {
        assert!(arriraw_size_estimate(&sample()) > 0);
    }

    #[test]
    fn test_metadata() {
        let s = arriraw_metadata_string(&sample());
        assert!(s.contains("ALEXA Mini"));
    }

    #[test]
    fn test_avg_iso() {
        assert!((arriraw_average_iso(&sample()) - 1200.0).abs() < 1e-3);
    }

    #[test]
    fn test_resolution() {
        assert_eq!(arriraw_resolution(&sample()), Some((2880, 2160)));
    }

    #[test]
    fn test_model_names() {
        assert_eq!(ArriModel::AlexaMiniLf.name(), "ALEXA Mini LF");
        assert_eq!(ArriModel::Amira.name(), "AMIRA");
    }

    #[test]
    fn test_empty_duration() {
        let exp = new_arriraw_export(ArriModel::Amira, 25.0);
        assert_eq!(arriraw_frame_count(&exp), 0);
    }
}
