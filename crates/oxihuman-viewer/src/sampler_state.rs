#![allow(dead_code)]

//! Texture sampler state (filter, wrap, anisotropy).

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    Nearest,
    Linear,
    Anisotropic,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrapMode {
    Repeat,
    ClampToEdge,
    ClampToBorder,
    MirroredRepeat,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SamplerState {
    pub min_filter: FilterMode,
    pub mag_filter: FilterMode,
    pub mip_filter: FilterMode,
    pub wrap_u: WrapMode,
    pub wrap_v: WrapMode,
    pub wrap_w: WrapMode,
    pub anisotropy: u32,
    pub lod_bias: f32,
    pub min_lod: f32,
    pub max_lod: f32,
}

#[allow(dead_code)]
pub fn default_sampler_state() -> SamplerState {
    SamplerState {
        min_filter: FilterMode::Linear,
        mag_filter: FilterMode::Linear,
        mip_filter: FilterMode::Linear,
        wrap_u: WrapMode::Repeat,
        wrap_v: WrapMode::Repeat,
        wrap_w: WrapMode::Repeat,
        anisotropy: 1,
        lod_bias: 0.0,
        min_lod: 0.0,
        max_lod: 16.0,
    }
}

#[allow(dead_code)]
pub fn ss_set_filter(state: &mut SamplerState, min: FilterMode, mag: FilterMode, mip: FilterMode) {
    state.min_filter = min;
    state.mag_filter = mag;
    state.mip_filter = mip;
}

#[allow(dead_code)]
pub fn ss_set_wrap(state: &mut SamplerState, u: WrapMode, v: WrapMode, w: WrapMode) {
    state.wrap_u = u;
    state.wrap_v = v;
    state.wrap_w = w;
}

#[allow(dead_code)]
pub fn ss_set_anisotropy(state: &mut SamplerState, level: u32) {
    state.anisotropy = level.clamp(1, 16);
}

#[allow(dead_code)]
pub fn ss_set_lod_bias(state: &mut SamplerState, bias: f32) {
    state.lod_bias = bias;
}

#[allow(dead_code)]
pub fn ss_set_lod_range(state: &mut SamplerState, min: f32, max: f32) {
    state.min_lod = min.max(0.0);
    state.max_lod = max.max(state.min_lod);
}

#[allow(dead_code)]
pub fn ss_is_anisotropic(state: &SamplerState) -> bool {
    state.anisotropy > 1 || state.min_filter == FilterMode::Anisotropic
}

#[allow(dead_code)]
pub fn ss_hash(state: &SamplerState) -> u64 {
    let a = state.anisotropy as u64;
    let b = state.lod_bias.to_bits() as u64;
    a.wrapping_mul(0x9e3779b97f4a7c15)
        .wrapping_add(b)
        .wrapping_add(state.min_lod.to_bits() as u64)
}

#[allow(dead_code)]
pub fn ss_to_json(state: &SamplerState) -> String {
    format!(
        "{{\"anisotropy\":{},\"lod_bias\":{},\"min_lod\":{},\"max_lod\":{}}}",
        state.anisotropy, state.lod_bias, state.min_lod, state.max_lod
    )
}

#[allow(dead_code)]
pub fn ss_nearest_mip() -> SamplerState {
    let mut s = default_sampler_state();
    s.mip_filter = FilterMode::Nearest;
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let s = default_sampler_state();
        assert_eq!(s.anisotropy, 1);
        assert_eq!(s.min_filter, FilterMode::Linear);
    }

    #[test]
    fn test_set_filter() {
        let mut s = default_sampler_state();
        ss_set_filter(&mut s, FilterMode::Nearest, FilterMode::Nearest, FilterMode::Nearest);
        assert_eq!(s.min_filter, FilterMode::Nearest);
    }

    #[test]
    fn test_set_wrap() {
        let mut s = default_sampler_state();
        ss_set_wrap(&mut s, WrapMode::ClampToEdge, WrapMode::ClampToEdge, WrapMode::ClampToEdge);
        assert_eq!(s.wrap_u, WrapMode::ClampToEdge);
    }

    #[test]
    fn test_set_anisotropy() {
        let mut s = default_sampler_state();
        ss_set_anisotropy(&mut s, 8);
        assert_eq!(s.anisotropy, 8);
    }

    #[test]
    fn test_anisotropy_clamped() {
        let mut s = default_sampler_state();
        ss_set_anisotropy(&mut s, 32);
        assert_eq!(s.anisotropy, 16);
    }

    #[test]
    fn test_is_anisotropic() {
        let mut s = default_sampler_state();
        assert!(!ss_is_anisotropic(&s));
        ss_set_anisotropy(&mut s, 4);
        assert!(ss_is_anisotropic(&s));
    }

    #[test]
    fn test_set_lod_range() {
        let mut s = default_sampler_state();
        ss_set_lod_range(&mut s, 2.0, 8.0);
        assert!((s.min_lod - 2.0).abs() < 1e-6);
        assert!((s.max_lod - 8.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_lod_bias() {
        let mut s = default_sampler_state();
        ss_set_lod_bias(&mut s, -0.5);
        assert!((s.lod_bias - (-0.5)).abs() < 1e-6);
    }

    #[test]
    fn test_hash_deterministic() {
        let s = default_sampler_state();
        assert_eq!(ss_hash(&s), ss_hash(&s));
    }

    #[test]
    fn test_to_json() {
        let s = default_sampler_state();
        let json = ss_to_json(&s);
        assert!(json.contains("anisotropy"));
    }
}
