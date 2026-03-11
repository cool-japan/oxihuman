// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Animation curve export (bezier key format for interop).

#[derive(Clone)]
pub struct BezierKey {
    pub time: f32,
    pub value: f32,
    pub in_tangent: f32,
    pub out_tangent: f32,
}

#[derive(Clone, PartialEq, Debug)]
pub enum CurveInfinity {
    Constant,
    Linear,
    Cycle,
    Oscillate,
}

#[derive(Clone)]
pub struct AnimCurve {
    pub name: String,
    pub keys: Vec<BezierKey>,
    pub pre_infinity: CurveInfinity,
    pub post_infinity: CurveInfinity,
}

pub struct AnimCurveExport {
    pub curves: Vec<AnimCurve>,
    pub fps: f32,
    pub duration: f32,
}

pub fn new_anim_curve(name: &str) -> AnimCurve {
    AnimCurve {
        name: name.to_string(),
        keys: Vec::new(),
        pre_infinity: CurveInfinity::Constant,
        post_infinity: CurveInfinity::Constant,
    }
}

pub fn add_key(curve: &mut AnimCurve, time: f32, value: f32, in_t: f32, out_t: f32) {
    let key = BezierKey {
        time,
        value,
        in_tangent: in_t,
        out_tangent: out_t,
    };
    let pos = curve.keys.partition_point(|k| k.time < time);
    curve.keys.insert(pos, key);
}

fn cubic_bezier(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let u = 1.0 - t;
    u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3
}

pub fn evaluate_curve(curve: &AnimCurve, time: f32) -> f32 {
    if curve.keys.is_empty() {
        return 0.0;
    }
    if curve.keys.len() == 1 {
        return curve.keys[0].value;
    }
    if time <= curve.keys[0].time {
        return curve.keys[0].value;
    }
    let last = &curve.keys[curve.keys.len() - 1];
    if time >= last.time {
        return last.value;
    }
    let idx = curve.keys.partition_point(|k| k.time <= time) - 1;
    let k0 = &curve.keys[idx];
    let k1 = &curve.keys[idx + 1];
    let dt = k1.time - k0.time;
    if dt < f32::EPSILON {
        return k0.value;
    }
    let t = (time - k0.time) / dt;
    let scale = dt / 3.0;
    let cp1 = k0.value + k0.out_tangent * scale;
    let cp2 = k1.value - k1.in_tangent * scale;
    cubic_bezier(k0.value, cp1, cp2, k1.value, t)
}

pub fn curve_duration(curve: &AnimCurve) -> f32 {
    if curve.keys.is_empty() {
        return 0.0;
    }
    curve.keys[curve.keys.len() - 1].time - curve.keys[0].time
}

pub fn curve_value_range(curve: &AnimCurve) -> (f32, f32) {
    if curve.keys.is_empty() {
        return (0.0, 0.0);
    }
    let mut min_v = f32::MAX;
    let mut max_v = f32::MIN;
    for k in &curve.keys {
        if k.value < min_v {
            min_v = k.value;
        }
        if k.value > max_v {
            max_v = k.value;
        }
    }
    (min_v, max_v)
}

pub fn auto_tangents(curve: &mut AnimCurve) {
    let n = curve.keys.len();
    if n < 2 {
        return;
    }
    let values: Vec<f32> = curve.keys.iter().map(|k| k.value).collect();
    let times: Vec<f32> = curve.keys.iter().map(|k| k.time).collect();
    for i in 0..n {
        let tangent = if i == 0 {
            if n > 1 {
                let dt = times[1] - times[0];
                if dt.abs() > f32::EPSILON {
                    (values[1] - values[0]) / dt
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else if i == n - 1 {
            let dt = times[n - 1] - times[n - 2];
            if dt.abs() > f32::EPSILON {
                (values[n - 1] - values[n - 2]) / dt
            } else {
                0.0
            }
        } else {
            let dt = times[i + 1] - times[i - 1];
            if dt.abs() > f32::EPSILON {
                (values[i + 1] - values[i - 1]) / dt
            } else {
                0.0
            }
        };
        curve.keys[i].in_tangent = tangent;
        curve.keys[i].out_tangent = tangent;
    }
}

pub fn flatten_tangents(curve: &mut AnimCurve) {
    for k in &mut curve.keys {
        k.in_tangent = 0.0;
        k.out_tangent = 0.0;
    }
}

pub fn export_to_json(export: &AnimCurveExport) -> String {
    let mut s = String::from("{");
    s.push_str(&format!("\"fps\":{},", export.fps));
    s.push_str(&format!("\"duration\":{},", export.duration));
    s.push_str("\"curves\":[");
    for (ci, curve) in export.curves.iter().enumerate() {
        if ci > 0 {
            s.push(',');
        }
        s.push('{');
        s.push_str(&format!("\"name\":\"{}\",", curve.name));
        s.push_str("\"keys\":[");
        for (ki, key) in curve.keys.iter().enumerate() {
            if ki > 0 {
                s.push(',');
            }
            s.push_str(&format!(
                "{{\"time\":{},\"value\":{},\"in_tangent\":{},\"out_tangent\":{}}}",
                key.time, key.value, key.in_tangent, key.out_tangent
            ));
        }
        s.push_str("]}");
    }
    s.push_str("]}");
    s
}

pub fn curves_to_csv(export: &AnimCurveExport, fps: f32) -> String {
    if export.curves.is_empty() {
        return String::new();
    }
    let mut s = String::from("time");
    for curve in &export.curves {
        s.push(',');
        s.push_str(&curve.name);
    }
    s.push('\n');
    let frame_count = (export.duration * fps).ceil() as u32;
    for frame in 0..=frame_count {
        let t = frame as f32 / fps;
        s.push_str(&format!("{}", t));
        for curve in &export.curves {
            let v = evaluate_curve(curve, t);
            s.push(',');
            s.push_str(&format!("{}", v));
        }
        s.push('\n');
    }
    s
}

pub fn new_anim_curve_export(fps: f32) -> AnimCurveExport {
    AnimCurveExport {
        curves: Vec::new(),
        fps,
        duration: 0.0,
    }
}

pub fn add_curve_to_export(export: &mut AnimCurveExport, curve: AnimCurve) {
    let dur = curve_duration(&curve);
    if dur > export.duration {
        export.duration = dur;
    }
    export.curves.push(curve);
}

pub fn resample_curve(curve: &AnimCurve, fps: f32) -> Vec<(f32, f32)> {
    if curve.keys.is_empty() {
        return Vec::new();
    }
    let start = curve.keys[0].time;
    let end = curve.keys[curve.keys.len() - 1].time;
    let frame_count = ((end - start) * fps).ceil() as u32;
    let mut samples = Vec::with_capacity(frame_count as usize + 1);
    for i in 0..=frame_count {
        let t = start + i as f32 / fps;
        samples.push((t, evaluate_curve(curve, t)));
    }
    samples
}

pub fn merge_curve_exports(mut a: AnimCurveExport, b: AnimCurveExport) -> AnimCurveExport {
    for curve in b.curves {
        add_curve_to_export(&mut a, curve);
    }
    if b.fps > 0.0 && b.fps > a.fps {
        a.fps = b.fps;
    }
    a
}

/* ── spec functions (wave 150B) ── */

/// A single key for the spec AnimCurveData.
#[derive(Debug, Clone)]
pub struct AnimCurveKey {
    pub time: f32,
    pub value: f32,
}

/// Spec-style animation curve data.
#[derive(Debug, Clone)]
pub struct AnimCurveData {
    pub name: String,
    pub keys: Vec<AnimCurveKey>,
}

/// Create a new `AnimCurveData`.
pub fn new_anim_curve_data(name: &str) -> AnimCurveData {
    AnimCurveData {
        name: name.to_string(),
        keys: Vec::new(),
    }
}

/// Push a key into an `AnimCurveData`.
pub fn anim_curve_push_key(data: &mut AnimCurveData, time: f32, value: f32) {
    data.keys.push(AnimCurveKey { time, value });
}

/// Evaluate with linear interpolation at `time`.
pub fn anim_curve_evaluate(data: &AnimCurveData, time: f32) -> f32 {
    if data.keys.is_empty() {
        return 0.0;
    }
    if data.keys.len() == 1 {
        return data.keys[0].value;
    }
    if time <= data.keys[0].time {
        return data.keys[0].value;
    }
    let last = &data.keys[data.keys.len() - 1];
    if time >= last.time {
        return last.value;
    }
    for i in 0..data.keys.len() - 1 {
        let k0 = &data.keys[i];
        let k1 = &data.keys[i + 1];
        if time >= k0.time && time <= k1.time {
            let dt = k1.time - k0.time;
            if dt < f32::EPSILON {
                return k0.value;
            }
            let t = (time - k0.time) / dt;
            return k0.value + (k1.value - k0.value) * t;
        }
    }
    0.0
}

/// Export `AnimCurveData` to JSON.
pub fn anim_curve_to_json(data: &AnimCurveData) -> String {
    format!(
        "{{\"name\":\"{}\",\"keys\":{}}}",
        data.name,
        data.keys.len()
    )
}

/// Duration of an `AnimCurveData`.
pub fn anim_curve_duration(data: &AnimCurveData) -> f32 {
    if data.keys.len() < 2 {
        return 0.0;
    }
    data.keys.last().map_or(0.0, |k| k.time) - data.keys[0].time
}

/// Number of keys.
pub fn anim_curve_key_count(data: &AnimCurveData) -> usize {
    data.keys.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_curve() -> AnimCurve {
        let mut c = new_anim_curve("pos_x");
        add_key(&mut c, 0.0, 0.0, 0.0, 0.0);
        add_key(&mut c, 1.0, 1.0, 0.0, 0.0);
        add_key(&mut c, 2.0, 0.5, 0.0, 0.0);
        c
    }

    #[test]
    fn test_new_anim_curve() {
        let c = new_anim_curve("rot");
        assert_eq!(c.name, "rot");
        assert!(c.keys.is_empty());
    }

    #[test]
    fn test_evaluate_at_key_time() {
        let c = make_curve();
        assert!((evaluate_curve(&c, 0.0) - 0.0).abs() < 1e-5);
        assert!((evaluate_curve(&c, 1.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_curve_duration() {
        let c = make_curve();
        assert!((curve_duration(&c) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_anim_curve_data_key_count() {
        /* spec AnimCurveData key count */
        let mut d = new_anim_curve_data("test");
        anim_curve_push_key(&mut d, 0.0, 0.0);
        anim_curve_push_key(&mut d, 1.0, 1.0);
        assert_eq!(anim_curve_key_count(&d), 2);
    }

    #[test]
    fn test_anim_curve_evaluate_linear() {
        let mut d = new_anim_curve_data("x");
        anim_curve_push_key(&mut d, 0.0, 0.0);
        anim_curve_push_key(&mut d, 1.0, 10.0);
        assert!((anim_curve_evaluate(&d, 0.5) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_anim_curve_duration() {
        let mut d = new_anim_curve_data("x");
        anim_curve_push_key(&mut d, 0.0, 0.0);
        anim_curve_push_key(&mut d, 2.0, 1.0);
        assert!((anim_curve_duration(&d) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_anim_curve_to_json() {
        let d = new_anim_curve_data("track");
        let j = anim_curve_to_json(&d);
        assert!(j.contains("track"));
    }
}
