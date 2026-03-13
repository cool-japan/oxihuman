// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Export keyframe easing data for animation curves.
#[allow(dead_code)]
pub enum EaseType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
    Step,
}

#[allow(dead_code)]
pub struct EasedKeyframe {
    pub time: f32,
    pub value: f32,
    pub ease: EaseType,
    pub ease_strength: f32,
}

#[allow(dead_code)]
pub struct KeyframeEaseExport {
    pub name: String,
    pub keyframes: Vec<EasedKeyframe>,
}

#[allow(dead_code)]
pub fn new_keyframe_ease_export(name: &str) -> KeyframeEaseExport {
    KeyframeEaseExport {
        name: name.to_string(),
        keyframes: vec![],
    }
}

#[allow(dead_code)]
pub fn add_eased_key(export: &mut KeyframeEaseExport, kf: EasedKeyframe) {
    export.keyframes.push(kf);
    export.keyframes.sort_by(|a, b| {
        a.time
            .partial_cmp(&b.time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

#[allow(dead_code)]
pub fn eased_key_count(export: &KeyframeEaseExport) -> usize {
    export.keyframes.len()
}

/// Apply easing function to normalized time t in `[0,1]`.
#[allow(dead_code)]
pub fn apply_ease(ease: &EaseType, t: f32, strength: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match ease {
        EaseType::Linear => t,
        EaseType::EaseIn => t.powf(1.0 + strength),
        EaseType::EaseOut => 1.0 - (1.0 - t).powf(1.0 + strength),
        EaseType::EaseInOut => {
            if t < 0.5 {
                2.0f32.powf(strength) * t.powf(1.0 + strength)
            } else {
                1.0 - (-2.0 * t + 2.0).powf(1.0 + strength) / 2.0f32.powf(strength)
            }
        }
        EaseType::Bounce => {
            let n1 = 7.5625f32;
            let d1 = 2.75f32;
            let t2 = if t < 1.0 / d1 {
                n1 * t * t
            } else if t < 2.0 / d1 {
                let t2 = t - 1.5 / d1;
                n1 * t2 * t2 + 0.75
            } else if t < 2.5 / d1 {
                let t2 = t - 2.25 / d1;
                n1 * t2 * t2 + 0.9375
            } else {
                let t2 = t - 2.625 / d1;
                n1 * t2 * t2 + 0.984375
            };
            t2 * strength.clamp(0.0, 1.0) + t * (1.0 - strength.clamp(0.0, 1.0))
        }
        EaseType::Elastic => {
            if t <= 0.0 {
                return 0.0;
            }
            if t >= 1.0 {
                return 1.0;
            }
            let p = 0.3f32;
            let s = p / 4.0;
            -((2.0f32.powf(10.0 * (t - 1.0))) * ((t - 1.0 - s) * std::f32::consts::TAU / p).sin())
        }
        EaseType::Step => {
            if t >= 0.5 {
                1.0
            } else {
                0.0
            }
        }
    }
}

#[allow(dead_code)]
pub fn evaluate_eased_curve(export: &KeyframeEaseExport, time: f32) -> f32 {
    let kfs = &export.keyframes;
    if kfs.is_empty() {
        return 0.0;
    }
    if time <= kfs[0].time {
        return kfs[0].value;
    }
    if time >= kfs[kfs.len() - 1].time {
        return kfs[kfs.len() - 1].value;
    }
    for i in 0..kfs.len() - 1 {
        let k0 = &kfs[i];
        let k1 = &kfs[i + 1];
        if time >= k0.time && time <= k1.time {
            let dt = k1.time - k0.time;
            let t = if dt < 1e-10 {
                0.0
            } else {
                (time - k0.time) / dt
            };
            let eased_t = apply_ease(&k0.ease, t, k0.ease_strength);
            return k0.value + (k1.value - k0.value) * eased_t;
        }
    }
    kfs[kfs.len() - 1].value
}

#[allow(dead_code)]
pub fn ease_type_name(e: &EaseType) -> &'static str {
    match e {
        EaseType::Linear => "linear",
        EaseType::EaseIn => "ease_in",
        EaseType::EaseOut => "ease_out",
        EaseType::EaseInOut => "ease_in_out",
        EaseType::Bounce => "bounce",
        EaseType::Elastic => "elastic",
        EaseType::Step => "step",
    }
}

#[allow(dead_code)]
pub fn keyframe_ease_to_json(export: &KeyframeEaseExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"key_count\":{}}}",
        export.name,
        export.keyframes.len()
    )
}

#[allow(dead_code)]
pub fn validate_ease_export(export: &KeyframeEaseExport) -> bool {
    !export.name.is_empty()
        && export
            .keyframes
            .iter()
            .all(|k| k.time.is_finite() && k.value.is_finite())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ramp_export() -> KeyframeEaseExport {
        let mut e = new_keyframe_ease_export("scale_y");
        add_eased_key(
            &mut e,
            EasedKeyframe {
                time: 0.0,
                value: 0.0,
                ease: EaseType::EaseInOut,
                ease_strength: 1.0,
            },
        );
        add_eased_key(
            &mut e,
            EasedKeyframe {
                time: 1.0,
                value: 1.0,
                ease: EaseType::Linear,
                ease_strength: 1.0,
            },
        );
        e
    }

    #[test]
    fn test_key_count() {
        let e = ramp_export();
        assert_eq!(eased_key_count(&e), 2);
    }

    #[test]
    fn test_evaluate_at_start() {
        let e = ramp_export();
        assert!((evaluate_eased_curve(&e, 0.0) - 0.0).abs() < 1e-4);
    }

    #[test]
    fn test_evaluate_at_end() {
        let e = ramp_export();
        assert!((evaluate_eased_curve(&e, 1.0) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_ease_in_slower_start() {
        let v1 = apply_ease(&EaseType::EaseIn, 0.1, 1.0);
        let v2 = apply_ease(&EaseType::Linear, 0.1, 1.0);
        assert!(v1 < v2);
    }

    #[test]
    fn test_ease_out_faster_start() {
        let v1 = apply_ease(&EaseType::EaseOut, 0.1, 1.0);
        let v2 = apply_ease(&EaseType::Linear, 0.1, 1.0);
        assert!(v1 > v2);
    }

    #[test]
    fn test_step_at_half() {
        assert_eq!(apply_ease(&EaseType::Step, 0.4, 1.0), 0.0);
        assert_eq!(apply_ease(&EaseType::Step, 0.6, 1.0), 1.0);
    }

    #[test]
    fn test_ease_type_name() {
        assert_eq!(ease_type_name(&EaseType::Linear), "linear");
        assert_eq!(ease_type_name(&EaseType::Bounce), "bounce");
    }

    #[test]
    fn test_validate() {
        let e = ramp_export();
        assert!(validate_ease_export(&e));
    }

    #[test]
    fn test_to_json() {
        let e = ramp_export();
        let j = keyframe_ease_to_json(&e);
        assert!(j.contains("scale_y"));
    }

    #[test]
    fn test_elastic_boundary() {
        let v0 = apply_ease(&EaseType::Elastic, 0.0, 1.0);
        let v1 = apply_ease(&EaseType::Elastic, 1.0, 1.0);
        assert_eq!(v0, 0.0);
        assert_eq!(v1, 1.0);
    }
}
