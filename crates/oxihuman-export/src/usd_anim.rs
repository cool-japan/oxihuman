// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use std::path::Path;

use anyhow::Context;
use oxihuman_mesh::MeshBuffers;

// ── Data types ────────────────────────────────────────────────────────────────

/// Configuration for USD animation export.
pub struct UsdAnimConfig {
    pub start_time: f32,
    pub end_time: f32,
    pub fps: f32,
    pub meters_per_unit: f32,
}

impl Default for UsdAnimConfig {
    fn default() -> Self {
        Self {
            start_time: 0.0,
            end_time: 1.0,
            fps: 24.0,
            meters_per_unit: 1.0,
        }
    }
}

/// A single time-sampled position frame.
pub struct UsdTimeSample {
    pub time: f32,
    pub positions: Vec<[f32; 3]>,
}

// ── Formatting helpers ────────────────────────────────────────────────────────

/// Format a position array as USD point list: `[(x, y, z), ...]`
pub fn format_usda_point_array(positions: &[[f32; 3]]) -> String {
    let inner: Vec<String> = positions
        .iter()
        .map(|p| format!("({:.6}, {:.6}, {:.6})", p[0], p[1], p[2]))
        .collect();
    format!("[{}]", inner.join(", "))
}

/// Build a `attr.timeSamples = { time: [...], ... }` block.
pub fn build_usda_time_samples_block(attr_name: &str, samples: &[UsdTimeSample]) -> String {
    if samples.is_empty() {
        return format!("{}.timeSamples = {{}}", attr_name);
    }

    let mut lines: Vec<String> = Vec::new();
    for sample in samples {
        // USD time codes are typically integer or fractional frame numbers
        let time_code = sample.time;
        let pts = format_usda_point_array(&sample.positions);
        lines.push(format!("            {:.4}: {},", time_code, pts));
    }

    format!(
        "{}.timeSamples = {{\n{}\n        }}",
        attr_name,
        lines.join("\n")
    )
}

// ── Main build function ───────────────────────────────────────────────────────

/// Produce a .usda file string with `def Mesh` + animated `points.timeSamples`.
pub fn build_usda_animated(
    mesh: &MeshBuffers,
    samples: &[UsdTimeSample],
    cfg: &UsdAnimConfig,
) -> String {
    let start_frame = cfg.start_time * cfg.fps;
    let end_frame = cfg.end_time * cfg.fps;

    let face_count = mesh.indices.len() / 3;

    // Build face vertex counts (all triangles = 3)
    let fvc: Vec<String> = vec!["3".to_string(); face_count];
    let fvi: Vec<String> = mesh.indices.iter().map(|i| i.to_string()).collect();

    // Static normals from first-frame mesh
    let normals_str = {
        let inner: Vec<String> = mesh
            .normals
            .iter()
            .map(|n| format!("({:.6}, {:.6}, {:.6})", n[0], n[1], n[2]))
            .collect();
        format!("[{}]", inner.join(", "))
    };

    // UV coordinates
    let uv_str = {
        let inner: Vec<String> = mesh
            .uvs
            .iter()
            .map(|uv| format!("({:.6}, {:.6})", uv[0], uv[1]))
            .collect();
        format!("[{}]", inner.join(", "))
    };

    let time_samples_block = build_usda_time_samples_block("points", samples);

    format!(
        r#"#usda 1.0
(
    defaultPrim = "Root"
    metersPerUnit = {meters_per_unit:.4}
    startTimeCode = {start:.4}
    endTimeCode = {end:.4}
    timeCodesPerSecond = {fps:.4}
    upAxis = "Y"
)

def Xform "Root"
{{
    def Mesh "Body"
    {{
        int[] faceVertexCounts = [{fvc}]
        int[] faceVertexIndices = [{fvi}]
        normal3f[] normals = {normals}
        texCoord2f[] primvars:st = {uvs} (
            interpolation = "vertex"
        )
        {time_samples}
    }}
}}
"#,
        meters_per_unit = cfg.meters_per_unit,
        start = start_frame,
        end = end_frame,
        fps = cfg.fps,
        fvc = fvc.join(", "),
        fvi = fvi.join(", "),
        normals = normals_str,
        uvs = uv_str,
        time_samples = time_samples_block,
    )
}

/// Write animated USDA to a file path.
pub fn export_usda_animated(
    mesh: &MeshBuffers,
    samples: &[UsdTimeSample],
    cfg: &UsdAnimConfig,
    path: &Path,
) -> anyhow::Result<()> {
    let content = build_usda_animated(mesh, samples, cfg);
    std::fs::write(path, content.as_bytes())
        .with_context(|| format!("Failed to write USD anim to {}", path.display()))?;
    Ok(())
}

/// Generate a sequence of stub time samples (all identical positions) for testing.
pub fn uniform_time_samples(
    base_positions: &[[f32; 3]],
    cfg: &UsdAnimConfig,
) -> Vec<UsdTimeSample> {
    let frame_count = ((cfg.end_time - cfg.start_time) * cfg.fps).ceil() as usize;
    let frame_count = frame_count.max(1);

    (0..frame_count)
        .map(|i| {
            let t = cfg.start_time + i as f32 / cfg.fps;
            UsdTimeSample {
                time: t,
                positions: base_positions.to_vec(),
            }
        })
        .collect()
}

/// Return a human-readable summary of the sample sequence.
pub fn usd_anim_stats(samples: &[UsdTimeSample]) -> String {
    if samples.is_empty() {
        return "UsdAnim: 0 samples".to_string();
    }
    let frame_count = samples.len();
    let vert_count = samples[0].positions.len();
    let t_start = samples.first().map(|s| s.time).unwrap_or(0.0);
    let t_end = samples.last().map(|s| s.time).unwrap_or(0.0);
    format!(
        "UsdAnim: frames={}, vertices_per_frame={}, time=[{:.3}..{:.3}]",
        frame_count, vert_count, t_start, t_end
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn stub_mesh() -> MeshBuffers {
        MeshBuffers {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            colors: None,
            has_suit: false,
        }
    }

    fn stub_cfg() -> UsdAnimConfig {
        UsdAnimConfig {
            start_time: 0.0,
            end_time: 1.0,
            fps: 24.0,
            meters_per_unit: 1.0,
        }
    }

    fn stub_samples(mesh: &MeshBuffers, n: usize) -> Vec<UsdTimeSample> {
        (0..n)
            .map(|i| UsdTimeSample {
                time: i as f32 / 24.0,
                positions: mesh.positions.clone(),
            })
            .collect()
    }

    #[test]
    fn build_usda_animated_contains_time_samples() {
        let mesh = stub_mesh();
        let cfg = stub_cfg();
        let samples = stub_samples(&mesh, 3);
        let usda = build_usda_animated(&mesh, &samples, &cfg);
        assert!(
            usda.contains("timeSamples"),
            "USDA must contain timeSamples"
        );
    }

    #[test]
    fn build_usda_animated_contains_mesh_prim() {
        let mesh = stub_mesh();
        let cfg = stub_cfg();
        let usda = build_usda_animated(&mesh, &[], &cfg);
        assert!(usda.contains("def Mesh"), "USDA must contain def Mesh");
        assert!(usda.contains("Body"), "USDA must contain Body prim");
    }

    #[test]
    fn build_usda_animated_contains_root_xform() {
        let mesh = stub_mesh();
        let cfg = stub_cfg();
        let usda = build_usda_animated(&mesh, &[], &cfg);
        assert!(usda.contains("def Xform \"Root\""));
    }

    #[test]
    fn format_usda_point_array_format_check() {
        let pts = &[[1.0f32, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let s = format_usda_point_array(pts);
        assert!(s.starts_with('['));
        assert!(s.ends_with(']'));
        assert!(s.contains("(1.000000, 2.000000, 3.000000)"));
        assert!(s.contains("(4.000000, 5.000000, 6.000000)"));
    }

    #[test]
    fn format_usda_point_array_empty() {
        let s = format_usda_point_array(&[]);
        assert_eq!(s, "[]");
    }

    #[test]
    fn build_usda_time_samples_block_has_both_times() {
        let samples = vec![
            UsdTimeSample {
                time: 0.0,
                positions: vec![[0.0, 0.0, 0.0]],
            },
            UsdTimeSample {
                time: 1.0,
                positions: vec![[1.0, 0.0, 0.0]],
            },
        ];
        let block = build_usda_time_samples_block("points", &samples);
        assert!(block.contains("0.0000"), "block must contain t=0");
        assert!(block.contains("1.0000"), "block must contain t=1");
        assert!(block.contains("timeSamples"));
    }

    #[test]
    fn build_usda_time_samples_block_empty() {
        let block = build_usda_time_samples_block("points", &[]);
        assert!(block.contains("timeSamples"));
        assert!(block.contains("{}"));
    }

    #[test]
    fn uniform_time_samples_frame_count() {
        let positions = vec![[0.0f32, 0.0, 0.0]];
        let cfg = UsdAnimConfig {
            start_time: 0.0,
            end_time: 1.0,
            fps: 24.0,
            meters_per_unit: 1.0,
        };
        let samples = uniform_time_samples(&positions, &cfg);
        // ceil((1.0 - 0.0) * 24.0) = 24
        assert_eq!(samples.len(), 24);
    }

    #[test]
    fn uniform_time_samples_fractional_ceil() {
        let positions = vec![[0.0f32, 0.0, 0.0]];
        let cfg = UsdAnimConfig {
            start_time: 0.0,
            end_time: 0.5,
            fps: 24.0,
            meters_per_unit: 1.0,
        };
        let samples = uniform_time_samples(&positions, &cfg);
        // ceil(0.5 * 24) = ceil(12) = 12
        assert_eq!(samples.len(), 12);
    }

    #[test]
    fn uniform_time_samples_identical_positions() {
        let positions = vec![[1.0f32, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let cfg = stub_cfg();
        let samples = uniform_time_samples(&positions, &cfg);
        for s in &samples {
            assert_eq!(s.positions, positions);
        }
    }

    #[test]
    fn usd_anim_stats_non_empty() {
        let mesh = stub_mesh();
        let samples = stub_samples(&mesh, 5);
        let s = usd_anim_stats(&samples);
        assert!(!s.is_empty());
        assert!(s.contains("5"));
    }

    #[test]
    fn usd_anim_stats_empty_samples() {
        let s = usd_anim_stats(&[]);
        assert!(!s.is_empty());
        assert!(s.contains('0'));
    }

    #[test]
    fn usd_anim_stats_single_sample() {
        let sample = UsdTimeSample {
            time: 0.5,
            positions: vec![[0.0f32, 0.0, 0.0]; 4],
        };
        let s = usd_anim_stats(&[sample]);
        assert!(s.contains("frames=1"));
        assert!(s.contains("vertices_per_frame=4"));
    }

    #[test]
    fn export_usda_animated_writes_file() {
        let mesh = stub_mesh();
        let cfg = stub_cfg();
        let samples = stub_samples(&mesh, 2);
        let path = std::path::Path::new("/tmp/test_usd_anim_export.usda");
        export_usda_animated(&mesh, &samples, &cfg, path).expect("should write file");
        assert!(path.exists());
        let content = std::fs::read_to_string(path).expect("should succeed");
        assert!(content.contains("timeSamples"));
    }
}
