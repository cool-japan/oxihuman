//! Light probe / environment lighting data.

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ProbeType {
    Sphere,
    Box,
    Infinite,
}

#[allow(dead_code)]
pub struct LightProbe {
    pub id: u32,
    pub name: String,
    pub position: [f32; 3],
    pub probe_type: ProbeType,
    pub radius: f32,
    pub intensity: f32,
    pub color_tint: [f32; 3],
    pub enabled: bool,
    pub sh_coefficients: Vec<[f32; 3]>,
}

#[allow(dead_code)]
pub struct LightProbeSet {
    pub probes: Vec<LightProbe>,
    pub next_id: u32,
    pub ambient_color: [f32; 3],
    pub ambient_intensity: f32,
}

#[allow(dead_code)]
pub fn new_light_probe_set() -> LightProbeSet {
    LightProbeSet {
        probes: Vec::new(),
        next_id: 0,
        ambient_color: [1.0, 1.0, 1.0],
        ambient_intensity: 1.0,
    }
}

#[allow(dead_code)]
pub fn add_light_probe(
    set: &mut LightProbeSet,
    name: &str,
    pos: [f32; 3],
    probe_type: ProbeType,
) -> u32 {
    let id = set.next_id;
    set.next_id += 1;
    let probe = LightProbe {
        id,
        name: name.to_string(),
        position: pos,
        probe_type,
        radius: 1.0,
        intensity: 1.0,
        color_tint: [1.0, 1.0, 1.0],
        enabled: true,
        sh_coefficients: default_sh_coefficients(),
    };
    set.probes.push(probe);
    id
}

#[allow(dead_code)]
pub fn get_light_probe(set: &LightProbeSet, id: u32) -> Option<&LightProbe> {
    set.probes.iter().find(|p| p.id == id)
}

#[allow(dead_code)]
pub fn probe_count(set: &LightProbeSet) -> usize {
    set.probes.len()
}

#[allow(dead_code)]
pub fn remove_light_probe(set: &mut LightProbeSet, id: u32) -> bool {
    if let Some(pos) = set.probes.iter().position(|p| p.id == id) {
        set.probes.remove(pos);
        true
    } else {
        false
    }
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[allow(dead_code)]
pub fn nearest_probe(set: &LightProbeSet, pos: [f32; 3]) -> Option<&LightProbe> {
    set.probes.iter().min_by(|a, b| {
        let da = dist3(a.position, pos);
        let db = dist3(b.position, pos);
        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
    })
}

#[allow(dead_code)]
pub fn set_probe_sh(probe: &mut LightProbe, coefficients: Vec<[f32; 3]>) {
    probe.sh_coefficients = coefficients;
}

/// Evaluate SH (L0-L2, 9 coefficients) at a given direction.
/// Uses real spherical harmonics Y_l^m evaluated at the direction.
#[allow(dead_code)]
pub fn sample_probe_sh(probe: &LightProbe, direction: [f32; 3]) -> [f32; 3] {
    let coeffs = &probe.sh_coefficients;
    if coeffs.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    // Normalize direction
    let len =
        (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2])
            .sqrt();
    let (x, y, z) = if len < f32::EPSILON {
        (0.0, 0.0, 1.0)
    } else {
        (direction[0] / len, direction[1] / len, direction[2] / len)
    };

    // SH basis Y values (real spherical harmonics, L0-L2)
    use std::f32::consts::PI;
    let y0 = 0.5 * (1.0 / PI).sqrt();
    let y1_m1 = (3.0 / (4.0 * PI)).sqrt() * y;
    let y1_0 = (3.0 / (4.0 * PI)).sqrt() * z;
    let y1_1 = (3.0 / (4.0 * PI)).sqrt() * x;
    let y2_m2 = 0.5 * (15.0 / PI).sqrt() * x * y;
    let y2_m1 = 0.5 * (15.0 / PI).sqrt() * y * z;
    let y2_0 = 0.25 * (5.0 / PI).sqrt() * (3.0 * z * z - 1.0);
    let y2_1 = 0.5 * (15.0 / PI).sqrt() * x * z;
    let y2_2 = 0.25 * (15.0 / PI).sqrt() * (x * x - y * y);

    let basis = [y0, y1_m1, y1_0, y1_1, y2_m2, y2_m1, y2_0, y2_1, y2_2];

    let mut result = [0.0f32; 3];
    let count = coeffs.len().min(9);
    for i in 0..count {
        result[0] += coeffs[i][0] * basis[i];
        result[1] += coeffs[i][1] * basis[i];
        result[2] += coeffs[i][2] * basis[i];
    }
    result
}

/// 9 neutral white SH coefficients (L0 = ambient white, L1-L2 = zero).
#[allow(dead_code)]
pub fn default_sh_coefficients() -> Vec<[f32; 3]> {
    let mut coeffs = vec![[0.0f32; 3]; 9];
    // L0 band: constant ambient
    coeffs[0] = [0.886_227, 0.886_227, 0.886_227]; // sqrt(1/(4*pi)) * pi
    coeffs
}

/// Blend two probes' SH colors at the neutral forward direction (0,0,1).
#[allow(dead_code)]
pub fn blend_probes(a: &LightProbe, b: &LightProbe, t: f32) -> [f32; 3] {
    let dir = [0.0, 0.0, 1.0];
    let ca = sample_probe_sh(a, dir);
    let cb = sample_probe_sh(b, dir);
    [
        ca[0] + (cb[0] - ca[0]) * t,
        ca[1] + (cb[1] - ca[1]) * t,
        ca[2] + (cb[2] - ca[2]) * t,
    ]
}

/// 1/d^2 falloff, clamped to [0, 1]. Returns 1 if distance is zero.
#[allow(dead_code)]
pub fn probe_influence_weight(probe: &LightProbe, pos: [f32; 3]) -> f32 {
    let d = dist3(probe.position, pos);
    if d < f32::EPSILON {
        return 1.0;
    }
    (1.0 / (d * d)).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn set_probe_enabled(set: &mut LightProbeSet, id: u32, enabled: bool) {
    if let Some(probe) = set.probes.iter_mut().find(|p| p.id == id) {
        probe.enabled = enabled;
    }
}

#[allow(dead_code)]
pub fn light_probe_set_to_json(set: &LightProbeSet) -> String {
    let probe_strs: Vec<String> = set
        .probes
        .iter()
        .map(|p| {
            let pt = match p.probe_type {
                ProbeType::Sphere => "sphere",
                ProbeType::Box => "box",
                ProbeType::Infinite => "infinite",
            };
            format!(
                r#"{{"id":{},"name":"{}","type":"{}","position":[{},{},{}],"radius":{},"intensity":{},"enabled":{}}}"#,
                p.id,
                p.name,
                pt,
                p.position[0],
                p.position[1],
                p.position[2],
                p.radius,
                p.intensity,
                p.enabled
            )
        })
        .collect();

    format!(
        r#"{{"probes":[{}],"ambient_color":[{},{},{}],"ambient_intensity":{}}}"#,
        probe_strs.join(","),
        set.ambient_color[0],
        set.ambient_color[1],
        set.ambient_color[2],
        set.ambient_intensity
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_light_probe_set() {
        let set = new_light_probe_set();
        assert_eq!(probe_count(&set), 0);
        assert_eq!(set.next_id, 0);
    }

    #[test]
    fn test_add_light_probe() {
        let mut set = new_light_probe_set();
        let id = add_light_probe(&mut set, "sky", [0.0, 10.0, 0.0], ProbeType::Infinite);
        assert_eq!(id, 0);
        assert_eq!(probe_count(&set), 1);
    }

    #[test]
    fn test_get_light_probe() {
        let mut set = new_light_probe_set();
        let id = add_light_probe(&mut set, "probe1", [0.0, 0.0, 0.0], ProbeType::Sphere);
        let probe = get_light_probe(&set, id);
        assert!(probe.is_some());
        assert_eq!(probe.unwrap().name, "probe1");
    }

    #[test]
    fn test_get_light_probe_not_found() {
        let set = new_light_probe_set();
        assert!(get_light_probe(&set, 99).is_none());
    }

    #[test]
    fn test_probe_count() {
        let mut set = new_light_probe_set();
        assert_eq!(probe_count(&set), 0);
        add_light_probe(&mut set, "a", [0.0, 0.0, 0.0], ProbeType::Sphere);
        add_light_probe(&mut set, "b", [1.0, 0.0, 0.0], ProbeType::Box);
        assert_eq!(probe_count(&set), 2);
    }

    #[test]
    fn test_remove_light_probe() {
        let mut set = new_light_probe_set();
        let id = add_light_probe(&mut set, "r", [0.0, 0.0, 0.0], ProbeType::Sphere);
        assert!(remove_light_probe(&mut set, id));
        assert_eq!(probe_count(&set), 0);
        assert!(!remove_light_probe(&mut set, id));
    }

    #[test]
    fn test_nearest_probe() {
        let mut set = new_light_probe_set();
        add_light_probe(&mut set, "far", [100.0, 0.0, 0.0], ProbeType::Sphere);
        add_light_probe(&mut set, "near", [1.0, 0.0, 0.0], ProbeType::Sphere);
        let nearest = nearest_probe(&set, [0.0, 0.0, 0.0]);
        assert!(nearest.is_some());
        assert_eq!(nearest.unwrap().name, "near");
    }

    #[test]
    fn test_nearest_probe_empty() {
        let set = new_light_probe_set();
        assert!(nearest_probe(&set, [0.0, 0.0, 0.0]).is_none());
    }

    #[test]
    fn test_default_sh_coefficients_length() {
        let coeffs = default_sh_coefficients();
        assert_eq!(coeffs.len(), 9);
    }

    #[test]
    fn test_sample_probe_sh() {
        let mut set = new_light_probe_set();
        let id = add_light_probe(&mut set, "p", [0.0, 0.0, 0.0], ProbeType::Sphere);
        let probe = get_light_probe(&set, id).unwrap();
        let color = sample_probe_sh(probe, [0.0, 0.0, 1.0]);
        // Should produce some positive value from L0 band
        assert!(color[0] >= 0.0);
    }

    #[test]
    fn test_probe_influence_weight_at_origin() {
        let mut set = new_light_probe_set();
        let id = add_light_probe(&mut set, "p", [0.0, 0.0, 0.0], ProbeType::Sphere);
        let probe = get_light_probe(&set, id).unwrap();
        let w = probe_influence_weight(probe, [0.0, 0.0, 0.0]);
        assert!((w - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_probe_influence_weight_falloff() {
        let mut set = new_light_probe_set();
        let id = add_light_probe(&mut set, "p", [0.0, 0.0, 0.0], ProbeType::Sphere);
        let probe = get_light_probe(&set, id).unwrap();
        let w = probe_influence_weight(probe, [10.0, 0.0, 0.0]);
        assert!(w < 1.0);
        assert!(w > 0.0);
    }

    #[test]
    fn test_blend_probes() {
        let mut set = new_light_probe_set();
        let id_a = add_light_probe(&mut set, "a", [0.0, 0.0, 0.0], ProbeType::Sphere);
        let id_b = add_light_probe(&mut set, "b", [1.0, 0.0, 0.0], ProbeType::Sphere);
        // Get probes separately to avoid borrow issues
        let ca = {
            let a = get_light_probe(&set, id_a).unwrap();
            sample_probe_sh(a, [0.0, 0.0, 1.0])
        };
        let cb = {
            let b = get_light_probe(&set, id_b).unwrap();
            sample_probe_sh(b, [0.0, 0.0, 1.0])
        };
        let a_probe = get_light_probe(&set, id_a).unwrap();
        let b_probe = get_light_probe(&set, id_b).unwrap();
        let blended = blend_probes(a_probe, b_probe, 0.5);
        let expected = (ca[0] + cb[0]) / 2.0;
        assert!((blended[0] - expected).abs() < 1e-5);
    }

    #[test]
    fn test_set_probe_enabled() {
        let mut set = new_light_probe_set();
        let id = add_light_probe(&mut set, "p", [0.0, 0.0, 0.0], ProbeType::Sphere);
        set_probe_enabled(&mut set, id, false);
        let probe = get_light_probe(&set, id).unwrap();
        assert!(!probe.enabled);
    }

    #[test]
    fn test_light_probe_set_to_json() {
        let mut set = new_light_probe_set();
        add_light_probe(&mut set, "env", [0.0, 5.0, 0.0], ProbeType::Infinite);
        let json = light_probe_set_to_json(&set);
        assert!(!json.is_empty());
        assert!(json.contains("probes"));
        assert!(json.contains("env"));
    }
}
