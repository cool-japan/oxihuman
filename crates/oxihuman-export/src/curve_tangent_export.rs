// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Export curve tangent data for animation curves.
#[allow(dead_code)]
pub struct CurveTangentKey {
    pub time: f32,
    pub value: f32,
    pub in_tangent: f32,
    pub out_tangent: f32,
}

#[allow(dead_code)]
pub struct CurveTangentExport {
    pub name: String,
    pub keys: Vec<CurveTangentKey>,
}

#[allow(dead_code)]
pub fn new_curve_tangent_export(name: &str) -> CurveTangentExport {
    CurveTangentExport {
        name: name.to_string(),
        keys: vec![],
    }
}

#[allow(dead_code)]
pub fn add_key(export: &mut CurveTangentExport, key: CurveTangentKey) {
    export.keys.push(key);
    export.keys.sort_by(|a, b| {
        a.time
            .partial_cmp(&b.time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

#[allow(dead_code)]
pub fn key_count(export: &CurveTangentExport) -> usize {
    export.keys.len()
}

#[allow(dead_code)]
pub fn curve_duration(export: &CurveTangentExport) -> f32 {
    if export.keys.is_empty() {
        return 0.0;
    }
    let first = export.keys.first().map_or(0.0, |k| k.time);
    let last = export.keys.last().map_or(0.0, |k| k.time);
    last - first
}

/// Evaluate cubic bezier segment at t in [0,1].
fn cubic_bezier_segment(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let u = 1.0 - t;
    u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3
}

/// Evaluate curve at given time using hermite interpolation.
#[allow(dead_code)]
pub fn evaluate_curve_tangent(export: &CurveTangentExport, time: f32) -> f32 {
    if export.keys.is_empty() {
        return 0.0;
    }
    let keys = &export.keys;
    if time <= keys[0].time {
        return keys[0].value;
    }
    if time >= keys[keys.len() - 1].time {
        return keys[keys.len() - 1].value;
    }
    for i in 0..keys.len() - 1 {
        let k0 = &keys[i];
        let k1 = &keys[i + 1];
        if time >= k0.time && time <= k1.time {
            let dt = k1.time - k0.time;
            if dt < 1e-10 {
                return k0.value;
            }
            let t = (time - k0.time) / dt;
            // Bezier control points from tangents
            let p0 = k0.value;
            let p3 = k1.value;
            let p1 = p0 + k0.out_tangent * dt / 3.0;
            let p2 = p3 - k1.in_tangent * dt / 3.0;
            return cubic_bezier_segment(p0, p1, p2, p3, t);
        }
    }
    keys[keys.len() - 1].value
}

#[allow(dead_code)]
pub fn value_range(export: &CurveTangentExport) -> (f32, f32) {
    if export.keys.is_empty() {
        return (0.0, 0.0);
    }
    let mut mn = export.keys[0].value;
    let mut mx = export.keys[0].value;
    for k in &export.keys {
        if k.value < mn {
            mn = k.value;
        }
        if k.value > mx {
            mx = k.value;
        }
    }
    (mn, mx)
}

#[allow(dead_code)]
pub fn curve_tangent_to_json(export: &CurveTangentExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"key_count\":{},\"duration\":{}}}",
        export.name,
        export.keys.len(),
        curve_duration(export)
    )
}

#[allow(dead_code)]
pub fn auto_tangents(export: &mut CurveTangentExport) {
    let n = export.keys.len();
    if n < 2 {
        return;
    }
    let values: Vec<f32> = export.keys.iter().map(|k| k.value).collect();
    let times: Vec<f32> = export.keys.iter().map(|k| k.time).collect();
    for i in 0..n {
        let tangent = if i == 0 {
            (values[1] - values[0]) / (times[1] - times[0]).max(1e-10)
        } else if i == n - 1 {
            (values[n - 1] - values[n - 2]) / (times[n - 1] - times[n - 2]).max(1e-10)
        } else {
            (values[i + 1] - values[i - 1]) / (times[i + 1] - times[i - 1]).max(1e-10)
        };
        export.keys[i].in_tangent = tangent;
        export.keys[i].out_tangent = tangent;
    }
}

#[allow(dead_code)]
pub fn flatten_all_tangents(export: &mut CurveTangentExport) {
    for k in &mut export.keys {
        k.in_tangent = 0.0;
        k.out_tangent = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ramp_curve() -> CurveTangentExport {
        let mut e = new_curve_tangent_export("pos_y");
        add_key(
            &mut e,
            CurveTangentKey {
                time: 0.0,
                value: 0.0,
                in_tangent: 1.0,
                out_tangent: 1.0,
            },
        );
        add_key(
            &mut e,
            CurveTangentKey {
                time: 1.0,
                value: 1.0,
                in_tangent: 1.0,
                out_tangent: 1.0,
            },
        );
        e
    }

    #[test]
    fn test_key_count() {
        let e = ramp_curve();
        assert_eq!(key_count(&e), 2);
    }

    #[test]
    fn test_curve_duration() {
        let e = ramp_curve();
        assert!((curve_duration(&e) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_at_start() {
        let e = ramp_curve();
        assert!((evaluate_curve_tangent(&e, 0.0) - 0.0).abs() < 1e-4);
    }

    #[test]
    fn test_evaluate_at_end() {
        let e = ramp_curve();
        assert!((evaluate_curve_tangent(&e, 1.0) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_evaluate_midpoint_between() {
        let e = ramp_curve();
        let v = evaluate_curve_tangent(&e, 0.5);
        assert!(v > 0.0 && v < 1.0);
    }

    #[test]
    fn test_value_range() {
        let e = ramp_curve();
        let (mn, mx) = value_range(&e);
        assert!((mn - 0.0).abs() < 1e-5);
        assert!((mx - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_auto_tangents() {
        let mut e = ramp_curve();
        auto_tangents(&mut e);
        assert!(e.keys.iter().all(|k| k.in_tangent.is_finite()));
    }

    #[test]
    fn test_flatten_tangents() {
        let mut e = ramp_curve();
        flatten_all_tangents(&mut e);
        for k in &e.keys {
            assert_eq!(k.in_tangent, 0.0);
            assert_eq!(k.out_tangent, 0.0);
        }
    }

    #[test]
    fn test_to_json() {
        let e = ramp_curve();
        let j = curve_tangent_to_json(&e);
        assert!(j.contains("pos_y"));
    }

    #[test]
    fn test_empty_curve_evaluates_zero() {
        let e = new_curve_tangent_export("empty");
        assert_eq!(evaluate_curve_tangent(&e, 0.5), 0.0);
    }
}
