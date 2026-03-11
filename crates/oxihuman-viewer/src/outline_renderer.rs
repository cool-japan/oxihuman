//! Silhouette outline and edge detection renderer.

// ── Enums ─────────────────────────────────────────────────────────────────────

/// Algorithm used to detect outline edges.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum OutlineMethod {
    SobelEdge,
    NormalDiff,
    DepthDiff,
    Inverted,
}

// ── Structs ───────────────────────────────────────────────────────────────────

/// Configuration for outline rendering.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OutlineConfig {
    pub method: OutlineMethod,
    pub thickness: f32,
    pub color: [f32; 4],
    pub depth_threshold: f32,
    pub normal_threshold: f32,
}

/// CPU-side buffer that stores per-pixel outline mask.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OutlineBuffer {
    pub width: u32,
    pub height: u32,
    pub outline_mask: Vec<bool>,
}

/// Result produced by outline detection.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OutlineResult {
    pub buffer: OutlineBuffer,
    pub outline_pixel_count: usize,
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Returns a sensible default `OutlineConfig`.
#[allow(dead_code)]
pub fn default_outline_config() -> OutlineConfig {
    OutlineConfig {
        method: OutlineMethod::SobelEdge,
        thickness: 1.0,
        color: [0.0, 0.0, 0.0, 1.0],
        depth_threshold: 0.01,
        normal_threshold: 0.5,
    }
}

/// Allocates a new `OutlineBuffer` filled with `false`.
#[allow(dead_code)]
pub fn new_outline_buffer(w: u32, h: u32) -> OutlineBuffer {
    OutlineBuffer {
        width: w,
        height: h,
        outline_mask: vec![false; (w * h) as usize],
    }
}

/// Detects outlines from depth and normal buffers.
#[allow(dead_code)]
pub fn detect_outlines(
    depth: &[f32],
    normals: &[[f32; 3]],
    w: u32,
    h: u32,
    cfg: &OutlineConfig,
) -> OutlineResult {
    let mut buf = new_outline_buffer(w, h);

    match cfg.method {
        OutlineMethod::SobelEdge => {
            let edge_strengths = sobel_edge_detect(depth, w, h);
            for (i, &s) in edge_strengths.iter().enumerate() {
                if s > cfg.depth_threshold && i < buf.outline_mask.len() {
                    buf.outline_mask[i] = true;
                }
            }
        }
        OutlineMethod::DepthDiff => {
            for y in 0..h {
                for x in 0..w {
                    let idx = (y * w + x) as usize;
                    if idx >= depth.len() {
                        continue;
                    }
                    let d = depth[idx];
                    let right = if x + 1 < w && idx + 1 < depth.len() {
                        depth[idx + 1]
                    } else {
                        d
                    };
                    let down = if y + 1 < h && idx + (w as usize) < depth.len() {
                        depth[idx + w as usize]
                    } else {
                        d
                    };
                    if (d - right).abs() > cfg.depth_threshold
                        || (d - down).abs() > cfg.depth_threshold
                    {
                        buf.outline_mask[idx] = true;
                    }
                }
            }
        }
        OutlineMethod::NormalDiff => {
            for y in 0..h {
                for x in 0..w {
                    let idx = (y * w + x) as usize;
                    if idx >= normals.len() {
                        continue;
                    }
                    let n = normals[idx];
                    let right_idx = (y * w + (x + 1).min(w - 1)) as usize;
                    let down_idx = ((y + 1).min(h - 1) * w + x) as usize;
                    let n_r = if right_idx < normals.len() { normals[right_idx] } else { n };
                    let n_d = if down_idx < normals.len() { normals[down_idx] } else { n };
                    let dot_r = n[0] * n_r[0] + n[1] * n_r[1] + n[2] * n_r[2];
                    let dot_d = n[0] * n_d[0] + n[1] * n_d[1] + n[2] * n_d[2];
                    if dot_r < cfg.normal_threshold || dot_d < cfg.normal_threshold {
                        buf.outline_mask[idx] = true;
                    }
                }
            }
        }
        OutlineMethod::Inverted => {
            // Mark everything as outline, then invert (for illustration)
            for v in buf.outline_mask.iter_mut() {
                *v = true;
            }
        }
    }

    let count = outline_pixel_count(&buf);
    OutlineResult {
        buffer: buf,
        outline_pixel_count: count,
    }
}

/// Applies a Sobel edge-detection filter to a depth buffer.
#[allow(dead_code)]
pub fn sobel_edge_detect(data: &[f32], w: u32, h: u32) -> Vec<f32> {
    let ww = w as usize;
    let hh = h as usize;
    let mut result = vec![0.0f32; ww * hh];

    let sample = |x: isize, y: isize| -> f32 {
        if x < 0 || y < 0 || x >= ww as isize || y >= hh as isize {
            return 0.0;
        }
        let idx = (y as usize) * ww + (x as usize);
        if idx < data.len() { data[idx] } else { 0.0 }
    };

    for y in 0..hh {
        for x in 0..ww {
            let xi = x as isize;
            let yi = y as isize;
            let gx = -sample(xi - 1, yi - 1)
                + sample(xi + 1, yi - 1)
                - 2.0 * sample(xi - 1, yi)
                + 2.0 * sample(xi + 1, yi)
                - sample(xi - 1, yi + 1)
                + sample(xi + 1, yi + 1);
            let gy = -sample(xi - 1, yi - 1)
                - 2.0 * sample(xi, yi - 1)
                - sample(xi + 1, yi - 1)
                + sample(xi - 1, yi + 1)
                + 2.0 * sample(xi, yi + 1)
                + sample(xi + 1, yi + 1);
            result[y * ww + x] = (gx * gx + gy * gy).sqrt();
        }
    }
    result
}

/// Returns the outline state of a single pixel.
#[allow(dead_code)]
pub fn outline_at(buf: &OutlineBuffer, x: u32, y: u32) -> bool {
    let idx = (y * buf.width + x) as usize;
    buf.outline_mask.get(idx).copied().unwrap_or(false)
}

/// Sets the outline state of a single pixel.
#[allow(dead_code)]
pub fn set_outline_pixel(buf: &mut OutlineBuffer, x: u32, y: u32, val: bool) {
    let idx = (y * buf.width + x) as usize;
    if idx < buf.outline_mask.len() {
        buf.outline_mask[idx] = val;
    }
}

/// Returns the total number of outline pixels.
#[allow(dead_code)]
pub fn outline_pixel_count(buf: &OutlineBuffer) -> usize {
    buf.outline_mask.iter().filter(|&&v| v).count()
}

/// Serialises the config to a JSON string.
#[allow(dead_code)]
pub fn outline_config_to_json(cfg: &OutlineConfig) -> String {
    format!(
        "{{\"method\":\"{}\",\"thickness\":{:.4},\"depth_threshold\":{:.6},\"normal_threshold\":{:.6}}}",
        outline_method_name(cfg),
        cfg.thickness,
        cfg.depth_threshold,
        cfg.normal_threshold,
    )
}

/// Returns the name of the outline method.
#[allow(dead_code)]
pub fn outline_method_name(cfg: &OutlineConfig) -> &'static str {
    match cfg.method {
        OutlineMethod::SobelEdge => "sobel_edge",
        OutlineMethod::NormalDiff => "normal_diff",
        OutlineMethod::DepthDiff => "depth_diff",
        OutlineMethod::Inverted => "inverted",
    }
}

/// Dilates the outline mask by one pixel in all four cardinal directions.
#[allow(dead_code)]
pub fn dilate_outline(buf: &mut OutlineBuffer) {
    let w = buf.width as usize;
    let h = buf.height as usize;
    let original = buf.outline_mask.clone();
    for y in 0..h {
        for x in 0..w {
            if original[y * w + x] {
                if x + 1 < w {
                    buf.outline_mask[y * w + x + 1] = true;
                }
                if x > 0 {
                    buf.outline_mask[y * w + x - 1] = true;
                }
                if y + 1 < h {
                    buf.outline_mask[(y + 1) * w + x] = true;
                }
                if y > 0 {
                    buf.outline_mask[(y - 1) * w + x] = true;
                }
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_sobel() {
        let cfg = default_outline_config();
        assert_eq!(cfg.method, OutlineMethod::SobelEdge);
        assert!((cfg.thickness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_buffer_all_false() {
        let buf = new_outline_buffer(4, 4);
        assert_eq!(buf.outline_mask.len(), 16);
        assert!(buf.outline_mask.iter().all(|&v| !v));
    }

    #[test]
    fn set_and_get_pixel() {
        let mut buf = new_outline_buffer(8, 8);
        set_outline_pixel(&mut buf, 3, 2, true);
        assert!(outline_at(&buf, 3, 2));
        assert!(!outline_at(&buf, 0, 0));
    }

    #[test]
    fn pixel_count_after_set() {
        let mut buf = new_outline_buffer(4, 4);
        set_outline_pixel(&mut buf, 1, 1, true);
        set_outline_pixel(&mut buf, 2, 2, true);
        assert_eq!(outline_pixel_count(&buf), 2);
    }

    #[test]
    fn dilate_expands_outline() {
        let mut buf = new_outline_buffer(5, 5);
        set_outline_pixel(&mut buf, 2, 2, true);
        let before = outline_pixel_count(&buf);
        dilate_outline(&mut buf);
        let after = outline_pixel_count(&buf);
        assert!(after > before);
    }

    #[test]
    fn method_names_correct() {
        let methods = [
            (OutlineMethod::SobelEdge, "sobel_edge"),
            (OutlineMethod::NormalDiff, "normal_diff"),
            (OutlineMethod::DepthDiff, "depth_diff"),
            (OutlineMethod::Inverted, "inverted"),
        ];
        for (m, expected) in methods {
            let cfg = OutlineConfig {
                method: m,
                thickness: 1.0,
                color: [0.0; 4],
                depth_threshold: 0.01,
                normal_threshold: 0.5,
            };
            assert_eq!(outline_method_name(&cfg), expected);
        }
    }

    #[test]
    fn detect_outlines_inverted_marks_all() {
        let depth = vec![0.5f32; 9];
        let normals = vec![[0.0f32, 1.0, 0.0]; 9];
        let mut cfg = default_outline_config();
        cfg.method = OutlineMethod::Inverted;
        let result = detect_outlines(&depth, &normals, 3, 3, &cfg);
        assert_eq!(result.outline_pixel_count, 9);
    }

    #[test]
    fn config_to_json_contains_method() {
        let cfg = default_outline_config();
        let json = outline_config_to_json(&cfg);
        assert!(json.contains("sobel_edge"));
    }
}
