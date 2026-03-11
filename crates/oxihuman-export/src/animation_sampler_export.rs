#![allow(dead_code)]
//! Animation sampler export.

/// Animation sampler export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct AnimSamplerExport {
    pub samplers: Vec<AnimSampler>,
}

/// A single animation sampler.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct AnimSampler {
    pub input: Vec<f32>,
    pub output: Vec<f32>,
    pub interpolation: String,
}

/// Export animation samplers.
#[allow(dead_code)]
pub fn export_anim_sampler(samplers: Vec<AnimSampler>) -> AnimSamplerExport {
    AnimSamplerExport { samplers }
}

/// Get input times for sampler.
#[allow(dead_code)]
pub fn sampler_input(e: &AnimSamplerExport, index: usize) -> &[f32] {
    if index < e.samplers.len() {
        &e.samplers[index].input
    } else {
        &[]
    }
}

/// Get output values for sampler.
#[allow(dead_code)]
pub fn sampler_output(e: &AnimSamplerExport, index: usize) -> &[f32] {
    if index < e.samplers.len() {
        &e.samplers[index].output
    } else {
        &[]
    }
}

/// Get interpolation method.
#[allow(dead_code)]
pub fn sampler_interpolation_ase(e: &AnimSamplerExport, index: usize) -> &str {
    if index < e.samplers.len() {
        &e.samplers[index].interpolation
    } else {
        ""
    }
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn sampler_to_json(e: &AnimSamplerExport, index: usize) -> String {
    if index < e.samplers.len() {
        let s = &e.samplers[index];
        format!(
            "{{\"interpolation\":\"{}\",\"keyframes\":{}}}",
            s.interpolation,
            s.input.len()
        )
    } else {
        "{}".to_string()
    }
}

/// Get sampler count.
#[allow(dead_code)]
pub fn sampler_count_ase(e: &AnimSamplerExport) -> usize {
    e.samplers.len()
}

/// Get export size estimate.
#[allow(dead_code)]
pub fn sampler_export_size(e: &AnimSamplerExport) -> usize {
    e.samplers
        .iter()
        .map(|s| (s.input.len() + s.output.len()) * 4)
        .sum()
}

/// Validate animation samplers.
#[allow(dead_code)]
pub fn validate_anim_sampler(e: &AnimSamplerExport) -> bool {
    let valid_interps = ["LINEAR", "STEP", "CUBICSPLINE"];
    e.samplers.iter().all(|s| {
        !s.input.is_empty() && !s.output.is_empty() && valid_interps.contains(&s.interpolation.as_str())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sampler() -> AnimSampler {
        AnimSampler {
            input: vec![0.0, 1.0],
            output: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            interpolation: "LINEAR".to_string(),
        }
    }

    #[test]
    fn test_export_anim_sampler() {
        let e = export_anim_sampler(vec![make_sampler()]);
        assert_eq!(e.samplers.len(), 1);
    }

    #[test]
    fn test_sampler_input() {
        let e = export_anim_sampler(vec![make_sampler()]);
        assert_eq!(sampler_input(&e, 0), &[0.0, 1.0]);
        assert!(sampler_input(&e, 5).is_empty());
    }

    #[test]
    fn test_sampler_output() {
        let e = export_anim_sampler(vec![make_sampler()]);
        assert_eq!(sampler_output(&e, 0).len(), 6);
    }

    #[test]
    fn test_sampler_interpolation() {
        let e = export_anim_sampler(vec![make_sampler()]);
        assert_eq!(sampler_interpolation_ase(&e, 0), "LINEAR");
        assert_eq!(sampler_interpolation_ase(&e, 5), "");
    }

    #[test]
    fn test_sampler_to_json() {
        let e = export_anim_sampler(vec![make_sampler()]);
        let j = sampler_to_json(&e, 0);
        assert!(j.contains("LINEAR"));
    }

    #[test]
    fn test_sampler_to_json_oob() {
        let e = export_anim_sampler(vec![]);
        assert_eq!(sampler_to_json(&e, 0), "{}");
    }

    #[test]
    fn test_sampler_count() {
        let e = export_anim_sampler(vec![make_sampler(), make_sampler()]);
        assert_eq!(sampler_count_ase(&e), 2);
    }

    #[test]
    fn test_sampler_export_size() {
        let e = export_anim_sampler(vec![make_sampler()]);
        assert!(sampler_export_size(&e) > 0);
    }

    #[test]
    fn test_validate_ok() {
        let e = export_anim_sampler(vec![make_sampler()]);
        assert!(validate_anim_sampler(&e));
    }

    #[test]
    fn test_validate_bad_interp() {
        let e = export_anim_sampler(vec![AnimSampler {
            input: vec![0.0],
            output: vec![0.0],
            interpolation: "INVALID".to_string(),
        }]);
        assert!(!validate_anim_sampler(&e));
    }
}
