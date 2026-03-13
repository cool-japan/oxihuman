// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Bone envelope capsule visualization.

#![allow(dead_code)]

/// Config for bone envelope rendering.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoneEnvelopeConfig {
    /// Inner radius scale.
    pub inner_radius_scale: f32,
    /// Outer radius scale.
    pub outer_radius_scale: f32,
    /// Envelope opacity.
    pub opacity: f32,
    /// Show wireframe.
    pub wireframe: bool,
    /// Color of envelope.
    pub color: [f32; 4],
}

#[allow(dead_code)]
impl Default for BoneEnvelopeConfig {
    fn default() -> Self {
        Self {
            inner_radius_scale: 0.5,
            outer_radius_scale: 1.0,
            opacity: 0.3,
            wireframe: false,
            color: [0.0, 0.5, 1.0, 0.3],
        }
    }
}

/// A bone capsule envelope.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoneEnvelope {
    pub bone_name: String,
    pub head: [f32; 3],
    pub tail: [f32; 3],
    pub radius_head: f32,
    pub radius_tail: f32,
}

/// Create default config.
#[allow(dead_code)]
pub fn new_bone_envelope_config() -> BoneEnvelopeConfig {
    BoneEnvelopeConfig::default()
}

/// Capsule length.
#[allow(dead_code)]
pub fn envelope_length(env: &BoneEnvelope) -> f32 {
    let d = [
        env.tail[0] - env.head[0],
        env.tail[1] - env.head[1],
        env.tail[2] - env.head[2],
    ];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// Check if a point is inside the envelope capsule.
#[allow(dead_code)]
pub fn point_in_envelope(p: [f32; 3], env: &BoneEnvelope) -> bool {
    let d = [
        env.tail[0] - env.head[0],
        env.tail[1] - env.head[1],
        env.tail[2] - env.head[2],
    ];
    let len_sq = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
    if len_sq < 1e-10 {
        let dp = [p[0] - env.head[0], p[1] - env.head[1], p[2] - env.head[2]];
        let dist_sq = dp[0] * dp[0] + dp[1] * dp[1] + dp[2] * dp[2];
        return dist_sq <= env.radius_head * env.radius_head;
    }
    let t =
        ((p[0] - env.head[0]) * d[0] + (p[1] - env.head[1]) * d[1] + (p[2] - env.head[2]) * d[2])
            / len_sq;
    let t = t.clamp(0.0, 1.0);
    let radius = env.radius_head + (env.radius_tail - env.radius_head) * t;
    let closest = [
        env.head[0] + t * d[0],
        env.head[1] + t * d[1],
        env.head[2] + t * d[2],
    ];
    let dp = [p[0] - closest[0], p[1] - closest[1], p[2] - closest[2]];
    let dist_sq = dp[0] * dp[0] + dp[1] * dp[1] + dp[2] * dp[2];
    dist_sq <= radius * radius
}

/// Set opacity.
#[allow(dead_code)]
pub fn bev_set_opacity(cfg: &mut BoneEnvelopeConfig, value: f32) {
    cfg.opacity = value.clamp(0.0, 1.0);
}

/// Toggle wireframe.
#[allow(dead_code)]
pub fn bev_toggle_wireframe(cfg: &mut BoneEnvelopeConfig) {
    cfg.wireframe = !cfg.wireframe;
}

/// Approximate volume of capsule.
#[allow(dead_code)]
pub fn envelope_approx_volume(env: &BoneEnvelope) -> f32 {
    use std::f32::consts::PI;
    let len = envelope_length(env);
    let r = (env.radius_head + env.radius_tail) * 0.5;
    PI * r * r * len + (4.0 / 3.0) * PI * r * r * r
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn bone_envelope_to_json(cfg: &BoneEnvelopeConfig) -> String {
    format!(
        r#"{{"inner_radius_scale":{:.4},"outer_radius_scale":{:.4},"opacity":{:.4},"wireframe":{}}}"#,
        cfg.inner_radius_scale, cfg.outer_radius_scale, cfg.opacity, cfg.wireframe
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = BoneEnvelopeConfig::default();
        assert!((c.inner_radius_scale - 0.5).abs() < 1e-6);
        assert!(!c.wireframe);
    }

    #[test]
    fn test_envelope_length() {
        let env = BoneEnvelope {
            bone_name: "test".to_string(),
            head: [0.0, 0.0, 0.0],
            tail: [0.0, 1.0, 0.0],
            radius_head: 0.1,
            radius_tail: 0.1,
        };
        assert!((envelope_length(&env) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_point_in_envelope_inside() {
        let env = BoneEnvelope {
            bone_name: "bone".to_string(),
            head: [0.0, 0.0, 0.0],
            tail: [0.0, 2.0, 0.0],
            radius_head: 0.5,
            radius_tail: 0.5,
        };
        assert!(point_in_envelope([0.0, 1.0, 0.0], &env));
    }

    #[test]
    fn test_point_in_envelope_outside() {
        let env = BoneEnvelope {
            bone_name: "bone".to_string(),
            head: [0.0, 0.0, 0.0],
            tail: [0.0, 2.0, 0.0],
            radius_head: 0.5,
            radius_tail: 0.5,
        };
        assert!(!point_in_envelope([5.0, 1.0, 0.0], &env));
    }

    #[test]
    fn test_set_opacity_clamped() {
        let mut c = BoneEnvelopeConfig::default();
        bev_set_opacity(&mut c, 5.0);
        assert!((c.opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_toggle_wireframe() {
        let mut c = BoneEnvelopeConfig::default();
        bev_toggle_wireframe(&mut c);
        assert!(c.wireframe);
    }

    #[test]
    fn test_approx_volume_positive() {
        let env = BoneEnvelope {
            bone_name: "b".to_string(),
            head: [0.0, 0.0, 0.0],
            tail: [0.0, 1.0, 0.0],
            radius_head: 0.2,
            radius_tail: 0.2,
        };
        assert!(envelope_approx_volume(&env) > 0.0);
    }

    #[test]
    fn test_to_json() {
        let j = bone_envelope_to_json(&BoneEnvelopeConfig::default());
        assert!(j.contains("opacity"));
    }
}
