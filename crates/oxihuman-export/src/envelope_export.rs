// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Bone envelope export for skinning weight falloff definitions.

/// Envelope for a bone (capsule shape for weight calculation).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EnvelopeExport {
    pub bone_name: String,
    pub head_radius: f32,
    pub tail_radius: f32,
    pub distance: f32,
}

/// Collection of envelopes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EnvelopeBundle {
    pub envelopes: Vec<EnvelopeExport>,
}

/// Create new bundle.
#[allow(dead_code)]
pub fn new_envelope_bundle() -> EnvelopeBundle {
    EnvelopeBundle { envelopes: vec![] }
}

/// Add envelope.
#[allow(dead_code)]
pub fn add_envelope(b: &mut EnvelopeBundle, bone: &str, head_r: f32, tail_r: f32, dist: f32) {
    b.envelopes.push(EnvelopeExport {
        bone_name: bone.to_string(),
        head_radius: head_r,
        tail_radius: tail_r,
        distance: dist,
    });
}

/// Envelope count.
#[allow(dead_code)]
pub fn env_count(b: &EnvelopeBundle) -> usize {
    b.envelopes.len()
}

/// Get by name.
#[allow(dead_code)]
pub fn get_envelope<'a>(b: &'a EnvelopeBundle, name: &str) -> Option<&'a EnvelopeExport> {
    b.envelopes.iter().find(|e| e.bone_name == name)
}

/// Average radius.
#[allow(dead_code)]
pub fn avg_radius(e: &EnvelopeExport) -> f32 {
    (e.head_radius + e.tail_radius) * 0.5
}

/// Approximate volume of envelope capsule.
#[allow(dead_code)]
pub fn envelope_volume(e: &EnvelopeExport) -> f32 {
    let r = avg_radius(e);
    std::f32::consts::PI * r * r * e.distance + (4.0 / 3.0) * std::f32::consts::PI * r * r * r
}

/// Validate.
#[allow(dead_code)]
pub fn env_validate(b: &EnvelopeBundle) -> bool {
    b.envelopes
        .iter()
        .all(|e| e.head_radius >= 0.0 && e.tail_radius >= 0.0 && e.distance >= 0.0)
}

/// Export to JSON.
#[allow(dead_code)]
pub fn envelope_bundle_to_json(b: &EnvelopeBundle) -> String {
    format!("{{\"count\":{}}}", env_count(b))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let b = new_envelope_bundle();
        assert_eq!(env_count(&b), 0);
    }
    #[test]
    fn test_add() {
        let mut b = new_envelope_bundle();
        add_envelope(&mut b, "arm", 0.1, 0.05, 0.3);
        assert_eq!(env_count(&b), 1);
    }
    #[test]
    fn test_get() {
        let mut b = new_envelope_bundle();
        add_envelope(&mut b, "leg", 0.1, 0.1, 0.5);
        assert!(get_envelope(&b, "leg").is_some());
    }
    #[test]
    fn test_get_missing() {
        let b = new_envelope_bundle();
        assert!(get_envelope(&b, "x").is_none());
    }
    #[test]
    fn test_avg_radius() {
        let e = EnvelopeExport {
            bone_name: "a".to_string(),
            head_radius: 0.2,
            tail_radius: 0.4,
            distance: 1.0,
        };
        assert!((avg_radius(&e) - 0.3).abs() < 1e-6);
    }
    #[test]
    fn test_volume() {
        let e = EnvelopeExport {
            bone_name: "a".to_string(),
            head_radius: 1.0,
            tail_radius: 1.0,
            distance: 1.0,
        };
        assert!(envelope_volume(&e) > 0.0);
    }
    #[test]
    fn test_validate() {
        let mut b = new_envelope_bundle();
        add_envelope(&mut b, "a", 0.1, 0.1, 0.5);
        assert!(env_validate(&b));
    }
    #[test]
    fn test_validate_bad() {
        let mut b = new_envelope_bundle();
        add_envelope(&mut b, "a", -0.1, 0.1, 0.5);
        assert!(!env_validate(&b));
    }
    #[test]
    fn test_to_json() {
        let b = new_envelope_bundle();
        assert!(envelope_bundle_to_json(&b).contains("\"count\":0"));
    }
    #[test]
    fn test_zero_distance() {
        let e = EnvelopeExport {
            bone_name: "a".to_string(),
            head_radius: 0.5,
            tail_radius: 0.5,
            distance: 0.0,
        };
        assert!(envelope_volume(&e) > 0.0);
    }
}
