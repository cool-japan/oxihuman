// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Reflection probe and environment cubemap management.

// ── Types ─────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ReflProbeType {
    Box,
    Sphere,
    Infinite,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ReflectionProbeConfig {
    pub probe_type: ReflProbeType,
    pub resolution: u32,
    pub influence_radius: f32,
    pub intensity: f32,
    pub mip_levels: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum CubeFace {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ReflectionProbe {
    pub config: ReflectionProbeConfig,
    pub position: [f32; 3],
    pub is_baked: bool,
    pub face_textures: [u32; 6],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProbeBlendResult {
    pub probe_a_weight: f32,
    pub probe_b_weight: f32,
    pub blend_factor: f32,
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_probe_config() -> ReflectionProbeConfig {
    ReflectionProbeConfig {
        probe_type: ReflProbeType::Sphere,
        resolution: 256,
        influence_radius: 5.0,
        intensity: 1.0,
        mip_levels: 8,
    }
}

#[allow(dead_code)]
pub fn new_reflection_probe(pos: [f32; 3], cfg: ReflectionProbeConfig) -> ReflectionProbe {
    ReflectionProbe {
        config: cfg,
        position: pos,
        is_baked: false,
        face_textures: [0u32; 6],
    }
}

/// Returns a 0..=1 influence weight for `point` relative to the probe.
#[allow(dead_code)]
pub fn probe_influence(probe: &ReflectionProbe, point: [f32; 3]) -> f32 {
    let dist = distance3(probe.position, point);
    let r = probe.config.influence_radius;
    if r < 1e-9 {
        return 0.0;
    }
    (1.0 - (dist / r)).clamp(0.0, 1.0) * probe.config.intensity
}

#[allow(dead_code)]
pub fn is_in_probe_range(probe: &ReflectionProbe, point: [f32; 3]) -> bool {
    distance3(probe.position, point) <= probe.config.influence_radius
}

/// Map a direction vector to the dominant cube face.
#[allow(dead_code)]
pub fn cubemap_direction_to_face(dir: [f32; 3]) -> CubeFace {
    let ax = dir[0].abs();
    let ay = dir[1].abs();
    let az = dir[2].abs();
    if ax >= ay && ax >= az {
        if dir[0] >= 0.0 {
            CubeFace::PosX
        } else {
            CubeFace::NegX
        }
    } else if ay >= ax && ay >= az {
        if dir[1] >= 0.0 {
            CubeFace::PosY
        } else {
            CubeFace::NegY
        }
    } else if dir[2] >= 0.0 {
        CubeFace::PosZ
    } else {
        CubeFace::NegZ
    }
}

#[allow(dead_code)]
pub fn face_name(face: &CubeFace) -> &'static str {
    match face {
        CubeFace::PosX => "+X",
        CubeFace::NegX => "-X",
        CubeFace::PosY => "+Y",
        CubeFace::NegY => "-Y",
        CubeFace::PosZ => "+Z",
        CubeFace::NegZ => "-Z",
    }
}

#[allow(dead_code)]
pub fn probe_type_name(probe: &ReflectionProbe) -> &'static str {
    match probe.config.probe_type {
        ReflProbeType::Box => "Box",
        ReflProbeType::Sphere => "Sphere",
        ReflProbeType::Infinite => "Infinite",
    }
}

#[allow(dead_code)]
pub fn blend_probes(
    a: &ReflectionProbe,
    b: &ReflectionProbe,
    point: [f32; 3],
) -> ProbeBlendResult {
    let wa = probe_influence(a, point);
    let wb = probe_influence(b, point);
    let total = wa + wb;
    let (na, nb, factor) = if total < 1e-9 {
        (0.5, 0.5, 0.5)
    } else {
        let na = wa / total;
        let nb = wb / total;
        (na, nb, nb) // blend_factor represents weight of probe_b
    };
    ProbeBlendResult {
        probe_a_weight: na,
        probe_b_weight: nb,
        blend_factor: factor,
    }
}

#[allow(dead_code)]
pub fn reflection_probe_to_json(probe: &ReflectionProbe) -> String {
    format!(
        r#"{{"position":[{:.4},{:.4},{:.4}],"is_baked":{},"resolution":{},"intensity":{:.4}}}"#,
        probe.position[0],
        probe.position[1],
        probe.position[2],
        probe.is_baked,
        probe.config.resolution,
        probe.config.intensity
    )
}

#[allow(dead_code)]
pub fn invalidate_probe(probe: &mut ReflectionProbe) {
    probe.is_baked = false;
    probe.face_textures = [0u32; 6];
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn distance3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_sphere_256() {
        let cfg = default_probe_config();
        assert_eq!(cfg.resolution, 256);
        assert_eq!(cfg.probe_type, ReflProbeType::Sphere);
    }

    #[test]
    fn new_probe_not_baked() {
        let probe = new_reflection_probe([0.0, 1.0, 0.0], default_probe_config());
        assert!(!probe.is_baked);
    }

    #[test]
    fn probe_influence_center_is_intensity() {
        let probe = new_reflection_probe([0.0, 0.0, 0.0], default_probe_config());
        let inf = probe_influence(&probe, [0.0, 0.0, 0.0]);
        assert!((inf - probe.config.intensity).abs() < 1e-5);
    }

    #[test]
    fn probe_influence_outside_range_is_zero() {
        let probe = new_reflection_probe([0.0, 0.0, 0.0], default_probe_config());
        let inf = probe_influence(&probe, [100.0, 0.0, 0.0]);
        assert!(inf < 1e-9);
    }

    #[test]
    fn is_in_probe_range_inside() {
        let probe = new_reflection_probe([0.0, 0.0, 0.0], default_probe_config());
        assert!(is_in_probe_range(&probe, [1.0, 0.0, 0.0]));
    }

    #[test]
    fn is_in_probe_range_outside() {
        let probe = new_reflection_probe([0.0, 0.0, 0.0], default_probe_config());
        assert!(!is_in_probe_range(&probe, [10.0, 0.0, 0.0]));
    }

    #[test]
    fn cubemap_direction_posx() {
        let face = cubemap_direction_to_face([1.0, 0.0, 0.0]);
        assert_eq!(face, CubeFace::PosX);
    }

    #[test]
    fn cubemap_direction_negy() {
        let face = cubemap_direction_to_face([0.0, -2.0, 0.5]);
        assert_eq!(face, CubeFace::NegY);
    }

    #[test]
    fn face_name_all_faces() {
        assert_eq!(face_name(&CubeFace::PosX), "+X");
        assert_eq!(face_name(&CubeFace::NegX), "-X");
        assert_eq!(face_name(&CubeFace::PosY), "+Y");
        assert_eq!(face_name(&CubeFace::NegY), "-Y");
        assert_eq!(face_name(&CubeFace::PosZ), "+Z");
        assert_eq!(face_name(&CubeFace::NegZ), "-Z");
    }

    #[test]
    fn probe_type_name_box() {
        let mut probe = new_reflection_probe([0.0; 3], default_probe_config());
        probe.config.probe_type = ReflProbeType::Box;
        assert_eq!(probe_type_name(&probe), "Box");
    }

    #[test]
    fn invalidate_probe_clears_baked() {
        let mut probe = new_reflection_probe([0.0; 3], default_probe_config());
        probe.is_baked = true;
        probe.face_textures = [1u32; 6];
        invalidate_probe(&mut probe);
        assert!(!probe.is_baked);
        assert_eq!(probe.face_textures, [0u32; 6]);
    }

    #[test]
    fn blend_probes_weights_sum_to_one() {
        let cfg_a = default_probe_config();
        let cfg_b = default_probe_config();
        let a = new_reflection_probe([0.0, 0.0, 0.0], cfg_a);
        let b = new_reflection_probe([3.0, 0.0, 0.0], cfg_b);
        let result = blend_probes(&a, &b, [1.5, 0.0, 0.0]);
        let sum = result.probe_a_weight + result.probe_b_weight;
        assert!((sum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn reflection_probe_to_json_contains_position() {
        let probe = new_reflection_probe([1.0, 2.0, 3.0], default_probe_config());
        let json = reflection_probe_to_json(&probe);
        assert!(json.contains("position"));
        assert!(json.contains("1.0000"));
    }
}
