// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Tangent space visualization (per-vertex TBN).

/// Which TBN component to visualize.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TbnComponent {
    Tangent,
    Bitangent,
    Normal,
    All,
}

impl TbnComponent {
    #[allow(dead_code)]
    pub fn name(self) -> &'static str {
        match self {
            TbnComponent::Tangent => "tangent",
            TbnComponent::Bitangent => "bitangent",
            TbnComponent::Normal => "normal",
            TbnComponent::All => "all",
        }
    }
}

/// Configuration for tangent space visualization.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TangentSpaceConfig {
    pub component: TbnComponent,
    pub line_length: f32,
    pub tangent_color: [f32; 3],
    pub bitangent_color: [f32; 3],
    pub normal_color: [f32; 3],
    pub enabled: bool,
}

impl Default for TangentSpaceConfig {
    fn default() -> Self {
        TangentSpaceConfig {
            component: TbnComponent::All,
            line_length: 0.1,
            tangent_color: [1.0, 0.0, 0.0],
            bitangent_color: [0.0, 1.0, 0.0],
            normal_color: [0.0, 0.0, 1.0],
            enabled: false,
        }
    }
}

#[allow(dead_code)]
pub fn default_tangent_space_config() -> TangentSpaceConfig {
    TangentSpaceConfig::default()
}

#[allow(dead_code)]
pub fn tsv_set_component(cfg: &mut TangentSpaceConfig, c: TbnComponent) {
    cfg.component = c;
}

#[allow(dead_code)]
pub fn tsv_set_line_length(cfg: &mut TangentSpaceConfig, v: f32) {
    cfg.line_length = v.clamp(0.001, 10.0);
}

#[allow(dead_code)]
pub fn tsv_enable(cfg: &mut TangentSpaceConfig) {
    cfg.enabled = true;
}

#[allow(dead_code)]
pub fn tsv_disable(cfg: &mut TangentSpaceConfig) {
    cfg.enabled = false;
}

/// Convert a TBN vector to a color (maps [-1,1] to `[0,1]`).
#[allow(dead_code)]
pub fn tsv_vector_to_color(v: [f32; 3]) -> [f32; 3] {
    [(v[0] + 1.0) * 0.5, (v[1] + 1.0) * 0.5, (v[2] + 1.0) * 0.5]
}

/// Compute the handedness/sign of the TBN frame.
#[allow(dead_code)]
pub fn tsv_handedness(tangent: [f32; 3], bitangent: [f32; 3], normal: [f32; 3]) -> f32 {
    let cross = [
        tangent[1] * normal[2] - tangent[2] * normal[1],
        tangent[2] * normal[0] - tangent[0] * normal[2],
        tangent[0] * normal[1] - tangent[1] * normal[0],
    ];
    let dot = cross[0] * bitangent[0] + cross[1] * bitangent[1] + cross[2] * bitangent[2];
    if dot >= 0.0 {
        1.0
    } else {
        -1.0
    }
}

/// Check if TBN frame is orthonormal (within tolerance).
#[allow(dead_code)]
pub fn tsv_is_orthonormal(tangent: [f32; 3], bitangent: [f32; 3], normal: [f32; 3]) -> bool {
    let dot_tn = tangent[0] * normal[0] + tangent[1] * normal[1] + tangent[2] * normal[2];
    let dot_tb = tangent[0] * bitangent[0] + tangent[1] * bitangent[1] + tangent[2] * bitangent[2];
    let dot_bn = bitangent[0] * normal[0] + bitangent[1] * normal[1] + bitangent[2] * normal[2];
    dot_tn.abs() < 0.01 && dot_tb.abs() < 0.01 && dot_bn.abs() < 0.01
}

#[allow(dead_code)]
pub fn tsv_to_json(cfg: &TangentSpaceConfig) -> String {
    format!(
        r#"{{"component":"{}","line_length":{:.4},"enabled":{}}}"#,
        cfg.component.name(),
        cfg.line_length,
        cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_disabled() {
        assert!(!default_tangent_space_config().enabled);
    }

    #[test]
    fn vector_to_color_origin() {
        let c = tsv_vector_to_color([0.0, 0.0, 0.0]);
        assert!((c[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn vector_to_color_positive_x() {
        let c = tsv_vector_to_color([1.0, 0.0, 0.0]);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn handedness_right_hand() {
        /* cross(T=[1,0,0], N=[0,0,1]) = [0,-1,0]; dot with B=[0,-1,0] = 1 → right-hand */
        let t = [1.0f32, 0.0, 0.0];
        let b = [0.0f32, -1.0, 0.0];
        let n = [0.0f32, 0.0, 1.0];
        assert!((tsv_handedness(t, b, n) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn orthonormal_standard_axes() {
        let t = [1.0f32, 0.0, 0.0];
        let b = [0.0f32, 1.0, 0.0];
        let n = [0.0f32, 0.0, 1.0];
        assert!(tsv_is_orthonormal(t, b, n));
    }

    #[test]
    fn not_orthonormal_same_axes() {
        let t = [1.0f32, 0.0, 0.0];
        let b = [1.0f32, 0.0, 0.0];
        let n = [1.0f32, 0.0, 0.0];
        assert!(!tsv_is_orthonormal(t, b, n));
    }

    #[test]
    fn set_line_length_clamps() {
        let mut cfg = default_tangent_space_config();
        tsv_set_line_length(&mut cfg, 0.0);
        assert!((cfg.line_length - 0.001).abs() < 1e-6);
    }

    #[test]
    fn enable_disable() {
        let mut cfg = default_tangent_space_config();
        tsv_enable(&mut cfg);
        assert!(cfg.enabled);
        tsv_disable(&mut cfg);
        assert!(!cfg.enabled);
    }

    #[test]
    fn to_json_has_component() {
        assert!(tsv_to_json(&default_tangent_space_config()).contains("component"));
    }
}
