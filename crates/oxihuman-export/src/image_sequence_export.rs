// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Image sequence export stub.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ImageFormat {
    PNG,
    EXR,
    JPG,
    TIFF,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ImageSequenceConfig {
    pub format: ImageFormat,
    pub frame_start: u32,
    pub frame_end: u32,
    pub fps: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ImageSequenceExport {
    pub config: ImageSequenceConfig,
    pub output_dir: String,
    pub frames_exported: u32,
}

#[allow(dead_code)]
pub fn default_image_sequence_config() -> ImageSequenceConfig {
    ImageSequenceConfig { format: ImageFormat::PNG, frame_start: 1, frame_end: 250, fps: 24.0 }
}

#[allow(dead_code)]
pub fn new_image_sequence_export(config: ImageSequenceConfig, output_dir: &str) -> ImageSequenceExport {
    ImageSequenceExport { config, output_dir: output_dir.to_string(), frames_exported: 0 }
}

#[allow(dead_code)]
pub fn ise_frame_count(exp: &ImageSequenceExport) -> u32 {
    exp.config.frame_end.saturating_sub(exp.config.frame_start) + 1
}

#[allow(dead_code)]
pub fn ise_duration(exp: &ImageSequenceExport) -> f32 {
    ise_frame_count(exp) as f32 / exp.config.fps.max(1e-6)
}

#[allow(dead_code)]
pub fn ise_format_extension(exp: &ImageSequenceExport) -> &'static str {
    match exp.config.format {
        ImageFormat::PNG => "png",
        ImageFormat::EXR => "exr",
        ImageFormat::JPG => "jpg",
        ImageFormat::TIFF => "tiff",
    }
}

#[allow(dead_code)]
pub fn ise_increment_frame(exp: &mut ImageSequenceExport) {
    exp.frames_exported += 1;
}

#[allow(dead_code)]
pub fn ise_to_json(exp: &ImageSequenceExport) -> String {
    format!(
        r#"{{"output_dir":"{}","frames":{},"fps":{:.2},"format":"{}","exported":{}}}"#,
        exp.output_dir,
        ise_frame_count(exp),
        exp.config.fps,
        ise_format_extension(exp),
        exp.frames_exported
    )
}

#[allow(dead_code)]
pub fn ise_validate(exp: &ImageSequenceExport) -> bool {
    exp.config.frame_end >= exp.config.frame_start
        && exp.config.fps > 0.0
        && !exp.output_dir.is_empty()
}

#[allow(dead_code)]
pub fn ise_frame_filename(exp: &ImageSequenceExport, frame: u32) -> String {
    format!("{}/frame_{:04}.{}", exp.output_dir, frame, ise_format_extension(exp))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_export() -> ImageSequenceExport {
        new_image_sequence_export(default_image_sequence_config(), "/tmp/frames")
    }

    #[test]
    fn default_config_frame_count() {
        let exp = make_export();
        assert_eq!(ise_frame_count(&exp), 250);
    }

    #[test]
    fn duration_correct() {
        let exp = make_export();
        let dur = ise_duration(&exp);
        // 250 frames / 24 fps ≈ 10.4
        assert!(dur > 10.0 && dur < 11.0);
    }

    #[test]
    fn format_extension_png() {
        let exp = make_export();
        assert_eq!(ise_format_extension(&exp), "png");
    }

    #[test]
    fn format_extension_exr() {
        let cfg = ImageSequenceConfig { format: ImageFormat::EXR, frame_start: 1, frame_end: 10, fps: 24.0 };
        let exp = new_image_sequence_export(cfg, "/tmp");
        assert_eq!(ise_format_extension(&exp), "exr");
    }

    #[test]
    fn increment_frame() {
        let mut exp = make_export();
        ise_increment_frame(&mut exp);
        ise_increment_frame(&mut exp);
        assert_eq!(exp.frames_exported, 2);
    }

    #[test]
    fn validate_ok() {
        let exp = make_export();
        assert!(ise_validate(&exp));
    }

    #[test]
    fn frame_filename_format() {
        let exp = make_export();
        let name = ise_frame_filename(&exp, 42);
        assert!(name.contains("0042"));
        assert!(name.contains(".png"));
    }

    #[test]
    fn to_json_has_fields() {
        let exp = make_export();
        let json = ise_to_json(&exp);
        assert!(json.contains("output_dir"));
        assert!(json.contains("fps"));
        assert!(json.contains("format"));
    }
}
