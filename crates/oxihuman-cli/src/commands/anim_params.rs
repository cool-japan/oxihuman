// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Time-varying morph-parameter sources for the `pc2` / `mdd` / `anim-bake`
//! animation export commands.
//!
//! Two JSON shapes are accepted by [`load_anim_source`]:
//!
//! 1. **Dense snapshot array** — one fully-resolved parameter object per
//!    output frame, e.g. `[{"height":0.0,"weight":0.5}, {"height":0.5,...}]`.
//!    This is the format produced by `WasmEngine::export_anim_json` (see
//!    `oxihuman-wasm/src/engine_anim.rs`). The array length fixes the frame
//!    count; unspecified fields fall back to [`ParamState::default`] and any
//!    unrecognised keys are stored in [`ParamState::extra`].
//!
//! 2. **Keyframed animation curve** — `{"interp":"linear","keyframes":[
//!    {"time":0.0,"params":{"height":0.0}},
//!    {"time":1.0,"params":{"height":1.0}}]}`. Each named parameter is
//!    interpolated independently via [`oxihuman_core::animation_curve::AnimCurve`]
//!    and sampled at `t = start_time + frame_index / fps`, matching the
//!    record/seek/play semantics mirrored in `oxihuman-wasm`'s `engine_anim`.
//!    Optional per-keyframe `tan_in` / `tan_out` objects supply Hermite
//!    tangents for `"interp":"cubic"`.

use std::collections::HashMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde_json::Value;

use oxihuman_core::animation_curve::{AnimCurve, InterpMode, Keyframe};
use oxihuman_morph::params::ParamState;

/// A source of time-varying morph parameters for baking animated frame
/// sequences. See the module docs for the accepted JSON shapes.
#[derive(Debug, Clone)]
pub enum AnimSource {
    /// One fully-resolved [`ParamState`] per output frame (frame count is fixed).
    Snapshots(Vec<ParamState>),
    /// Continuous per-parameter animation curves, keyed by parameter name,
    /// sampled at an arbitrary time in seconds.
    Curves(HashMap<String, AnimCurve>),
}

impl AnimSource {
    /// The frame count implied by this source, if it is fixed (dense form only).
    pub fn fixed_frame_count(&self) -> Option<usize> {
        match self {
            AnimSource::Snapshots(snaps) => Some(snaps.len()),
            AnimSource::Curves(_) => None,
        }
    }

    /// A reasonable default frame count for curve sources: enough frames at
    /// `fps` to cover the longest-running parameter curve, at least one frame.
    pub fn suggested_frame_count(&self, fps: f32) -> usize {
        match self {
            AnimSource::Snapshots(snaps) => snaps.len().max(1),
            AnimSource::Curves(curves) => {
                let max_duration = curves
                    .values()
                    .map(|c| c.duration())
                    .fold(0.0_f32, f32::max);
                let safe_fps = if fps > 0.0 { fps } else { 1.0 };
                ((max_duration * safe_fps).round() as usize + 1).max(1)
            }
        }
    }

    /// Resolve the [`ParamState`] to apply for output frame `i`.
    ///
    /// For the dense (snapshot) form, `i` indexes directly into the array
    /// (clamped to the last entry so callers never index out of range). For
    /// the curve form, each named parameter curve is sampled at
    /// `t = start_time + i / fps`.
    pub fn params_at_frame(&self, i: usize, fps: f32, start_time: f32) -> ParamState {
        match self {
            AnimSource::Snapshots(snaps) => {
                let idx = i.min(snaps.len().saturating_sub(1));
                snaps.get(idx).cloned().unwrap_or_else(ParamState::default)
            }
            AnimSource::Curves(curves) => {
                let safe_fps = if fps > 0.0 { fps } else { 1.0 };
                let t = start_time + (i as f32) / safe_fps;
                let mut params = ParamState::default();
                for (name, curve) in curves {
                    apply_named_value(&mut params, name, curve.evaluate(t));
                }
                params
            }
        }
    }
}

/// Load an [`AnimSource`] from a JSON file on disk.
pub fn load_anim_source(path: &Path) -> Result<AnimSource> {
    let src = std::fs::read_to_string(path)
        .with_context(|| format!("reading anim JSON: {}", path.display()))?;
    let value: Value = serde_json::from_str(&src)
        .with_context(|| format!("parsing anim JSON: {}", path.display()))?;
    parse_anim_value(&value)
}

/// Write `value` into the matching field of `params` (built-ins get their
/// dedicated field, everything else lands in `extra`).
fn apply_named_value(params: &mut ParamState, name: &str, value: f32) {
    match name {
        "height" => params.height = value,
        "weight" => params.weight = value,
        "muscle" => params.muscle = value,
        "age" => params.age = value,
        other => {
            params.extra.insert(other.to_string(), value);
        }
    }
}

/// A dense-form array element is any object that is *not* a keyframe wrapper
/// (i.e. it lacks the `time` + `params` pairing used by the curve form).
fn looks_like_keyframe(v: &Value) -> bool {
    matches!(v, Value::Object(map) if map.contains_key("time") && map.contains_key("params"))
}

/// Parse a lenient parameter snapshot: known fields (`height`/`weight`/
/// `muscle`/`age`) override the default, unknown numeric fields become
/// `extra` entries. Non-numeric values for a known/unknown key are an error.
fn snapshot_from_map(map: &serde_json::Map<String, Value>) -> Result<ParamState> {
    let mut params = ParamState::default();
    for (key, value) in map {
        let f = value
            .as_f64()
            .with_context(|| format!("param '{}' must be a number", key))? as f32;
        apply_named_value(&mut params, key, f);
    }
    Ok(params)
}

fn parse_anim_value(value: &Value) -> Result<AnimSource> {
    match value {
        Value::Array(entries) => {
            if entries.is_empty() {
                bail!("anim JSON array must not be empty");
            }
            if entries.iter().all(looks_like_keyframe) {
                Ok(AnimSource::Curves(build_curves(
                    entries,
                    InterpMode::Linear,
                )?))
            } else {
                let snapshots = entries
                    .iter()
                    .map(|entry| {
                        entry
                            .as_object()
                            .context("each anim JSON array entry must be an object")
                            .and_then(snapshot_from_map)
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(AnimSource::Snapshots(snapshots))
            }
        }
        Value::Object(map) => {
            let keyframes = map
                .get("keyframes")
                .and_then(Value::as_array)
                .context("anim JSON object must contain a \"keyframes\" array")?;
            if keyframes.is_empty() {
                bail!("\"keyframes\" array must not be empty");
            }
            let interp = match map.get("interp").and_then(Value::as_str) {
                None | Some("linear") => InterpMode::Linear,
                Some("step") => InterpMode::Step,
                Some("cubic") => InterpMode::Cubic,
                Some(other) => bail!("unknown interp mode '{}': use linear|step|cubic", other),
            };
            Ok(AnimSource::Curves(build_curves(keyframes, interp)?))
        }
        _ => bail!("anim JSON must be an array or an object with a \"keyframes\" array"),
    }
}

/// Build one [`AnimCurve`] per named parameter from a list of keyframe
/// objects `{"time":f32,"params":{name: value, ...},"tan_in":{...},"tan_out":{...}}`.
fn build_curves(entries: &[Value], interp: InterpMode) -> Result<HashMap<String, AnimCurve>> {
    let mut curves: HashMap<String, AnimCurve> = HashMap::new();
    for entry in entries {
        let obj = entry
            .as_object()
            .context("each keyframe must be a JSON object")?;
        let time = obj
            .get("time")
            .and_then(Value::as_f64)
            .context("keyframe missing numeric \"time\"")? as f32;
        let params = obj
            .get("params")
            .and_then(Value::as_object)
            .context("keyframe missing \"params\" object")?;
        let tan_in = obj.get("tan_in").and_then(Value::as_object);
        let tan_out = obj.get("tan_out").and_then(Value::as_object);

        for (name, raw_value) in params {
            let value = raw_value
                .as_f64()
                .with_context(|| format!("keyframe param '{}' must be a number", name))?
                as f32;
            let t_in = tan_in
                .and_then(|m| m.get(name))
                .and_then(Value::as_f64)
                .unwrap_or(0.0) as f32;
            let t_out = tan_out
                .and_then(|m| m.get(name))
                .and_then(Value::as_f64)
                .unwrap_or(0.0) as f32;
            curves
                .entry(name.clone())
                .or_insert_with(|| AnimCurve::new(interp))
                .insert(Keyframe::with_tangents(time, value, t_in, t_out));
        }
    }
    Ok(curves)
}

#[cfg(test)]
mod tests {
    use super::*;

    // 1. dense array with full snapshots parses and preserves values.
    #[test]
    fn dense_snapshots_parse() {
        let v: Value = serde_json::from_str(
            r#"[{"height":0.1,"weight":0.5,"muscle":0.5,"age":0.5},
                {"height":0.9,"weight":0.5,"muscle":0.5,"age":0.5}]"#,
        )
        .expect("valid json");
        let src = parse_anim_value(&v).expect("should parse");
        assert_eq!(src.fixed_frame_count(), Some(2));
        let p0 = src.params_at_frame(0, 24.0, 0.0);
        let p1 = src.params_at_frame(1, 24.0, 0.0);
        assert!((p0.height - 0.1).abs() < 1e-6);
        assert!((p1.height - 0.9).abs() < 1e-6);
    }

    // 2. dense array with partial fields defaults the rest and keeps extras.
    #[test]
    fn dense_snapshots_partial_fields_default_and_extra() {
        let v: Value =
            serde_json::from_str(r#"[{"height":0.2,"custom":0.75}]"#).expect("valid json");
        let src = parse_anim_value(&v).expect("should parse");
        let p = src.params_at_frame(0, 24.0, 0.0);
        assert!((p.height - 0.2).abs() < 1e-6);
        assert!((p.weight - ParamState::default().weight).abs() < 1e-6);
        assert_eq!(p.extra.get("custom").copied(), Some(0.75));
    }

    // 3. snapshot index clamps to the last frame rather than panicking.
    #[test]
    fn dense_snapshots_index_clamps() {
        let v: Value = serde_json::from_str(r#"[{"height":0.3}]"#).expect("valid json");
        let src = parse_anim_value(&v).expect("should parse");
        let p = src.params_at_frame(50, 24.0, 0.0);
        assert!((p.height - 0.3).abs() < 1e-6);
    }

    // 4. keyframe object form builds a sampleable curve.
    #[test]
    fn keyframe_object_form_samples_linearly() {
        let v: Value = serde_json::from_str(
            r#"{"interp":"linear","keyframes":[
                {"time":0.0,"params":{"height":0.0}},
                {"time":1.0,"params":{"height":1.0}}]}"#,
        )
        .expect("valid json");
        let src = parse_anim_value(&v).expect("should parse");
        assert_eq!(src.fixed_frame_count(), None);
        let p_mid = src.params_at_frame(12, 24.0, 0.0); // t = 0.5s
        assert!((p_mid.height - 0.5).abs() < 1e-3);
    }

    // 5. step interpolation holds the previous keyframe's value.
    #[test]
    fn keyframe_step_holds_value() {
        let v: Value = serde_json::from_str(
            r#"{"interp":"step","keyframes":[
                {"time":0.0,"params":{"weight":0.2}},
                {"time":2.0,"params":{"weight":0.8}}]}"#,
        )
        .expect("valid json");
        let src = parse_anim_value(&v).expect("should parse");
        let p = src.params_at_frame(24, 24.0, 0.0); // t = 1.0s, before next keyframe
        assert!((p.weight - 0.2).abs() < 1e-6);
    }

    // 6. bare (unwrapped) keyframe array is also accepted.
    #[test]
    fn bare_keyframe_array_accepted() {
        let v: Value = serde_json::from_str(
            r#"[{"time":0.0,"params":{"age":0.1}},{"time":2.0,"params":{"age":0.9}}]"#,
        )
        .expect("valid json");
        let src = parse_anim_value(&v).expect("should parse");
        assert_eq!(src.fixed_frame_count(), None);
    }

    // 7. suggested_frame_count derives from curve duration and fps.
    #[test]
    fn suggested_frame_count_matches_duration() {
        let v: Value = serde_json::from_str(
            r#"{"keyframes":[{"time":0.0,"params":{"height":0.0}},
                              {"time":2.0,"params":{"height":1.0}}]}"#,
        )
        .expect("valid json");
        let src = parse_anim_value(&v).expect("should parse");
        // 2 seconds at 10 fps -> 20 steps -> 21 frames (inclusive of both ends)
        assert_eq!(src.suggested_frame_count(10.0), 21);
    }

    // 8. empty array is rejected.
    #[test]
    fn empty_array_rejected() {
        let v: Value = serde_json::from_str("[]").expect("valid json");
        assert!(parse_anim_value(&v).is_err());
    }

    // 9. object without "keyframes" is rejected.
    #[test]
    fn object_without_keyframes_rejected() {
        let v: Value = serde_json::from_str(r#"{"foo":1}"#).expect("valid json");
        assert!(parse_anim_value(&v).is_err());
    }

    // 10. unknown interp mode is rejected.
    #[test]
    fn unknown_interp_rejected() {
        let v: Value = serde_json::from_str(
            r#"{"interp":"bogus","keyframes":[{"time":0.0,"params":{"height":0.0}}]}"#,
        )
        .expect("valid json");
        assert!(parse_anim_value(&v).is_err());
    }

    // 11. non-numeric snapshot value is rejected.
    #[test]
    fn non_numeric_snapshot_value_rejected() {
        let v: Value = serde_json::from_str(r#"[{"height":"tall"}]"#).expect("valid json");
        assert!(parse_anim_value(&v).is_err());
    }

    // 12. top-level scalar is rejected.
    #[test]
    fn top_level_scalar_rejected() {
        let v: Value = serde_json::from_str("42").expect("valid json");
        assert!(parse_anim_value(&v).is_err());
    }

    // 13. cubic interpolation with tangents evaluates at endpoints.
    #[test]
    fn cubic_tangents_endpoints() {
        let v: Value = serde_json::from_str(
            r#"{"interp":"cubic","keyframes":[
                {"time":0.0,"params":{"muscle":0.0},"tan_out":{"muscle":0.0}},
                {"time":1.0,"params":{"muscle":1.0},"tan_in":{"muscle":0.0}}]}"#,
        )
        .expect("valid json");
        let src = parse_anim_value(&v).expect("should parse");
        let p0 = src.params_at_frame(0, 1.0, 0.0);
        let p1 = src.params_at_frame(1, 1.0, 0.0);
        assert!((p0.muscle - 0.0).abs() < 1e-3);
        assert!((p1.muscle - 1.0).abs() < 1e-3);
    }
}
