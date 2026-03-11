#![allow(dead_code)]
//! Scene light probe: a light probe for indirect illumination sampling.

/// A scene light probe.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SceneLightProbe {
    position: [f32; 3],
    radius: f32,
    sh_coefficients: Vec<f32>,
    intensity: f32,
    active: bool,
}

/// Create a new scene light probe.
#[allow(dead_code)]
pub fn new_scene_light_probe(position: [f32; 3], radius: f32) -> SceneLightProbe {
    SceneLightProbe {
        position,
        radius,
        sh_coefficients: vec![0.0; 9],
        intensity: 1.0,
        active: true,
    }
}

/// Return the probe position.
#[allow(dead_code)]
pub fn probe_position_slp(probe: &SceneLightProbe) -> [f32; 3] {
    probe.position
}

/// Return the probe radius.
#[allow(dead_code)]
pub fn probe_radius(probe: &SceneLightProbe) -> f32 {
    probe.radius
}

/// Return the SH coefficients.
#[allow(dead_code)]
pub fn probe_sh_coefficients(probe: &SceneLightProbe) -> &[f32] {
    &probe.sh_coefficients
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn probe_to_json(probe: &SceneLightProbe) -> String {
    format!(
        "{{\"position\":[{},{},{}],\"radius\":{},\"intensity\":{},\"active\":{}}}",
        probe.position[0], probe.position[1], probe.position[2],
        probe.radius, probe.intensity, probe.active
    )
}

/// Check if the probe is active.
#[allow(dead_code)]
pub fn probe_is_active(probe: &SceneLightProbe) -> bool {
    probe.active
}

/// Return the probe intensity.
#[allow(dead_code)]
pub fn probe_intensity(probe: &SceneLightProbe) -> f32 {
    probe.intensity
}

/// Reset the probe to defaults.
#[allow(dead_code)]
pub fn probe_reset(probe: &mut SceneLightProbe) {
    probe.sh_coefficients = vec![0.0; 9];
    probe.intensity = 1.0;
    probe.active = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_probe() {
        let p = new_scene_light_probe([0.0, 1.0, 0.0], 5.0);
        assert!(probe_is_active(&p));
    }

    #[test]
    fn test_position() {
        let p = new_scene_light_probe([1.0, 2.0, 3.0], 5.0);
        let pos = probe_position_slp(&p);
        assert!((pos[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_radius() {
        let p = new_scene_light_probe([0.0; 3], 10.0);
        assert!((probe_radius(&p) - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_sh_coefficients() {
        let p = new_scene_light_probe([0.0; 3], 5.0);
        assert_eq!(probe_sh_coefficients(&p).len(), 9);
    }

    #[test]
    fn test_intensity() {
        let p = new_scene_light_probe([0.0; 3], 5.0);
        assert!((probe_intensity(&p) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let p = new_scene_light_probe([0.0; 3], 5.0);
        let json = probe_to_json(&p);
        assert!(json.contains("\"active\":true"));
    }

    #[test]
    fn test_reset() {
        let mut p = new_scene_light_probe([0.0; 3], 5.0);
        p.intensity = 0.0;
        p.active = false;
        probe_reset(&mut p);
        assert!(probe_is_active(&p));
        assert!((probe_intensity(&p) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_inactive_probe() {
        let mut p = new_scene_light_probe([0.0; 3], 5.0);
        p.active = false;
        assert!(!probe_is_active(&p));
    }

    #[test]
    fn test_probe_default_coeffs() {
        let p = new_scene_light_probe([0.0; 3], 5.0);
        let sh = probe_sh_coefficients(&p);
        assert!(sh.iter().all(|&v| v.abs() < 1e-9));
    }

    #[test]
    fn test_probe_json_position() {
        let p = new_scene_light_probe([1.0, 2.0, 3.0], 5.0);
        let json = probe_to_json(&p);
        assert!(json.contains("\"position\":[1,2,3]"));
    }
}
