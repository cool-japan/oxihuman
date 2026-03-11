#![allow(dead_code)]
//! Sampler export.

/// Sampler export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SamplerExport {
    pub wrap_s: u32,
    pub wrap_t: u32,
    pub mag_filter: u32,
    pub min_filter: u32,
}

/// Export a sampler.
#[allow(dead_code)]
pub fn export_sampler(wrap_s: u32, wrap_t: u32, mag_filter: u32, min_filter: u32) -> SamplerExport {
    SamplerExport { wrap_s, wrap_t, mag_filter, min_filter }
}

/// Get wrap S mode.
#[allow(dead_code)]
pub fn sampler_wrap_s(e: &SamplerExport) -> u32 {
    e.wrap_s
}

/// Get wrap T mode.
#[allow(dead_code)]
pub fn sampler_wrap_t(e: &SamplerExport) -> u32 {
    e.wrap_t
}

/// Get magnification filter.
#[allow(dead_code)]
pub fn sampler_mag_filter(e: &SamplerExport) -> u32 {
    e.mag_filter
}

/// Get minification filter.
#[allow(dead_code)]
pub fn sampler_min_filter(e: &SamplerExport) -> u32 {
    e.min_filter
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn sampler_to_json(e: &SamplerExport) -> String {
    format!(
        "{{\"wrapS\":{},\"wrapT\":{},\"magFilter\":{},\"minFilter\":{}}}",
        e.wrap_s, e.wrap_t, e.mag_filter, e.min_filter
    )
}

/// Get export size estimate.
#[allow(dead_code)]
pub fn sampler_export_size(_e: &SamplerExport) -> usize {
    16 // 4 u32
}

/// Validate sampler.
#[allow(dead_code)]
pub fn validate_sampler(e: &SamplerExport) -> bool {
    let valid_wraps = [10497, 33071, 33648]; // REPEAT, CLAMP_TO_EDGE, MIRRORED_REPEAT
    valid_wraps.contains(&e.wrap_s) && valid_wraps.contains(&e.wrap_t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_sampler() {
        let e = export_sampler(10497, 10497, 9729, 9729);
        assert_eq!(e.wrap_s, 10497);
    }

    #[test]
    fn test_sampler_wrap_s() {
        let e = export_sampler(33071, 10497, 9729, 9729);
        assert_eq!(sampler_wrap_s(&e), 33071);
    }

    #[test]
    fn test_sampler_wrap_t() {
        let e = export_sampler(10497, 33648, 9729, 9729);
        assert_eq!(sampler_wrap_t(&e), 33648);
    }

    #[test]
    fn test_sampler_mag_filter() {
        let e = export_sampler(10497, 10497, 9728, 9729);
        assert_eq!(sampler_mag_filter(&e), 9728);
    }

    #[test]
    fn test_sampler_min_filter() {
        let e = export_sampler(10497, 10497, 9729, 9987);
        assert_eq!(sampler_min_filter(&e), 9987);
    }

    #[test]
    fn test_sampler_to_json() {
        let e = export_sampler(10497, 10497, 9729, 9729);
        let j = sampler_to_json(&e);
        assert!(j.contains("wrapS"));
    }

    #[test]
    fn test_sampler_export_size() {
        let e = export_sampler(10497, 10497, 9729, 9729);
        assert_eq!(sampler_export_size(&e), 16);
    }

    #[test]
    fn test_validate_ok() {
        let e = export_sampler(10497, 10497, 9729, 9729);
        assert!(validate_sampler(&e));
    }

    #[test]
    fn test_validate_bad_wrap() {
        let e = export_sampler(0, 10497, 9729, 9729);
        assert!(!validate_sampler(&e));
    }

    #[test]
    fn test_validate_clamp_to_edge() {
        let e = export_sampler(33071, 33071, 9729, 9729);
        assert!(validate_sampler(&e));
    }
}
