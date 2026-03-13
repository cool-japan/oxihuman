// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export bone envelope data (capsule-based influence volumes).

/// A bone envelope represented as a capsule.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoneEnvelope {
    pub bone_name: String,
    pub head: [f32; 3],
    pub tail: [f32; 3],
    pub radius_head: f32,
    pub radius_tail: f32,
    pub weight: f32,
}

/// Collection of bone envelopes.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct EnvelopeSet {
    pub envelopes: Vec<BoneEnvelope>,
}

/// Create a new envelope set.
#[allow(dead_code)]
pub fn new_envelope_set() -> EnvelopeSet {
    EnvelopeSet::default()
}

/// Add an envelope to the set.
#[allow(dead_code)]
pub fn add_envelope(set: &mut EnvelopeSet, env: BoneEnvelope) {
    set.envelopes.push(env);
}

/// Length (bone length) of an envelope.
#[allow(dead_code)]
pub fn envelope_length(env: &BoneEnvelope) -> f32 {
    let d = [
        env.tail[0] - env.head[0],
        env.tail[1] - env.head[1],
        env.tail[2] - env.head[2],
    ];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// Check if a point lies inside the bone envelope capsule.
#[allow(dead_code)]
pub fn point_in_envelope(p: [f32; 3], env: &BoneEnvelope) -> bool {
    // Project p onto the bone segment and check radius at that t.
    let seg = [
        env.tail[0] - env.head[0],
        env.tail[1] - env.head[1],
        env.tail[2] - env.head[2],
    ];
    let len2 = seg[0] * seg[0] + seg[1] * seg[1] + seg[2] * seg[2];
    if len2 < 1e-10 {
        let d2 = sq_dist(p, env.head);
        return d2 <= env.radius_head * env.radius_head;
    }
    let hp = [p[0] - env.head[0], p[1] - env.head[1], p[2] - env.head[2]];
    let t = (hp[0] * seg[0] + hp[1] * seg[1] + hp[2] * seg[2]) / len2;
    let t = t.clamp(0.0, 1.0);
    let closest = [
        env.head[0] + t * seg[0],
        env.head[1] + t * seg[1],
        env.head[2] + t * seg[2],
    ];
    let d2 = sq_dist(p, closest);
    let r = env.radius_head + t * (env.radius_tail - env.radius_head);
    d2 <= r * r
}

fn sq_dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
}

/// Compute the volume of the capsule envelope (approximation).
#[allow(dead_code)]
pub fn envelope_volume(env: &BoneEnvelope) -> f32 {
    use std::f32::consts::PI;
    let l = envelope_length(env);
    let r = (env.radius_head + env.radius_tail) * 0.5;
    PI * r * r * l + (4.0 / 3.0) * PI * r * r * r
}

/// Find envelope by bone name.
#[allow(dead_code)]
pub fn find_envelope<'a>(set: &'a EnvelopeSet, name: &str) -> Option<&'a BoneEnvelope> {
    set.envelopes.iter().find(|e| e.bone_name == name)
}

/// Serialise envelopes to a flat buffer: `[hx,hy,hz,tx,ty,tz,rh,rt,w]` per envelope.
#[allow(dead_code)]
pub fn serialise_envelopes(set: &EnvelopeSet) -> Vec<f32> {
    set.envelopes
        .iter()
        .flat_map(|e| {
            [
                e.head[0],
                e.head[1],
                e.head[2],
                e.tail[0],
                e.tail[1],
                e.tail[2],
                e.radius_head,
                e.radius_tail,
                e.weight,
            ]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_env() -> BoneEnvelope {
        BoneEnvelope {
            bone_name: "arm".to_string(),
            head: [0.0, 0.0, 0.0],
            tail: [0.0, 1.0, 0.0],
            radius_head: 0.1,
            radius_tail: 0.05,
            weight: 1.0,
        }
    }

    #[test]
    fn test_add_envelope() {
        let mut s = new_envelope_set();
        add_envelope(&mut s, sample_env());
        assert_eq!(s.envelopes.len(), 1);
    }

    #[test]
    fn test_envelope_length() {
        let e = sample_env();
        assert!((envelope_length(&e) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_point_in_envelope_inside() {
        let e = sample_env();
        assert!(point_in_envelope([0.0, 0.5, 0.0], &e));
    }

    #[test]
    fn test_point_in_envelope_outside() {
        let e = sample_env();
        assert!(!point_in_envelope([1.0, 0.5, 0.0], &e));
    }

    #[test]
    fn test_envelope_volume_positive() {
        assert!(envelope_volume(&sample_env()) > 0.0);
    }

    #[test]
    fn test_find_envelope_found() {
        let mut s = new_envelope_set();
        add_envelope(&mut s, sample_env());
        assert!(find_envelope(&s, "arm").is_some());
    }

    #[test]
    fn test_find_envelope_not_found() {
        let s = new_envelope_set();
        assert!(find_envelope(&s, "leg").is_none());
    }

    #[test]
    fn test_serialise_envelopes_length() {
        let mut s = new_envelope_set();
        add_envelope(&mut s, sample_env());
        assert_eq!(serialise_envelopes(&s).len(), 9);
    }

    #[test]
    fn test_serialise_empty() {
        let s = new_envelope_set();
        assert!(serialise_envelopes(&s).is_empty());
    }

    #[test]
    fn test_weight_stored() {
        let e = sample_env();
        assert!((e.weight - 1.0).abs() < 1e-6);
    }
}
