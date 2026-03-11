// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A blended animation clip entry.
#[allow(dead_code)]
pub struct BlendClipEntry {
    pub name: String,
    pub weight: f32,
    pub start: f32,
    pub end: f32,
}

/// Export bundle for blended anim clips.
#[allow(dead_code)]
#[derive(Default)]
pub struct AnimClipBlendExport {
    pub clips: Vec<BlendClipEntry>,
}

/// Create a new anim clip blend export.
#[allow(dead_code)]
pub fn new_anim_clip_blend_export() -> AnimClipBlendExport {
    AnimClipBlendExport::default()
}

/// Add a blend clip entry.
#[allow(dead_code)]
pub fn add_blend_clip(
    export: &mut AnimClipBlendExport,
    name: &str,
    weight: f32,
    start: f32,
    end: f32,
) {
    export.clips.push(BlendClipEntry {
        name: name.to_string(),
        weight,
        start,
        end,
    });
}

/// Count blend clips.
#[allow(dead_code)]
pub fn blend_clip_count(export: &AnimClipBlendExport) -> usize {
    export.clips.len()
}

/// Sum of all weights.
#[allow(dead_code)]
pub fn total_blend_weight(export: &AnimClipBlendExport) -> f32 {
    export.clips.iter().map(|c| c.weight).sum()
}

/// Normalize blend weights to sum to 1.
#[allow(dead_code)]
pub fn normalize_blend_weights(export: &mut AnimClipBlendExport) {
    let sum = total_blend_weight(export);
    if sum > 1e-9 {
        for c in &mut export.clips {
            c.weight /= sum;
        }
    }
}

/// Maximum clip duration.
#[allow(dead_code)]
pub fn max_clip_duration(export: &AnimClipBlendExport) -> f32 {
    export
        .clips
        .iter()
        .map(|c| c.end - c.start)
        .fold(0.0_f32, f32::max)
}

/// Find clip by name.
#[allow(dead_code)]
pub fn find_blend_clip<'a>(
    export: &'a AnimClipBlendExport,
    name: &str,
) -> Option<&'a BlendClipEntry> {
    export.clips.iter().find(|c| c.name == name)
}

/// Validate that all weights are in [0, 1].
#[allow(dead_code)]
pub fn validate_blend_clips(export: &AnimClipBlendExport) -> bool {
    export
        .clips
        .iter()
        .all(|c| (0.0..=1.0).contains(&c.weight) && c.end >= c.start)
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn anim_clip_blend_to_json(export: &AnimClipBlendExport) -> String {
    format!(
        r#"{{"blend_clips":{},"total_weight":{:.4}}}"#,
        export.clips.len(),
        total_blend_weight(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_count() {
        let mut e = new_anim_clip_blend_export();
        add_blend_clip(&mut e, "walk", 0.5, 0.0, 1.0);
        assert_eq!(blend_clip_count(&e), 1);
    }

    #[test]
    fn total_weight() {
        let mut e = new_anim_clip_blend_export();
        add_blend_clip(&mut e, "a", 0.3, 0.0, 1.0);
        add_blend_clip(&mut e, "b", 0.7, 0.0, 1.0);
        assert!((total_blend_weight(&e) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn normalize() {
        let mut e = new_anim_clip_blend_export();
        add_blend_clip(&mut e, "a", 2.0, 0.0, 1.0);
        add_blend_clip(&mut e, "b", 2.0, 0.0, 1.0);
        normalize_blend_weights(&mut e);
        assert!((total_blend_weight(&e) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn max_duration() {
        let mut e = new_anim_clip_blend_export();
        add_blend_clip(&mut e, "a", 0.5, 0.0, 2.0);
        add_blend_clip(&mut e, "b", 0.5, 0.0, 3.0);
        assert!((max_clip_duration(&e) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn find_clip() {
        let mut e = new_anim_clip_blend_export();
        add_blend_clip(&mut e, "run", 1.0, 0.0, 1.0);
        assert!(find_blend_clip(&e, "run").is_some());
    }

    #[test]
    fn find_missing() {
        let e = new_anim_clip_blend_export();
        assert!(find_blend_clip(&e, "missing").is_none());
    }

    #[test]
    fn validate_valid() {
        let mut e = new_anim_clip_blend_export();
        add_blend_clip(&mut e, "a", 0.5, 0.0, 1.0);
        assert!(validate_blend_clips(&e));
    }

    #[test]
    fn json_has_count() {
        let mut e = new_anim_clip_blend_export();
        add_blend_clip(&mut e, "a", 1.0, 0.0, 1.0);
        let j = anim_clip_blend_to_json(&e);
        assert!(j.contains("\"blend_clips\":1"));
    }

    #[test]
    fn empty_default() {
        let e = new_anim_clip_blend_export();
        assert_eq!(blend_clip_count(&e), 0);
    }

    #[test]
    fn normalize_zero_sum_safe() {
        let mut e = new_anim_clip_blend_export();
        // no clips, sum is 0 - should not panic
        normalize_blend_weights(&mut e);
    }
}
