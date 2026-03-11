// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! RenderDocCapture — stub for RenderDoc frame capture integration.

#![allow(dead_code)]

/// Configuration for the capture session.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    pub enabled: bool,
    pub output_path: String,
    pub api_version: u32,
    pub frame_count: u32,
}

/// RenderDoc capture state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderDocCapture {
    pub config: CaptureConfig,
    pub capturing: bool,
}

/// Create a `CaptureConfig` with the given parameters.
#[allow(dead_code)]
pub fn new_capture_config(output_path: &str, api_version: u32) -> CaptureConfig {
    CaptureConfig {
        enabled: true,
        output_path: output_path.to_owned(),
        api_version,
        frame_count: 0,
    }
}

/// Create a default `CaptureConfig`.
#[allow(dead_code)]
pub fn default_capture_config() -> CaptureConfig {
    new_capture_config("renderdoc_capture", 1)
}

/// Begin a capture session (stub).
#[allow(dead_code)]
pub fn begin_capture(cap: &mut RenderDocCapture) {
    if cap.config.enabled {
        cap.capturing = true;
    }
}

/// End a capture session and increment frame count (stub).
#[allow(dead_code)]
pub fn end_capture(cap: &mut RenderDocCapture) {
    if cap.capturing {
        cap.config.frame_count += 1;
        cap.capturing = false;
    }
}

/// Return whether capturing is enabled.
#[allow(dead_code)]
pub fn capture_enabled(cap: &RenderDocCapture) -> bool {
    cap.config.enabled
}

/// Return the number of captured frames.
#[allow(dead_code)]
pub fn capture_frame_count(cap: &RenderDocCapture) -> u32 {
    cap.config.frame_count
}

/// Return the output path.
#[allow(dead_code)]
pub fn capture_output_path(cap: &RenderDocCapture) -> &str {
    &cap.config.output_path
}

/// Return the API version.
#[allow(dead_code)]
pub fn capture_api_version(cap: &RenderDocCapture) -> u32 {
    cap.config.api_version
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_capture() -> RenderDocCapture {
        RenderDocCapture { config: default_capture_config(), capturing: false }
    }

    #[test]
    fn test_default_config() {
        let cfg = default_capture_config();
        assert!(capture_enabled(&RenderDocCapture { config: cfg, capturing: false }));
    }

    #[test]
    fn test_new_capture_config() {
        let cfg = new_capture_config("out/", 2);
        assert_eq!(cfg.api_version, 2);
        assert_eq!(cfg.output_path, "out/");
    }

    #[test]
    fn test_begin_end_capture() {
        let mut cap = make_capture();
        begin_capture(&mut cap);
        assert!(cap.capturing);
        end_capture(&mut cap);
        assert!(!cap.capturing);
        assert_eq!(capture_frame_count(&cap), 1);
    }

    #[test]
    fn test_capture_enabled() {
        let cap = make_capture();
        assert!(capture_enabled(&cap));
    }

    #[test]
    fn test_capture_frame_count_initial() {
        let cap = make_capture();
        assert_eq!(capture_frame_count(&cap), 0);
    }

    #[test]
    fn test_capture_output_path() {
        let cap = make_capture();
        assert!(!capture_output_path(&cap).is_empty());
    }

    #[test]
    fn test_capture_api_version() {
        let cap = make_capture();
        assert!(capture_api_version(&cap) >= 1);
    }

    #[test]
    fn test_multiple_captures() {
        let mut cap = make_capture();
        for _ in 0..3 {
            begin_capture(&mut cap);
            end_capture(&mut cap);
        }
        assert_eq!(capture_frame_count(&cap), 3);
    }

    #[test]
    fn test_end_without_begin_no_effect() {
        let mut cap = make_capture();
        end_capture(&mut cap);
        assert_eq!(capture_frame_count(&cap), 0);
    }
}
